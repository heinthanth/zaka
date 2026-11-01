use zakac_ir_builder::ir::CmpOp;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::LogicOp;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::PrefixOp;
use zakac_ir_builder::value::Value;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::interval::eval;
use crate::state::Interval;
use crate::state::SlotState;
use crate::state::State;

/// Narrow the facts on one edge of a test, under `and`, `or` and `not`:
/// the owner state of a slot compared with `ဗလာ`, and, when `ints` gives
/// the platform, the interval of an integer slot compared with a value.
///
/// # Arguments
///
/// * `cond` - The test.
/// * `holds` - `true` on the edge where the test holds.
/// * `state` - The facts, narrowed in place.
/// * `ints` - The bit count of the platform-sized integer kinds, or `None` to
///   leave the integers alone, as for a test whose calls can change them after
///   it reads them.
pub(crate) fn narrow(cond: &Expr, holds: bool, state: &mut State, ints: Option<u16>)
{
    match &cond.node
    {
        ExprNode::Compare { op, lhs, rhs } =>
        {
            let null_edge = (*op == CmpOp::Eq) == holds;
            let tests_null = matches!(op, CmpOp::Eq | CmpOp::Ne);
            let place = null_compared(lhs, rhs).filter(|_| tests_null);
            do_if!(let Some(place) = place, narrow_slot(place, null_edge, state));
            let edge = tern!(holds, *op, opposite(*op));
            do_if!(let Some(platform) = ints, narrow_ints(edge, lhs, rhs, state, platform));
        }
        ExprNode::Logic { op, lhs, rhs } => logic(*op, lhs, rhs, holds, state, ints),
        ExprNode::Prefix { op: PrefixOp::Not, operand } =>
        {
            narrow(operand, !holds, state, ints)
        }
        _ => (),
    }
}

/// The comparison that holds where `op` does not.
///
/// # Arguments
///
/// * `op` - The comparison.
///
/// # Returns
///
/// Its negation.
fn opposite(op: CmpOp) -> CmpOp
{
    match op
    {
        CmpOp::Eq => CmpOp::Ne,
        CmpOp::Ne => CmpOp::Eq,
        CmpOp::Lt => CmpOp::Ge,
        CmpOp::Ge => CmpOp::Lt,
        CmpOp::Gt => CmpOp::Le,
        CmpOp::Le => CmpOp::Gt,
    }
}

/// The comparison with its operands swapped.
///
/// # Arguments
///
/// * `op` - The comparison.
///
/// # Returns
///
/// The comparison that holds for `rhs op' lhs` where `lhs op rhs` holds.
fn swapped(op: CmpOp) -> CmpOp
{
    match op
    {
        CmpOp::Lt => CmpOp::Gt,
        CmpOp::Gt => CmpOp::Lt,
        CmpOp::Le => CmpOp::Ge,
        CmpOp::Ge => CmpOp::Le,
        other => other,
    }
}

/// Narrow the interval of an integer slot on either side of a comparison
/// that holds.
///
/// # Arguments
///
/// * `op` - The comparison that holds on the edge.
/// * `lhs` - The left operand.
/// * `rhs` - The right operand.
/// * `state` - The facts, narrowed in place.
/// * `platform` - The bit count of the platform-sized integer kinds.
fn narrow_ints(op: CmpOp, lhs: &Expr, rhs: &Expr, state: &mut State, platform: u16)
{
    let (left, right) = (eval(lhs, state, platform), eval(rhs, state, platform));
    let slot = |e: &Expr| int_slot(e, platform);
    do_if!(let Some(place) = slot(lhs), clamp(place, op, right, state));
    do_if!(let Some(place) = slot(rhs), clamp(place, swapped(op), left, state));
    let pair = below_count(op, lhs, rhs, platform);
    let pair = pair.or_else(|| below_count(swapped(op), rhs, lhs, platform));
    do_if!(let Some(pair) = pair, { state.below.insert(pair); });
}

/// The integer local and the slice local of a test that holds when the
/// integer is below the count of the slice: `n < s ၏ အကွက်အရေအတွက်`.
///
/// # Arguments
///
/// * `op` - The comparison that holds, `lhs` on its left.
/// * `lhs` - The left operand.
/// * `rhs` - The right operand.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The integer and the slice, or `None` for another test.
fn below_count(
    op: CmpOp, lhs: &Expr, rhs: &Expr, platform: u16,
) -> Option<(Place, Place)>
{
    ret_if!(op != CmpOp::Lt, None);
    let n = int_slot(lhs, platform).filter(|p| matches!(p, Place::Local(_)))?;
    Some((n, count_slot(rhs)?))
}

/// The slice local whose count an operand reads, under each conversion: a
/// count that a conversion cuts is still at least each value it compares
/// below, since the count is a `usize`.
///
/// # Arguments
///
/// * `e` - The operand.
///
/// # Returns
///
/// The slice, or `None` for another operand.
fn count_slot(e: &Expr) -> Option<Place>
{
    let ExprNode::Load { addr } = &peel_converts(e).node
    else
    {
        return None;
    };
    let ExprNode::FieldAddr { base, field: 1, .. } = &addr.node
    else
    {
        return None;
    };
    let read = tern!(let ExprNode::Read(place) = base.node, Some(place), None);
    read.filter(|p| base.kind.is_slice() && matches!(p, Place::Local(_)))
}

/// An expression with each conversion around it taken off.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The innermost operand.
fn peel_converts(e: &Expr) -> &Expr
{
    tern!(let ExprNode::Convert { operand, .. } = &e.node, peel_converts(operand), e)
}

/// The integer slot an operand reads, under conversions that keep each
/// value.
///
/// # Arguments
///
/// * `e` - The operand.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The slot, or `None` for another operand.
pub(crate) fn int_slot(e: &Expr, platform: u16) -> Option<Place>
{
    match &e.node
    {
        ExprNode::Read(place) if e.kind.is_int() => Some(*place),
        ExprNode::Convert { operand, implicit: true }
            if keeps_values(operand, e, platform) =>
        {
            int_slot(operand, platform)
        }
        _ => None,
    }
}

/// Whether a conversion keeps each value of its operand.
///
/// # Arguments
///
/// * `from` - The operand.
/// * `to` - The conversion.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// `true` when the range of the operand kind lies in that of the result.
fn keeps_values(from: &Expr, to: &Expr, platform: u16) -> bool
{
    let (Some(a), Some(b)) = (from.kind.int_kind(), to.kind.int_kind())
    else
    {
        return false;
    };
    let (a, b) =
        (Interval::of_int_kind(a, platform), Interval::of_int_kind(b, platform));
    b.lo <= a.lo && a.hi <= b.hi
}

/// Narrow an integer slot whose value is known to lie between two ends,
/// so each end is a value some path gives it, to the values that pass a
/// comparison with `other`. A slot not known so is left alone: a
/// comparison alone gives no value a path reaches. An edge no value
/// passes cannot run, and the slot is left not known there.
///
/// # Arguments
///
/// * `place` - The slot.
/// * `op` - The comparison that holds, the slot on its left.
/// * `other` - The values of the other side.
/// * `state` - The facts, narrowed in place.
fn clamp(place: Place, op: CmpOp, other: Interval, state: &mut State)
{
    let SlotState::Int(x) = *state.get(place)
    else
    {
        return;
    };
    ret_if!(!x.is_bounded() || !other.is_bounded());
    let (lo, hi) = match op
    {
        CmpOp::Lt => (x.lo, x.hi.min(other.hi - 1)),
        CmpOp::Le => (x.lo, x.hi.min(other.hi)),
        CmpOp::Gt => (x.lo.max(other.lo + 1), x.hi),
        CmpOp::Ge => (x.lo.max(other.lo), x.hi),
        CmpOp::Eq => (x.lo.max(other.lo), x.hi.min(other.hi)),
        CmpOp::Ne => not_equal(x, other),
    };
    let next = tern!(lo <= hi, Interval { lo, hi }, Interval::TOP);
    state.set(place, SlotState::Int(next));
}

/// The ends of an interval with one value left out, when that value is
/// one of its ends.
///
/// # Arguments
///
/// * `x` - The interval.
/// * `other` - The value left out, when it is one value.
///
/// # Returns
///
/// The new ends.
fn not_equal(x: Interval, other: Interval) -> (i128, i128)
{
    ret_if!(other.lo != other.hi, (x.lo, x.hi));
    let lo = tern!(x.lo == other.lo, x.lo + 1, x.lo);
    let hi = tern!(x.hi == other.lo, x.hi - 1, x.hi);
    (lo, hi)
}

/// Narrow on one edge of `and` or `or`. When the edge does not tell which
/// operand decided, the two cases are joined.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `lhs` - The left test, which runs first.
/// * `rhs` - The right test.
/// * `holds` - `true` on the edge where the whole test holds.
/// * `state` - The facts, narrowed in place.
/// * `ints` - The platform, when integers are narrowed too.
fn logic(
    op: LogicOp, lhs: &Expr, rhs: &Expr, holds: bool, state: &mut State,
    ints: Option<u16>,
)
{
    let both = (op == LogicOp::And) == holds;
    ret_if!(both, narrow(lhs, holds, state, ints), narrow(rhs, holds, state, ints));
    let mut left = state.clone();
    narrow(lhs, holds, &mut left, ints);
    narrow(lhs, !holds, state, ints);
    narrow(rhs, holds, state, ints);
    *state = left.join(state);
}

/// The slot a comparison with `ဗလာ` tests.
///
/// # Arguments
///
/// * `lhs` - The left operand.
/// * `rhs` - The right operand.
///
/// # Returns
///
/// The slot read on one side when the other side is `ဗလာ`.
pub(crate) fn null_compared(lhs: &Expr, rhs: &Expr) -> Option<Place>
{
    ret_if!(is_null(rhs), slot_read(lhs));
    ret_if!(is_null(lhs), slot_read(rhs));
    None
}

/// The load of a list cell a comparison with `ဗလာ` tests.
///
/// # Arguments
///
/// * `lhs` - The left operand.
/// * `rhs` - The right operand.
///
/// # Returns
///
/// The load of a cell of a list slot on one side, under its conversions,
/// when the other side is `ဗလာ`.
pub(crate) fn null_compared_cell<'e>(lhs: &'e Expr, rhs: &'e Expr) -> Option<&'e Expr>
{
    ret_if!(is_null(rhs), cell_load(lhs));
    ret_if!(is_null(lhs), cell_load(rhs));
    None
}

/// The load of a cell of a list slot, or of a field of a struct slot, an
/// expression reads.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The load, under conversions and lends too.
fn cell_load(e: &Expr) -> Option<&Expr>
{
    let load = peel(e);
    let ExprNode::Load { addr } = &load.node
    else
    {
        return None;
    };
    let list =
        match &addr.node
        {
            ExprNode::CellAddr { list, .. }
            | ExprNode::FieldAddr { base: list, .. } => list,
            _ => return None,
        };
    let slot = list.kind.is_aggregate() && matches!(list.node, ExprNode::Read(_));
    slot.then_some(load)
}

/// Whether an expression is the constant `ဗလာ`.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// `true` for `ဗလာ`, under conversions too.
fn is_null(e: &Expr) -> bool { peel(e).as_const() == Some(Value::NULL) }

/// The slot an expression reads.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The slot of a read, under conversions and lends too.
fn slot_read(e: &Expr) -> Option<Place>
{
    let ExprNode::Read(place) = peel(e).node
    else
    {
        return None;
    };
    Some(place)
}

/// The expression under its conversions and lends, which keep the
/// address.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The inner expression.
fn peel(e: &Expr) -> &Expr
{
    match &e.node
    {
        ExprNode::Convert { operand, .. } | ExprNode::Lend { operand } => peel(operand),
        _ => e,
    }
}

/// Narrow the owner state of one slot.
///
/// # Arguments
///
/// * `place` - The slot.
/// * `null_edge` - `true` on the edge where the slot is `ဗလာ`.
/// * `state` - The facts, narrowed in place.
fn narrow_slot(place: Place, null_edge: bool, state: &mut State)
{
    let Some(info) = state.ptr_mut(place)
    else
    {
        return;
    };
    info.state = info.state.narrow(null_edge);
}

#[cfg(test)]
mod tests
{
    use zakac_ir_builder::ir::CmpOp;
    use zakac_ir_builder::ir::ExprNode;
    use zakac_ir_builder::ir::LogicOp;
    use zakac_ir_builder::ir::Place;
    use zakac_ir_builder::ir::PrefixOp;
    use zakac_ir_builder::value::Value;
    use zakac_semcheck::kind::DataKind;
    use zakac_span::span::Span;

    use super::*;
    use crate::state::OwnerState;
    use crate::state::PtrInfo;

    /// An expression of `kind`.
    fn expr(kind: DataKind, node: ExprNode) -> Expr
    {
        Expr::builder().kind(kind).node(node).span(Span::default()).build()
    }

    /// A comparison of global `g` with `ဗလာ`, the constant on the left
    /// when `flip` holds.
    fn null_test(op: CmpOp, g: u32, flip: bool) -> Expr
    {
        let ptr = DataKind::ISIZE.pointer_to();
        let read = Box::new(expr(ptr, ExprNode::Read(Place::Global(g))));
        let null = Box::new(expr(ptr, ExprNode::Const(Value::NULL)));
        let (lhs, rhs) = if flip { (null, read) } else { (read, null) };
        expr(DataKind::Bool, ExprNode::Compare { op, lhs, rhs })
    }

    /// A state of global owners in the states given.
    fn owners(states: &[OwnerState]) -> State
    {
        let slot = |&s: &OwnerState| {
            SlotState::Ptr(
                PtrInfo::builder().state(s).offset(Interval::point(0)).build(),
            )
        };
        State::builder().globals(states.iter().map(slot).collect()).build()
    }

    /// The owner state of global `g`.
    fn at(state: &State, g: u32) -> OwnerState
    {
        state.ptr(Place::Global(g)).expect("the slot is a pointer").state
    }

    #[test]
    fn ne_null_edge_holds_and_eq_null_edge_is_null()
    {
        let start = owners(&[OwnerState::UNKNOWN, OwnerState::FRESH]);
        let ne = null_test(CmpOp::Ne, 0, false);
        let mut taken = start.clone();
        narrow(&ne, true, &mut taken, None);
        assert_eq!(at(&taken, 0), OwnerState::HELD);
        let mut no = start.clone();
        narrow(&ne, false, &mut no, None);
        assert_eq!(at(&no, 0), OwnerState::NULL);
        let eq = null_test(CmpOp::Eq, 1, true);
        let mut taken = start.clone();
        narrow(&eq, true, &mut taken, None);
        assert_eq!(
            (at(&taken, 0), at(&taken, 1)),
            (OwnerState::UNKNOWN, OwnerState::NULL)
        );
        let mut no = start;
        narrow(&eq, false, &mut no, None);
        assert_eq!(at(&no, 1), OwnerState::HELD);
    }

    #[test]
    fn and_or_and_not_combine_the_edges()
    {
        let start = owners(&[OwnerState::UNKNOWN, OwnerState::UNKNOWN]);
        let both = |op| {
            let lhs = Box::new(null_test(CmpOp::Ne, 0, false));
            let rhs = Box::new(null_test(CmpOp::Ne, 1, false));
            expr(DataKind::Bool, ExprNode::Logic { op, lhs, rhs })
        };
        let mut taken = start.clone();
        narrow(&both(LogicOp::And), true, &mut taken, None);
        assert_eq!(
            (at(&taken, 0), at(&taken, 1)),
            (OwnerState::HELD, OwnerState::HELD)
        );
        let mut no = start.clone();
        narrow(&both(LogicOp::And), false, &mut no, None);
        assert_eq!(
            (at(&no, 0), at(&no, 1)),
            (OwnerState::UNKNOWN, OwnerState::UNKNOWN)
        );
        let mut no = start.clone();
        narrow(&both(LogicOp::Or), false, &mut no, None);
        assert_eq!((at(&no, 0), at(&no, 1)), (OwnerState::NULL, OwnerState::NULL));
        let operand = Box::new(null_test(CmpOp::Eq, 0, false));
        let not = expr(DataKind::Bool, ExprNode::Prefix { op: PrefixOp::Not, operand });
        let mut taken = start;
        narrow(&not, true, &mut taken, None);
        assert_eq!(at(&taken, 0), OwnerState::HELD);
    }
}
