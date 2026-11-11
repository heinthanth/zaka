use zakac_diagnostics::ownership::local_borrow_returned;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Place;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::checks::report;
use crate::checks::slot_name;
use crate::digest::Root;
use crate::digest::Seg;
use crate::flow::Flow;
use crate::roots::roots_of;
use crate::state::State;
use crate::state::holds_address;

/// Check a value the function gives back, save a read of its result
/// variable, whose sets were checked each.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `value` - The value returned.
/// * `span` - The return.
/// * `state` - The facts before the value runs.
pub(crate) fn check_return(flow: &mut Flow, value: &Expr, span: Span, state: &State)
{
    let reads_result = matches!(value.node, ExprNode::Read(p) if is_result(flow, p));
    do_if!(!reads_result, check_result_roots(flow, value, span, state));
}

/// Check a value set into the result variable, which the function gives
/// back unless a later set replaces it.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `place` - The slot set.
/// * `value` - The value it gets.
/// * `span` - The statement.
/// * `state` - The facts before the value runs.
pub(crate) fn check_result_set(
    flow: &mut Flow, place: Place, value: &Expr, span: Span, state: &State,
)
{
    do_if!(is_result(flow, place), check_result_roots(flow, value, span, state));
}

/// Report an address the function gives back that points into a place
/// that dies with the call: a local or a parameter slot, or a cell of one.
/// An address through the block a parameter points at reaches the caller's
/// places, and an address of a global a place that outlasts the call, so
/// each such path goes into the digest as one the result can point into.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `value` - The value given back.
/// * `span` - The statement.
/// * `state` - The facts before the value runs.
pub(crate) fn check_result_roots(
    flow: &mut Flow, value: &Expr, span: Span, state: &State,
)
{
    ret_if!(!flow.recording());
    ret_if!(!flow.routine.ret.is_some_and(holds_address));
    let roots = roots_of(value, state, &flow.ctx);
    let lent: Vec<u32> = roots.iter().filter_map(|r| lent_param(flow, r)).collect();
    flow.result_from.extend(lent);
    let outlasts = |r: &&Root| r.is_global() || lent_param(flow, r).is_some();
    let kept: Vec<Root> = roots.iter().filter(outlasts).cloned().collect();
    flow.result_roots.extend(kept);
    let dead = roots.iter().find(|r| dies(r));
    let Some(dead) = dead
    else
    {
        return;
    };
    flow.result_roots_ok = false;
    let name = slot_name(flow, dead.place);
    report(flow, local_borrow_returned(span, name));
}

/// Whether a slot is the result variable of the routine.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `place` - The slot.
///
/// # Returns
///
/// `true` for the result variable.
fn is_result(flow: &Flow, place: Place) -> bool
{
    flow.routine.result.is_some_and(|n| place == Place::Local(n))
}

/// The parameter a path reaches the caller's places through: it starts at
/// the parameter and goes through the block it points at.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `root` - The path.
///
/// # Returns
///
/// The parameter number, or `None` for each other path.
fn lent_param(flow: &Flow, root: &Root) -> Option<u32>
{
    let Place::Local(n) = root.place
    else
    {
        return None;
    };
    ret_if!(n >= flow.routine.params, None);
    root.path.contains(&Seg::Block).then_some(n)
}

/// Whether a path leads into a place that dies with the call: it starts
/// at a local slot and takes no step into a block, which a local can hold
/// from the heap or from the caller.
///
/// # Arguments
///
/// * `root` - The path.
///
/// # Returns
///
/// `true` when the place ends when the function returns.
fn dies(root: &Root) -> bool
{
    matches!(root.place, Place::Local(_)) && !root.path.contains(&Seg::Block)
}
