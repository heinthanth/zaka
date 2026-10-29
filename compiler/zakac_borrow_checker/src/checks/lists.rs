use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Stmt;
use zakac_span::span::Span;

use crate::cfg::NodeKind;
use crate::checks::loans::is_call_value;
use crate::flow::Flow;
use crate::roots::Roots;
use crate::roots::list_slot;
use crate::state::SlotState;
use crate::state::State;

/// Add what a value stored into a cell of a list of borrows points into
/// to the roots of the list. The list counts as one variable whose roots
/// are those of each of its cells; the store writes one cell and leaves
/// the others as before, so the list keeps the roots it had: a weak set.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `addr` - The address of the store.
/// * `stored` - The value stored, with its roots.
/// * `state` - The facts, changed in place.
pub(crate) fn store_into_cell(
    flow: &Flow, addr: &Expr, stored: &SlotState, state: &mut State,
)
{
    let Some(place) = borrow_list(flow, addr)
    else
    {
        return;
    };
    let added = match stored
    {
        SlotState::Ptr(info) | SlotState::List(info) => info.roots.clone(),
        SlotState::Int(_) | SlotState::None => Roots::new(),
    };
    let Some(info) = state.ptr_mut(place)
    else
    {
        return;
    };
    info.roots.extend(added);
}

/// The list of borrows the cell address of the node that runs stores the
/// result of a call into, with the roots the list had before the node.
/// Those roots remain loans whatever the call renews, since the store
/// writes one cell alone.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `call` - The call.
///
/// # Returns
///
/// The list slot and its roots before the node, or `None` for each other
/// node, value or address.
pub(crate) fn cell_result_target(flow: &Flow, call: Span) -> Option<(Place, Roots)>
{
    let kind = &flow.cfg.nodes[flow.node as usize].kind;
    let NodeKind::Leaf(Stmt::Store { addr, value, .. }) = kind
    else
    {
        return None;
    };
    let direct = is_call_value(value, call);
    let place = borrow_list(flow, addr).filter(|_| direct)?;
    let before = flow.ins.get(flow.node as usize)?.as_ref()?;
    Some((place, before.ptr(place)?.roots.clone()))
}

/// The list slot of borrows a cell address names, through the cells of
/// nested lists.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `addr` - The address.
///
/// # Returns
///
/// The slot, or `None` for an address that is no cell of a list slot of
/// borrows.
fn borrow_list(flow: &Flow, addr: &Expr) -> Option<Place>
{
    let place = list_slot(addr)?;
    let kind = flow.ctx.kind_of(place)?;
    (kind.is_list() && kind.holds_borrow()).then_some(place)
}
