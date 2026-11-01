use std::collections::BTreeMap;
use std::io::Write;

use bon::Builder;
use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen::bc::FmtArg;
use zakac_codegen::bc::Module;
use zakac_codegen::bc::Op;
use zakac_codegen::bc::SliceSource;
use zakac_codegen::bc::Slot;
use zakac_codegen::bc::TrapCode;
use zakac_i18n::t;
use zakac_ir_builder::ir::CmpOp;
use zakac_ir_builder::ir::InfixOp;
use zakac_ir_builder::ir::PrefixOp;
use zakac_ir_builder::value::EvalError;
use zakac_ir_builder::value::Value;
use zakac_ir_builder::value::compare;
use zakac_ir_builder::value::convert;
use zakac_ir_builder::value::infix;
use zakac_ir_builder::value::infix_wrapping;
use zakac_ir_builder::value::prefix;
use zakac_ir_builder::value::prefix_wrapping;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::do_while;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::IntKind;
use zakac_semcheck::kind::Width;

use crate::mem::Mem;
use crate::mem::Origin;
use crate::print::list_text;
use crate::print::slice_parts;
use crate::print::text;
use crate::print::text_at;
use crate::trap::Trap;
use crate::trap::TrapKind;
use crate::trap::hex;

/// The kind of a pointer difference.
const ISIZE: IntKind = IntKind::new(true, Width::Platform);

/// How the variables of a run sit in the address space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum FrameMode
{
    /// Each variable in a block of its own with a red zone after it, so a
    /// read or write past a variable traps.
    #[default]
    Checked,
    /// The locals of a call next to each other in one block at C offsets,
    /// and the globals next to each other in one more, so a read or write past
    /// a variable reaches the next one, as in C.
    Packed,
}

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
    /// The most octets of the stack region; a call with no room for its
    /// locals traps.
    #[builder(default = 1 << 20)]
    pub max_stack: u64,
    /// How the variables sit in the address space.
    #[builder(default)]
    pub frames: FrameMode,
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
    /// The depth of the operand stack when the call began, after its
    /// arguments were taken; the values over it belong to the call.
    pub depth: usize,
    /// The address of each local of the call, parameters first.
    #[builder(default)]
    pub locals: Vec<u64>,
    /// The blocks the call took in the stack region, freed when it
    /// returns: one per local, or one for all of them when packed.
    #[builder(default)]
    pub blocks: Vec<u64>,
    /// The top of the stack region before the call took its blocks.
    #[builder(default)]
    pub sp: u64,
}

/// The variables a stack or static block holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct Owner
{
    /// The function of a local or a frame; `None` for globals.
    pub func: Option<u32>,
    /// The slot index in the locals of the function, or in the globals;
    /// `None` for a block that holds them all.
    pub slot: Option<u32>,
}

/// What a stack or static block holds, named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Held<'m>
{
    /// One local of a call.
    Local
    {
        /// The name of the local.
        name: &'m str,
        /// The name of the function.
        func: &'m str,
    },
    /// One global.
    Global
    {
        /// The name of the global.
        name: &'m str,
    },
    /// Each local of a call, packed.
    Frame
    {
        /// The name of the function.
        func: &'m str,
    },
    /// Each global, packed.
    Globals,
    /// A static text, with its index in the module.
    Text(u32),
}

/// A variable of a packed block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct PackedVar<'m>
{
    /// Its offset from the start of the block.
    pub offset: u64,
    /// Its octet count.
    pub len: u64,
    /// Its name.
    pub name: &'m str,
    /// Its kind.
    pub kind: DataKind,
}

/// A slot as a variable of a packed block.
///
/// # Arguments
///
/// * `s` - The slot.
/// * `offset` - Its offset in the block.
/// * `platform` - The platform bits.
///
/// # Returns
///
/// The variable.
fn packed_var(s: &Slot, offset: u64, platform: u16) -> PackedVar<'_>
{
    let len = u64::from(s.kind.size(platform));
    PackedVar::builder()
        .offset(offset)
        .len(len)
        .name(s.name.as_str())
        .kind(s.kind)
        .build()
}

/// Where each slot sits in a packed block, set out as C sets out the
/// fields of a struct, and the octet count of the block.
///
/// # Arguments
///
/// * `slots` - The slots.
/// * `platform` - The platform bits.
///
/// # Returns
///
/// The offset of each slot, and the size.
fn packed(slots: &[Slot], platform: u16) -> (Vec<u64>, u64)
{
    let mut end = 0u64;
    let mut place = |s: &Slot| {
        let at = end.next_multiple_of(u64::from(s.kind.align(platform).max(1)));
        end = at + u64::from(s.kind.size(platform));
        at
    };
    let offsets = slots.iter().map(&mut place).collect();
    (offsets, end)
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
    /// The static block of each global slot.
    #[builder(default)]
    globals: Vec<u64>,
    /// The frozen static block of each static text of the module.
    #[builder(default)]
    texts: Vec<u64>,
    /// The operand stack.
    #[builder(default)]
    stack: Vec<Value>,
    /// The locals of the calls in progress.
    #[builder(default)]
    slots: usize,
    /// The calls in progress, innermost last.
    #[builder(default)]
    frames: Vec<Frame>,
    /// The number of ops run.
    #[builder(default)]
    steps: u64,
    /// The most stack slots in use at once so far: the locals of the calls
    /// and the operand stack.
    #[builder(default)]
    peak_stack: usize,
    /// The function and op index of the op that ran last.
    at: (u32, u32),
    /// What print ops wrote and no one took.
    #[builder(default)]
    out: String,
    /// The static data, the stack region and the heap.
    mem: Mem,
    /// The variable of each stack and static block, at the block start.
    /// It outlives the block, so a trap can still name the variable.
    #[builder(default)]
    owners: BTreeMap<u64, Owner>,
    /// Where the run stands.
    #[builder(default = State::Running)]
    state: State,
}

/// The addresses of the locals of a new call and the blocks those take, or
/// the trap.
type Taken = Result<(Vec<u64>, Vec<u64>), TrapKind>;

/// What a block holds, as the text of a trap.
///
/// # Arguments
///
/// * `held` - What the block holds.
///
/// # Returns
///
/// The text.
fn held_text(held: Held) -> String
{
    let text = match held
    {
        Held::Local { name, func } => t!("vm.trap.stack_of", name = name, func = func),
        Held::Global { name } => t!("vm.trap.global", name = name),
        Held::Frame { func } => t!("vm.trap.frame_of", func = func),
        Held::Globals => t!("vm.trap.globals"),
        Held::Text(n) => t!("vm.trap.static_text", n = n),
    };
    text.into_owned()
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
/// The trap of an operator that failed: an overflow names the operator and
/// the kind; each other failure gives the trap of [`eval_trap`].
///
/// # Arguments
///
/// * `e` - The failure.
/// * `op` - The name of the operator.
/// * `operand` - An operand, which gives the kind.
///
/// # Returns
///
/// The trap.
fn overflow_trap(e: EvalError, op: &'static str, operand: Value) -> TrapKind
{
    ret_if!(e != EvalError::Overflow, eval_trap(e));
    TrapKind::Overflow { op: op.to_string(), kind: operand.kind().name() }
}

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
        let main = module.main;
        let mem = Mem::new(module.platform_bits, options.max_heap);
        #[rustfmt::skip]
        let mut machine = Machine::builder().module(module).options(options)
            .at((main, 0)).mem(mem).build();
        let placed = machine.place_globals().and_then(|()| machine.place_texts());
        machine.mem.open_stack(options.max_stack);
        let entered = placed.and_then(|()| machine.enter(main));
        do_if!(let Err(kind) = entered, machine.state = State::Trapped(machine.trap(kind)));
        machine.raise_peak();
        machine
    }

    /// Give the globals their static room, before the stack region opens.
    ///
    /// # Returns
    ///
    /// `Ok`, or `StaticFull` when the address space has no room.
    fn place_globals(&mut self) -> Step
    {
        let platform = self.module.platform_bits;
        ret_if!(self.options.frames == FrameMode::Packed, self.pack_globals());
        let sizes: Vec<u64> = self
            .module
            .globals
            .iter()
            .map(|s| u64::from(s.kind.size(platform)))
            .collect();
        (0..).zip(sizes).try_for_each(|(n, size)| self.place_global(n, size))
    }

    /// Give each static text a frozen static block, before the stack region
    /// opens. Each literal of one text shares its block.
    ///
    /// # Returns
    ///
    /// `Ok`, or `StaticFull` when the address space has no room.
    fn place_texts(&mut self) -> Step
    {
        for text in &self.module.texts
        {
            let size = text.len() as u64;
            let made = self.mem.place_frozen(text.as_bytes());
            self.texts.push(made.ok_or(TrapKind::StaticFull { size })?);
        }
        Ok(())
    }

    /// Give one global a static block.
    ///
    /// # Arguments
    ///
    /// * `n` - The slot index.
    /// * `size` - The octet count.
    ///
    /// # Returns
    ///
    /// `Ok`, or `StaticFull` when the address space has no room.
    fn place_global(&mut self, n: u32, size: u64) -> Step
    {
        let made = self.mem.alloc(size, Origin::Static, 0);
        let addr = made.ok_or(TrapKind::StaticFull { size })?;
        self.owners.insert(addr, Owner::builder().slot(n).build());
        self.globals.push(addr);
        Ok(())
    }

    /// Give each global its place in one static block.
    ///
    /// # Returns
    ///
    /// `Ok`, or `StaticFull` when the address space has no room.
    fn pack_globals(&mut self) -> Step
    {
        let (offsets, size) = packed(&self.module.globals, self.module.platform_bits);
        ret_if!(offsets.is_empty(), Ok(()));
        let made = self.mem.alloc(size, Origin::Static, 0);
        let base = made.ok_or(TrapKind::StaticFull { size })?;
        self.owners.insert(base, Owner::builder().build());
        self.globals = offsets.iter().map(|o| base + o).collect();
        Ok(())
    }

    /// Raise the peak of the stack to the slots in use now.
    fn raise_peak(&mut self)
    {
        let slots = self.slots + self.stack.len();
        self.peak_stack = self.peak_stack.max(slots);
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
        self.raise_peak();
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

    /// The blocks of the locals of one call, parameters first.
    ///
    /// # Arguments
    ///
    /// * `frame` - The index in [`Machine::frames`].
    ///
    /// # Returns
    ///
    /// The block starts; none for a frame that is not there.
    pub fn locals(&self, frame: usize) -> &[u64]
    {
        self.frames.get(frame).map_or(&[], |f| &f.locals)
    }

    /// The blocks of the global slots.
    ///
    /// # Returns
    ///
    /// The block starts.
    pub fn globals(&self) -> &[u64] { &self.globals }

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

    /// The most stack slots in use at once so far: the locals of the calls
    /// and the operand stack; returns leave it.
    ///
    /// # Returns
    ///
    /// The slots.
    pub fn peak_stack(&self) -> usize { self.peak_stack }

    /// The static data, the stack region and the heap.
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

    /// What a stack or static block holds, or held last.
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the block.
    ///
    /// # Returns
    ///
    /// The variables, named; `None` when no variable ever started there.
    pub fn owner(&self, start: u64) -> Option<Held<'_>>
    {
        let text = self.texts.iter().position(|a| *a == start);
        ret_if!(let Some(n) = text, Some(Held::Text(n as u32)));
        let owner = self.owners.get(&start)?;
        let func = owner.func.and_then(|f| self.module.funcs.get(f as usize));
        let slots = func.map_or(&self.module.globals, |f| &f.locals);
        let named = |n: u32| slots.get(n as usize).map_or("?", |s| s.name.as_str());
        let name = owner.slot.map(named);
        let held = match (func, name)
        {
            (Some(f), Some(name)) => Held::Local { name, func: &f.name },
            (None, Some(name)) => Held::Global { name },
            (Some(f), None) => Held::Frame { func: &f.name },
            (None, None) => Held::Globals,
        };
        Some(held)
    }

    /// The kind of the variable a block holds, or held last.
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the block.
    ///
    /// # Returns
    ///
    /// The kind; `None` for a heap block, or a packed block that holds
    /// several variables.
    pub fn held_kind(&self, start: u64) -> Option<DataKind>
    {
        let owner = self.owners.get(&start)?;
        let func = owner.func.and_then(|f| self.module.funcs.get(f as usize));
        let slots = func.map_or(&self.module.globals, |f| &f.locals);
        slots.get(owner.slot? as usize).map(|s| s.kind)
    }

    /// The variables a packed block holds, or held last, in offset order.
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the block.
    ///
    /// # Returns
    ///
    /// The variables; `None` for a block that holds one variable or none,
    /// such as each block of a checked run.
    pub fn packed_vars(&self, start: u64) -> Option<Vec<PackedVar<'_>>>
    {
        let owner = self.owners.get(&start).filter(|o| o.slot.is_none())?;
        let func = owner.func.and_then(|f| self.module.funcs.get(f as usize));
        let slots = func.map_or(&self.module.globals, |f| &f.locals);
        let platform = self.module.platform_bits;
        let (offsets, _) = packed(slots, platform);
        let vars = slots.iter().zip(offsets).map(|(s, o)| packed_var(s, o, platform));
        Some(vars.collect())
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
        let named = self.owner(block.start).map(held_text);
        let origin = named.unwrap_or_else(|| <&str>::from(block.origin).to_string());
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
            Op::LocalAddr(n) => self.local_addr(n),
            Op::ResultAddr => self.result_addr(),
            Op::GlobalAddr(n) => self.global_addr(n),
            Op::Prefix(op) => self.prefix(op),
            Op::Infix(op) => self.infix(op),
            Op::Wrap(op) =>
            {
                let (r, l) = (self.pop()?, self.pop()?);
                let platform = self.module.platform_bits;
                self.push(infix_wrapping(op, l, r, platform).map_err(eval_trap)?)
            }
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
            Op::Take(kind) => self.take(kind),
            Op::PtrAdd(stride) => self.ptr_add(stride),
            Op::PtrSub(stride) => self.ptr_sub(stride),
            Op::Alloc(stride) => self.alloc(stride),
            Op::Realloc(stride) => self.realloc(stride),
            Op::Free => self.free(),
            Op::Dup => self.dup(),
            Op::Index { len, first } => self.index(len, first),
            Op::Blit(kind) => self.blit(kind),
            Op::Zero(kind) =>
            {
                let addr = self.pop_ptr("zero")?;
                self.mem.zero(addr, kind)
            }
            Op::ListEq(kind) => self.list_eq(kind),
            Op::PrintList(kind) =>
            {
                let addr = self.pop_ptr("print_list")?;
                let line = list_text(&self.mem, addr, kind)?;
                write_line(&mut self.out, &line);
                Ok(())
            }
            Op::MakeSlice { stride, from } => self.make_slice(stride, from),
            Op::SliceCell { stride, first } => self.slice_cell(stride, first),
            Op::Text(t) => self.static_text(t),
            Op::Format(f) => self.format(f),
            Op::Drop(kind) =>
            {
                let addr = self.pop_ptr("drop")?;
                self.drop_value(addr, kind)
            }
            Op::CharCount => self.char_count(),
            Op::CharAt { first } => self.char_at(first),
            Op::TextEdges => self.text_edges(),
            Op::CheckUtf8 => self.check_utf8(),
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

    /// Push the address of the block of a local of the innermost call.
    ///
    /// # Arguments
    ///
    /// * `n` - The slot index.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap when the slot is out of range.
    fn local_addr(&mut self, n: u32) -> Step
    {
        let frame = self.frames.last().ok_or_else(|| malformed("no call"))?;
        let addr = frame.locals.get(n as usize).copied();
        let addr = addr.ok_or_else(|| malformed("local slot out of range"))?;
        self.push(Value::Ptr(addr))
    }

    /// Push the address of the caller's slot for the list or struct result
    /// of the running call: the value just under its arguments.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap when the call has no such slot.
    fn result_addr(&mut self) -> Step
    {
        let frame = self.frames.last().ok_or_else(|| malformed("no call"))?;
        let below = frame.depth.checked_sub(1).and_then(|at| self.stack.get(at));
        let slot = below.and_then(|v| v.as_ptr());
        let slot = slot.ok_or_else(|| malformed("no slot for the result"))?;
        self.push(Value::Ptr(slot))
    }

    /// Push the address of the block of a global.
    ///
    /// # Arguments
    ///
    /// * `n` - The slot index.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap when the slot is out of range.
    fn global_addr(&mut self, n: u32) -> Step
    {
        let addr = self.globals.get(n as usize).copied();
        let addr = addr.ok_or_else(|| malformed("global slot out of range"))?;
        self.push(Value::Ptr(addr))
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
        let stop = self.module.overflow == OverflowMode::Stop;
        let found =
            tern!(stop, prefix(op, v, platform), prefix_wrapping(op, v, platform));
        self.push(found.map_err(|e| overflow_trap(e, op.into(), v))?)
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
        let stop = self.module.overflow == OverflowMode::Stop;
        let found =
            tern!(stop, infix(op, l, r, platform), infix_wrapping(op, l, r, platform));
        self.push(found.map_err(|e| overflow_trap(e, op.into(), l))?)
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

    /// Start a call: each local takes a block of zeros in the stack region,
    /// then the arguments move from the stack into the parameters.
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
        let platform = self.module.platform_bits;
        let packed = tern!(
            self.options.frames == FrameMode::Packed,
            Some(packed(&func.locals, platform)),
            None
        );
        let sizes: Vec<u64> =
            func.locals.iter().map(|s| u64::from(s.kind.size(platform))).collect();
        let kinds: Vec<DataKind> = func.locals.iter().map(|s| s.kind).collect();
        let sp = self.mem.stack_top();
        let (locals, blocks) = match packed
        {
            Some((offsets, size)) => self.take_frame(f, &offsets, size)?,
            None => self.take_blocks(f, &sizes)?,
        };
        let args = self.stack.split_off(self.stack.len() - params);
        let mut passed = locals.iter().zip(args).zip(kinds);
        passed.try_for_each(|((a, v), kind)| self.pass(*a, v, kind))?;
        let depth = self.stack.len();
        self.slots += locals.len();
        #[rustfmt::skip]
        self.frames.push(Frame::builder().func(f).pc(0).depth(depth).locals(locals)
            .blocks(blocks).sp(sp).build());
        Ok(())
    }

    /// Take one block of the stack region for each local of a new call,
    /// packed at their offsets.
    ///
    /// # Arguments
    ///
    /// * `f` - The function index.
    /// * `offsets` - The offset of each local.
    /// * `size` - The octet count of the block.
    ///
    /// # Returns
    ///
    /// The address of each local and the block, or `StackFull`.
    fn take_frame(&mut self, f: u32, offsets: &[u64], size: u64) -> Taken
    {
        ret_if!(offsets.is_empty(), Ok((Vec::new(), Vec::new())));
        let made = self.mem.push_stack(size, self.frames.len() + 1);
        let base = made.ok_or(TrapKind::StackFull { size })?;
        self.owners.insert(base, Owner::builder().func(f).build());
        Ok((offsets.iter().map(|o| base + o).collect(), vec![base]))
    }

    /// Take a block of the stack region for each local of a new call. When
    /// one does not fit, the blocks taken so far go back.
    ///
    /// # Arguments
    ///
    /// * `f` - The function index.
    /// * `sizes` - The octet count of each local.
    ///
    /// # Returns
    ///
    /// The address of each local and the blocks, the same starts, or
    /// `StackFull`.
    fn take_blocks(&mut self, f: u32, sizes: &[u64]) -> Taken
    {
        let sp = self.mem.stack_top();
        let owner = self.frames.len() + 1;
        let mut made = Vec::with_capacity(sizes.len());
        for size in sizes
        {
            let Some(addr) = self.mem.push_stack(*size, owner)
            else
            {
                self.mem.pop_stack(&made, sp);
                return Err(TrapKind::StackFull { size: *size });
            };
            made.push(addr);
        }
        let owner = |slot| Owner::builder().func(f).slot(slot).build();
        do_for!((slot, a) in (0..).zip(&made), _ = self.owners.insert(*a, owner(slot)));
        Ok((made.clone(), made))
    }

    /// End the innermost call and give back its stack blocks; a result
    /// remains on the stack. A list or struct result is first copied to the
    /// caller's slot under it, unless it is built there, and the
    /// address of the slot remains.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap when there is no call.
    fn leave(&mut self) -> Step
    {
        let frame = self.frames.pop().ok_or_else(|| malformed("no call"))?;
        let ret = self.module.funcs.get(frame.func as usize).and_then(|f| f.ret);
        do_if!(let Some(kind) = ret.filter(|k| k.is_aggregate()), self.dup_result(kind)?);
        self.mem.pop_stack(&frame.blocks, frame.sp);
        self.slots -= frame.locals.len();
        Ok(())
    }

    /// Duplicate a list or struct result into the caller's slot, the address
    /// under it on the stack.
    ///
    /// # Arguments
    ///
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap either access hits.
    fn dup_result(&mut self, kind: DataKind) -> Step
    {
        let from = self.pop_ptr("ret")?;
        let to = self.stack.last().and_then(|v| v.as_ptr());
        let to = to.ok_or_else(|| malformed("ret needs the slot for the result"))?;
        ret_if!(from == to, Ok(()));
        self.mem.blit(to, from, kind)
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

    /// Pop an address, push the value of a kind at it and leave the default
    /// of the kind there, `ဗလာ` for an owner.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind; never a list or a struct, which moves through a
    ///   blit and a zero.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap the access hits.
    fn take(&mut self, kind: DataKind) -> Step
    {
        debug_assert!(
            !kind.is_aggregate(),
            "a list or struct moves through blit and zero"
        );
        let addr = self.pop_ptr("take")?;
        let v = self.mem.load(addr, kind)?;
        self.mem.store(addr, Value::default_of(kind))?;
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

    /// Write an argument into the slot of its parameter. A list or struct
    /// argument is the address of the value, and its octets are copied.
    ///
    /// # Arguments
    ///
    /// * `to` - The address of the parameter slot.
    /// * `v` - The argument.
    /// * `kind` - The kind of the parameter.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap the write hits.
    fn pass(&mut self, to: u64, v: Value, kind: DataKind) -> Step
    {
        ret_if!(!kind.is_aggregate(), self.mem.store(to, v));
        let from = v
            .as_ptr()
            .ok_or_else(|| malformed("a list or struct argument is an address"))?;
        self.mem.blit(to, from, kind)
    }

    /// Pop a cell number and push it counted from 0, or stop the program
    /// when it is outside the list.
    ///
    /// # Arguments
    ///
    /// * `len` - The number of cells.
    /// * `first` - The number of the first cell: 0 or 1.
    ///
    /// # Returns
    ///
    /// `Ok`, or `IndexOutOfRange`.
    fn index(&mut self, len: u32, first: u8) -> Step
    {
        let cell = self.pop_cell(u64::from(len), first)?;
        let usize = IntKind::new(false, Width::Platform);
        let platform = self.module.platform_bits;
        self.push(Value::int(usize, u128::from(cell), platform))
    }

    /// Pop a cell number and check it against a count of cells.
    ///
    /// # Arguments
    ///
    /// * `len` - The number of cells.
    /// * `first` - The number of the first cell: 0 or 1.
    ///
    /// # Returns
    ///
    /// The cell counted from 0, or `IndexOutOfRange`.
    fn pop_cell(&mut self, len: u64, first: u8) -> Result<u64, TrapKind>
    {
        let Value::Int { kind, raw } = self.pop()?
        else
        {
            return Err(malformed("index needs an integer"));
        };
        let wide = i128::try_from(raw).unwrap_or(i128::MAX);
        let index = tern!(kind.signed, raw as i128, wide);
        let lo = i128::from(first);
        let outside = index < lo || index >= lo + i128::from(len);
        ret_if!(outside, Err(TrapKind::IndexOutOfRange { index, first, len }));
        Ok((index - lo) as u64)
    }

    /// Pop a length, a start, a source and the address of a slice slot, and
    /// make the slice of the cells from `start` on. A range past the end of
    /// a list or a slice traps; a range in a heap block must keep to the
    /// block, as a pointer step must.
    ///
    /// # Arguments
    ///
    /// * `stride` - The size of a cell.
    /// * `from` - What the source is.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn make_slice(&mut self, stride: u32, from: SliceSource) -> Step
    {
        let len = self.pop_int("make_slice")?;
        let start = self.pop_int("make_slice")?;
        let source = self.pop_ptr("make_slice")?;
        let slot = self.pop_ptr("make_slice")?;
        let (len, start) = (
            u64::try_from(len).unwrap_or(u64::MAX),
            u64::try_from(start).unwrap_or(u64::MAX),
        );
        let (base, limit) = match from
        {
            SliceSource::Slice =>
            {
                slice_parts(&self.mem, source).map(|(b, c)| (b, Some(c)))?
            }
            SliceSource::List(n) => (source, Some(u64::from(n))),
            SliceSource::Checked | SliceSource::Block => (source, None),
        };
        let past = |limit| TrapKind::SliceOutOfRange { start, len, limit };
        let end = start.checked_add(len);
        do_if!(let Some(limit) = limit, ret_if!(end.is_none_or(|e| e > limit), Err(past(limit))));
        let stride = u64::from(stride);
        let first = self.wrap_addr(base.wrapping_add(start.wrapping_mul(stride)));
        let last = self.wrap_addr(first.wrapping_add(len.wrapping_mul(stride)));
        let block = from == SliceSource::Block;
        ret_if!(
            block && base == 0 && end != Some(0),
            Err(TrapKind::NullDeref { addr: base })
        );
        let inside =
            self.mem.steps_within(base, first) && self.mem.steps_within(base, last);
        ret_if!(block && !inside, Err(TrapKind::OutOfBounds { addr: last, size: 0 }));
        let word = u64::from(self.module.platform_bits / 8);
        let usize = IntKind::new(false, Width::Platform);
        self.mem.store(slot, Value::Ptr(first))?;
        let count = Value::int(usize, u128::from(len), self.module.platform_bits);
        self.mem.store(slot + word, count)?;
        self.push(Value::Ptr(slot))
    }

    /// Pop the address of a text slot, write a static text there with room
    /// 0, and push the address.
    ///
    /// # Arguments
    ///
    /// * `t` - The index of the text.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn static_text(&mut self, t: u32) -> Step
    {
        let slot = self.pop_ptr("text")?;
        let at =
            self.texts.get(t as usize).copied().ok_or_else(|| malformed("no text"))?;
        let len = self.module.texts[t as usize].len() as u64;
        self.write_text(slot, at, len, 0)?;
        self.push(Value::Ptr(slot))
    }

    /// Pop the values of the pieces of a format, then the address of a text
    /// slot; write the text of each piece in a new heap block, keep it in
    /// the slot, and push the address.
    ///
    /// # Arguments
    ///
    /// * `f` - The index of the format.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap: `OutOfMem` when the heap has no room.
    fn format(&mut self, f: u32) -> Step
    {
        let args = self.module.formats.get(f as usize).cloned();
        let args = args.ok_or_else(|| malformed("no format"))?;
        let count = args.iter().filter(|a| matches!(a, FmtArg::Value(_))).count();
        let at = self.stack.len().checked_sub(count);
        let values =
            self.stack.split_off(at.ok_or_else(|| malformed("too few pieces"))?);
        let slot = self.pop_ptr("format")?;
        let mut values = values.into_iter();
        let mut out = String::new();
        for arg in args
        {
            let piece = match arg
            {
                FmtArg::Text(t) => self.module.texts.get(t as usize).cloned(),
                FmtArg::Value(kind) =>
                {
                    values.next().map(|v| self.piece_text(v, kind)).transpose()?
                }
            };
            out.push_str(&piece.ok_or_else(|| malformed("a piece is missing"))?);
        }
        let size = out.len() as u64;
        let block = self.mem.alloc(size, Origin::Heap, 0);
        let block = block.ok_or(TrapKind::OutOfMem { size })?;
        self.mem.fill_block(block, out.as_bytes())?;
        self.write_text(slot, block, size, size)?;
        self.push(Value::Ptr(slot))
    }

    /// The text a print writes for a piece of a format.
    ///
    /// # Arguments
    ///
    /// * `v` - The value, or the address of a list, a struct or text.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The text, or the trap a read hits.
    fn piece_text(&self, v: Value, kind: DataKind) -> Result<String, TrapKind>
    {
        ret_if!(!kind.is_aggregate(), Ok(text(v)));
        let addr = v.as_ptr().ok_or_else(|| malformed("a piece is no address"))?;
        list_text(&self.mem, addr, kind)
    }

    /// Pop the address of text and push the number of its characters.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap a read hits.
    fn char_count(&mut self) -> Step
    {
        let addr = self.pop_ptr("char_count")?;
        let count = text_at(&self.mem, addr)?.chars().count();
        let (usize, platform) =
            (IntKind::new(false, Width::Platform), self.module.platform_bits);
        self.push(Value::int(usize, count as u128, platform))
    }

    /// Pop a character number, then the address of text, and push the
    /// character at that number.
    ///
    /// # Arguments
    ///
    /// * `first` - The number of the first character: 0 or 1.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap for a number outside the text.
    fn char_at(&mut self, first: u8) -> Step
    {
        let index = self.pop()?;
        let addr = self.pop_ptr("char_at")?;
        let text = text_at(&self.mem, addr)?;
        self.push(index)?;
        let at =
            self.pop_cell(text.chars().count() as u64, first).map_err(|e| match e
            {
                TrapKind::IndexOutOfRange { index, first, len } =>
                {
                    TrapKind::CharOutOfRange { index, first, len }
                }
                other => other,
            })?;
        let found =
            text.chars().nth(at as usize).ok_or_else(|| malformed("no character"))?;
        self.push(Value::Char(found))
    }

    /// The view of text on the stack, which remains there: its first u8 and
    /// its count.
    ///
    /// # Arguments
    ///
    /// * `op` - The name of the op, for the trap text.
    ///
    /// # Returns
    ///
    /// The address of the first u8 and the count, or a trap.
    fn top_view(&self, op: &str) -> Result<(u64, u64), TrapKind>
    {
        let addr = self.stack.last().and_then(|v| v.as_ptr());
        let addr = addr.ok_or_else(|| malformed(&format!("{op} needs a view")))?;
        slice_parts(&self.mem, addr)
    }

    /// Stop the program when the view of text on the stack starts or ends
    /// inside a character: the u8 at an edge continues a character.
    ///
    /// # Returns
    ///
    /// `Ok`, or `TextCut`.
    fn text_edges(&mut self) -> Step
    {
        let (base, count) = self.top_view("text_edges")?;
        ret_if!(base == 0 && count == 0, Ok(()));
        let cut = [base, base + count]
            .into_iter()
            .find(|at| self.mem.octet_at(*at).is_some_and(|b| b & 0xC0 == 0x80));
        cut.map_or(Ok(()), |addr| Err(TrapKind::TextCut { addr }))
    }

    /// Stop the program when the u8s of the slice on the stack are not
    /// UTF-8.
    ///
    /// # Returns
    ///
    /// `Ok`, or `BadUtf8` at the first u8 that is not.
    fn check_utf8(&mut self) -> Step
    {
        let (base, count) = self.top_view("check_utf8")?;
        let raw = self.mem.octets(base, count)?;
        std::str::from_utf8(&raw)
            .map(drop)
            .map_err(|e| TrapKind::BadUtf8 { addr: base + e.valid_up_to() as u64 })
    }

    /// Write the three words of a text.
    ///
    /// # Arguments
    ///
    /// * `slot` - The address of the text.
    /// * `at` - The address of its u8s.
    /// * `len` - Their count.
    /// * `room` - The size of its heap block; 0 for static text.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap a write hits.
    fn write_text(&mut self, slot: u64, at: u64, len: u64, room: u64) -> Step
    {
        let platform = self.module.platform_bits;
        let word = u64::from(platform / 8);
        let usize = IntKind::new(false, Width::Platform);
        self.mem.store(slot, Value::Ptr(at))?;
        self.mem.store(slot + word, Value::int(usize, u128::from(len), platform))?;
        self.mem.store(slot + 2 * word, Value::int(usize, u128::from(room), platform))
    }

    /// Free each heap text in a value and leave each text blank: a text
    /// with room 0 sits in static storage or was moved out, so its u8s
    /// remain.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address of the value.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap a read or the free hits.
    fn drop_value(&mut self, addr: u64, kind: DataKind) -> Step
    {
        let platform = self.module.platform_bits;
        if let Some(s) = kind.as_struct()
        {
            let fields =
                s.fields().iter().enumerate().filter(|(_, (_, k))| k.holds_text());
            let fields: Vec<_> =
                fields.map(|(i, (_, k))| (s.offset(i, platform), *k)).collect();
            return fields
                .into_iter()
                .try_for_each(|(o, k)| self.drop_value(addr + u64::from(o), k));
        }
        if let Some(l) = kind.as_list().filter(|l| l.cell.holds_text())
        {
            let stride = u64::from(l.cell.size(platform));
            return (0..u64::from(l.len))
                .try_for_each(|i| self.drop_value(addr + i * stride, l.cell));
        }
        ret_if!(kind != DataKind::Text, Ok(()));
        let word = u64::from(platform / 8);
        let room = self.mem.load(addr + 2 * word, DataKind::USIZE)?;
        let held = matches!(room, Value::Int { raw, .. } if raw > 0);
        let at = self.mem.load(addr, DataKind::OPAQUE)?.as_ptr().unwrap_or(0);
        do_if!(held, self.mem.free(at)?);
        self.mem.zero(addr, kind)
    }

    /// Pop a cell number, then the address of a slice, and push the address
    /// of that cell; a number outside the slice traps.
    ///
    /// # Arguments
    ///
    /// * `stride` - The size of a cell.
    /// * `first` - The number of the first cell: 0 or 1.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn slice_cell(&mut self, stride: u32, first: u8) -> Step
    {
        let index = self.pop()?;
        let slice = self.pop_ptr("slice_cell")?;
        let (base, count) = slice_parts(&self.mem, slice)?;
        self.push(index)?;
        let cell = self.pop_cell(count, first)?;
        let to = base.wrapping_add(cell.wrapping_mul(u64::from(stride)));
        self.push(Value::Ptr(self.wrap_addr(to)))
    }

    /// Pop a source address and a destination address, and duplicate a value
    /// of a kind from one to the other.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the value, such as a list.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap either access hits.
    fn blit(&mut self, kind: DataKind) -> Step
    {
        let from = self.pop_ptr("blit")?;
        let to = self.pop_ptr("blit")?;
        self.mem.blit(to, from, kind)
    }

    /// Pop the addresses of two lists and push whether their cells are
    /// equal, one cell at a time.
    ///
    /// # Arguments
    ///
    /// * `kind` - The list kind.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap a read of a cell hits.
    fn list_eq(&mut self, kind: DataKind) -> Step
    {
        let r = self.pop_ptr("list_eq")?;
        let l = self.pop_ptr("list_eq")?;
        let same = self.cells_equal(l, r, kind)?;
        self.push(Value::Bool(same))
    }

    /// Whether the values of a kind at two addresses are equal; for a list,
    /// whether each cell equals the cell at the same place.
    ///
    /// # Arguments
    ///
    /// * `l` - The first address.
    /// * `r` - The second address.
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// Whether the two are equal, or the trap a read hits.
    fn cells_equal(&self, l: u64, r: u64, kind: DataKind) -> Result<bool, TrapKind>
    {
        let Some(list) = kind.as_list()
        else
        {
            let (a, b) = (self.mem.load(l, kind)?, self.mem.load(r, kind)?);
            return compare(CmpOp::Eq, a, b).map_err(eval_trap);
        };
        let stride = u64::from(list.cell.size(self.module.platform_bits));
        for at in (0..u64::from(list.len)).map(|i| i * stride)
        {
            ret_if!(!self.cells_equal(l + at, r + at, list.cell)?, Ok(false));
        }
        Ok(true)
    }

    /// Pop an element offset and an address, and push the address moved
    /// `offset` elements, wrapped at the platform width. A step off
    /// null traps as a null access: no offset turns null into the address
    /// of a block, whatever number it adds. A step from an address in a
    /// block must land in that block, one past its end included, or it
    /// traps as out of bounds with no octets, since the step accesses
    /// nothing: no offset leads from one block into another.
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
        ret_if!(addr == 0 && offset != 0, Err(TrapKind::NullDeref { addr }));
        let moved = offset.wrapping_mul(i128::from(stride)) as u64;
        let to = self.wrap_addr(addr.wrapping_add(moved));
        let outside = TrapKind::OutOfBounds { addr: to, size: 0 };
        ret_if!(!self.mem.steps_within(addr, to), Err(outside));
        self.push(Value::Ptr(to))
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
