use std::collections::BTreeMap;

use bon::Builder;
use zakac_codegen::bc;
use zakac_codegen::bc::Op;
use zakac_codegen::listing::comment;
use zakac_codegen::listing::labels;
use zakac_codegen::listing::op_text;

use super::Unit;
use crate::compile::spans::Range;

/// A named slot, as the slot tables show it.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct SlotRow
{
    /// The slot index.
    pub index: u32,
    /// The source name, or a `%` name for a slot the generator added.
    #[builder(into)]
    pub name: String,
    /// The data kind.
    #[builder(into)]
    pub kind: String,
}

/// One op of a function, as the asm tables show it.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct OpRow
{
    /// The op index in its function.
    pub pc: u32,
    /// The address of the op in the code, where the functions sit one
    /// after another in module order.
    pub addr: u32,
    /// The label number when a jump goes here.
    pub label: Option<usize>,
    /// The op with its operand; a jump names its label.
    #[builder(into)]
    pub text: String,
    /// The slot or function the op names.
    pub note: Option<String>,
    /// The source region the op comes from, if it has one.
    pub range: Option<Range>,
    /// 1-based line of the start of the region.
    pub line: Option<usize>,
    /// Whether the op is a jump with no condition, as the jump back at the
    /// end of a loop is; a break point on its line skips it.
    #[builder(default)]
    pub plain_jump: bool,
}

/// One function of the ops, as the asm tables show it.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct AsmFunc
{
    /// The function index.
    pub index: u32,
    /// The address of its first op in the code.
    pub start: u32,
    /// The source name.
    #[builder(into)]
    pub name: String,
    /// The number of parameters.
    pub params: u32,
    /// The result kind, if the function has one.
    pub ret: Option<String>,
    /// The local slots, parameters first.
    pub locals: Vec<SlotRow>,
    /// The ops.
    pub rows: Vec<OpRow>,
}

impl AsmFunc
{
    /// The head line of the function: `f0 name (params 2) -> isize`.
    ///
    /// # Returns
    ///
    /// The text.
    pub fn head(&self) -> String
    {
        let ret = self.ret.as_ref().map(|k| format!(" -> {k}")).unwrap_or_default();
        format!("f{} {} (params {}){ret}", self.index, self.name, self.params)
    }
}

/// Rows of a slot table.
///
/// # Arguments
///
/// * `slots` - The slots.
///
/// # Returns
///
/// The rows.
pub fn slot_rows(slots: &[bc::Slot]) -> Vec<SlotRow>
{
    let row = |(i, s): (usize, &bc::Slot)| {
        SlotRow::builder()
            .index(i as u32)
            .name(s.name.as_str())
            .kind(s.kind.name())
            .build()
    };
    slots.iter().enumerate().map(row).collect()
}

/// The address of the first op of each function, where the functions sit
/// one after another in module order.
///
/// # Arguments
///
/// * `module` - The ops.
///
/// # Returns
///
/// The addresses, in function index order.
pub fn code_starts(module: &bc::Module) -> Vec<u32>
{
    let mut at = 0;
    let start = |f: &bc::Func| {
        let start = at;
        at += f.code.len() as u32;
        start
    };
    module.funcs.iter().map(start).collect()
}

/// Rows of each function of a module.
///
/// # Arguments
///
/// * `unit` - The source the spans point into.
/// * `module` - The ops.
///
/// # Returns
///
/// The functions, in module order.
pub(super) fn asm_rows(unit: &Unit, module: &bc::Module) -> Vec<AsmFunc>
{
    let starts = code_starts(module);
    let place = Place::builder().module(module).starts(&starts).build();
    let funcs = module.funcs.iter().enumerate();
    funcs.map(|(i, f)| func_rows(unit, place, i as u32, f)).collect()
}

/// The module and where its functions start, for the notes and addresses
/// of the rows.
#[derive(Clone, Copy, Builder)]
struct Place<'a>
{
    /// The module.
    module: &'a bc::Module,
    /// The address of the first op of each function.
    starts: &'a [u32],
}

/// Rows of one function.
///
/// # Arguments
///
/// * `unit` - The source the spans point into.
/// * `place` - The module and where its functions start.
/// * `index` - The function index.
/// * `func` - The function.
///
/// # Returns
///
/// The function.
fn func_rows(unit: &Unit, place: Place, index: u32, func: &bc::Func) -> AsmFunc
{
    let marks = labels(&func.code);
    let start = place.starts[index as usize];
    let ops = func.code.iter().enumerate();
    let row = |(pc, op): (usize, &Op)| {
        let at = OpAt::builder().func(func).start(start).pc(pc as u32).op(*op).build();
        op_row(unit, place, &marks, at)
    };
    AsmFunc::builder()
        .index(index)
        .start(start)
        .name(func.name.as_str())
        .params(func.params)
        .maybe_ret(func.ret.map(|k| k.name().to_string()))
        .locals(slot_rows(&func.locals))
        .rows(ops.map(row).collect())
        .build()
}

/// An op and where it sits.
#[derive(Clone, Copy, Builder)]
struct OpAt<'a>
{
    /// The function the op is in.
    func: &'a bc::Func,
    /// The address of the first op of the function.
    start: u32,
    /// The op index in the function.
    pc: u32,
    /// The op.
    op: Op,
}

/// Row of one op. A call notes the address of the function it calls.
///
/// # Arguments
///
/// * `unit` - The source the spans point into.
/// * `place` - The module and where its functions start.
/// * `marks` - The label number of each jump target.
/// * `at` - The op and where it sits.
///
/// # Returns
///
/// The row.
fn op_row(unit: &Unit, place: Place, marks: &BTreeMap<u32, usize>, at: OpAt) -> OpRow
{
    let (func, pc, op) = (at.func, at.pc, at.op);
    let span = func.spans.get(pc as usize).filter(|s| !s.is_placeholder());
    let note = comment(op, place.module, func);
    let target = match op
    {
        Op::Call(f) => place.starts.get(f as usize),
        _ => None,
    };
    let note = match (note, target)
    {
        (Some(n), Some(t)) => Some(format!("{n} @ {t}")),
        (note, _) => note,
    };
    OpRow::builder()
        .pc(pc)
        .addr(at.start + pc)
        .maybe_label(marks.get(&pc).copied())
        .text(op_text(op, marks))
        .maybe_note(note)
        .maybe_range(span.map(|s| unit.range(s)))
        .maybe_line(span.map(|s| unit.line_col(s).0))
        .plain_jump(matches!(op, Op::Jump(_)))
        .build()
}

/// The ops a break point on a source line stops at: the ops of the line where
/// the run comes onto it, so the first op of each function on the line, an op
/// after an op of another line, and the target of a jump, as the head of a
/// loop is. A jump with no condition is left out: the jump back at the end of
/// a loop carries the line of the loop, and the head stops the run next.
///
/// # Arguments
///
/// * `asm` - The functions.
/// * `line` - The 1-based line.
///
/// # Returns
///
/// Function and op index pairs.
pub fn line_stops(asm: &[AsmFunc], line: usize) -> Vec<(u32, u32)>
{
    asm.iter().flat_map(|f| entries(f, line)).collect()
}

/// The ops of one function where the run comes onto a source line.
///
/// # Arguments
///
/// * `func` - The function.
/// * `line` - The 1-based line.
///
/// # Returns
///
/// Function and op index pairs.
fn entries(func: &AsmFunc, line: usize) -> impl Iterator<Item = (u32, u32)> + '_
{
    let on = move |r: &OpRow| r.line == Some(line);
    let rows = func.rows.iter().enumerate();
    let comes_on = move |(i, r): &(usize, &OpRow)| {
        let after_other = *i == 0 || !on(&func.rows[i - 1]);
        on(r) && !r.plain_jump && (after_other || r.label.is_some())
    };
    rows.filter(comes_on).map(move |(_, r)| (func.index, r.pc))
}

/// Whether a source line has ops, so a break point there can stop a run.
///
/// # Arguments
///
/// * `asm` - The functions.
/// * `line` - The 1-based line.
///
/// # Returns
///
/// `true` when an op comes from the line.
pub fn has_ops(asm: &[AsmFunc], line: usize) -> bool
{
    asm.iter().flat_map(|f| &f.rows).any(|r| r.line == Some(line))
}

/// Where an op comes from in the source, so a break point on it finds the
/// op again in new ops of the same text, as after a change of the compile
/// options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Builder)]
pub struct OpSpot
{
    /// The function index.
    pub func: u32,
    /// The source region of the op, as UTF-8 offsets, if it has one.
    pub range: Option<(u32, u32)>,
    /// The number of ops of the function from the same region before it.
    pub nth: usize,
}

/// The place in the source of an op.
///
/// # Arguments
///
/// * `asm` - The functions.
/// * `func` - The function index.
/// * `pc` - The op index.
///
/// # Returns
///
/// The place, or `None` for an op that is not there.
pub fn op_spot(asm: &[AsmFunc], func: u32, pc: u32) -> Option<OpSpot>
{
    let rows = &asm.get(func as usize)?.rows;
    let range = region(rows.get(pc as usize)?);
    let nth = rows[..pc as usize].iter().filter(|r| region(r) == range).count();
    Some(OpSpot::builder().func(func).maybe_range(range).nth(nth).build())
}

/// The op at a place in the source.
///
/// # Arguments
///
/// * `asm` - The functions.
/// * `spot` - The place.
///
/// # Returns
///
/// The function and op index, or `None` when no op comes from the place.
pub fn find_spot(asm: &[AsmFunc], spot: &OpSpot) -> Option<(u32, u32)>
{
    let rows = &asm.get(spot.func as usize)?.rows;
    let same = rows.iter().filter(|r| region(r) == spot.range);
    same.map(|r| (spot.func, r.pc)).nth(spot.nth)
}

/// The source region of an op as a pair of offsets.
///
/// # Arguments
///
/// * `row` - The op.
///
/// # Returns
///
/// The start and end, or `None` without a region.
fn region(row: &OpRow) -> Option<(u32, u32)> { row.range.map(|r| (r.lo, r.hi)) }
