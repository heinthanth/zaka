use zakac_diagnostics::ownership::bad_free;
use zakac_diagnostics::ownership::borrow_stored;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::Place;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::checks::report;
use crate::digest::Root;
use crate::digest::Seg;
use crate::flow::Flow;
use crate::roots::list_slot;
use crate::roots::roots_of;
use crate::state::SlotState;
use crate::state::State;

/// Report a free or a resize through a pointer that holds the address of
/// a variable on each path, not of a heap block: each place it can point at
/// is a slot, or a cell or field the slot holds in place, with no step into a
/// block on the path, as `x ၏ လိပ်စာ` and `x ၏ y ၏ လိပ်စာ` give it.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `dest` - The pointer freed or resized.
/// * `span` - The statement.
/// * `state` - The facts before it runs.
///
/// # Returns
///
/// `true` when the statement is rejected.
pub(crate) fn check_free_dest(
    flow: &mut Flow, dest: &Dest, span: Span, state: &State,
) -> bool
{
    let Dest::Slot(place) = dest
    else
    {
        return false;
    };
    let SlotState::Ptr(info) = state.get(*place)
    else
    {
        return false;
    };
    let in_variable = |r: &Root| !r.path.contains(&Seg::Block);
    let rejected = !info.roots.is_empty() && info.roots.iter().all(in_variable);
    do_if!(rejected, report(flow, bad_free(span)));
    rejected
}

/// Report an address of a variable of the function stored through an
/// address: in a heap block or behind a pointer it could outlast the call.
/// A cell of a list slot is that slot, so it follows the rule of a set of
/// the slot.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `addr` - The address written to.
/// * `value` - The value stored.
/// * `span` - The statement.
/// * `state` - The facts before the store.
pub(crate) fn check_stored_borrow(
    flow: &mut Flow, addr: &Expr, value: &Expr, span: Span, state: &State,
)
{
    ret_if!(!points_into_local(flow, value, state));
    let Some(place) = list_slot(addr)
    else
    {
        return report(flow, borrow_stored(span));
    };
    check_global_set(flow, place, value, span, state);
}

/// Report an address of a variable of the function stored in a global from a
/// function: the global outlasts the call, and the variable dies with it.
/// The main program keeps its variables for the whole run.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `place` - The slot set.
/// * `value` - The value it gets.
/// * `span` - The statement.
/// * `state` - The facts before the set.
pub(crate) fn check_global_set(
    flow: &mut Flow, place: Place, value: &Expr, span: Span, state: &State,
)
{
    ret_if!(flow.routine.is_main || matches!(place, Place::Local(_)));
    do_if!(points_into_local(flow, value, state), report(flow, borrow_stored(span)));
}

/// Whether a value can point at a variable of the function: a slot of its
/// own, or a cell or field the slot holds in place, with no step into a
/// block on the path. The constant null points nowhere.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `value` - The value.
/// * `state` - The facts where it is read.
///
/// # Returns
///
/// `true` when storing the value can keep the address of a variable.
fn points_into_local(flow: &Flow, value: &Expr, state: &State) -> bool
{
    ret_if!(!value.kind.holds_address(), false);
    let local =
        |r: &Root| matches!(r.place, Place::Local(_)) && !r.path.contains(&Seg::Block);
    roots_of(value, state, &flow.ctx).iter().any(local)
}
