use zakac_diagnostics::Level;
use zakac_diagnostics::ownership::block_leaked;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Place;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

use crate::checks::report;
use crate::checks::slot_name;
use crate::digest::Root;
use crate::flow::Flow;
use crate::state::OwnerState;
use crate::state::SlotState;
use crate::state::State;

/// Report a set or an allocation into an owner that still holds a block:
/// that block is lost. It applies to each owner slot, in functions and
/// in the main program alike. The slot must hold a block on each path,
/// or hold an allocation of this routine on some path: a slot that is
/// just not known (a parameter, a global in a function, a call result,
/// a slot a call can write) is silent, since the check cannot tell
/// whether its block went elsewhere; so is a slot whose block a call can
/// have given back.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `place` - The slot written.
/// * `span` - The statement.
/// * `state` - The facts after the new value runs, before it is stored.
pub(crate) fn check_overwrite(flow: &mut Flow, place: Place, span: Span, state: &State)
{
    let lost = |s: OwnerState| !s.shares_given() && (s.shares_fresh() || s.must_hold());
    check_held(flow, place, span, state, lost);
}

/// Report each owner of a function that can still hold a block when the
/// function returns, save one whose block a call can have given back.
/// The main program is not checked: its blocks live to the end of the
/// run, which `--leaks` reports.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `span` - The exit: a return or the end.
/// * `state` - The facts there, after a value given back moved out.
pub(crate) fn check_exit(flow: &mut Flow, span: Span, state: &State)
{
    ret_if!(flow.routine.is_main);
    let count = flow.routine.locals.len() as u32;
    let held = |s: OwnerState| s.can_hold() && !s.shares_given();
    do_for!(place in (0..count).map(Place::Local), {
        check_held(flow, place, span, state, held);
    });
}

/// The span a leak at a return goes to: the end of the function for the
/// return the compiler adds after the last statement, which reads the
/// result variable under the span of the return itself, or which ends where
/// the function ends: no written return reaches past the closing words.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `value` - The value given back.
/// * `span` - The return.
///
/// # Returns
///
/// The span.
pub(crate) fn return_span(flow: &Flow, value: Option<&Expr>, span: Span) -> Span
{
    let result = flow.routine.result.map(Place::Local);
    let added = value.is_some_and(|v| {
        matches!(v.node, ExprNode::Read(p) if Some(p) == result) && v.span == span
    });
    let at_end = span.hi == flow.routine.span.hi;
    tern!(added || at_end, flow.routine.span.shrink_to_hi(), span)
}

/// Report an owner slot whose state loses a block, as `lost` tells. A list
/// or a struct with owners in it is reported under strict alone, as a
/// warning: its cell numbers are not followed, so the check cannot tell
/// which cells still hold a block. A plain list of owners is the one
/// exception, see [`report_cells`]. Text is dropped as its scope ends, so
/// it is never lost, but an owner beside it in a struct or a list still is.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `place` - The slot.
/// * `span` - Where the block is lost.
/// * `state` - The facts there.
/// * `lost` - Whether an owner in a state loses its block there.
fn check_held(
    flow: &mut Flow, place: Place, span: Span, state: &State,
    lost: impl Fn(OwnerState) -> bool,
)
{
    let Some(kind) = flow.ctx.kind_of(place)
    else
    {
        return;
    };
    ret_if!(owner_cells(kind), {
        do_if!(let Some(info) = state.ptr(place), {
            report_cells(flow, place, span, info.cells, lost);
        });
    });
    let owner = kind.is_owner_ptr();
    let list = kind.is_aggregate() && kind.holds_owner_ptr() && flow.options.strict;
    ret_if!(!owner && !list);
    ret_if!(!state.ptr(place).is_some_and(|info| lost(info.state)));
    let found = block_leaked(span, slot_name(flow, place));
    report(flow, tern!(owner, found, found.with_level(Level::Warning)));
}

/// Report the cells of a plain list of owners where the cells lose a block.
/// When each cell holds a block for certain, the one lost does too: an error in
/// each mode. When a cell just can hold one, as `lost` tells, the check
/// cannot tell which cell, so it is a warning under strict alone.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `place` - The list slot.
/// * `span` - Where the block is lost.
/// * `cells` - What the cells hold there.
/// * `lost` - Whether a cell in a state loses its block there.
fn report_cells(
    flow: &mut Flow, place: Place, span: Span, cells: OwnerState,
    lost: impl Fn(OwnerState) -> bool,
)
{
    let sure = !cells.shares_given() && cells.must_hold();
    ret_if!(!sure && !(flow.options.strict && lost(cells)));
    let found = block_leaked(span, slot_name(flow, place));
    report(flow, tern!(sure, found, found.with_level(Level::Warning)));
}

/// Check a remove from a plain list slot of owners: the last cell goes, with
/// the block it can hold. The cells left can then be `ဗလာ`, so a later loss
/// of the same list is not reported twice as certain.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `dest` - The list.
/// * `span` - The remove.
/// * `state` - The facts, changed in place.
pub(crate) fn check_removed(flow: &mut Flow, dest: &Dest, span: Span, state: &mut State)
{
    let Some(place) = owner_list(flow, dest)
    else
    {
        return;
    };
    let Some(info) = state.ptr_mut(place)
    else
    {
        return;
    };
    let cells = info.cells;
    info.cells = cells.join(OwnerState::NULL);
    report_cells(flow, place, span, cells, |s| s.can_hold() && !s.shares_given());
}

/// Note the owner an add puts in a plain list slot of owners.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `dest` - The list.
/// * `added` - The value added.
/// * `state` - The facts, changed in place.
pub(crate) fn add_owner(flow: &Flow, dest: &Dest, added: &SlotState, state: &mut State)
{
    let held = match added
    {
        SlotState::Ptr(info) => info.state,
        _ => OwnerState::UNKNOWN,
    };
    let info = owner_list(flow, dest).and_then(|place| state.ptr_mut(place));
    do_if!(let Some(info) = info, info.cells = info.cells.join(held));
}

/// Make the cells of each plain list of owners a write or a call can reach
/// not known: the check does not follow which cell it changes.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `roots` - What the write or the call can reach.
/// * `state` - The facts, changed in place.
pub(crate) fn blur_cells<'a>(
    flow: &Flow, roots: impl IntoIterator<Item = &'a Root>, state: &mut State,
)
{
    let kind = |r: &&Root| flow.ctx.kind_of(r.place).is_some_and(owner_cells);
    for root in roots.into_iter().filter(kind)
    {
        let info = state.ptr_mut(root.place);
        do_if!(let Some(info) = info, {
            info.cells = info.cells.join(OwnerState::UNKNOWN);
        });
    }
}

/// The plain list slot of owners an add or a remove edits.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `dest` - The list.
///
/// # Returns
///
/// The slot, or `None` for a list through an address or of other cells.
fn owner_list(flow: &Flow, dest: &Dest) -> Option<Place>
{
    let Dest::Slot(place) = dest
    else
    {
        return None;
    };
    flow.ctx.kind_of(*place).is_some_and(owner_cells).then_some(*place)
}

/// Whether a kind is a plain list whose cells are owner pointers.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` for such a list.
fn owner_cells(kind: DataKind) -> bool
{
    kind.as_seq().is_some_and(|s| s.cell.is_owner_ptr())
}
