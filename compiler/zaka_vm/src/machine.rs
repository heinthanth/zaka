use std::io::Write;

use bon::Builder;
use zakac_codegen::bc::Module;
use zakac_codegen::bc::Op;
use zakac_codegen::bc::TrapCode;
use zakac_ir_builder::ir::CmpOp;
use zakac_ir_builder::ir::InfixOp;
use zakac_ir_builder::ir::PrefixOp;
use zakac_ir_builder::value::EvalError;
use zakac_ir_builder::value::Value;
use zakac_ir_builder::value::compare;
use zakac_ir_builder::value::convert;
use zakac_ir_builder::value::infix_wrapping;
use zakac_ir_builder::value::prefix_wrapping;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::do_while;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::kind::DataKind;

use crate::print::text;
use crate::trap::Trap;
use crate::trap::TrapKind;

/// Limits for a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct RunOptions
{
    /// The most nested calls, the main function included.
    #[builder(default = 4096)]
    pub max_depth: u32,
    /// The most ops to run, or `None` for no limit.
    pub max_steps: Option<u64>,
}

impl Default for RunOptions
{
    fn default() -> Self { RunOptions::builder().build() }
}

/// Run a module with the default limits, writing its output to `out`.
///
/// # Arguments
///
/// * `module` - The module.
/// * `out` - Where print ops write.
///
/// # Returns
///
/// `Ok` when the main function returns, or the trap that stopped it.
pub fn run(module: &Module, out: &mut dyn Write) -> Result<(), Trap>
{
    run_with(module, out, &RunOptions::default())
}

/// Run a module, writing its output to `out` as it goes.
///
/// # Arguments
///
/// * `module` - The module.
/// * `out` - Where print ops write.
/// * `options` - The limits.
///
/// # Returns
///
/// `Ok` when the main function returns, or the trap that stopped it.
pub fn run_with(
    module: &Module, out: &mut dyn Write, options: &RunOptions,
) -> Result<(), Trap>
{
    let mut machine = Machine::new(module.clone(), *options);
    while machine.running()
    {
        machine.step();
        drain(&mut machine, out)?;
    }
    machine.state().trap().cloned().map_or(Ok(()), Err)
}

/// Write what the machine printed since the last call, and clear it.
///
/// # Arguments
///
/// * `machine` - The machine.
/// * `out` - Where the text goes.
///
/// # Returns
///
/// `Ok`, or a trap at the op that printed when the write fails.
fn drain(machine: &mut Machine, out: &mut dyn Write) -> Result<(), Trap>
{
    ret_if!(machine.out.is_empty(), Ok(()));
    let written = out.write_all(machine.out.as_bytes());
    machine.out.clear();
    written.map_err(|e| machine.trap(TrapKind::Write { msg: e.to_string() }))
}

/// A call in progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct Frame
{
    /// The function index.
    pub func: u32,
    /// The index of the next op.
    pub pc: u32,
    /// Where the locals of the call start in the store.
    pub base: usize,
    /// The depth of the operand stack when the call began, after its
    /// arguments were taken; the values over it belong to the call.
    pub depth: usize,
}

/// Where a run stands. A trap is final: further steps change nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State
{
    /// Ops remain to run.
    Running,
    /// The main function returned.
    Finished,
    /// The program stopped before its end.
    Trapped(Trap),
}

impl State
{
    /// The trap, when the run stopped before its end.
    ///
    /// # Returns
    ///
    /// The trap, or `None`.
    pub fn trap(&self) -> Option<&Trap>
    {
        match self
        {
            State::Trapped(t) => Some(t),
            _ => None,
        }
    }
}

/// The result of one op.
type Step = Result<(), TrapKind>;

/// A machine that runs a module one op at a time. It owns the module and
/// keeps what print ops write, so a caller can hold it, look at its state
/// between ops, and clone it to go back later.
#[derive(Debug, Clone, Builder)]
#[builder(start_fn(vis = ""))]
pub struct Machine
{
    /// The module.
    module: Module,
    /// The limits.
    options: RunOptions,
    /// The global slots.
    globals: Vec<Value>,
    /// The operand stack.
    #[builder(default)]
    stack: Vec<Value>,
    /// The locals of each call, one after the other.
    #[builder(default)]
    store: Vec<Value>,
    /// The calls in progress, innermost last.
    #[builder(default)]
    frames: Vec<Frame>,
    /// The number of ops run.
    #[builder(default)]
    steps: u64,
    /// The function and op index of the op that ran last.
    at: (u32, u32),
    /// What print ops wrote and no one took.
    #[builder(default)]
    out: String,
    /// Where the run stands.
    #[builder(default = State::Running)]
    state: State,
}

/// A malformed-ops trap.
///
/// # Arguments
///
/// * `reason` - What is wrong.
///
/// # Returns
///
/// The trap kind.
fn malformed(reason: &str) -> TrapKind
{
    TrapKind::Malformed { reason: reason.to_string() }
}

/// The trap for an error of an operation on values.
///
/// # Arguments
///
/// * `e` - The error.
///
/// # Returns
///
/// The trap kind.
fn eval_trap(e: EvalError) -> TrapKind
{
    match e
    {
        EvalError::ZeroDivisor => TrapKind::ZeroDivisor,
        EvalError::ShiftTooFar { distance, bits } =>
        {
            TrapKind::ShiftTooFar { distance, bits }
        }
        EvalError::Overflow => malformed("checked overflow at run time"),
        EvalError::Mismatch => malformed("operands do not suit the op"),
    }
}

/// Add a line to the output.
///
/// # Arguments
///
/// * `out` - The output.
/// * `line` - The line without its break.
fn write_line(out: &mut String, line: &str)
{
    out.push_str(line);
    out.push('\n');
}

impl Machine
{
    /// Make a machine at the first op of the main function. A depth limit
    /// of zero traps here.
    ///
    /// # Arguments
    ///
    /// * `module` - The module.
    /// * `options` - The limits.
    ///
    /// # Returns
    ///
    /// The machine.
    pub fn new(module: Module, options: RunOptions) -> Self
    {
        let globals =
            module.globals.iter().map(|s| Value::default_of(s.kind)).collect();
        let main = module.main;
        #[rustfmt::skip]
        let mut machine = Machine::builder().module(module).options(options)
            .globals(globals).at((main, 0)).build();
        let entered = machine.enter(main);
        do_if!(let Err(kind) = entered, machine.state = State::Trapped(machine.trap(kind)));
        machine
    }

    /// Run one op. Once the run is finished or trapped, this does nothing.
    ///
    /// # Returns
    ///
    /// Where the run stands.
    pub fn step(&mut self) -> &State
    {
        ret_if!(!self.running(), &self.state);
        let ran = self.fetch();
        self.state = match ran
        {
            Err(kind) => State::Trapped(self.trap(kind)),
            Ok(()) if self.frames.is_empty() => State::Finished,
            Ok(()) => State::Running,
        };
        &self.state
    }

    /// Run one op; a call runs to its return.
    ///
    /// # Returns
    ///
    /// Where the run stands.
    pub fn step_over(&mut self) -> &State
    {
        let depth = self.frames.len();
        let call = matches!(self.next_op(), Some(Op::Call(_)));
        self.step();
        do_while!(call && self.running() && self.frames.len() > depth, let _ = self.step());
        &self.state
    }

    /// Run until the innermost call returns.
    ///
    /// # Returns
    ///
    /// Where the run stands.
    pub fn step_out(&mut self) -> &State
    {
        let depth = self.frames.len();
        do_while!(self.running() && self.frames.len() >= depth, let _ = self.step());
        &self.state
    }

    /// Run until the next op is one of `stops`, the run ends, or the op
    /// budget of the limits runs out. At least one op runs, so a run can go
    /// on from a stop.
    ///
    /// # Arguments
    ///
    /// * `stops` - Function and op index pairs to stop at.
    ///
    /// # Returns
    ///
    /// Where the run stands.
    pub fn run_until(&mut self, stops: &[(u32, u32)]) -> &State
    {
        let stopped = |m: &Self| m.position().is_some_and(|p| stops.contains(&p));
        self.step();
        do_while!(self.running() && !stopped(self), let _ = self.step());
        &self.state
    }

    /// Where the run stands.
    ///
    /// # Returns
    ///
    /// The state.
    pub fn state(&self) -> &State { &self.state }

    /// Whether ops remain to run.
    ///
    /// # Returns
    ///
    /// `true` while the run is neither finished nor trapped.
    pub fn running(&self) -> bool { self.state == State::Running }

    /// The function and op index of the next op; after a trap, of the op
    /// that trapped.
    ///
    /// # Returns
    ///
    /// The place, or `None` once the run is finished.
    pub fn position(&self) -> Option<(u32, u32)>
    {
        match self.state
        {
            State::Trapped(_) => Some(self.at),
            State::Finished => None,
            State::Running => self.frames.last().map(|f| (f.func, f.pc)),
        }
    }

    /// The next op to run.
    ///
    /// # Returns
    ///
    /// The op, or `None` once the run has ended.
    pub fn next_op(&self) -> Option<Op>
    {
        ret_if!(!self.running(), None);
        let (func, pc) = self.position()?;
        self.module.funcs.get(func as usize)?.code.get(pc as usize).copied()
    }

    /// The calls in progress, innermost last.
    ///
    /// # Returns
    ///
    /// The frames.
    pub fn frames(&self) -> &[Frame] { &self.frames }

    /// The operand stack, top last.
    ///
    /// # Returns
    ///
    /// The values.
    pub fn stack(&self) -> &[Value] { &self.stack }

    /// The locals of one call, parameters first.
    ///
    /// # Arguments
    ///
    /// * `frame` - The index in [`Machine::frames`].
    ///
    /// # Returns
    ///
    /// The values; none for a frame that is not there.
    pub fn locals(&self, frame: usize) -> &[Value]
    {
        let Some(f) = self.frames.get(frame)
        else
        {
            return &[];
        };
        let end = self.frames.get(frame + 1).map_or(self.store.len(), |n| n.base);
        &self.store[f.base..end]
    }

    /// The global slots.
    ///
    /// # Returns
    ///
    /// The values.
    pub fn globals(&self) -> &[Value] { &self.globals }

    /// What print ops wrote and no one took.
    ///
    /// # Returns
    ///
    /// The text.
    pub fn output(&self) -> &str { &self.out }

    /// Take what print ops wrote so far.
    ///
    /// # Returns
    ///
    /// The text; the machine keeps none of it.
    pub fn take_output(&mut self) -> String { std::mem::take(&mut self.out) }

    /// The number of ops run.
    ///
    /// # Returns
    ///
    /// The count.
    pub fn steps(&self) -> u64 { self.steps }

    /// The module the machine runs.
    ///
    /// # Returns
    ///
    /// The module.
    pub fn module(&self) -> &Module { &self.module }

    /// Place a trap at the op that ran last.
    ///
    /// # Arguments
    ///
    /// * `kind` - The reason the program stopped.
    ///
    /// # Returns
    ///
    /// The trap.
    fn trap(&self, kind: TrapKind) -> Trap
    {
        let (func, pc) = self.at;
        let name =
            self.module.funcs.get(func as usize).map_or("?", |f| f.name.as_str());
        Trap::builder().kind(kind).func(name).pc(pc).build()
    }

    /// Run the next op of the innermost call.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason the program stops.
    fn fetch(&mut self) -> Step
    {
        self.steps += 1;
        let limit = self.options.max_steps.filter(|max| self.steps > *max);
        ret_if!(let Some(steps) = limit, Err(TrapKind::StepLimit { steps }));
        let frame = self.frames.last_mut().ok_or_else(|| malformed("no call"))?;
        let code = &self.module.funcs[frame.func as usize].code;
        let op = code.get(frame.pc as usize).copied();
        self.at = (frame.func, frame.pc);
        frame.pc += 1;
        self.exec(op.ok_or_else(|| malformed("ran past the last op"))?)
    }

    /// Run one op.
    ///
    /// # Arguments
    ///
    /// * `op` - The op.
    ///
    /// # Returns
    ///
    /// `Ok`, or the reason the program stops.
    fn exec(&mut self, op: Op) -> Step
    {
        match op
        {
            Op::Push(v) => self.push(v),
            Op::LoadLocal(n) => self.load_local(n),
            Op::StoreLocal(n) => self.store_local(n),
            Op::LoadGlobal(n) => self.load_global(n),
            Op::StoreGlobal(n) => self.store_global(n),
            Op::Prefix(op) => self.prefix(op),
            Op::Infix(op) => self.infix(op),
            Op::Compare(op) => self.compare(op),
            Op::Convert(kind) => self.convert(kind),
            Op::Jump(t) => self.goto(t),
            Op::JumpIfFalse(t) => self.branch(t, false),
            Op::JumpIfTrue(t) => self.branch(t, true),
            Op::Call(f) => self.enter(f),
            Op::Ret => self.leave(),
            Op::Print => self.print(),
            Op::PrintText(i) => self.print_text(i),
            Op::Pop => self.pop().map(drop),
            Op::Trap(TrapCode::StepNotPositive) => Err(TrapKind::StepNotPositive),
        }
    }

    /// Push a value.
    ///
    /// # Arguments
    ///
    /// * `v` - The value.
    ///
    /// # Returns
    ///
    /// `Ok`.
    fn push(&mut self, v: Value) -> Step
    {
        self.stack.push(v);
        Ok(())
    }

    /// Pop a value.
    ///
    /// # Returns
    ///
    /// The value, or a trap when the stack is blank.
    fn pop(&mut self) -> Result<Value, TrapKind>
    {
        self.stack.pop().ok_or_else(|| malformed("pop from a blank stack"))
    }

    /// Pop a boolean.
    ///
    /// # Returns
    ///
    /// The boolean, or a trap for another value.
    fn pop_bool(&mut self) -> Result<bool, TrapKind>
    {
        self.pop()?.as_bool().ok_or_else(|| malformed("a jump needs a boolean"))
    }

    /// The store index of a local slot of the innermost call.
    ///
    /// # Arguments
    ///
    /// * `n` - The slot index.
    ///
    /// # Returns
    ///
    /// The store index, or a trap when the slot is out of range.
    fn local(&self, n: u32) -> Result<usize, TrapKind>
    {
        let base = self.frames.last().map_or(0, |f| f.base);
        let at = base + n as usize;
        ret_if!(at >= self.store.len(), Err(malformed("local slot out of range")));
        Ok(at)
    }

    /// Check a global slot index.
    ///
    /// # Arguments
    ///
    /// * `n` - The slot index.
    ///
    /// # Returns
    ///
    /// The index, or a trap when it is out of range.
    fn global(&self, n: u32) -> Result<usize, TrapKind>
    {
        let at = n as usize;
        ret_if!(at >= self.globals.len(), Err(malformed("global slot out of range")));
        Ok(at)
    }

    /// Push a local slot.
    ///
    /// # Arguments
    ///
    /// * `n` - The slot index.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn load_local(&mut self, n: u32) -> Step
    {
        let at = self.local(n)?;
        self.push(self.store[at])
    }

    /// Pop into a local slot.
    ///
    /// # Arguments
    ///
    /// * `n` - The slot index.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn store_local(&mut self, n: u32) -> Step
    {
        let at = self.local(n)?;
        self.store[at] = self.pop()?;
        Ok(())
    }

    /// Push a global slot.
    ///
    /// # Arguments
    ///
    /// * `n` - The slot index.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn load_global(&mut self, n: u32) -> Step
    {
        let at = self.global(n)?;
        self.push(self.globals[at])
    }

    /// Pop into a global slot.
    ///
    /// # Arguments
    ///
    /// * `n` - The slot index.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn store_global(&mut self, n: u32) -> Step
    {
        let at = self.global(n)?;
        self.globals[at] = self.pop()?;
        Ok(())
    }

    /// Work out a prefix operator on the top value.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn prefix(&mut self, op: PrefixOp) -> Step
    {
        let v = self.pop()?;
        let platform = self.module.platform_bits;
        self.push(prefix_wrapping(op, v, platform).map_err(eval_trap)?)
    }

    /// Work out an infix operator on the two top values.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn infix(&mut self, op: InfixOp) -> Step
    {
        let (r, l) = (self.pop()?, self.pop()?);
        let platform = self.module.platform_bits;
        self.push(infix_wrapping(op, l, r, platform).map_err(eval_trap)?)
    }

    /// Compare the two top values.
    ///
    /// # Arguments
    ///
    /// * `op` - The comparison.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn compare(&mut self, op: CmpOp) -> Step
    {
        let (r, l) = (self.pop()?, self.pop()?);
        self.push(Value::Bool(compare(op, l, r).map_err(eval_trap)?))
    }

    /// Convert the top value.
    ///
    /// # Arguments
    ///
    /// * `kind` - The target kind.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn convert(&mut self, kind: DataKind) -> Step
    {
        let v = self.pop()?;
        let platform = self.module.platform_bits;
        self.push(convert(v, kind, platform).map_err(eval_trap)?)
    }

    /// Go to an op of the innermost call.
    ///
    /// # Arguments
    ///
    /// * `target` - The op index.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap when there is no call.
    fn goto(&mut self, target: u32) -> Step
    {
        let frame = self.frames.last_mut().ok_or_else(|| malformed("no call"))?;
        frame.pc = target;
        Ok(())
    }

    /// Pop a boolean and go to an op when it equals `when`.
    ///
    /// # Arguments
    ///
    /// * `target` - The op index.
    /// * `when` - The boolean that takes the jump.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn branch(&mut self, target: u32, when: bool) -> Step
    {
        let taken = self.pop_bool()? == when;
        do_if!(taken, self.goto(target)?);
        Ok(())
    }

    /// Start a call: its locals take their defaults, then the arguments
    /// move from the stack into the parameters.
    ///
    /// # Arguments
    ///
    /// * `f` - The function index.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn enter(&mut self, f: u32) -> Step
    {
        let depth = self.options.max_depth;
        ret_if!(
            self.frames.len() as u32 >= depth,
            Err(TrapKind::StackOverflow { depth })
        );
        let func = self
            .module
            .funcs
            .get(f as usize)
            .ok_or_else(|| malformed("no function"))?;
        let params = func.params as usize;
        let short = self.stack.len() < params || func.locals.len() < params;
        ret_if!(short, Err(malformed("missing arguments")));
        let base = self.store.len();
        self.store.extend(func.locals.iter().map(|s| Value::default_of(s.kind)));
        let args = self.stack.split_off(self.stack.len() - params);
        self.store[base..base + params].copy_from_slice(&args);
        let depth = self.stack.len();
        self.frames
            .push(Frame::builder().func(f).pc(0).base(base).depth(depth).build());
        Ok(())
    }

    /// End the innermost call; a result remains on the stack.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap when there is no call.
    fn leave(&mut self) -> Step
    {
        let frame = self.frames.pop().ok_or_else(|| malformed("no call"))?;
        self.store.truncate(frame.base);
        Ok(())
    }

    /// Pop a value and write it.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn print(&mut self) -> Step
    {
        let v = self.pop()?;
        write_line(&mut self.out, &text(v));
        Ok(())
    }

    /// Write a pooled text.
    ///
    /// # Arguments
    ///
    /// * `i` - The pool index.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn print_text(&mut self, i: u32) -> Step
    {
        let texts = &self.module.texts;
        let line = texts.get(i as usize).ok_or_else(|| malformed("no text"))?;
        write_line(&mut self.out, line);
        Ok(())
    }
}
