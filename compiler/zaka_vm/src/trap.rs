use std::fmt;
use std::sync::Arc;

use bon::Builder;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

/// The reason a program stopped before its end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrapKind
{
    /// An integer division or remainder with zero as divisor.
    ZeroDivisor,
    /// A shift distance not below the bit width.
    ShiftTooFar
    {
        /// The distance.
        distance: u128,
        /// The bit width of the shifted value.
        bits: u16,
    },
    /// The calls nest too deep.
    StackOverflow
    {
        /// The limit.
        depth: u32,
    },
    /// The op budget ran out.
    StepLimit
    {
        /// The limit.
        steps: u64,
    },
    /// A counted loop step is zero or negative.
    StepNotPositive,
    /// The ops break a rule the generator keeps, such as a slot index out
    /// of range.
    Malformed
    {
        /// What is wrong.
        reason: String,
    },
    /// The output could not be written.
    Write
    {
        /// The I/O error text.
        msg: String,
    },
    /// An access below the first block, in the reserved region.
    NullDeref
    {
        /// The address.
        addr: u64,
    },
    /// An access outside each block, or across the end of one, or a
    /// pointer step out of the block it starts in.
    OutOfBounds
    {
        /// The address.
        addr: u64,
        /// The octet count of the access; 0 for a step, which accesses
        /// nothing.
        size: u32,
    },
    /// An access inside a freed block.
    UseAfterFree
    {
        /// The address.
        addr: u64,
    },
    /// A write inside a frozen block: static text, which each literal of
    /// the same text shares.
    FrozenWrite
    {
        /// The address.
        addr: u64,
    },
    /// A range of text starts or ends inside a character.
    TextCut
    {
        /// The address of the edge.
        addr: u64,
    },
    /// U8s that become text are not UTF-8.
    BadUtf8
    {
        /// The address of the first u8 that is not.
        addr: u64,
    },
    /// A free of a heap block that was freed before.
    DoubleFree
    {
        /// The start of the block.
        addr: u64,
    },
    /// A free of an address that is not the start of a heap block.
    BadFree
    {
        /// The address.
        addr: u64,
    },
    /// An access at an address that is not a multiple of the alignment of
    /// its kind.
    Misaligned
    {
        /// The address.
        addr: u64,
        /// The alignment of the kind.
        align: u32,
    },
    /// A load of octets that hold no value of the kind.
    BadValue
    {
        /// The address.
        addr: u64,
        /// The name of the kind.
        kind: String,
    },
    /// The address space had no room for the block of a global.
    StaticFull
    {
        /// The octet count of the global.
        size: u64,
    },
    /// A call found no room in the stack region for a local.
    StackFull
    {
        /// The octet count of the local.
        size: u64,
    },
    /// A resize found no heap room.
    OutOfMem
    {
        /// The octet count asked for.
        size: u64,
    },
    /// A subtraction of pointers into different blocks.
    MixedBlocks,
    /// The two ranges of a raw duplicate that must stand apart overlap.
    Overlap
    {
        /// The address the cells go to.
        addr: u64,
    },
    /// A cell number outside its list.
    IndexOutOfRange
    {
        /// The cell number.
        index: i128,
        /// The number of the first cell: 0 or 1.
        first: u8,
        /// The number of cells.
        len: u64,
    },
    /// A call went through a function value that holds no function, the zero
    /// word.
    NullCall,
    /// A field was read or written through an object variable that holds no
    /// object, the zero word.
    NullObject,
    /// A handle went to C after a C function that takes it over closed it.
    ClosedHandle,
    /// C passed NULL to a callback for a parameter of plain text.
    NullFromC,
    /// C called a closure after the call that gave it ended, with no leave
    /// from `ခေါ်ဆိုခွင့် ပြုမည်`, or after that leave was taken back.
    ClosureEscape,
    /// A C function was called in a build of the VM that cannot call C,
    /// such as the one in the browser.
    ForeignCall
    {
        /// The C name.
        name: String,
    },
    /// The C functions of the program could not be reached: a lib that does
    /// not open, a C name a lib does not hold, or a platform size that is not
    /// the host's.
    ForeignLink
    {
        /// What went wrong.
        reason: String,
    },
    /// A remove statement found a list with no cells.
    NoCellToRemove,
    /// A remove statement found text with no characters.
    NoCharToRemove,
    /// An add in front of a cell or a character named a number past the
    /// count, which is the last place a value can go.
    NoPlaceToInsert
    {
        /// The number named.
        index: i128,
        /// The number of the first cell or character: 0 or 1.
        first: u8,
        /// The number of cells or characters.
        len: u64,
    },
    /// A character number lies outside its text.
    CharOutOfRange
    {
        /// The character number.
        index: i128,
        /// The number of the first character: 0 or 1.
        first: u8,
        /// The number of characters.
        len: u64,
    },
    /// A replace statement named a range whose last cell or character
    /// comes before its first.
    EndBeforeStart
    {
        /// The first number, as written.
        start: i128,
        /// The last number, as written.
        end: i128,
    },
    /// With `--overflow stop`, an integer operation whose result does not fit
    /// its kind.
    Overflow
    {
        /// The operator, such as `+`.
        op: String,
        /// The integer kind, as its short name.
        kind: String,
    },
    /// A result that holds a failure was converted to its good result.
    NotGood
    {
        /// The error value, as `ပြသမည်` writes it; blank for a failure with
        /// no error value.
        text: String,
    },
    /// A good result was converted to its error value.
    NotFault,
    /// An optional that holds no data was read.
    NoData,
    /// A statement wrote through the value of a row of a ဇယား, and no row had
    /// the index.
    NoRow,
    /// A range of cells that passes the end of its list or slice.
    SliceOutOfRange
    {
        /// The first cell of the range.
        start: u64,
        /// The number of cells of the range.
        len: u64,
        /// The number of cells of the source.
        limit: u64,
    },
}

impl fmt::Display for TrapKind
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let text = match self
        {
            TrapKind::ZeroDivisor => t!("vm.trap.zero_divisor"),
            TrapKind::ShiftTooFar { distance, bits } =>
            {
                t!("vm.trap.shift_too_far", distance = distance, bits = bits)
            }
            TrapKind::StackOverflow { depth } =>
            {
                t!("vm.trap.stack_overflow", depth = depth)
            }
            TrapKind::StepLimit { steps } => t!("vm.trap.step_limit", steps = steps),
            TrapKind::StepNotPositive => t!("vm.trap.step_not_positive"),
            TrapKind::Malformed { reason } => t!("vm.trap.malformed", reason = reason),
            TrapKind::Write { msg } => t!("vm.trap.write", msg = msg),
            TrapKind::NullDeref { addr } =>
            {
                t!("vm.trap.null_deref", addr = hex(*addr))
            }
            TrapKind::OutOfBounds { addr, size: 0 } =>
            {
                t!("vm.trap.step_out_of_bounds", addr = hex(*addr))
            }
            TrapKind::OutOfBounds { addr, size } =>
            {
                t!("vm.trap.out_of_bounds", addr = hex(*addr), size = size)
            }
            TrapKind::UseAfterFree { addr } =>
            {
                t!("vm.trap.use_after_free", addr = hex(*addr))
            }
            TrapKind::FrozenWrite { addr } =>
            {
                t!("vm.trap.frozen_write", addr = hex(*addr))
            }
            TrapKind::TextCut { addr } => t!("vm.trap.text_cut", addr = hex(*addr)),
            TrapKind::BadUtf8 { addr } => t!("vm.trap.bad_utf8", addr = hex(*addr)),
            TrapKind::DoubleFree { addr } =>
            {
                t!("vm.trap.double_free", addr = hex(*addr))
            }
            TrapKind::BadFree { addr } => t!("vm.trap.bad_free", addr = hex(*addr)),
            TrapKind::Misaligned { addr, align } =>
            {
                t!("vm.trap.misaligned", addr = hex(*addr), align = align)
            }
            TrapKind::BadValue { addr, kind } =>
            {
                t!("vm.trap.bad_value", addr = hex(*addr), kind = kind)
            }
            TrapKind::StaticFull { size } => t!("vm.trap.static_full", size = size),
            TrapKind::StackFull { size } => t!("vm.trap.stack_full", size = size),
            TrapKind::OutOfMem { size } => t!("vm.trap.out_of_mem", size = size),
            TrapKind::MixedBlocks => t!("vm.trap.mixed_blocks"),
            TrapKind::Overlap { addr } => t!("vm.trap.overlap", addr = hex(*addr)),
            TrapKind::IndexOutOfRange { index, first, len } =>
            {
                let last = i128::from(*len) + i128::from(*first) - 1;
                t!("vm.trap.index_out_of_range", index = index, lo = first, hi = last)
            }
            TrapKind::CharOutOfRange { index, first, len } =>
            {
                let last = i128::from(*len) + i128::from(*first) - 1;
                t!("vm.trap.char_out_of_range", index = index, lo = first, hi = last)
            }
            TrapKind::EndBeforeStart { start, end } =>
            {
                t!("vm.trap.end_before_start", start = start, end = end)
            }
            TrapKind::NullCall => t!("vm.trap.null_call"),
            TrapKind::NullObject => t!("vm.trap.null_object"),
            TrapKind::ClosedHandle => t!("vm.trap.closed_handle"),
            TrapKind::NullFromC => t!("vm.trap.null_from_c"),
            TrapKind::ClosureEscape => t!("vm.trap.closure_escape"),
            TrapKind::ForeignCall { name } => t!("vm.trap.foreign_call", name = name),
            TrapKind::ForeignLink { reason } =>
            {
                t!("vm.trap.foreign_link", reason = reason)
            }
            TrapKind::NoCellToRemove => t!("vm.trap.no_cell_to_remove"),
            TrapKind::NoCharToRemove => t!("vm.trap.no_char_to_remove"),
            TrapKind::NoPlaceToInsert { index, first, len } =>
            {
                let last = i128::from(*len) + i128::from(*first);
                t!("vm.trap.no_place_to_insert", index = index, lo = first, hi = last)
            }
            TrapKind::Overflow { op, kind } =>
            {
                t!("vm.trap.overflow", op = op, kind = kind)
            }
            TrapKind::NotGood { text } if text.is_empty() =>
            {
                t!("vm.trap.not_good_bare")
            }
            TrapKind::NotGood { text } => t!("vm.trap.not_good", e = text),
            TrapKind::NotFault => t!("vm.trap.not_fault"),
            TrapKind::NoData => t!("vm.trap.no_data"),
            TrapKind::NoRow => t!("vm.trap.no_row"),
            TrapKind::SliceOutOfRange { start, len, limit } =>
            {
                t!(
                    "vm.trap.slice_out_of_range",
                    start = start,
                    len = len,
                    limit = limit
                )
            }
        };
        f.write_str(&text)
    }
}

impl TrapKind
{
    /// The address an access or free trap names.
    ///
    /// # Returns
    ///
    /// The address, or `None` for a trap that names none.
    pub fn addr(&self) -> Option<u64>
    {
        match self
        {
            TrapKind::NullDeref { addr }
            | TrapKind::OutOfBounds { addr, .. }
            | TrapKind::UseAfterFree { addr }
            | TrapKind::FrozenWrite { addr }
            | TrapKind::TextCut { addr }
            | TrapKind::BadUtf8 { addr }
            | TrapKind::DoubleFree { addr }
            | TrapKind::BadFree { addr }
            | TrapKind::Misaligned { addr, .. }
            | TrapKind::BadValue { addr, .. }
            | TrapKind::Overlap { addr } => Some(*addr),
            TrapKind::ZeroDivisor
            | TrapKind::ShiftTooFar { .. }
            | TrapKind::StackOverflow { .. }
            | TrapKind::StepLimit { .. }
            | TrapKind::StepNotPositive
            | TrapKind::Malformed { .. }
            | TrapKind::Write { .. }
            | TrapKind::StaticFull { .. }
            | TrapKind::StackFull { .. }
            | TrapKind::OutOfMem { .. }
            | TrapKind::MixedBlocks
            | TrapKind::IndexOutOfRange { .. }
            | TrapKind::CharOutOfRange { .. }
            | TrapKind::EndBeforeStart { .. }
            | TrapKind::NullCall
            | TrapKind::NullObject
            | TrapKind::ClosedHandle
            | TrapKind::NullFromC
            | TrapKind::ClosureEscape
            | TrapKind::ForeignCall { .. }
            | TrapKind::ForeignLink { .. }
            | TrapKind::NoCellToRemove
            | TrapKind::NoCharToRemove
            | TrapKind::NoPlaceToInsert { .. }
            | TrapKind::SliceOutOfRange { .. }
            | TrapKind::NotGood { .. }
            | TrapKind::NotFault
            | TrapKind::NoData
            | TrapKind::NoRow
            | TrapKind::Overflow { .. } => None,
        }
    }
}

/// An address as `0x` and lowercase hex digits.
///
/// # Arguments
///
/// * `addr` - The address.
///
/// # Returns
///
/// The text.
pub fn hex(addr: u64) -> String { format!("{addr:#x}") }

/// A trap with the place it happened.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Trap
{
    /// The reason the program stopped.
    pub kind: TrapKind,
    /// The name of the function that ran.
    #[builder(into)]
    pub func: String,
    /// The index of the op that trapped.
    pub pc: u32,
    /// The block the address of the trap falls in, as text: its start,
    /// size and origin.
    pub block: Option<String>,
    /// The calls in progress, the one that ran first: its function and op,
    /// then each caller with the op of its call.
    #[builder(default)]
    pub frames: Vec<TrapFrame>,
}

/// One call in progress when the program stopped.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct TrapFrame
{
    /// The name of the function.
    #[builder(into)]
    pub func: String,
    /// The index of the op that ran, or of the call it waits on.
    pub pc: u32,
    /// The source region of that op; `None` in a release build, which
    /// leaves it out.
    pub span: Option<Span>,
    /// Whether the compiler added the function, as the main function around
    /// the start block, which no source names.
    #[builder(default)]
    pub added: bool,
}

/// How much of the calls in progress a report lists, from the variable
/// `ZAKA_BACKTRACE`, as Rust reads `RUST_BACKTRACE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Backtrace
{
    /// The line of the stop alone: `0`.
    Off,
    /// The calls of the program, a run of one call once, the first
    /// [`SHOWN_CALLS`] lines: unset, or another word.
    #[default]
    Short,
    /// Each call, the main function the compiler adds too: `full`.
    Full,
}

impl Backtrace
{
    /// The name of the variable that picks it.
    pub const VAR: &str = "ZAKA_BACKTRACE";

    /// The choice a value of the variable makes.
    ///
    /// # Arguments
    ///
    /// * `value` - The value, `None` when the variable is not set.
    ///
    /// # Returns
    ///
    /// The choice.
    pub fn from_value(value: Option<&str>) -> Self
    {
        match value
        {
            Some("0") => Backtrace::Off,
            Some("full") => Backtrace::Full,
            _ => Backtrace::Short,
        }
    }
}

/// The most lines of calls a report shows; a deep recursion shows its
/// first calls, then a count of the rest.
const SHOWN_CALLS: usize = 16;

impl Trap
{
    /// The trap as the command line writes it: the reason and where it
    /// happened, its line when the build kept the lines, then the calls
    /// in progress when there is more than one, as much of them as
    /// `backtrace` asks.
    ///
    /// # Arguments
    ///
    /// * `sm` - The source map the regions point into.
    /// * `backtrace` - How much of the calls to list.
    ///
    /// # Returns
    ///
    /// The text, one line for each call.
    pub fn report(&self, sm: &SourceMap, backtrace: Backtrace) -> String
    {
        let what = self.kind.to_string();
        let place = self.frames.first().and_then(|f| place(sm, f.span));
        let head = match place
        {
            Some(place) =>
            {
                t!("vm.trap.at_line", what = what, func = self.func, place = place)
            }
            None => t!("vm.trap.at", what = what, func = self.func, pc = self.pc),
        };
        let block = self.block.as_ref().map(|b| format!("; {b}")).unwrap_or_default();
        let mut out = format!("{head}{block}");
        let lines = match backtrace
        {
            Backtrace::Off => return out,
            Backtrace::Short => short_calls(sm, &self.frames),
            Backtrace::Full => self.frames.iter().map(|f| call_line(sm, f)).collect(),
        };
        ret_if!(lines.len() < 2, out);
        out.push('\n');
        out.push_str(&t!("vm.trap.backtrace"));
        lines.iter().for_each(|line| {
            out.push('\n');
            out.push_str(line);
        });
        out
    }
}

/// The lines of the calls in progress of the program: the calls of the
/// main function the compiler adds are left out but for the first, a run of
/// calls of one op shows once, with the count of its repeats, and the lines
/// past [`SHOWN_CALLS`] show as a count. A last line tells how to list each
/// call when one was left out.
///
/// # Arguments
///
/// * `sm` - The source map the regions point into.
/// * `frames` - The calls, the one that ran first.
///
/// # Returns
///
/// The lines.
fn short_calls(sm: &SourceMap, frames: &[TrapFrame]) -> Vec<String>
{
    let shown = frames.iter().enumerate().filter(|(n, f)| *n == 0 || !f.added);
    let mut runs: Vec<(&TrapFrame, usize)> = Vec::new();
    for (_, frame) in shown
    {
        match runs.last_mut()
        {
            Some((last, n)) if *last == frame => *n += 1,
            _ => runs.push((frame, 1)),
        }
    }
    let hidden: usize = runs.iter().skip(SHOWN_CALLS).map(|(_, n)| n).sum();
    let runs_shown = runs.iter().take(SHOWN_CALLS);
    let mut lines: Vec<String> =
        runs_shown.flat_map(|(f, n)| call_lines(sm, f, *n)).collect();
    do_if!(hidden > 0, lines.push(t!("vm.trap.calls_more", n = hidden).to_string()));
    let left_out = runs.len() < frames.len();
    let hint = t!("vm.trap.full_hint", var = Backtrace::VAR);
    do_if!(left_out && lines.len() > 1, lines.push(hint.to_string()));
    lines
}

/// The line of one call, and a line for its repeats.
///
/// # Arguments
///
/// * `sm` - The source map the regions point into.
/// * `frame` - The call.
/// * `n` - The count of the call in its run, itself included.
///
/// # Returns
///
/// The lines.
fn call_lines(sm: &SourceMap, frame: &TrapFrame, n: usize) -> Vec<String>
{
    let again = (n > 1).then(|| t!("vm.trap.call_again", n = n - 1).to_string());
    std::iter::once(call_line(sm, frame)).chain(again).collect()
}

/// The line of one call: its function and its line, or its op in a release
/// build.
///
/// # Arguments
///
/// * `sm` - The source map the regions point into.
/// * `frame` - The call.
///
/// # Returns
///
/// The line.
fn call_line(sm: &SourceMap, frame: &TrapFrame) -> String
{
    let func = &frame.func;
    let line = match place(sm, frame.span)
    {
        Some(place) => t!("vm.trap.call_at", func = func, place = place),
        None => t!("vm.trap.call_op", func = func, pc = frame.pc),
    };
    line.to_string()
}

/// A source region as `file:line:column`, the column counted from 1.
///
/// # Arguments
///
/// * `sm` - The source map.
/// * `span` - The region.
///
/// # Returns
///
/// The text, or `None` for no region or one outside each file of the map.
fn place(sm: &SourceMap, span: Option<Span>) -> Option<String>
{
    let lo = span.filter(|s| !s.is_placeholder())?.lo;
    let inside = |f: &Arc<SourceFile>| f.start_pos <= lo && lo <= f.end_pos();
    ret_if!(!sm.files().iter().any(inside), None);
    let loc = sm.lookup_char_pos(&lo);
    let path = loc.file.name.virtual_path_string();
    Some(format!("{path}:{}:{}", loc.line, loc.col.as_u32() + 1))
}

impl fmt::Display for Trap
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let what = self.kind.to_string();
        let text = t!("vm.trap.at", what = what, func = self.func, pc = self.pc);
        f.write_str(&text)?;
        self.block.as_ref().map_or(Ok(()), |b| write!(f, "; {b}"))
    }
}
