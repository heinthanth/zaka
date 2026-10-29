use zakac_diagnostics::ownership::mixed_blocks;
use zakac_diagnostics::ownership::offset_past_end;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Place;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::checks::report;
use crate::digest::Root;
use crate::digest::Seg;
use crate::flow::Flow;
use crate::roots::Roots;
use crate::state::Interval;
use crate::state::SlotState;
use crate::state::State;

/// Report a pointer whose offset leaves its block, when the element count
/// of the block and both ends of the offset are known: a read or a write
/// at the end of the block or past it, or a step past one past the end.
/// The end the report names is a value some path reaches, as counted
/// loops give exact ranges and joins keep the ends of each path.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `at` - The value of the pointer.
/// * `span` - The pointer.
/// * `access` - `true` for a read or a write through it, `false` for a step,
///   which can stop one past the end.
pub(crate) fn check_offset(flow: &mut Flow, at: &SlotState, span: Span, access: bool)
{
    let SlotState::Ptr(info) = at
    else
    {
        return;
    };
    let Some(len) = info.len
    else
    {
        return;
    };
    let last = i128::from(len) - i128::from(access);
    let Some(off) = outside(info.offset, 0, last)
    else
    {
        return;
    };
    report(flow, offset_past_end(span, off, len));
}

/// Check a cell number against the cells of its list. A number proven
/// inside each time the cell address runs lets code generation leave out
/// its run-time check; one with a known end outside the list is reported.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `index` - The values the cell number can take.
/// * `len` - The number of cells.
/// * `first` - The number of the first cell: 0 or 1.
/// * `span` - The cell address.
pub(crate) fn check_cell(
    flow: &mut Flow, index: Interval, len: u32, first: u8, span: Span,
)
{
    let lo = i128::from(first);
    let last = lo + i128::from(len) - 1;
    let inside = index.is_bounded() && lo <= index.lo && index.hi <= last;
    do_if!(flow.recording(), {
        tern!(inside, flow.proofs.push(span), flow.doubts.push(span));
    });
    let Some(at) = outside(index, lo, last)
    else
    {
        return;
    };
    let off = at.saturating_sub(i64::from(first));
    report(flow, offset_past_end(span, off, u64::from(len)));
}

/// The end of an interval that lies outside a range, when both ends are
/// known.
///
/// # Arguments
///
/// * `n` - The interval.
/// * `lo` - The least value inside.
/// * `hi` - The greatest value inside.
///
/// # Returns
///
/// The upper end when it passes `hi`, else the lower end when it is below
/// `lo`; `None` when the interval has an end not known or lies inside.
fn outside(n: Interval, lo: i128, hi: i128) -> Option<i64>
{
    ret_if!(!n.is_bounded(), None);
    let end = tern!(n.hi > hi, n.hi, n.lo);
    ret_if!(lo <= end && end <= hi, None);
    Some(i64::try_from(end).unwrap_or(tern!(end < 0, i64::MIN, i64::MAX)))
}

/// Report a subtraction of two pointers that point into different blocks
/// for certain: each side names the blocks it can point into, each of
/// them a block the routine knows on its own, and no block of one side
/// can be a block of the other.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `lhs` - The later pointer.
/// * `rhs` - The earlier pointer.
/// * `span` - The subtraction.
/// * `state` - The facts before it runs.
pub(crate) fn check_ptr_sub(
    flow: &mut Flow, lhs: &Expr, rhs: &Expr, span: Span, state: &State,
)
{
    ret_if!(!flow.recording());
    let (Some(a), Some(b)) = (blocks_of(flow, lhs, state), blocks_of(flow, rhs, state))
    else
    {
        return;
    };
    let shared = a.iter().any(|mine| b.iter().any(|theirs| mine.overlaps(theirs)));
    do_if!(!shared, report(flow, mixed_blocks(span)));
}

/// The blocks a pointer can point into, as paths that each name one
/// block: a slot, the block of an owner, or a path through owners from
/// them. An owner holds its block alone, and each slot is a block of its
/// own, so two such paths that do not overlap name two blocks.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `e` - The pointer.
/// * `state` - The facts.
///
/// # Returns
///
/// The paths, or `None` when one of the blocks is not known, as for a
/// raw address, a parameter, a call result or a load.
fn blocks_of(flow: &Flow, e: &Expr, state: &State) -> Option<Roots>
{
    ret_if!(e.kind.is_raw_ptr(), None);
    match &e.node
    {
        ExprNode::AddrOf(place) => Some(Roots::from([Root::slot(*place)])),
        ExprNode::Read(place) => read_blocks(flow, *place, state),
        ExprNode::PtrAdd { base: inner, .. }
        | ExprNode::Lend { operand: inner }
        | ExprNode::Convert { operand: inner, .. } => blocks_of(flow, inner, state),
        ExprNode::CellAddr { list, .. } if list.kind.is_list() =>
        {
            let ExprNode::Read(place) = list.node
            else
            {
                return None;
            };
            Some(Roots::from([Root::slot(place)]))
        }
        ExprNode::CellAddr { list, .. } => blocks_of(flow, list, state),
        _ => None,
    }
}

/// The blocks a pointer slot can point into: the block of an owner, or
/// the roots of a borrow the routine set itself. A parameter and, in a
/// function, a global point where the caller or another routine made
/// them point, so their blocks are not known here.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `place` - The slot.
/// * `state` - The facts.
///
/// # Returns
///
/// The paths, or `None` when one of the blocks is not known.
fn read_blocks(flow: &Flow, place: Place, state: &State) -> Option<Roots>
{
    let kind = flow.ctx.kind_of(place)?;
    ret_if!(kind.is_owner_ptr(), Some(Roots::from([Root::slot(place).block()])));
    let outer = Root::slot(place).is_param(flow.routine.params)
        || (matches!(place, Place::Global(_)) && !flow.routine.is_main);
    ret_if!(outer || !kind.holds_borrow(), None);
    let roots = &state.ptr(place)?.roots;
    let known = !roots.is_empty() && roots.iter().all(|r| owned_here(flow, r));
    known.then(|| roots.iter().map(block_path).collect())
}

/// Whether a root names a block the routine knows on its own: one that
/// does not stand for what a borrow parameter points at.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `root` - The root.
///
/// # Returns
///
/// `false` for a path through the block of a parameter that is no owner.
fn owned_here(flow: &Flow, root: &Root) -> bool
{
    ret_if!(!root.is_param(flow.routine.params) || root.path.is_empty(), true);
    flow.ctx.kind_of(root.place).is_some_and(|k| k.is_owner_ptr())
}

/// The path of the block a path points into: the path without the cells
/// it ends with, since the cells of a list lie in the block of the list.
///
/// # Arguments
///
/// * `root` - The path.
///
/// # Returns
///
/// The path of the block.
fn block_path(root: &Root) -> Root
{
    let keep = root.path.iter().rposition(|s| *s == Seg::Block).map_or(0, |n| n + 1);
    Root::builder().place(root.place).path(root.path[..keep].to_vec()).build()
}
