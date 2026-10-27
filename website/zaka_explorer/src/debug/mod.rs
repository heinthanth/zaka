use std::borrow::Cow;
use std::collections::BTreeSet;

use bon::Builder;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoStaticStr;
use zaka_vm::machine::Machine;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::State;
use zaka_vm::mem::Mem;
use zaka_vm::print::text;
use zaka_vm::trap::hex;
use zakac_codegen::bc;
use zakac_i18n::t;
use zakac_ir_builder::value::Value;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::compile::RUN_STEPS;
use crate::compile::asm::code_starts;
use crate::compile::trap_text;

/// Where a run in the debugger stands; its static name is a CSS class
/// suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum RunStatus
{
    /// No op has run.
    Start,
    /// Ops run on a timer. The view shows it; a session never reports it.
    Running,
    /// Some ops have run and more remain.
    Paused,
    /// The run was stopped before its end.
    Stopped,
    /// The main function returned.
    Finished,
    /// The program stopped before its end.
    Trapped,
}

impl RunStatus
{
    /// Name in the status chip.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn label(self) -> Cow<'static, str>
    {
        match self
        {
            RunStatus::Start => t!("explorer.run.status_start"),
            RunStatus::Running => t!("explorer.run.status_running"),
            RunStatus::Paused => t!("explorer.run.status_paused"),
            RunStatus::Stopped => t!("explorer.run.status_stopped"),
            RunStatus::Finished => t!("explorer.run.status_finished"),
            RunStatus::Trapped => t!("explorer.run.status_trapped"),
        }
    }
}

/// How fast the debugger steps on its own, in ops a second; its static name
/// is the value of the speed picker.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    EnumIter,
    EnumString,
    IntoStaticStr
)]
pub enum Speed
{
    /// One op a second.
    #[strum(serialize = "1")]
    Ops1,
    /// Two ops a second.
    #[strum(serialize = "2")]
    Ops2,
    /// Five ops a second.
    #[default]
    #[strum(serialize = "5")]
    Ops5,
    /// Ten ops a second.
    #[strum(serialize = "10")]
    Ops10,
    /// 20 ops a second.
    #[strum(serialize = "20")]
    Ops20,
    /// 50 ops a second.
    #[strum(serialize = "50")]
    Ops50,
    /// 100 ops a second.
    #[strum(serialize = "100")]
    Ops100,
}

impl Speed
{
    /// The number of ops a second.
    ///
    /// # Returns
    ///
    /// The count.
    pub fn rate(self) -> u64
    {
        let name: &'static str = self.into();
        name.parse().unwrap_or(1)
    }

    /// Name in the speed picker, such as `5 ops/s`.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn label(self) -> Cow<'static, str>
    {
        let rate = self.rate();
        tern!(
            rate == 1,
            t!("explorer.run.speed_one"),
            t!("explorer.run.speed_rate", count = rate)
        )
    }

    /// The time between two ops.
    ///
    /// # Returns
    ///
    /// The time in ms.
    pub fn millis(self) -> u64 { 1000 / self.rate() }
}

/// A call in progress, as the call stack panel shows it.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct FrameRow
{
    /// The index in the frames of the machine; the main function is 0.
    pub index: usize,
    /// The function index.
    pub func: u32,
    /// The function name.
    #[builder(into)]
    pub name: String,
    /// The op the call stands at: the next op in the innermost call, the
    /// call op in the others.
    pub pc: u32,
    /// The address of that op in the code.
    pub addr: u32,
}

/// A slot and its value, as the locals and globals panels show it.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct ValueRow
{
    /// The slot index.
    pub index: u32,
    /// The slot name.
    #[builder(into)]
    pub name: String,
    /// The data kind.
    #[builder(into)]
    pub kind: String,
    /// The value as the program would print it.
    #[builder(into)]
    pub value: String,
}

/// The registers of the machine, as the register strip shows them.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Registers
{
    /// The program counter: the address of the next op in the code, where
    /// the functions sit one after another; `None` once the run is
    /// finished.
    pub pc: Option<u32>,
    /// The stack pointer: the address over the top of the stack, which is
    /// where the next value goes.
    pub sp: usize,
    /// The base pointer: the address where the frame of the innermost call
    /// starts; `None` with no call in progress.
    pub bp: Option<usize>,
}

/// The frame of a call, as the stack panel shows it. The machine keeps the
/// locals of the calls and the operand stack apart; the panel sets them out
/// as one stack, where each frame holds the locals of its call and then the
/// values the call pushed.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct FrameBlock
{
    /// The call.
    pub call: FrameRow,
    /// The address where the frame starts.
    pub start: usize,
    /// Its locals, parameters first.
    pub slots: Vec<ValueRow>,
    /// The values it pushed, bottom first.
    pub temps: Vec<ValueRow>,
}

/// A debugger session: a machine over the generated ops, the break points
/// and the call the panels show.
#[derive(Debug, Clone, Default, Builder)]
pub struct Session
{
    /// The machine, or `None` when code generation did not run.
    machine: Option<Machine>,
    /// Function and op index pairs where a run stops, set on the ops.
    #[builder(default)]
    breaks: BTreeSet<(u32, u32)>,
    /// Function and op index pairs where a run stops for the break points
    /// on source lines.
    #[builder(default)]
    line_stops: BTreeSet<(u32, u32)>,
    /// The index of the call the panels show.
    #[builder(default)]
    viewed: usize,
    /// How the last run ended, once it was stopped or finished; the
    /// machine is then back at the first op.
    ended: Option<RunStatus>,
    /// What the ended run wrote, kept until the next run starts.
    #[builder(default)]
    kept: String,
    /// The number of ops the ended run ran.
    #[builder(default)]
    kept_steps: u64,
}

/// The values of the operand stack that belong to a call: those pushed since
/// it began and before the next call began.
///
/// # Arguments
///
/// * `m` - The machine.
/// * `index` - The index of the call in the frames of the machine.
///
/// # Returns
///
/// The values, bottom first.
fn temps_of(m: &Machine, index: usize) -> &[Value]
{
    let frames = m.frames();
    let hi = frames.get(index + 1).map_or(m.stack().len(), |f| f.depth);
    let lo = frames[index].depth.min(hi);
    &m.stack()[lo..hi]
}

/// Where each frame starts on the stack, outermost first, and the address
/// over the top.
///
/// # Arguments
///
/// * `m` - The machine.
///
/// # Returns
///
/// The starts and the top.
fn frame_starts(m: &Machine) -> (Vec<usize>, usize)
{
    let size = |i: usize| m.locals(i).len() + temps_of(m, i).len();
    let mut top = 0;
    let starts = (0..m.frames().len())
        .map(|i| {
            let start = top;
            top += size(i);
            start
        })
        .collect();
    (starts, top)
}

/// Rows of values the stack holds over the locals of a call.
///
/// # Arguments
///
/// * `values` - The values, bottom first.
///
/// # Returns
///
/// The rows, without names.
fn temp_rows(values: &[Value]) -> Vec<ValueRow>
{
    let row = |(i, v): (usize, &Value)| {
        ValueRow::builder()
            .index(i as u32)
            .name("")
            .kind(v.kind().name())
            .value(text(*v))
            .build()
    };
    values.iter().enumerate().map(row).collect()
}

/// A machine at the start of the ops, with the op and heap budgets of the
/// explorer.
///
/// # Arguments
///
/// * `ops` - The ops.
///
/// # Returns
///
/// The machine.
fn machine_for(ops: &bc::Module) -> Machine
{
    let options = RunOptions::builder().max_steps(RUN_STEPS).max_heap(16 << 20).build();
    Machine::new(ops.clone(), options)
}

/// The text of a slot value. An addressed slot holds the address of its
/// cell, so it shows the value in the cell, then ` @` and the address; a
/// cell that no longer holds a value shows the address alone.
///
/// # Arguments
///
/// * `mem` - The address space of the machine.
/// * `slot` - The slot.
/// * `v` - Its value.
///
/// # Returns
///
/// The text.
fn slot_text(mem: &Mem, slot: &bc::Slot, v: Value) -> String
{
    let cell = v.as_ptr().filter(|_| slot.addressed);
    let Some(addr) = cell
    else
    {
        return text(v);
    };
    let held = mem.load(addr, slot.kind).ok();
    held.map_or_else(|| hex(addr), |held| format!("{} @{}", text(held), hex(addr)))
}

/// Rows of slots and their values.
///
/// # Arguments
///
/// * `mem` - The address space of the machine, which holds the values of
///   addressed slots.
/// * `slots` - The slots.
/// * `values` - The values, in slot order.
///
/// # Returns
///
/// The rows.
fn value_rows(mem: &Mem, slots: &[bc::Slot], values: &[Value]) -> Vec<ValueRow>
{
    let row = |(i, (s, v)): (usize, (&bc::Slot, &Value))| {
        ValueRow::builder()
            .index(i as u32)
            .name(s.name.as_str())
            .kind(s.kind.name())
            .value(slot_text(mem, s, *v))
            .build()
    };
    slots.iter().zip(values).enumerate().map(row).collect()
}

impl Session
{
    /// Start a session over the ops.
    ///
    /// # Arguments
    ///
    /// * `ops` - The ops, or `None` when code generation did not run.
    ///
    /// # Returns
    ///
    /// The session, at the first op.
    pub fn new(ops: Option<&bc::Module>) -> Self
    {
        Session::builder().maybe_machine(ops.map(machine_for)).build()
    }

    /// Whether there are ops to run.
    ///
    /// # Returns
    ///
    /// `true` when code generation ran.
    pub fn is_live(&self) -> bool { self.machine.is_some() }

    /// Go back to the first op; the break points remain.
    pub fn reset(&mut self)
    {
        self.machine = self.machine.as_ref().map(|m| machine_for(m.module()));
        self.viewed = 0;
        self.ended = None;
        self.kept.clear();
        self.kept_steps = 0;
    }

    /// End a run that has started and not ended, and go back to the first
    /// op. What the run wrote is kept until the next run starts.
    pub fn stop(&mut self)
    {
        ret_if!(self.status() != RunStatus::Paused);
        self.rewind(RunStatus::Stopped);
    }

    /// Go back to the first op after a run ended, and keep what it wrote
    /// and how far it got until the next run starts.
    ///
    /// # Arguments
    ///
    /// * `how` - How the run ended.
    fn rewind(&mut self, how: RunStatus)
    {
        let (kept, steps) = (self.output(), self.steps());
        self.reset();
        self.ended = Some(how);
        self.kept = kept;
        self.kept_steps = steps;
    }

    /// Run one op.
    pub fn step(&mut self) { self.drive(|m| _ = m.step()); }

    /// Run one op; a call runs to its return.
    pub fn step_over(&mut self) { self.drive(|m| _ = m.step_over()); }

    /// Run until the innermost call returns.
    pub fn step_out(&mut self) { self.drive(|m| _ = m.step_out()); }

    /// Run until a break point, the end, a trap or the op budget.
    pub fn resume(&mut self)
    {
        let stops: Vec<(u32, u32)> = self.all_breaks().collect();
        self.drive(|m| _ = m.run_until(&stops));
    }

    /// Run one op for the timed stepping.
    ///
    /// # Returns
    ///
    /// Whether the stepping goes on: the run has not ended and the next op
    /// is not a break point.
    pub fn tick(&mut self) -> bool { self.advance(true) }

    /// Run a batch of ops, so a long run can go on over a number of frames and
    /// stop between them.
    ///
    /// # Arguments
    ///
    /// * `count` - The most ops to run.
    /// * `breaks` - Whether a break point stops the run.
    ///
    /// # Returns
    ///
    /// Whether the run goes on: it has not ended and, with `breaks`, the
    /// next op is not a break point.
    pub fn run_for(&mut self, count: u32, breaks: bool) -> bool
    {
        (0..count).all(|_| self.advance(breaks))
    }

    /// Run one op.
    ///
    /// # Arguments
    ///
    /// * `breaks` - Whether a break point stops the run.
    ///
    /// # Returns
    ///
    /// Whether the run goes on.
    fn advance(&mut self, breaks: bool) -> bool
    {
        self.step();
        let at = self.position();
        let at_break = breaks && at.is_some_and(|p| self.is_break(p));
        self.status() == RunStatus::Paused && !at_break
    }

    /// Run a command on the machine, then show the innermost call.
    ///
    /// # Arguments
    ///
    /// * `f` - The command.
    fn drive(&mut self, f: impl FnOnce(&mut Machine))
    {
        self.ended = None;
        self.kept.clear();
        self.kept_steps = 0;
        let Some(m) = self.machine.as_mut()
        else
        {
            return;
        };
        f(m);
        self.viewed = m.frames().len().saturating_sub(1);
        let finished = m.state() == &State::Finished;
        do_if!(finished, self.rewind(RunStatus::Finished));
    }

    /// Set or clear a break point.
    ///
    /// # Arguments
    ///
    /// * `func` - The function index.
    /// * `pc` - The op index.
    pub fn toggle_break(&mut self, func: u32, pc: u32)
    {
        let at = (func, pc);
        tern!(self.breaks.remove(&at), (), _ = self.breaks.insert(at));
    }

    /// The op indices with a break point in one function.
    ///
    /// # Arguments
    ///
    /// * `func` - The function index.
    ///
    /// # Returns
    ///
    /// The op indices.
    pub fn breaks_in(&self, func: u32) -> Vec<u32>
    {
        let within = self.all_breaks().filter(|(f, _)| *f == func);
        within.map(|(_, pc)| pc).collect()
    }

    /// Replace the break points on ops.
    ///
    /// # Arguments
    ///
    /// * `stops` - Function and op index pairs.
    pub fn set_op_stops(&mut self, stops: impl IntoIterator<Item = (u32, u32)>)
    {
        self.breaks = stops.into_iter().collect();
    }

    /// Replace the stops of the break points on source lines.
    ///
    /// # Arguments
    ///
    /// * `stops` - Function and op index pairs.
    pub fn set_line_stops(&mut self, stops: impl IntoIterator<Item = (u32, u32)>)
    {
        self.line_stops = stops.into_iter().collect();
    }

    /// Whether a stop comes from a break point on a source line.
    ///
    /// # Arguments
    ///
    /// * `func` - The function index.
    /// * `pc` - The op index.
    ///
    /// # Returns
    ///
    /// `true` for a stop of a source line.
    pub fn from_line(&self, func: u32, pc: u32) -> bool
    {
        self.line_stops.contains(&(func, pc))
    }

    /// Whether a run stops at an op, for a break point on it or on its line.
    ///
    /// # Arguments
    ///
    /// * `at` - The function and op index.
    ///
    /// # Returns
    ///
    /// `true` at a stop.
    fn is_break(&self, at: (u32, u32)) -> bool
    {
        self.breaks.contains(&at) || self.line_stops.contains(&at)
    }

    /// All stops: the break points on ops and on source lines.
    ///
    /// # Returns
    ///
    /// Function and op index pairs, in order, each once.
    fn all_breaks(&self) -> impl Iterator<Item = (u32, u32)> + '_
    {
        self.breaks.union(&self.line_stops).copied()
    }

    /// Show the panels of another call.
    ///
    /// # Arguments
    ///
    /// * `index` - The index in the frames of the machine.
    pub fn view_frame(&mut self, index: usize)
    {
        let count = self.machine.as_ref().map_or(0, |m| m.frames().len());
        ret_if!(index >= count);
        self.viewed = index;
    }

    /// Where the run stands.
    ///
    /// # Returns
    ///
    /// The status.
    pub fn status(&self) -> RunStatus
    {
        let Some(m) = &self.machine
        else
        {
            return RunStatus::Start;
        };
        ret_if!(let Some(how) = self.ended, how);
        match m.state()
        {
            State::Finished => RunStatus::Finished,
            State::Trapped(_) => RunStatus::Trapped,
            State::Running =>
            {
                tern!(m.steps() == 0, RunStatus::Start, RunStatus::Paused)
            }
        }
    }

    /// Whether ops remain to run.
    ///
    /// # Returns
    ///
    /// `true` while the run has not ended.
    pub fn running(&self) -> bool
    {
        self.machine.as_ref().is_some_and(Machine::running)
    }

    /// The function and op index of the next op; after a trap, of the op
    /// that trapped.
    ///
    /// # Returns
    ///
    /// The place, or `None` once the run is finished.
    pub fn position(&self) -> Option<(u32, u32)> { self.machine.as_ref()?.position() }

    /// The function of the call the panels show; the main function once the
    /// run is finished.
    ///
    /// # Returns
    ///
    /// The function index, or `None` without ops.
    pub fn viewed_func(&self) -> Option<u32>
    {
        let m = self.machine.as_ref()?;
        let main = m.module().main;
        Some(m.frames().get(self.viewed).map_or(main, |f| f.func))
    }

    /// The op the call the panels show stands at.
    ///
    /// # Returns
    ///
    /// The op index, or `None` once the run is finished.
    pub fn marked_pc(&self) -> Option<u32>
    {
        self.frames().into_iter().find(|f| f.index == self.viewed).map(|f| f.pc)
    }

    /// The calls in progress, innermost first.
    ///
    /// # Returns
    ///
    /// The rows.
    pub fn frames(&self) -> Vec<FrameRow>
    {
        let Some(m) = &self.machine
        else
        {
            return Vec::new();
        };
        let inner = m.frames().len().saturating_sub(1);
        let at = m.position();
        let starts = code_starts(m.module());
        let row = |(i, f): (usize, &zaka_vm::machine::Frame)| {
            let pc =
                tern!(i == inner, at.map_or(f.pc, |p| p.1), f.pc.saturating_sub(1));
            let name = m.module().funcs[f.func as usize].name.as_str();
            let addr = starts[f.func as usize] + pc;
            FrameRow::builder()
                .index(i)
                .func(f.func)
                .name(name)
                .pc(pc)
                .addr(addr)
                .build()
        };
        let rows = m.frames().iter().enumerate().map(row);
        rows.rev().collect()
    }

    /// The index of the call the panels show.
    ///
    /// # Returns
    ///
    /// The index in the frames of the machine.
    pub fn viewed(&self) -> usize { self.viewed }

    /// The frames of the calls in progress, innermost first.
    ///
    /// # Returns
    ///
    /// The blocks.
    pub fn frame_blocks(&self) -> Vec<FrameBlock>
    {
        let Some(m) = &self.machine
        else
        {
            return Vec::new();
        };
        let (starts, _) = frame_starts(m);
        let block = |call: FrameRow| {
            let i = call.index;
            let slots = &m.module().funcs[call.func as usize].locals;
            FrameBlock::builder()
                .start(starts[i])
                .slots(value_rows(m.mem(), slots, m.locals(i)))
                .temps(temp_rows(temps_of(m, i)))
                .call(call)
                .build()
        };
        self.frames().into_iter().map(block).collect()
    }

    /// The registers of the machine, as addresses on the stack the frames
    /// make up.
    ///
    /// # Returns
    ///
    /// The registers; all zero or none without ops.
    pub fn registers(&self) -> Registers
    {
        let frames = self.machine.as_ref().map(frame_starts);
        let bp = frames.as_ref().and_then(|(starts, _)| starts.last().copied());
        let sp = frames.map_or(0, |(_, top)| top);
        let pc = self
            .position()
            .zip(self.machine.as_ref())
            .map(|((func, pc), m)| code_starts(m.module())[func as usize] + pc);
        Registers::builder().maybe_pc(pc).sp(sp).maybe_bp(bp).build()
    }

    /// The locals of the call the panels show.
    ///
    /// # Returns
    ///
    /// The rows, parameters first.
    pub fn locals(&self) -> Vec<ValueRow>
    {
        let Some(m) = &self.machine
        else
        {
            return Vec::new();
        };
        let Some(frame) = m.frames().get(self.viewed)
        else
        {
            return Vec::new();
        };
        let slots = &m.module().funcs[frame.func as usize].locals;
        value_rows(m.mem(), slots, m.locals(self.viewed))
    }

    /// The global slots.
    ///
    /// # Returns
    ///
    /// The rows.
    pub fn globals(&self) -> Vec<ValueRow>
    {
        let Some(m) = &self.machine
        else
        {
            return Vec::new();
        };
        value_rows(m.mem(), &m.module().globals, m.globals())
    }

    /// The operand stack, top first, each value with its kind.
    ///
    /// # Returns
    ///
    /// The values as text.
    pub fn stack(&self) -> Vec<String>
    {
        let values = self.machine.as_ref().map_or(&[][..], |m| m.stack());
        values.iter().rev().map(Value::to_string).collect()
    }

    /// What the program wrote so far.
    ///
    /// # Returns
    ///
    /// The text.
    pub fn output(&self) -> String
    {
        ret_if!(self.ended.is_some(), self.kept.clone());
        self.machine.as_ref().map(|m| m.output().to_string()).unwrap_or_default()
    }

    /// The text of the trap that stopped the run.
    ///
    /// # Returns
    ///
    /// The text, or `None` while the run has not trapped.
    pub fn trap(&self) -> Option<String>
    {
        self.machine.as_ref()?.state().trap().map(trap_text)
    }

    /// The number of ops run.
    ///
    /// # Returns
    ///
    /// The count.
    pub fn steps(&self) -> u64
    {
        ret_if!(self.ended.is_some(), self.kept_steps);
        self.machine.as_ref().map_or(0, Machine::steps)
    }
}
