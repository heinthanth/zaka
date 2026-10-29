use zakac_diagnostics::ownership::Invalidation;
use zakac_diagnostics::ownership::invalidated_loan;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

use crate::cfg::NodeKind;
use crate::checks::lists::cell_result_target;
use crate::checks::report;
use crate::checks::slot_name;
use crate::digest::Root;
use crate::flow::Flow;
use crate::flow::Loss;
use crate::liveness::LiveMap;
use crate::liveness::merge;
use crate::liveness::node_exprs;
use crate::liveness::reads;
use crate::roots::Roots;
use crate::roots::freed_paths;
use crate::state::State;

/// Report a free or a resize of the block at `dest` while a borrow of it
/// is still read later. A slot that holds `ဗလာ` on each path, moved out
/// or never filled, gives nothing back: a resize of it is an allocation.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `dest` - Where the owner lives.
/// * `what` - What happens to the block.
/// * `span` - The statement.
/// * `state` - The facts before it runs.
pub(crate) fn check_heap(
    flow: &mut Flow, dest: &Dest, what: Invalidation, span: Span, state: &State,
)
{
    ret_if!(!flow.recording());
    ret_if!(matches!(dest, Dest::Slot(place) if holds_blank(*place, state)));
    let freed = freed_paths(dest, state, &flow.ctx);
    let loss = Loss::builder().paths(freed).what(what).span(span).build();
    check_invalidation(flow, &loss, state);
}

/// Report a block that loses its owner while a borrow of it is still
/// read after the node that runs: a free, a resize or a move out. Each
/// freed path that starts at a parameter or a global goes into what the
/// function frees, for the checks at its calls. A call whose callee
/// itself still reads the block is reported for that when no later read
/// of the routine is.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `loss` - The paths that lose their block, what happens to it and the
///   operation.
/// * `state` - The facts, which hold the roots of each borrow.
pub(crate) fn check_invalidation(flow: &mut Flow, loss: &Loss, state: &State)
{
    ret_if!(!flow.recording());
    record_frees(flow, &loss.paths);
    let Some((owner, borrow, witness)) = first_hit(flow, loss, state)
    else
    {
        do_if!(let Some(clash) = &loss.clash, report(flow, clash.clone()));
        return;
    };
    let (owner, borrow) = (slot_name(flow, owner), slot_name(flow, borrow));
    let what = loss.what.text();
    report(flow, invalidated_loan(loss.span, owner, &what, borrow, witness));
}

/// Report each block a node took from its owner, through a move out or
/// through a call, while a borrow of the block is still read later. The
/// roots come from the facts after the node, so a borrow the node sets to
/// point elsewhere no longer holds a loan on the block.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `losses` - The blocks that lose their owner.
/// * `state` - The facts after the node.
pub(crate) fn check_losses(flow: &mut Flow, losses: &[Loss], state: &State)
{
    do_for!(loss in losses, check_invalidation(flow, loss, state));
}

/// Whether an owner slot holds `ဗလာ` on each path.
///
/// # Arguments
///
/// * `place` - The slot.
/// * `state` - The facts.
///
/// # Returns
///
/// `true` when each path leaves it `ဗလာ` or moved out.
pub(crate) fn holds_blank(place: Place, state: &State) -> bool
{
    state.ptr(place).is_some_and(|info| info.state.as_read().is_null())
}

/// The first borrow, read after the operation, that points into a freed
/// path. A slot the node sets to the result of the call that lost the
/// block holds no loan on the paths the callee renewed; a list of borrows
/// whose cell the node sets to that result keeps the loans it held before
/// the node.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `loss` - The paths that lose their block and the operation.
/// * `state` - The facts, which hold the roots of each borrow.
///
/// # Returns
///
/// The slot the freed path starts at, the borrow slot and the span of
/// its later read.
fn first_hit(flow: &Flow, loss: &Loss, state: &State) -> Option<(Place, Place, Span)>
{
    let live = live_after(flow, loss.span);
    let target = result_target(flow, loss.span);
    let target = target.or_else(|| cell_result_target(flow, loss.span));
    let (set, kept) = target.unzip();
    let kept = kept.unwrap_or_default();
    live.iter().find_map(|(&borrow, &witness)| {
        let roots = &state.ptr(borrow)?.roots;
        let handed = |r: &&Root| {
            set == Some(borrow) && loss.returned.contains(*r) && !kept.contains(*r)
        };
        let loans: Vec<&Root> = roots.iter().filter(|r| !handed(r)).collect();
        let hit = loss.paths.iter().find(|f| loans.iter().any(|r| f.reaches(r)))?;
        Some((hit.place, borrow, witness))
    })
}

/// The slot the node that runs sets to the result of a call, the value
/// being that call itself, under lends, conversions to a checked pointer
/// and pointer steps at most.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `call` - The call.
///
/// # Returns
///
/// The slot, with no root it keeps, `None` for each other node or value.
fn result_target(flow: &Flow, call: Span) -> Option<(Place, Roots)>
{
    let kind = &flow.cfg.nodes[flow.node as usize].kind;
    let NodeKind::Leaf(Stmt::Set { place, value, .. }) = kind
    else
    {
        return None;
    };
    is_call_value(value, call).then(|| (*place, Roots::new()))
}

/// Whether a value is a call itself, under lends, conversions to a checked
/// pointer and pointer steps at most.
///
/// # Arguments
///
/// * `value` - The value.
/// * `call` - The call.
///
/// # Returns
///
/// `true` when [`pointer_source`] of `value` is the call at `call`.
pub(crate) fn is_call_value(value: &Expr, call: Span) -> bool
{
    let source = pointer_source(value);
    matches!(source.node, ExprNode::Call { .. }) && source.span == call
}

/// The expression whose pointer `e` hands on unchanged in its block:
/// under lends, conversions to a checked pointer and pointer steps. A
/// conversion to an integer or to a raw address hands on nothing the
/// checker follows.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The innermost operand.
pub(crate) fn pointer_source(e: &Expr) -> &Expr
{
    let checked = e.kind.is_checked_ptr();
    match &e.node
    {
        ExprNode::Lend { operand } | ExprNode::PtrAdd { base: operand, .. } =>
        {
            pointer_source(operand)
        }
        ExprNode::Convert { operand, .. } if checked => pointer_source(operand),
        _ => e,
    }
}

/// The borrow slots read after an operation: those live after the node
/// that runs it, and those the node itself reads further on, past the
/// end of the operation.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `span` - The operation.
///
/// # Returns
///
/// The slots, each with the span of its first read after the operation.
fn live_after(flow: &Flow, span: Span) -> LiveMap
{
    let mut live = flow.live.get(flow.node as usize).cloned().unwrap_or_default();
    let mut found = Vec::new();
    let kind = &flow.cfg.nodes[flow.node as usize].kind;
    do_for!(e in node_exprs(kind), reads(e, &mut found));
    let borrow = |p: Place| flow.ctx.kind_of(p).is_some_and(DataKind::holds_borrow);
    let later = found.into_iter().filter(|&(p, at)| at.lo >= span.hi && borrow(p));
    let mut seen = LiveMap::new();
    do_for!((place, at) in later, { seen.entry(place).or_insert(at); });
    merge(&mut live, &seen);
    live
}

/// Keep each freed path that starts at a parameter or a global, for the
/// digest of a function.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `freed` - The paths that lose their block.
fn record_frees(flow: &mut Flow, freed: &Roots)
{
    ret_if!(flow.routine.is_main);
    let params = flow.routine.params;
    let outer = freed.iter().filter(|r| r.is_outer(params));
    flow.frees.extend(outer.cloned());
}
