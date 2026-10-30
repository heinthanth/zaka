use zakac_diagnostics::Level;
use zakac_diagnostics::ownership::block_leaked;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Place;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::checks::report;
use crate::checks::slot_name;
use crate::flow::Flow;
use crate::state::OwnerState;
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
/// result variable under the span of the return itself.
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
    tern!(added, flow.routine.span.shrink_to_hi(), span)
}

/// Report an owner slot whose state loses a block, as `lost` tells. A
/// list of owners is
/// reported under strict alone, as a warning: its cell numbers are not
/// followed, so the check cannot tell which cells still hold a block.
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
    let owner = kind.is_owner_ptr();
    let list = kind.is_aggregate() && kind.is_move_kind() && flow.options.strict;
    ret_if!(!owner && !list);
    ret_if!(!state.ptr(place).is_some_and(|info| lost(info.state)));
    let found = block_leaked(span, slot_name(flow, place));
    report(flow, tern!(owner, found, found.with_level(Level::Warning)));
}
