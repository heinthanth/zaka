use zakac_diagnostics::ownership::null_deref;
use zakac_diagnostics::ownership::null_deref_possible;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Place;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::checks::report;
use crate::checks::slot_name;
use crate::flow::Flow;
use crate::state::State;

/// Report a dereference through a pointer slot that is `ဗလာ` on each
/// path, and, under strict, through one that can hold an allocation not
/// compared with `ဗလာ`. A slot that is not known (a parameter, a global
/// in a function or after a call, a call result) or that is `ဗလာ` on
/// some paths is silent: warning there would flag most code. A slot moved out
/// on each path is left to the move check.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `addr` - The address read or written through.
/// * `state` - The facts when it runs.
pub(crate) fn check_deref(flow: &mut Flow, addr: &Expr, state: &State)
{
    let Some(place) = deref_slot(addr)
    else
    {
        return;
    };
    let Some(info) = state.ptr(place)
    else
    {
        return;
    };
    ret_if!(info.state.is_moved());
    let seen = info.state.as_read();
    let name = slot_name(flow, place);
    ret_if!(seen.is_null(), report(flow, null_deref(addr.span, name)));
    let warn = flow.options.strict && seen.shares_fresh();
    do_if!(warn, report(flow, null_deref_possible(addr.span, name)));
}

/// The pointer slot an address comes from.
///
/// # Arguments
///
/// * `addr` - The address.
///
/// # Returns
///
/// The slot read, under conversions, lends and offsets, which keep
/// whether the address is `ဗလာ`.
fn deref_slot(addr: &Expr) -> Option<Place>
{
    match &addr.node
    {
        ExprNode::Read(place) => Some(*place),
        ExprNode::Convert { operand, .. }
        | ExprNode::Lend { operand }
        | ExprNode::PtrAdd { base: operand, .. } => deref_slot(operand),
        _ => None,
    }
}
