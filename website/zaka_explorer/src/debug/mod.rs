use std::borrow::Cow;
use std::collections::BTreeSet;
use std::time::Duration;

use bon::Builder;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoStaticStr;
use web_time::Instant;
use zaka_vm::machine::FrameMode;
use zaka_vm::machine::Held;
use zaka_vm::machine::Machine;
use zaka_vm::machine::PackedVar;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::State;
use zaka_vm::mem::Block;
use zaka_vm::mem::Mem;
use zaka_vm::mem::Origin;
use zaka_vm::mem::RED_ZONE;
use zaka_vm::print::list_text;
use zaka_vm::print::text;
use zakac_codegen::bc;
use zakac_i18n::t;
use zakac_ir_builder::value::Value;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;

use crate::compile::RUN_STEPS;
use crate::compile::asm::code_starts;
use crate::compile::trap_text;

/// The cap on live heap octets of a session until one is set.
pub const MAX_HEAP: u64 = 16 << 20;

/// Octets in a KiB.
const KIB: u64 = 1 << 10;

/// Octets in a MiB.
const MIB: u64 = 1 << 20;

/// Octets in a GiB.
const GIB: u64 = 1 << 30;

/// The most octets of a block the heap panel keeps for its dump.
pub const DUMP_CAP: usize = 256;

/// The most blocks the heap panel lists; the rest are counted.
pub const MAX_BLOCKS: usize = 500;

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
    /// The address the value leads to: a pointer that is not null.
    pub points_at: Option<u64>,
    /// The address of the block that holds a variable; `None` for a value
    /// on the operand stack.
    pub home: Option<u64>,
}

/// Whether a block is in use, as the heap panel shows it; its static name
/// is a CSS class suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum BlockState
{
    /// In use.
    Live,
    /// Freed and in quarantine.
    Freed,
    /// A heap block still in use when the run finished.
    Leaked,
}

impl BlockState
{
    /// Name in the badge of the block.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn label(self) -> Cow<'static, str>
    {
        match self
        {
            BlockState::Live => t!("explorer.heap.live"),
            BlockState::Freed => t!("explorer.heap.freed"),
            BlockState::Leaked => t!("explorer.heap.leaked"),
        }
    }
}

/// A block of the address space, as the heap panel shows it.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct BlockRow
{
    /// The address of the first octet.
    pub start: u64,
    /// The octet count.
    pub len: u64,
    /// Where the block came from.
    pub origin: Origin,
    /// Whether the block is in use.
    pub state: BlockState,
    /// What a stack or static block holds, as text, such as `က of ပြ`;
    /// `None` for a heap block.
    pub owner: Option<String>,
    /// The first octets of the contents, at most [`DUMP_CAP`]; none once
    /// the block is freed.
    #[builder(default)]
    pub data: Vec<u8>,
    /// The variables of a packed block, in offset order; none for a block
    /// that holds one variable or none.
    #[builder(default)]
    pub vars: Vec<VarRow>,
    /// The cell sizes of the list the block holds, outermost first, such as
    /// `[24, 8]` for a `[2][3]isize`; none for a block that holds no list.
    #[builder(default)]
    pub cells: Vec<u64>,
}

/// A variable of a packed block, as the map shows it.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct VarRow
{
    /// Its offset from the start of the block.
    pub offset: u64,
    /// Its octet count.
    pub len: u64,
    /// Its name.
    #[builder(into)]
    pub name: String,
    /// What it is, as text, such as `က of ပြ` or `Global x`.
    #[builder(into)]
    pub owner: String,
    /// The cell sizes of the list it is, outermost first; none for a
    /// variable that is no list.
    #[builder(default)]
    pub cells: Vec<u64>,
}

impl BlockRow
{
    /// Whether an address lies inside the block or just past its end.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    ///
    /// # Returns
    ///
    /// `true` when `start <= addr <= start + len`.
    pub fn reaches(&self, addr: u64) -> bool
    {
        addr >= self.start && addr <= self.start.saturating_add(self.len)
    }

    /// The rows the heap panel shows for the block: one for each variable
    /// of a packed block, at its own address, as a checked run shows them;
    /// the block itself otherwise.
    ///
    /// # Returns
    ///
    /// The rows, in address order.
    pub fn rows(&self) -> Vec<BlockRow>
    {
        ret_if!(self.vars.is_empty(), vec![self.clone()]);
        self.vars.iter().map(|v| self.var_row(v)).collect()
    }

    /// The row of a variable of a packed block, with its own octets.
    ///
    /// # Arguments
    ///
    /// * `v` - The variable.
    ///
    /// # Returns
    ///
    /// The row.
    fn var_row(&self, v: &VarRow) -> BlockRow
    {
        let at = v.offset as usize;
        let data = self.data.get(at..at + v.len as usize).unwrap_or_default();
        BlockRow::builder()
            .start(self.start + v.offset)
            .len(v.len)
            .origin(self.origin)
            .state(self.state)
            .owner(v.owner.clone())
            .data(data.to_vec())
            .cells(v.cells.clone())
            .build()
    }

    /// The cell an octet of the block sits in, as a path of cell numbers
    /// counted from 0, outermost first, such as `[1][2]`.
    ///
    /// # Arguments
    ///
    /// * `at` - The offset of the octet from the start of the block.
    ///
    /// # Returns
    ///
    /// The cell numbers; none for a block that holds no list.
    pub fn cell_path(&self, at: u64) -> Vec<u64>
    {
        let mut rest = at;
        let step = |size: &u64| {
            let n = rest / size;
            rest %= size;
            n
        };
        self.cells.iter().map(step).collect()
    }
}

/// The cell sizes of a list kind, outermost first.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The size of a cell at each list level; none for a kind that is no list.
pub fn cell_sizes(kind: DataKind, platform: u16) -> Vec<u64>
{
    let level = |k: &DataKind| k.as_list().map(|l| l.cell);
    let cells: Vec<DataKind> = std::iter::successors(level(&kind), level).collect();
    cells.iter().map(|c| u64::from(c.size(platform))).collect()
}

/// The blocks of the address space at one point of a run, with the counts
/// the heap panel shows over them.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct HeapShot
{
    /// The blocks in address order, at most [`MAX_BLOCKS`].
    #[builder(default)]
    pub blocks: Vec<BlockRow>,
    /// The blocks left out of the list.
    #[builder(default)]
    pub omitted: usize,
    /// The live heap octets.
    #[builder(default)]
    pub live_heap: u64,
    /// The cap on live heap octets.
    #[builder(default = MAX_HEAP)]
    pub cap: u64,
    /// The octets of the stack region the calls in progress take.
    #[builder(default)]
    pub stack_used: u64,
    /// The octets of the stack region.
    #[builder(default)]
    pub stack_cap: u64,
    /// The blocks in use.
    #[builder(default)]
    pub live: usize,
    /// The blocks in quarantine.
    #[builder(default)]
    pub freed: usize,
    /// The address an access or free of the run trapped at.
    pub fault: Option<u64>,
}

/// The width and height in px of the cell of a small block in the map.
pub const CELL: u64 = 12;

/// The most cells wide the cell of a large block grows.
pub const MAX_CELLS: u64 = 10;

/// The width in px of the cell of a block: one cell of [`CELL`] px for up to
/// 8 octets, and one more for each doubling past that, up to [`MAX_CELLS`]
/// cells, so a larger block stands out while a huge one does not crowd the
/// lane.
///
/// # Arguments
///
/// * `len` - The octet count.
///
/// # Returns
///
/// The width.
pub fn cell_width(len: u64) -> u64
{
    let doublings = u64::from((len.max(8) / 8).ilog2());
    CELL * (1 + doublings).min(MAX_CELLS)
}

/// A part of the heap map, in address order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapPart
{
    /// The block at this index of [`HeapShot::blocks`].
    Block(usize),
    /// The octets between two blocks, where each access traps: the red zone
    /// after a block, the room left to align the next one, and room no block
    /// uses.
    Gap
    {
        /// The first octet.
        start: u64,
        /// The octet count.
        len: u64,
    },
}

impl Default for HeapShot
{
    fn default() -> Self { HeapShot::builder().build() }
}

impl HeapShot
{
    /// The block an address leads into.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    ///
    /// # Returns
    ///
    /// The start of the block, or `None` when the address is outside each
    /// listed block.
    pub fn block_of(&self, addr: u64) -> Option<u64>
    {
        self.blocks.iter().rev().find(|b| b.reaches(addr)).map(|b| b.start)
    }

    /// The row an address leads into: a variable of a packed block, or a
    /// block.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address.
    ///
    /// # Returns
    ///
    /// The start of the row, or `None` when the address is outside each
    /// listed block.
    pub fn row_of(&self, addr: u64) -> Option<u64>
    {
        let start = self.block_of(addr)?;
        let block = self.blocks.iter().find(|b| b.start == start)?;
        let held = block.rows().into_iter().rev().find(|r| r.reaches(addr));
        Some(held.map_or(start, |r| r.start))
    }

    /// The parts of the map: each block, with the gap before the next one,
    /// so the static data, the stack and the heap follow one another as the
    /// machine places them in RAM. The last block has its red zone after it,
    /// grown to reach a trap past it.
    ///
    /// # Returns
    ///
    /// The parts, in address order.
    pub fn parts(&self) -> Vec<MapPart>
    {
        let tail = |end: u64| {
            let past = self.fault.filter(|f| *f >= end).map_or(0, |f| f - end + 1);
            end.saturating_add(RED_ZONE.max(past))
        };
        let gap = |i: usize| {
            let a = &self.blocks[i];
            let end = a.start.saturating_add(a.len);
            let next = self.blocks.get(i + 1).map_or_else(|| tail(end), |b| b.start);
            (next > end).then(|| MapPart::Gap { start: end, len: next - end })
        };
        let around = |i| std::iter::once(MapPart::Block(i)).chain(gap(i));
        (0..self.blocks.len()).flat_map(around).collect()
    }

    /// The hex digits of the highest address the panel shows: the end of
    /// the last block with its red zone, or the trap past it. Each address
    /// is padded to it, so the addresses line up.
    ///
    /// # Returns
    ///
    /// The digit count, at least 4.
    pub fn addr_digits(&self) -> usize
    {
        let end = |b: &BlockRow| b.start.saturating_add(b.len).saturating_add(RED_ZONE);
        let top = self.blocks.last().map_or(0, end).max(self.fault.unwrap_or(0));
        (top.max(1).ilog2() / 4 + 1).max(4) as usize
    }

    /// Whether the run trapped inside a range of octets.
    ///
    /// # Arguments
    ///
    /// * `start` - The first octet.
    /// * `len` - The octet count.
    ///
    /// # Returns
    ///
    /// `true` when the trap address lies in `[start, start + len)`.
    pub fn faults_in(&self, start: u64, len: u64) -> bool
    {
        self.fault.is_some_and(|f| f >= start && f < start.saturating_add(len))
    }
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
#[derive(Debug, Clone, Builder)]
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
    /// The blocks the ended run left.
    #[builder(default)]
    kept_heap: HeapShot,
    /// The cap on live heap octets of each run.
    #[builder(default = MAX_HEAP)]
    max_heap: u64,
    /// How the variables of each run sit in the address space.
    #[builder(default)]
    frames: FrameMode,
    /// How long the run has taken so far.
    #[builder(default)]
    spent: Duration,
    /// How long the ended run took.
    #[builder(default)]
    kept_spent: Duration,
    /// The peak heap octets and stack slots of the ended run.
    #[builder(default)]
    kept_usage: (u64, usize),
}

impl Default for Session
{
    fn default() -> Self { Session::builder().build() }
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
            .maybe_points_at(pointee(*v))
            .build()
    };
    values.iter().enumerate().map(row).collect()
}

/// A machine at the start of the ops, with the op budget of the explorer.
///
/// # Arguments
///
/// * `ops` - The ops.
/// * `max_heap` - The cap on live heap octets.
/// * `frames` - How the variables sit in the address space.
///
/// # Returns
///
/// The machine.
fn machine_for(ops: &bc::Module, max_heap: u64, frames: FrameMode) -> Machine
{
    #[rustfmt::skip]
    let options = RunOptions::builder().max_steps(RUN_STEPS).max_heap(max_heap)
        .frames(frames).build();
    Machine::new(ops.clone(), options)
}

/// An octet count with a unit.
///
/// # Arguments
///
/// * `n` - The count.
///
/// # Returns
///
/// The text, such as `48 B`, `1.5 KiB`, `16 MiB` or `4 GiB`.
pub fn octets(n: u64) -> String
{
    ret_if!(n >= GIB && n.is_multiple_of(GIB), format!("{} GiB", n / GIB));
    ret_if!(n >= MIB && n.is_multiple_of(MIB), format!("{} MiB", n / MIB));
    ret_if!(n >= MIB, format!("{:.1} MiB", n as f64 / MIB as f64));
    ret_if!(n >= KIB, format!("{:.1} KiB", n as f64 / KIB as f64));
    format!("{n} B")
}

/// The address a value leads to.
///
/// # Arguments
///
/// * `v` - The value.
///
/// # Returns
///
/// The address, or `None` for a value that is not a pointer or is null.
fn pointee(v: Value) -> Option<u64> { v.as_ptr().filter(|a| *a != 0) }

/// A row of the heap panel for a block.
///
/// # Arguments
///
/// * `m` - The machine.
/// * `b` - The block.
/// * `finished` - Whether the run finished, so a live heap block leaked.
///
/// # Returns
///
/// The row.
fn block_row(m: &Machine, b: &Block, finished: bool) -> BlockRow
{
    let leaked = finished && b.origin == Origin::Heap;
    let state = match (b.live, leaked)
    {
        (false, _) => BlockState::Freed,
        (true, true) => BlockState::Leaked,
        (true, false) => BlockState::Live,
    };
    let held = m.owner(b.start);
    let owner = held.map(held_text);
    let var_owner = |name: &str| match held
    {
        Some(Held::Frame { func }) =>
        {
            t!("explorer.heap.stack_of", name = name, func = func)
        }
        _ => t!("explorer.heap.global", name = name),
    };
    let platform = m.module().platform_bits;
    let var = |v: PackedVar| {
        let owner = var_owner(v.name).into_owned();
        let cells = cell_sizes(v.kind, platform);
        #[rustfmt::skip]
        return VarRow::builder().offset(v.offset).len(v.len).name(v.name).owner(owner)
            .cells(cells).build();
    };
    let vars =
        m.packed_vars(b.start).unwrap_or_default().into_iter().map(var).collect();
    let data = b.data.iter().take(DUMP_CAP).copied().collect();
    BlockRow::builder()
        .start(b.start)
        .len(b.len)
        .origin(b.origin)
        .state(state)
        .maybe_owner(owner)
        .vars(vars)
        .data(data)
        .cells(m.held_kind(b.start).map_or_else(Vec::new, |k| cell_sizes(k, platform)))
        .build()
}

/// What a block holds, as the heap panel names it.
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
        Held::Local { name, func } =>
        {
            t!("explorer.heap.stack_of", name = name, func = func)
        }
        Held::Global { name } => t!("explorer.heap.global", name = name),
        Held::Frame { func } => t!("explorer.heap.frame_of", func = func),
        Held::Globals => t!("explorer.heap.globals"),
        Held::Text(n) => t!("explorer.heap.static_text", n = n),
    };
    text.into_owned()
}

/// The blocks of a machine for the heap panel.
///
/// # Arguments
///
/// * `m` - The machine.
/// * `finished` - Whether the run finished, so the live heap blocks leaked.
///
/// # Returns
///
/// The blocks and their counts.
fn heap_shot(m: &Machine, finished: bool) -> HeapShot
{
    let mem = m.mem();
    let live = mem.blocks().filter(|b| b.live).count();
    let total = mem.blocks().count();
    let rows = mem.blocks().take(MAX_BLOCKS).map(|b| block_row(m, b, finished));
    let fault = m.state().trap().and_then(|t| t.kind.addr());
    HeapShot::builder()
        .maybe_fault(fault)
        .blocks(rows.collect())
        .omitted(total.saturating_sub(MAX_BLOCKS))
        .live_heap(mem.live_heap())
        .cap(m.options().max_heap)
        .stack_used(mem.stack_used())
        .stack_cap(mem.stack_cap())
        .live(live)
        .freed(total - live)
        .build()
}

/// Rows of slots and the values their blocks hold.
///
/// # Arguments
///
/// * `mem` - The address space of the machine, which holds the values.
/// * `slots` - The slots.
/// * `homes` - The block of each slot, in slot order.
///
/// # Returns
///
/// The rows; a value the block does not hold shows as `?`.
fn value_rows(mem: &Mem, slots: &[bc::Slot], homes: &[u64]) -> Vec<ValueRow>
{
    let row = |(i, (s, home)): (usize, (&bc::Slot, &u64))| {
        let held = mem.load(*home, s.kind).ok();
        let shown = list_text(mem, *home, s.kind).ok();
        ValueRow::builder()
            .index(i as u32)
            .name(s.name.as_str())
            .kind(s.kind.name())
            .value(shown.unwrap_or_else(|| "?".to_string()))
            .maybe_points_at(held.and_then(pointee))
            .home(*home)
            .build()
    };
    slots.iter().zip(homes).enumerate().map(row).collect()
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
        let machine = ops.map(|o| machine_for(o, MAX_HEAP, FrameMode::default()));
        Session::builder().maybe_machine(machine).build()
    }

    /// Set the cap on live heap octets; a new cap starts the run again.
    ///
    /// # Arguments
    ///
    /// * `cap` - The cap.
    pub fn set_max_heap(&mut self, cap: u64)
    {
        ret_if!(cap == self.max_heap);
        self.max_heap = cap;
        self.reset();
    }

    /// Set how the variables sit in the address space; a new form
    /// starts the run again.
    ///
    /// # Arguments
    ///
    /// * `frames` - The form.
    pub fn set_frames(&mut self, frames: FrameMode)
    {
        ret_if!(frames == self.frames);
        self.frames = frames;
        self.reset();
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
        let (cap, frames) = (self.max_heap, self.frames);
        let fresh = |m: &Machine| machine_for(m.module(), cap, frames);
        self.machine = self.machine.as_ref().map(fresh);
        self.viewed = 0;
        self.ended = None;
        self.kept.clear();
        self.kept_steps = 0;
        self.kept_heap = HeapShot::default();
        self.spent = Duration::ZERO;
        self.kept_spent = Duration::ZERO;
        self.kept_usage = (0, 0);
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
        let (spent, usage) = (self.run_time(), self.usage());
        let finished = how == RunStatus::Finished;
        let heap = self.machine.as_ref().map(|m| heap_shot(m, finished));
        self.reset();
        self.ended = Some(how);
        self.kept = kept;
        self.kept_steps = steps;
        self.kept_heap = heap.unwrap_or_default();
        self.kept_spent = spent;
        self.kept_usage = usage;
    }

    /// Run one op.
    pub fn step(&mut self) { self.clocked(|s| s.drive(|m| _ = m.step())) }

    /// Run one op; a call runs to its return.
    pub fn step_over(&mut self) { self.clocked(|s| s.drive(|m| _ = m.step_over())) }

    /// Run until the innermost call returns.
    pub fn step_out(&mut self) { self.clocked(|s| s.drive(|m| _ = m.step_out())) }

    /// Run until a break point, the end, a trap or the op budget.
    pub fn resume(&mut self)
    {
        let stops: Vec<(u32, u32)> = self.all_breaks().collect();
        self.clocked(|s| s.drive(|m| _ = m.run_until(&stops)));
    }

    /// Run one op for the timed stepping.
    ///
    /// # Returns
    ///
    /// Whether the stepping goes on: the run has not ended and the next op
    /// is not a break point.
    pub fn tick(&mut self) -> bool
    {
        let mut going = false;
        self.clocked(|s| going = s.advance(true));
        going
    }

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
        let mut going = false;
        self.clocked(|s| going = (0..count).all(|_| s.advance(breaks)));
        going
    }

    /// Run a command and add the time it took to the run, or to the ended
    /// run when the command ended it. The clock wraps whole commands and
    /// batches, not each op, so a fast run does not lose speed to it.
    ///
    /// # Arguments
    ///
    /// * `command` - The command.
    fn clocked(&mut self, command: impl FnOnce(&mut Self))
    {
        let start = Instant::now();
        command(self);
        let spent = start.elapsed();
        tern!(self.ended.is_some(), self.kept_spent += spent, self.spent += spent);
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
        self.drive(|m| _ = m.step());
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
        self.kept_heap = HeapShot::default();
        self.kept_spent = Duration::ZERO;
        self.kept_usage = (0, 0);
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

    /// How long the run took: the ops, not the pauses between them.
    ///
    /// # Returns
    ///
    /// The time of the run so far, or of the ended run.
    pub fn run_time(&self) -> Duration
    {
        tern!(self.ended.is_some(), self.kept_spent, self.spent)
    }

    /// The most heap and stack the run held at once.
    ///
    /// # Returns
    ///
    /// The peak heap octets and the peak stack slots, of the run so far or
    /// of the ended run.
    pub fn usage(&self) -> (u64, usize)
    {
        ret_if!(self.ended.is_some(), self.kept_usage);
        let peaks = |m: &Machine| (m.mem().peak_heap(), m.peak_stack());
        self.machine.as_ref().map_or((0, 0), peaks)
    }

    /// The blocks of the address space: those of the machine, or those the
    /// ended run left.
    ///
    /// # Returns
    ///
    /// The blocks and their counts.
    pub fn heap(&self) -> HeapShot
    {
        ret_if!(self.ended.is_some(), self.kept_heap.clone());
        let shot = self.machine.as_ref().map(|m| heap_shot(m, false));
        shot.unwrap_or_else(|| HeapShot::builder().cap(self.max_heap).build())
    }
}
