use zakac_diagnostics::ownership::moved_out;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Place;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::cfg::NodeId;
use crate::checks::report;
use crate::checks::slot_name;
use crate::digest::Root;
use crate::digest::Seg;
use crate::flow::Flow;
use crate::flow::is_loop_test;
use crate::flow::leave_moved;
use crate::roots::Roots;
use crate::roots::roots_of;
use crate::roots::take_paths;
use crate::state::State;

/// Report a read of an owner moved out on each path. A test against
/// `ဗလာ` reads the slot without using its value, so it is silent.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `place` - The slot read.
/// * `span` - The read.
/// * `state` - The facts when it runs.
/// * `null_compare` - Whether the read is a test against `ဗလာ`.
pub(crate) fn check_read(
    flow: &mut Flow, place: Place, span: Span, state: &State, null_compare: bool,
)
{
    ret_if!(null_compare);
    let moved = state.ptr(place).is_some_and(|info| info.state.is_moved());
    ret_if!(!moved);
    let name = slot_name(flow, place);
    report(flow, moved_out(span, name));
}

/// Move a value out of the place a [`ExprNode::Take`] names. A slot,
/// checked for a value moved out before, is left moved out; a place in
/// mem is read through its address, and the slots the address can name
/// are not known after it.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `e` - The move.
/// * `state` - The facts, changed in place.
///
/// # Returns
///
/// The paths that lose their block.
pub(crate) fn run_take(flow: &mut Flow, e: &Expr, state: &mut State) -> Roots
{
    let ExprNode::Take { addr } = &e.node
    else
    {
        return Roots::new();
    };
    let paths = take_paths(e, state, &flow.ctx);
    let ExprNode::AddrOf(place) = addr.node
    else
    {
        flow.deref(addr, state);
        take_through(flow, addr, state);
        return paths;
    };
    check_read(flow, place, e.span, state, false);
    leave_moved(place, e.kind, state);
    paths
}

/// Move a value out through an address that is not a slot address. An
/// address that names one slot alone leaves that slot moved out, as a
/// move out of the slot does; one that can name several slots makes them
/// not known, and marks each so a leak check skips it.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `addr` - The address.
/// * `state` - The facts, changed in place.
fn take_through(flow: &Flow, addr: &Expr, state: &mut State)
{
    let roots = roots_of(addr, state, &flow.ctx);
    let slots: Vec<&Root> =
        roots.iter().filter(|r| !r.path.contains(&Seg::Block)).collect();
    let single = match slots.as_slice()
    {
        [root] if root.path.is_empty() => Some(root.place),
        _ => None,
    };
    let kind = single.and_then(|place| flow.ctx.kind_of(place));
    ret_if!(let (Some(place), Some(kind)) = (single, kind), leave_moved(place, kind, state));
    flow.havoc_through(addr, state);
    do_for!(root in slots, flow.mark_given(root.place, state));
}

/// Report each read, in a loop, of an owner that each round moves out
/// and does not fill again, so the next round reads it moved out. Each
/// loop test gets the facts the rounds bring back, without those of the
/// edge into the loop; the flow runs again from there, and the loops
/// report reads of moved-out owners alone.
///
/// # Arguments
///
/// * `flow` - The flow, solved.
pub(crate) fn check_loop_moves(flow: &mut Flow)
{
    let pinned = round_states(flow);
    let heads: Vec<NodeId> =
        (0..pinned.len() as NodeId).filter(|&n| pinned[n as usize].is_some()).collect();
    ret_if!(heads.is_empty());
    let ins = flow.fixpoint(&pinned);
    let inside = loop_nodes(flow, &heads);
    flow.reporting = true;
    flow.moves_alone = true;
    for n in flow.cfg.rpo.clone()
    {
        let state = ins[n as usize].clone().filter(|_| inside[n as usize]);
        do_if!(let Some(state) = state, { flow.transfer(n, state); });
    }
    flow.reporting = false;
    flow.moves_alone = false;
}

/// The facts each loop test gets back from the rounds of its loop.
///
/// # Arguments
///
/// * `flow` - The flow, solved.
///
/// # Returns
///
/// For each node, the join of the out-states of the edges that come back
/// to it; `None` for a node no edge comes back to.
fn round_states(flow: &mut Flow) -> Vec<Option<State>>
{
    let mut back: Vec<Option<State>> = vec![None; flow.cfg.nodes.len()];
    for (s, out) in back_edges(flow)
    {
        let slot = &mut back[s as usize];
        *slot = Some(slot.take().map_or_else(|| out.clone(), |old| old.join(&out)));
    }
    back
}

/// The edges that come back to a loop test, with their out-states.
///
/// # Arguments
///
/// * `flow` - The flow, solved.
///
/// # Returns
///
/// The loop test and the facts on each edge back to it.
fn back_edges(flow: &mut Flow) -> Vec<(NodeId, State)>
{
    let rank = flow.ranks();
    let mut found = Vec::new();
    for n in flow.cfg.rpo.clone()
    {
        let Some(state) = flow.ins[n as usize].clone()
        else
        {
            continue;
        };
        let outs = flow.transfer(n, state);
        let succ = flow.cfg.nodes[n as usize].succ.clone();
        let back = |s: &NodeId| {
            let kind = &flow.cfg.nodes[*s as usize].kind;
            is_loop_test(kind) && rank[*s as usize] <= rank[n as usize]
        };
        found.extend(succ.into_iter().zip(outs).filter(|(s, _)| back(s)));
    }
    found
}

/// The loop tests and the nodes of their loops.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `heads` - The loop tests.
///
/// # Returns
///
/// For each node, whether it is one of the tests or runs inside one of
/// their loops.
fn loop_nodes(flow: &Flow, heads: &[NodeId]) -> Vec<bool>
{
    let mut inside = vec![false; flow.cfg.nodes.len()];
    for &head in heads
    {
        do_for!(n in loop_nodes_of(flow, head), inside[n as usize] = true);
    }
    inside
}

/// The nodes of one loop: its test, and each node its first edge reaches
/// before the test again or the node after the loop.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `head` - The loop test.
///
/// # Returns
///
/// The nodes.
fn loop_nodes_of(flow: &Flow, head: NodeId) -> Vec<NodeId>
{
    let succ = &flow.cfg.nodes[head as usize].succ;
    let (first, after) = (succ[0], succ[1]);
    let mut seen = vec![false; flow.cfg.nodes.len()];
    seen[head as usize] = true;
    seen[after as usize] = true;
    let mut found = vec![head];
    let mut work = vec![first];
    while let Some(n) = work.pop()
    {
        let fresh = !std::mem::replace(&mut seen[n as usize], true);
        do_if!(fresh, {
            found.push(n);
            work.extend(&flow.cfg.nodes[n as usize].succ);
        });
    }
    found
}

#[cfg(test)]
mod tests
{
    use zakac_diagnostics::Code;

    use super::check_loop_moves;
    use crate::flow::Flow;
    use crate::options::BorrowCheckOptions;
    use crate::roots::global_roots;
    use crate::routine::Routine;
    use crate::testing::program;

    #[test]
    fn recording_is_off_outside_the_report_pass_and_in_the_loop_rerun()
    {
        let prog = program("ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။\n");
        let module = &prog.module;
        let roots = global_roots(module, &[]);
        let options = BorrowCheckOptions::default();
        let mut flow =
            Flow::new(module, Routine::main(module), &[], &roots, &options, 64);
        let mut states = Vec::new();
        for (reporting, moves_alone) in [(false, false), (true, false), (true, true)]
        {
            flow.reporting = reporting;
            flow.moves_alone = moves_alone;
            states.push(flow.recording());
        }
        assert_eq!(states, [false, true, false]);
    }

    #[test]
    fn the_loop_rerun_records_reads_of_moved_owners_alone()
    {
        let prog = program(
            "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ဂ တွင် သိမ်းထားမည်။
ကိန်းရှင် ဗ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
က သည် ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ဂ ၏ တန်ဖိုး ကို ပြသပါ။
လိပ်စာ ဂ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ဗ ကို ပ ဟု ထားပါ။
လိပ်စာ ဗ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
က ကို က + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
",
        );
        let module = &prog.module;
        let roots = global_roots(module, &[]);
        let options = BorrowCheckOptions::default();
        let mut flow =
            Flow::new(module, Routine::main(module), &[], &roots, &options, 64);
        flow.solve();
        flow.report_pass();
        let found = flow.diags.len();
        let kept = (flow.proofs.clone(), flow.frees.clone(), flow.result_from.clone());
        let result = (flow.result_roots.clone(), flow.result_roots_ok);
        check_loop_moves(&mut flow);
        let added: Vec<(Code, usize)> =
            flow.diags[found..].iter().map(|d| (d.code, prog.line(d.span))).collect();
        assert_eq!(added, [(Code::MovedOut, 10)]);
        assert_eq!((flow.proofs, flow.frees, flow.result_from), kept);
        assert_eq!((flow.result_roots, flow.result_roots_ok), result);
    }
}
