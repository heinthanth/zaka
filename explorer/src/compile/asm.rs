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
        .build()
}
