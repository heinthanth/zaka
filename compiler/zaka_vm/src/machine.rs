use std::collections::BTreeMap;
use std::io::Write;

use bon::Builder;
use zakac_codegen::bc::Module;
use zakac_codegen::bc::Op;
use zakac_codegen::bc::TrapCode;
use zakac_i18n::t;
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
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::IntKind;
use zakac_semcheck::kind::Width;

use crate::mem::Mem;
use crate::mem::Origin;
use crate::print::text;
use crate::trap::Trap;
use crate::trap::TrapKind;
use crate::trap::hex;

/// The kind of a pointer difference.
const ISIZE: IntKind = IntKind::new(true, Width::Platform);

/// Limits for a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct RunOptions
{
    /// The most nested calls, the main function included.
    #[builder(default = 4096)]
    pub max_depth: u32,
    /// The most ops to run, or `None` for no limit.
    pub max_steps: Option<u64>,
    /// The most live heap octets; an allocation past it fails.
    #[builder(default = 256 << 20)]
    pub max_heap: u64,
    /// Whether the command line lists the heap blocks left unfreed when
    /// the run ends; the machine itself keeps no list.
    #[builder(default)]
    pub leaks: bool,
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
    run_and_keep(module, out, options).1
}

/// Run a module like [`run_with`], and keep the machine, so the caller can
/// look at what the run left behind, such as the heap blocks never freed.
///
/// # Arguments
///
/// * `module` - The module.
/// * `out` - Where print ops write.
/// * `options` - The limits.
///
/// # Returns
///
/// The machine where the run stopped, and `Ok` when the main function
/// returned or the trap that stopped it.
pub fn run_and_keep(
    module: &Module, out: &mut dyn Write, options: &RunOptions,
) -> (Machine, Result<(), Trap>)
{
    let mut machine = Machine::new(module.clone(), *options);
    let result = drive(&mut machine, out);
    (machine, result)
}

/// Run a machine to its end, writing its output as it goes.
///
/// # Arguments
///
/// * `machine` - The machine.
/// * `out` - Where print ops write.
///
/// # Returns
///
/// `Ok` when the main function returns, or the trap that stopped it.
fn drive(machine: &mut Machine, out: &mut dyn Write) -> Result<(), Trap>
{
    while machine.running()
    {
        machine.step();
        drain(machine, out)?;
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
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
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
    /// The start of each cell the call made, freed when it returns.
    #[builder(default)]
    pub cells: Vec<u64>,
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
    /// The heap, the cells and the static data.
    mem: Mem,
    /// The index of the function that made each cell, at the cell start.
    /// It outlives the cell, so a trap can still name the function.
    #[builder(default)]
    cells: BTreeMap<u64, u32>,
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
        let mem = Mem::new(module.platform_bits, options.max_heap);
        #[rustfmt::skip]
        let mut machine = Machine::builder().module(module).options(options)
            .globals(globals).at((main, 0)).mem(mem).build();
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

    /// The heap, the cells and the static data.
    ///
    /// # Returns
    ///
    /// The address space.
    pub fn mem(&self) -> &Mem { &self.mem }

    /// The heap blocks never freed.
    ///
    /// # Returns
    ///
    /// The start and length of each live heap block, in address order.
    pub fn leaks(&self) -> Vec<(u64, u64)> { self.mem.leaks() }

    /// The module the machine runs.
    ///
    /// # Returns
    ///
    /// The module.
    pub fn module(&self) -> &Module { &self.module }

    /// The limits of the run.
    ///
    /// # Returns
    ///
    /// The limits.
    pub fn options(&self) -> RunOptions { self.options }

    /// The name of the function that made a cell.
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the cell.
    ///
    /// # Returns
    ///
    /// The name, or `None` when no cell ever started there.
    pub fn cell_maker(&self, start: u64) -> Option<&str>
    {
        self.cells.get(&start).map(|f| self.func_name(*f))
    }

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
        let block = kind.addr().and_then(|a| self.block_text(a));
        #[rustfmt::skip]
        let trap = Trap::builder().kind(kind).func(self.func_name(func)).pc(pc)
            .maybe_block(block).build();
        trap
    }

    /// The name of a function.
    ///
    /// # Arguments
    ///
    /// * `func` - The function index.
    ///
    /// # Returns
    ///
    /// The name, or `?` for an index with no function.
    fn func_name(&self, func: u32) -> &str
    {
        self.module.funcs.get(func as usize).map_or("?", |f| f.name.as_str())
    }

    /// Describe the block an address falls in, or ends just before.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    ///
    /// # Returns
    ///
    /// The start, size and origin of the block, with the function that
    /// made a cell; `None` when the address is outside each block.
    fn block_text(&self, addr: u64) -> Option<String>
    {
        let block = self.mem.block_near(addr)?;
        let maker = || self.cell_maker(block.start).unwrap_or("?");
        let origin = match block.origin
        {
            Origin::Cell => t!("vm.trap.cell_of", func = maker()).into_owned(),
            Origin::Heap | Origin::Static => <&str>::from(block.origin).to_string(),
        };
        let (start, size) = (hex(block.start), block.len);
        Some(
            t!("vm.trap.block", start = start, size = size, origin = origin)
                .into_owned(),
        )
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
            Op::Load(kind) => self.load(kind),
            Op::Store(_) => self.store(),
            Op::Cell(size, _) => self.cell(size),
            Op::PtrAdd(stride) => self.ptr_add(stride),
            Op::PtrSub(stride) => self.ptr_sub(stride),
            Op::Alloc(stride) => self.alloc(stride),
            Op::Realloc(stride) => self.realloc(stride),
            Op::Free => self.free(),
            Op::Dup => self.dup(),
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

    /// Pop an address.
    ///
    /// # Arguments
    ///
    /// * `op` - The name of the op, for the trap text.
    ///
    /// # Returns
    ///
    /// The address, or a trap for another value.
    fn pop_ptr(&mut self, op: &str) -> Result<u64, TrapKind>
    {
        let v = self.pop()?;
        v.as_ptr().ok_or_else(|| malformed(&format!("{op} needs an address")))
    }

    /// Pop an integer, whatever its kind.
    ///
    /// # Arguments
    ///
    /// * `op` - The name of the op, for the trap text.
    ///
    /// # Returns
    ///
    /// The bit pattern, sign-extended for a signed kind, or a trap for
    /// another value.
    fn pop_int(&mut self, op: &str) -> Result<u128, TrapKind>
    {
        match self.pop()?
        {
            Value::Int { raw, .. } => Ok(raw),
            _ => Err(malformed(&format!("{op} needs an integer"))),
        }
    }

    /// Cut an address to the platform width.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    ///
    /// # Returns
    ///
    /// The address with the bits past the width dropped.
    fn wrap_addr(&self, addr: u64) -> u64
    {
        let bits = self.module.platform_bits;
        tern!(bits >= 64, addr, addr & ((1u64 << bits) - 1))
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

    /// End the innermost call and free its cells; a result remains on the
    /// stack.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap when there is no call.
    fn leave(&mut self) -> Step
    {
        let frame = self.frames.pop().ok_or_else(|| malformed("no call"))?;
        self.mem.release(&frame.cells);
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

    /// Pop an address and push the value of a kind there.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind to read.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap the access hits.
    fn load(&mut self, kind: DataKind) -> Step
    {
        let addr = self.pop_ptr("load")?;
        let v = self.mem.load(addr, kind)?;
        self.push(v)
    }

    /// Pop a value, then an address, and write the value there.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap the access hits.
    fn store(&mut self) -> Step
    {
        let v = self.pop()?;
        let addr = self.pop_ptr("store")?;
        self.mem.store(addr, v)
    }

    /// Make a cell for an addressed local of the innermost call and push
    /// its address. Blocks start at a multiple of 16, which covers each
    /// alignment.
    ///
    /// # Arguments
    ///
    /// * `size` - The octet count.
    ///
    /// # Returns
    ///
    /// `Ok`, or `OutOfMem` when the address space is full.
    fn cell(&mut self, size: u32) -> Step
    {
        let size = u64::from(size);
        let made = self.mem.alloc(size, Origin::Cell, self.frames.len());
        let addr = made.ok_or(TrapKind::OutOfMem { size })?;
        let frame = self.frames.last_mut().ok_or_else(|| malformed("no call"))?;
        frame.cells.push(addr);
        self.cells.insert(addr, self.at.0);
        self.push(Value::Ptr(addr))
    }

    /// Pop an element offset and an address, and push the address moved
    /// `offset` elements, wrapped at the platform width.
    ///
    /// # Arguments
    ///
    /// * `stride` - The element size.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn ptr_add(&mut self, stride: u32) -> Step
    {
        let offset = self.pop_int("ptr_add")? as i128;
        let addr = self.pop_ptr("ptr_add")?;
        let moved = offset.wrapping_mul(i128::from(stride)) as u64;
        self.push(Value::Ptr(self.wrap_addr(addr.wrapping_add(moved))))
    }

    /// Pop two addresses in one block and push their distance in elements,
    /// an `isize`.
    ///
    /// # Arguments
    ///
    /// * `stride` - The element size.
    ///
    /// # Returns
    ///
    /// `Ok`, or `MixedBlocks` when no live block reaches both.
    fn ptr_sub(&mut self, stride: u32) -> Step
    {
        ret_if!(stride == 0, Err(malformed("ptr_sub needs a stride")));
        let r = self.pop_ptr("ptr_sub")?;
        let l = self.pop_ptr("ptr_sub")?;
        ret_if!(!self.mem.same_block(l, r), Err(TrapKind::MixedBlocks));
        let distance = (i128::from(l) - i128::from(r)) / i128::from(stride);
        self.push(Value::int(ISIZE, distance as u128, self.module.platform_bits))
    }

    /// Pop an element count and push the start of a new heap block that
    /// holds `count` elements, or null when the heap has no room or the
    /// octet count overflows.
    ///
    /// # Arguments
    ///
    /// * `stride` - The element size.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn alloc(&mut self, stride: u32) -> Step
    {
        let count = self.pop_int("alloc")?;
        let size = count.checked_mul(u128::from(stride));
        let size = size.and_then(|s| u64::try_from(s).ok());
        let addr = size.and_then(|s| self.mem.alloc(s, Origin::Heap, 0));
        self.push(addr.map_or(Value::NULL, Value::Ptr))
    }

    /// Pop an element count and the start of a heap block, and push the
    /// start of the block resized to `count` elements.
    ///
    /// # Arguments
    ///
    /// * `stride` - The element size.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap: `OutOfMem` when the heap has no room or the octet
    /// count overflows, `BadFree` or `DoubleFree` for an address that is
    /// not the start of a live heap block.
    fn realloc(&mut self, stride: u32) -> Step
    {
        let count = self.pop_int("realloc")?;
        let addr = self.pop_ptr("realloc")?;
        let wide = count.saturating_mul(u128::from(stride));
        let full = TrapKind::OutOfMem { size: u64::MAX };
        let size = u64::try_from(wide).map_err(|_| full)?;
        let moved = self.mem.realloc(addr, size)?;
        self.push(Value::Ptr(moved))
    }

    /// Pop the start of a heap block and free it; null does nothing.
    ///
    /// # Returns
    ///
    /// `Ok`, or `DoubleFree` or `BadFree`.
    fn free(&mut self) -> Step
    {
        let addr = self.pop_ptr("free")?;
        self.mem.free(addr)
    }

    /// Push the top value again.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap when the stack is blank.
    fn dup(&mut self) -> Step
    {
        let top = self.stack.last().copied();
        self.push(top.ok_or_else(|| malformed("dup of a blank stack"))?)
    }
}
