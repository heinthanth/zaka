use std::collections::BTreeMap;
use std::collections::BTreeSet;

use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::callgraph::callees;
use crate::cfg::Cfg;
use crate::cfg::NodeKind;
use crate::routine::Routine;
use crate::walk::each_expr;
use crate::walk::each_stmt;
use crate::walk::stmt_exprs;

/// The borrow slots that can still be read, each with the span of a read
/// that follows.
pub(crate) type LiveMap = BTreeMap<Place, Span>;

/// The borrow slots live after each node: a backward fixpoint, with the
/// earliest read in the source as the witness where paths meet.
///
/// # Arguments
///
/// * `cfg` - The graph of the routine.
/// * `module` - The module.
/// * `routine` - The routine.
///
/// # Returns
///
/// The live slots after each node, in node order.
pub(crate) fn live_out(cfg: &Cfg, module: &Module, routine: &Routine) -> Vec<LiveMap>
{
    let slots = Slots { module, routine, globals: global_reads(module) };
    let order = cfg.postorder();
    let mut outs = vec![LiveMap::new(); cfg.nodes.len()];
    let mut ins = vec![LiveMap::new(); cfg.nodes.len()];
    let mut changed = true;
    while changed
    {
        changed = false;
        for &n in &order
        {
            let node = &cfg.nodes[n as usize];
            let mut out = LiveMap::new();
            do_for!(&s in &node.succ, merge(&mut out, &ins[s as usize]));
            let live_in = slots.live_in(&node.kind, &out);
            changed |= live_in != ins[n as usize];
            ins[n as usize] = live_in;
            outs[n as usize] = out;
        }
    }
    outs
}

/// Add the live slots of a successor, keeping the earlier witness.
///
/// # Arguments
///
/// * `into` - The live slots so far.
/// * `from` - The live slots of the successor.
pub(crate) fn merge(into: &mut LiveMap, from: &LiveMap)
{
    for (&place, &span) in from
    {
        let later = into.get(&place).is_some_and(|old| old.lo <= span.lo);
        do_if!(!later, {
            into.insert(place, span);
        });
    }
}

/// What the liveness reads of the slots of a routine.
struct Slots<'m>
{
    /// The module.
    module: &'m Module,
    /// The routine.
    routine: &'m Routine<'m>,
    /// The global borrows each function reads, itself or through its
    /// callees.
    globals: Vec<BTreeSet<u32>>,
}

impl Slots<'_>
{
    /// Whether the liveness follows a slot: one that holds a borrow.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// `true` for a borrow slot or a list of borrows.
    fn tracked(&self, place: Place) -> bool
    {
        let slot = match place
        {
            Place::Global(n) => self.module.globals.get(n as usize),
            Place::Local(n) => self.routine.locals.get(n as usize),
        };
        slot.is_some_and(|s| s.kind.holds_borrow())
    }

    /// The live slots before a node.
    ///
    /// # Arguments
    ///
    /// * `kind` - What the node runs.
    /// * `out` - The live slots after it.
    ///
    /// # Returns
    ///
    /// The slots after it less the one it sets, plus those it reads, each
    /// read its own witness.
    fn live_in(&self, kind: &NodeKind, out: &LiveMap) -> LiveMap
    {
        let mut live = out.clone();
        do_if!(let NodeKind::Leaf(Stmt::Set { place, .. }) = kind, { live.remove(place); });
        for (place, span) in self.uses(kind)
        {
            do_if!(self.tracked(place), {
                live.insert(place, span);
            });
        }
        live
    }

    /// The slots a node reads, with the span of each read. A call reads
    /// the global borrows its function reads.
    ///
    /// # Arguments
    ///
    /// * `kind` - What the node runs.
    ///
    /// # Returns
    ///
    /// The reads.
    fn uses(&self, kind: &NodeKind) -> Vec<(Place, Span)>
    {
        let exprs = node_exprs(kind);
        let mut out: Vec<(Place, Span)> = heap_slot(kind).into_iter().collect();
        do_for!(e in &exprs, reads(e, &mut out));
        let mut calls = Vec::new();
        do_if!(let NodeKind::Leaf(Stmt::Call { func, span, .. }) = kind, calls.push((*func, *span)));
        do_for!(e in &exprs, each_expr(e, &mut |sub| calls.extend(call_of(sub))));
        for (func, span) in calls
        {
            let globals = self.globals.get(func as usize).into_iter().flatten();
            out.extend(globals.map(|&g| (Place::Global(g), span)));
        }
        out
    }
}

/// The expressions a node runs.
///
/// # Arguments
///
/// * `kind` - What the node runs.
///
/// # Returns
///
/// The expressions, in the order the node runs them.
pub(crate) fn node_exprs<'m>(kind: &NodeKind<'m>) -> Vec<&'m Expr>
{
    match kind
    {
        NodeKind::Leaf(s) | NodeKind::CountedInit(s) | NodeKind::CountedHead(s) =>
        {
            stmt_exprs(s)
        }
        NodeKind::Branch(e) | NodeKind::LoopHead(e) => vec![e],
        NodeKind::Start | NodeKind::End | NodeKind::Exit => Vec::new(),
    }
}

/// The slot a free or a resize reads the address of.
///
/// # Arguments
///
/// * `kind` - What the node runs.
///
/// # Returns
///
/// The slot and the statement, for a free or a resize of a slot.
fn heap_slot(kind: &NodeKind) -> Option<(Place, Span)>
{
    let NodeKind::Leaf(s) = kind
    else
    {
        return None;
    };
    match s
    {
        Stmt::Free { dest: Dest::Slot(place), span }
        | Stmt::Resize { dest: Dest::Slot(place), span, .. } => Some((*place, *span)),
        _ => None,
    }
}

/// The function and the span of a call expression.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The call, or `None` for another expression.
fn call_of(e: &Expr) -> Option<(u32, Span)>
{
    tern!(let ExprNode::Call { func, .. } = &e.node, Some((*func, e.span)), None)
}

/// The global borrows each function reads, itself or through the
/// functions it calls, to a fixpoint.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The global numbers, in function order.
pub(crate) fn global_reads(module: &Module) -> Vec<BTreeSet<u32>>
{
    let is_borrow =
        |g: u32| module.globals.get(g as usize).is_some_and(|s| s.kind.holds_borrow());
    let direct = |stmts: &[Stmt]| {
        let mut found = Vec::new();
        each_stmt(stmts, &mut |s| do_for!(e in stmt_exprs(s), reads(e, &mut found)));
        let globals = found.into_iter().filter_map(|(place, _)| global_of(place));
        globals.filter(|&g| is_borrow(g)).collect::<BTreeSet<u32>>()
    };
    let mut sets: Vec<BTreeSet<u32>> =
        module.funcs.iter().map(|f| direct(&f.block)).collect();
    let calls: Vec<BTreeSet<u32>> = module
        .funcs
        .iter()
        .map(|f| {
            let mut out = BTreeSet::new();
            callees(&f.block, &mut out);
            out
        })
        .collect();
    let mut changed = true;
    while changed
    {
        changed = false;
        for (f, callees) in calls.iter().enumerate()
        {
            let more: BTreeSet<u32> =
                callees.iter().flat_map(|&c| sets[c as usize].clone()).collect();
            let before = sets[f].len();
            sets[f].extend(more);
            changed |= sets[f].len() != before;
        }
    }
    sets
}

/// The global number of a place.
///
/// # Arguments
///
/// * `place` - The place.
///
/// # Returns
///
/// The number of a global, `None` for a local.
fn global_of(place: Place) -> Option<u32>
{
    tern!(let Place::Global(g) = place, Some(g), None)
}

/// Add each slot an expression reads, with the span of the read.
///
/// # Arguments
///
/// * `e` - The expression.
/// * `out` - The reads so far.
pub(crate) fn reads(e: &Expr, out: &mut Vec<(Place, Span)>)
{
    each_expr(
        e,
        &mut |sub| do_if!(let ExprNode::Read(place) = sub.node, out.push((place, sub.span))),
    );
}

#[cfg(test)]
mod tests
{
    use zakac_ir_builder::ir::Stmt;

    use super::*;
    use crate::cfg::NodeKind;
    use crate::cfg::build_cfg;
    use crate::testing::program;

    #[test]
    fn borrow_read_after_free_is_live_with_witness()
    {
        let prog = program(
            "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် မိတ္တူ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
မိတ္တူ ၏ တန်ဖိုး ကို ပြသပါ။
",
        );
        let routine = Routine::main(&prog.module);
        let cfg = build_cfg(routine.block, routine.span);
        let live = live_out(&cfg, &prog.module, &routine);
        let free = cfg
            .nodes
            .iter()
            .position(|n| matches!(n.kind, NodeKind::Leaf(Stmt::Free { .. })));
        let after_free = &live[free.expect("the free is a node")];
        let witness = after_free.get(&Place::Global(1)).expect("the borrow is live");
        assert_eq!(prog.line(*witness), 5);
        assert!(!after_free.contains_key(&Place::Global(0)));
        let print = cfg
            .nodes
            .iter()
            .position(|n| matches!(n.kind, NodeKind::Leaf(Stmt::Print { .. })));
        assert!(live[print.unwrap()].is_empty());
    }

    #[test]
    fn a_set_ends_the_live_range_and_a_loop_carries_it_back()
    {
        let prog = program(
            "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၅ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် ညွှန်း သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ကာ ညွှန်း သည် ပ + ၅ အောက် \
             ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ညွှန်း ၏ တန်ဖိုး ကို ပြသပါ။
ပြီးလျှင် ညွှန်း ကို ညွှန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ပ လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
",
        );
        let routine = Routine::main(&prog.module);
        let cfg = build_cfg(routine.block, routine.span);
        let live = live_out(&cfg, &prog.module, &routine);
        let at = |pick: &dyn Fn(&NodeKind) -> bool| {
            cfg.nodes
                .iter()
                .position(|n| pick(&n.kind))
                .expect("the node is in the graph")
        };
        let walker = Place::Global(1);
        let first_set = at(
            &|k| matches!(k, NodeKind::Leaf(Stmt::Set { place, .. }) if *place == walker),
        );
        let head = at(&|k| matches!(k, NodeKind::LoopHead(_)));
        let free = at(&|k| matches!(k, NodeKind::Leaf(Stmt::Free { .. })));
        assert!(live[first_set].contains_key(&walker));
        assert!(live[head].contains_key(&walker));
        assert!(!live[free].contains_key(&walker));
        let alloc = at(&|k| matches!(k, NodeKind::Leaf(Stmt::Alloc { .. })));
        assert!(!live[alloc].contains_key(&walker));
    }
}
