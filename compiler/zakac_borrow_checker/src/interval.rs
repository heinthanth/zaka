use std::collections::BTreeSet;
use std::collections::HashSet;

use zakac_ast::stmt::CountMode;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::InfixOp;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::PrefixOp;
use zakac_ir_builder::ir::Stmt;
use zakac_ir_builder::value::Value;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

use crate::routine::Routine;
use crate::state::Interval;
use crate::state::SlotState;
use crate::state::State;
use crate::walk::each_expr;
use crate::walk::each_expr_mut;
use crate::walk::each_stmt;
use crate::walk::each_stmt_mut;
use crate::walk::stmt_exprs;

/// The cell addresses the report passes met, with a proof that the cell
/// number is in range and without one.
#[derive(Debug, Clone, Default)]
pub(crate) struct CellProofs
{
    /// The cell addresses proven in range.
    proofs: Vec<Span>,
    /// The cell addresses met without a proof.
    doubts: Vec<Span>,
}

impl CellProofs
{
    /// Add what one flow found.
    ///
    /// # Arguments
    ///
    /// * `proofs` - The cell addresses it proved in range.
    /// * `doubts` - The cell addresses it met without a proof.
    pub(crate) fn add(&mut self, proofs: &[Span], doubts: &[Span])
    {
        self.proofs.extend_from_slice(proofs);
        self.doubts.extend_from_slice(doubts);
    }

    /// The cell addresses proven in range each time a flow met them.
    ///
    /// # Returns
    ///
    /// The spans.
    pub(crate) fn proven(&self) -> HashSet<Span>
    {
        let doubts: HashSet<&Span> = self.doubts.iter().collect();
        self.proofs.iter().filter(|s| !doubts.contains(s)).copied().collect()
    }
}

/// Whether a module holds a raw address kind at all: in a slot, a
/// result or an expression, a cast target included. Such a program can
/// write an integer through an address the flow cannot see, so none of
/// its cells is proven in range.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// `true` when some kind has a raw level.
pub(crate) fn uses_raw(module: &Module) -> bool
{
    let funcs = module.funcs.iter();
    let slots = module.globals.iter().chain(funcs.clone().flat_map(|f| &f.locals));
    let results = funcs.clone().filter_map(|f| f.ret);
    ret_if!(slots.map(|s| s.kind).chain(results).any(has_raw), true);
    let mut found = false;
    let mut look = |e: &Expr| found |= has_raw(e.kind);
    let blocks = funcs.map(|f| f.block.as_slice()).chain([module.main.as_slice()]);
    for block in blocks
    {
        each_stmt(block, &mut |s| do_for!(e in stmt_exprs(s), each_expr(e, &mut look)));
    }
    found
}

/// Whether a kind has a raw address level, at whatever depth, cell or
/// field.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` for a raw level.
fn has_raw(kind: DataKind) -> bool
{
    ret_if!(let Some(list) = kind.as_list(), has_raw(list.cell));
    ret_if!(let Some(s) = kind.as_struct(), s.fields().iter().any(|(_, k)| has_raw(*k)));
    kind.ptr_kind().is_some_and(|p| p.raw != 0)
}

/// The slots of a routine a write the flow does not follow can change:
/// each slot whose address the program takes, through which a callee, a
/// borrow parameter or a raw address can write it. When the module turns
/// a number into an address, which can name each slot, all of them.
///
/// # Arguments
///
/// * `module` - The module.
/// * `routine` - The routine.
///
/// # Returns
///
/// The slots.
pub(crate) fn addressed_slots(module: &Module, routine: &Routine) -> BTreeSet<Place>
{
    let funcs = module.funcs.iter().map(|f| f.block.as_slice());
    let (globals, from_number) = taken_addresses(funcs.chain([module.main.as_slice()]));
    let (locals, _) = taken_addresses([routine.block]);
    let global = |n: usize| Place::Global(n as u32);
    let local = |n: usize| Place::Local(n as u32);
    let marked_global = |n: &usize| module.globals[*n].addressed;
    let marked_local = |n: &usize| routine.locals[*n].addressed;
    let all_globals =
        (0..module.globals.len()).filter(|n| from_number || marked_global(n));
    let all_locals =
        (0..routine.locals.len()).filter(|n| from_number || marked_local(n));
    let taken = globals.into_iter().filter(|p| matches!(p, Place::Global(_)));
    let taken_locals = locals.into_iter().filter(|p| matches!(p, Place::Local(_)));
    let marked = all_globals.map(global).chain(all_locals.map(local));
    marked.chain(taken).chain(taken_locals).collect()
}

/// The slots whose address some statement of the blocks takes, and
/// whether one turns a number into an address, a cast folded to a
/// constant address included.
///
/// # Arguments
///
/// * `blocks` - The statement lists.
///
/// # Returns
///
/// The slots and the flag.
fn taken_addresses<'m>(
    blocks: impl IntoIterator<Item = &'m [Stmt]>,
) -> (BTreeSet<Place>, bool)
{
    let mut found = BTreeSet::new();
    let mut from_number = false;
    let mut look = |e: &Expr| {
        do_if!(let ExprNode::AddrOf(place) = e.node, { found.insert(place); });
        let number = |op: &Expr| op.kind.is_int() && e.kind.is_ptr();
        let cast =
            matches!(&e.node, ExprNode::Convert { operand, .. } if number(operand));
        let folded = e.kind.is_ptr() && e.as_const().is_some_and(|v| v != Value::NULL);
        from_number |= cast || folded;
    };
    for block in blocks
    {
        each_stmt(block, &mut |s| do_for!(e in stmt_exprs(s), each_expr(e, &mut look)));
    }
    (found, from_number)
}

/// The value of an integer constant.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The value, or `None` for an expression that is no integer constant.
pub(crate) fn const_int(e: &Expr) -> Option<i128>
{
    let Some(Value::Int { kind, raw }) = e.as_const()
    else
    {
        return None;
    };
    tern!(kind.signed, Some(raw as i128), i128::try_from(raw).ok())
}

/// The interval of an integer expression, without running it.
///
/// # Arguments
///
/// * `e` - The expression.
/// * `state` - The facts.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The interval; [`Interval::TOP`] when it leaves the range of the kind
/// of `e`, which wraps at run time.
pub(crate) fn eval(e: &Expr, state: &State, platform: u16) -> Interval
{
    let sub = |inner: &Expr| eval(inner, state, platform);
    let n = match &e.node
    {
        ExprNode::Const(_) => const_int(e).map_or(Interval::TOP, Interval::point),
        ExprNode::Read(place) => int_of(state.get(*place)),
        ExprNode::Infix { op, lhs, rhs } => arith(*op, sub(lhs), sub(rhs)),
        ExprNode::Prefix { op: PrefixOp::Neg, operand } => sub(operand).negate(),
        ExprNode::Convert { operand, implicit: true } => sub(operand),
        _ => Interval::TOP,
    };
    n.fit(e.kind, platform)
}

/// The interval of an integer slot fact.
///
/// # Arguments
///
/// * `fact` - The fact.
///
/// # Returns
///
/// The interval, [`Interval::TOP`] for a fact that is no integer.
pub(crate) fn int_of(fact: &SlotState) -> Interval
{
    tern!(let SlotState::Int(n) = fact, *n, Interval::TOP)
}

/// The interval of an arithmetic operation.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `lhs` - The left interval.
/// * `rhs` - The right interval.
///
/// # Returns
///
/// The interval of `+`, `-` and `*`; [`Interval::TOP`] for the others.
pub(crate) fn arith(op: InfixOp, lhs: Interval, rhs: Interval) -> Interval
{
    match op
    {
        InfixOp::Add => lhs.plus(rhs),
        InfixOp::Sub => lhs.minus(rhs),
        InfixOp::Mul => lhs.times(rhs),
        _ => Interval::TOP,
    }
}

/// The values the variable of a counted loop takes inside the loop. With
/// constant bounds and step, the last value is the last one the step
/// reaches; otherwise each value between the bounds counts. A loop that
/// cannot run, and a step that is not known to be positive, give
/// [`Interval::TOP`].
///
/// # Arguments
///
/// * `from` - The first value.
/// * `to` - The last value.
/// * `step` - The step; one when absent.
/// * `mode` - Which direction the variable moves.
/// * `state` - The facts at the step of the loop.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The interval.
pub(crate) fn counted_range(
    from: &Expr, to: &Expr, step: Option<&Expr>, mode: CountMode, state: &State,
    platform: u16,
) -> Interval
{
    let bound = |e: &Expr| eval(e, state, platform);
    let (first, last) = (bound(from), bound(to));
    let step = step.map_or(Interval::point(1), bound);
    ret_if!(step.lo < 1 || !step.is_bounded(), Interval::TOP);
    let up = match mode
    {
        CountMode::Increase => Some(true),
        CountMode::Decrease => Some(false),
        CountMode::Count => direction(first, last),
    };
    let Some(up) = up
    else
    {
        return first.hull(last);
    };
    tern!(up, rising(first, last, step), falling(first, last, step))
}

/// The direction a counting loop goes in, when the bounds tell.
///
/// # Arguments
///
/// * `first` - The first value.
/// * `last` - The last value.
///
/// # Returns
///
/// `Some(true)` up, `Some(false)` down, `None` when either can happen.
fn direction(first: Interval, last: Interval) -> Option<bool>
{
    ret_if!(first.hi <= last.lo, Some(true));
    ret_if!(first.lo >= last.hi, Some(false));
    None
}

/// The values of a loop that adds its step.
///
/// # Arguments
///
/// * `first` - The first value.
/// * `last` - The bound it stops at.
/// * `step` - The step, positive.
///
/// # Returns
///
/// The interval.
fn rising(first: Interval, last: Interval, step: Interval) -> Interval
{
    ret_if!(first.lo > last.hi, Interval::TOP);
    let points = first.lo == first.hi && last.lo == last.hi && step.lo == step.hi;
    ret_if!(!points, Interval { lo: first.lo, hi: last.hi });
    let span = last.lo.checked_sub(first.lo).map(|d| d / step.lo * step.lo);
    let hi = span.and_then(|d| first.lo.checked_add(d));
    hi.map_or(Interval::TOP, |hi| Interval { lo: first.lo, hi })
}

/// The values of a loop that subtracts its step.
///
/// # Arguments
///
/// * `first` - The first value.
/// * `last` - The bound it stops at.
/// * `step` - The step, positive.
///
/// # Returns
///
/// The interval.
fn falling(first: Interval, last: Interval, step: Interval) -> Interval
{
    let flip = |n: Interval| n.negate();
    let up = rising(flip(first), flip(last), step);
    tern!(up == Interval::TOP, up, up.negate())
}

/// The element count of a block, when its size is one known value that
/// holds a whole number of elements of the pointee.
///
/// # Arguments
///
/// * `count` - The count written.
/// * `unit` - The kind it counts; `None` for a count of octets.
/// * `pointee` - The kind the owner points at; `None` for an opaque one.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The element count, or `None` when it is not known.
pub(crate) fn elements(
    count: Interval, unit: Option<DataKind>, pointee: Option<DataKind>, platform: u16,
) -> Option<u64>
{
    ret_if!(count.lo != count.hi, None);
    let count = u64::try_from(count.lo).ok()?;
    let unit = unit.map_or(1, |k| u64::from(k.size(platform)));
    let size = u64::from(pointee?.size(platform));
    let octets = count.checked_mul(unit)?;
    ret_if!(size == 0 || !octets.is_multiple_of(size), None);
    Some(octets / size)
}

/// Set the `checked` flag on each cell address proven in range, so code
/// generation leaves its run-time check out. A flag set before is kept.
///
/// # Arguments
///
/// * `module` - The module, changed in place.
/// * `proofs` - The spans of the cell addresses proven in range.
pub fn mark_checked(module: &mut Module, proofs: &HashSet<Span>)
{
    ret_if!(proofs.is_empty());
    let mut mark = |e: &mut Expr| {
        let span = e.span;
        let ExprNode::CellAddr { checked, .. } = &mut e.node
        else
        {
            return;
        };
        do_if!(proofs.contains(&span), *checked = true);
    };
    let mut each = |s: &mut Stmt| each_expr_mut(s, &mut mark);
    do_for!(func in module.funcs.iter_mut(), each_stmt_mut(&mut func.block, &mut each));
    each_stmt_mut(&mut module.main, &mut each);
}

#[cfg(test)]
mod tests
{
    use zakac_semcheck::kind::IntKind;
    use zakac_semcheck::kind::Width;

    use super::*;

    #[test]
    fn elements_count_whole_pointees()
    {
        let i32_kind = DataKind::Int(IntKind::new(true, Width::Bits(32)));
        let u8_kind = DataKind::Int(IntKind::new(false, Width::Bits(8)));
        let n = Interval::point;
        assert_eq!(elements(n(3), Some(i32_kind), Some(i32_kind), 64), Some(3));
        assert_eq!(elements(n(8), None, Some(i32_kind), 64), Some(2));
        assert_eq!(elements(n(7), None, Some(i32_kind), 64), None);
        assert_eq!(elements(n(2), Some(i32_kind), Some(u8_kind), 64), Some(8));
        assert_eq!(elements(n(8), None, None, 64), None);
        assert_eq!(elements(Interval { lo: 1, hi: 2 }, None, Some(u8_kind), 64), None);
    }
}
