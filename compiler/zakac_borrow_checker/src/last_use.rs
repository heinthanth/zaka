use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::ownership::duplicate_made;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Func;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

use crate::callgraph::callees;
use crate::cfg::NodeKind;
use crate::cfg::build_cfg;
use crate::liveness::live_out_of;
use crate::liveness::node_exprs;
use crate::liveness::reads;
use crate::routine::Routine;
use crate::walk::each_expr;
use crate::walk::each_expr_mut;
use crate::walk::each_stmt;
use crate::walk::each_stmt_mut;
use crate::walk::stmt_exprs;

/// A duplicate in a routine: the span of the read and the place it reads.
type DupAt = (Span, Place);

/// Turn each duplicate of a text place that the program reads no more into a
/// move, so the text changes owner and nothing is copied. A duplicate remains
/// when the place is a parameter, whose cells can belong to the caller; a
/// global read in a function, which a later call can read; a global a call
/// of the same statement writes, whose block the check would see freed
/// under the moved text; a place that a lend, a range or an address reads,
/// so a move never ends a live view; or a place the same statement reads
/// again.
///
/// # Arguments
///
/// * `module` - The module, changed in place.
pub(crate) fn move_last_uses(module: &mut Module)
{
    let mut lent = lent_in(&module.main);
    do_for!(f in &module.funcs, lent.extend(lent_in(&f.block)));
    let lent_globals: BTreeSet<Place> =
        lent.into_iter().filter(|p| matches!(p, Place::Global(_))).collect();
    let writes = global_writes(module);
    let main = moves_in(module, &Routine::main(module), &lent_globals, &writes);
    let funcs: Vec<HashSet<DupAt>> = module
        .funcs
        .iter()
        .map(|f| moves_in(module, &Routine::func(f), &lent_globals, &writes))
        .collect();
    rewrite(&mut module.main, &main);
    do_for!((f, moves) in module.funcs.iter_mut().zip(&funcs), rewrite(&mut f.block, moves));
}

/// The duplicates of one routine that can become moves.
///
/// # Arguments
///
/// * `module` - The module.
/// * `routine` - The routine.
/// * `lent_globals` - The globals a lend, a range or an address reads in some
///   routine.
/// * `writes` - The globals each function writes, at each depth.
///
/// # Returns
///
/// The duplicates, each with the span of its read and its place.
fn moves_in(
    module: &Module, routine: &Routine, lent_globals: &BTreeSet<Place>,
    writes: &[BTreeSet<u32>],
) -> HashSet<DupAt>
{
    let cfg = build_cfg(routine.block, routine.span);
    let live = live_out_of(&cfg, module, routine, DataKind::holds_text);
    let lent = lent_in(routine.block);
    let mut verdicts: HashMap<DupAt, bool> = HashMap::new();
    for (node, out) in cfg.nodes.iter().zip(&live)
    {
        let exprs = node_exprs(&node.kind);
        let mut read = Vec::new();
        do_for!(e in &exprs, reads(e, &mut read));
        let written = call_writes(&node.kind, &exprs, writes);
        let mut judge = |e: &Expr| {
            let Some(place) = dup_of(e)
            else
            {
                return;
            };
            let once = read.iter().filter(|(p, _)| *p == place).count() == 1;
            let kept = lent.contains(&place) || lent_globals.contains(&place);
            let kept =
                kept || matches!(place, Place::Global(g) if written.contains(&g));
            let ok =
                once && movable(routine, place) && !kept && !out.contains_key(&place);
            verdicts.entry((e.span, place)).and_modify(|v| *v &= ok).or_insert(ok);
        };
        do_for!(e in &exprs, each_expr(e, &mut judge));
    }
    verdicts.into_iter().filter(|(_, ok)| *ok).map(|(at, _)| at).collect()
}

/// The globals the calls of a node write, at each depth.
///
/// # Arguments
///
/// * `kind` - What the node runs.
/// * `exprs` - The expressions it runs.
/// * `writes` - The globals each function writes, at each depth.
///
/// # Returns
///
/// The global numbers.
fn call_writes(
    kind: &NodeKind, exprs: &[&Expr], writes: &[BTreeSet<u32>],
) -> BTreeSet<u32>
{
    let mut called = BTreeSet::new();
    do_if!(let NodeKind::Leaf(Stmt::Call { func, .. }) = kind, { called.insert(*func); });
    let mut note = |e: &Expr| do_if!(let ExprNode::Call { func, .. } = e.node, { called.insert(func); });
    do_for!(e in exprs, each_expr(e, &mut note));
    called.iter().filter_map(|f| writes.get(*f as usize)).flatten().copied().collect()
}

/// Whether a move out of a place can stand for a duplicate in a routine: a
/// local that is no parameter, or a global of the main program.
///
/// # Arguments
///
/// * `routine` - The routine.
/// * `place` - The place.
///
/// # Returns
///
/// `true` if it can.
fn movable(routine: &Routine, place: Place) -> bool
{
    match place
    {
        Place::Local(n) => n >= routine.params,
        Place::Global(_) => routine.is_main,
    }
}

/// The places a lend, a range or an address reads in a list of statements,
/// nested blocks included. The address under a move is no such read: the
/// move ends the place, so no view of it outlives the move.
///
/// # Arguments
///
/// * `stmts` - The statements.
///
/// # Returns
///
/// The places.
fn lent_in(stmts: &[Stmt]) -> BTreeSet<Place>
{
    let mut found = BTreeSet::new();
    let mut addrs: HashMap<(Span, Place), i32> = HashMap::new();
    let mut note = |e: &Expr| match &e.node
    {
        ExprNode::AddrOf(place) => *addrs.entry((e.span, *place)).or_default() += 1,
        ExprNode::Take { addr } =>
        {
            let moved = tern!(let ExprNode::AddrOf(p) = addr.node, Some(p), None);
            do_if!(let Some(p) = moved, *addrs.entry((addr.span, p)).or_default() -= 1);
        }
        ExprNode::Lend { operand } | ExprNode::MakeSlice { source: operand, .. } =>
        {
            let mut inside = Vec::new();
            reads(operand, &mut inside);
            found.extend(inside.into_iter().map(|(place, _)| place));
        }
        _ => (),
    };
    each_stmt(stmts, &mut |s| do_for!(e in stmt_exprs(s), each_expr(e, &mut note)));
    found.extend(addrs.into_iter().filter(|(_, n)| *n > 0).map(|((_, p), _)| p));
    found
}

/// The place a duplicate reads, when it reads a whole place.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The place, or `None` for each other expression.
fn dup_of(e: &Expr) -> Option<Place>
{
    let ExprNode::Dup { value, .. } = &e.node
    else
    {
        return None;
    };
    tern!(let ExprNode::Read(place) = value.node, Some(place), None)
}

/// Turn the chosen duplicates of a list of statements into moves.
///
/// # Arguments
///
/// * `stmts` - The statements, changed in place.
/// * `moves` - The duplicates to turn.
fn rewrite(stmts: &mut [Stmt], moves: &HashSet<DupAt>)
{
    ret_if!(moves.is_empty());
    let mut turn = |e: &mut Expr| {
        let moved = dup_of(e).is_some_and(|place| moves.contains(&(e.span, place)));
        do_if!(moved, *e = take_of(e));
    };
    each_stmt_mut(stmts, &mut |s| each_expr_mut(s, &mut turn));
}

/// The move that stands for a duplicate of a whole place.
///
/// # Arguments
///
/// * `e` - The duplicate.
///
/// # Returns
///
/// The move out of the place, of the same kind.
fn take_of(e: &Expr) -> Expr
{
    let place = dup_of(e).expect("a duplicate of a whole place");
    let at = ExprNode::AddrOf(place);
    let addr = Expr::builder().kind(e.kind.pointer_to()).node(at).span(e.span).build();
    let node = ExprNode::Take { addr: Box::new(addr) };
    Expr::builder().kind(e.kind).node(node).span(e.span).build()
}

/// Report each duplicate left in a module, for `--warn-duplicates`.
///
/// # Arguments
///
/// * `module` - The module, after [`move_last_uses`].
///
/// # Returns
///
/// One `ZK0720` warning for each duplicate.
pub(crate) fn duplicate_reports(module: &Module) -> Vec<Diagnostic>
{
    let mut found = Vec::new();
    let mut scan = |stmts: &[Stmt], locals: &[zakac_ir_builder::ir::Slot]| {
        let mut note = |e: &Expr| {
            let ExprNode::Dup { value, lent: false } = &e.node
            else
            {
                return;
            };
            let name = root_name(value, module, locals);
            found.push(duplicate_made(e.span, &name));
        };
        each_stmt(stmts, &mut |s| do_for!(e in stmt_exprs(s), each_expr(e, &mut note)));
    };
    scan(&module.main, &[]);
    do_for!(f in &module.funcs, scan(&f.block, &f.locals));
    found
}

/// The name of the place a value is read out of: the variable itself, or
/// the variable whose field or cell it is.
///
/// # Arguments
///
/// * `value` - The value read.
/// * `module` - The module.
/// * `locals` - The local slots of the routine.
///
/// # Returns
///
/// The name, or `?` when no slot is found.
fn root_name(
    value: &Expr, module: &Module, locals: &[zakac_ir_builder::ir::Slot],
) -> String
{
    let mut places = Vec::new();
    each_expr(value, &mut |sub| match sub.node
    {
        ExprNode::Read(place) | ExprNode::AddrOf(place) => places.push(place),
        _ => (),
    });
    let slot = places.first().and_then(|place| match place
    {
        Place::Global(n) => module.globals.get(*n as usize),
        Place::Local(n) => locals.get(*n as usize),
    });
    slot.map_or_else(|| "?".to_string(), |s| s.name.clone())
}

/// Lend each text argument a call does not take: the parameter reads the
/// caller's u8s with a room of 0 for the length of the call, and nothing is
/// copied unless the callee keeps the text. A duplicate remains when the
/// call, or a call inside its arguments, can change the source while the
/// loan lives: a global one of them writes, or a place whose address or a
/// view of it is taken somewhere. The duplicate a loop starts from is lent
/// too, under the same terms, see [`lend_loop_dup`].
///
/// # Arguments
///
/// * `module` - The module, after [`move_last_uses`], changed in place.
pub(crate) fn lend_arguments(module: &mut Module)
{
    let writes = global_writes(module);
    let mut all = lent_in(&module.main);
    do_for!(f in &module.funcs, all.extend(lent_in(&f.block)));
    let globals: BTreeSet<Place> =
        all.into_iter().filter(|p| matches!(p, Place::Global(_))).collect();
    let with_globals = |mut own: BTreeSet<Place>| {
        own.extend(globals.iter().copied());
        own
    };
    let main = with_globals(lent_in(&module.main));
    let funcs: Vec<BTreeSet<Place>> =
        module.funcs.iter().map(|f| with_globals(lent_in(&f.block))).collect();
    lend_in(&mut module.main, &main, &writes);
    do_for!((f, kept) in module.funcs.iter_mut().zip(&funcs), lend_in(&mut f.block, kept, &writes));
}

/// Lend the text arguments of each call in a list of statements.
///
/// # Arguments
///
/// * `stmts` - The statements, changed in place.
/// * `kept` - The places a lend, a range or an address reads.
/// * `writes` - The globals each function writes, at each depth.
fn lend_in(stmts: &mut [Stmt], kept: &BTreeSet<Place>, writes: &[BTreeSet<u32>])
{
    each_stmt_mut(stmts, &mut |s| {
        lend_loop_dup(s, kept, writes);
        do_if!(let Stmt::Call { func, args, .. } = s, lend_args(*func, args, kept, writes));
        each_expr_mut(s, &mut |e| {
            do_if!(let ExprNode::Call { func, args, .. } = &mut e.node, lend_args(*func, args, kept, writes));
        });
    });
}

/// Lend the text arguments of one call that no call of it can change.
///
/// # Arguments
///
/// * `func` - The function called.
/// * `args` - The arguments, changed in place.
/// * `kept` - The places a lend, a range or an address reads.
/// * `writes` - The globals each function writes, at each depth.
fn lend_args(
    func: u32, args: &mut [Expr], kept: &BTreeSet<Place>, writes: &[BTreeSet<u32>],
)
{
    let mut called = BTreeSet::from([func]);
    let mut note = |e: &Expr| do_if!(let ExprNode::Call { func, .. } = e.node, { called.insert(func); });
    do_for!(a in args.iter(), each_expr(a, &mut note));
    let written: BTreeSet<u32> = called
        .iter()
        .filter_map(|f| writes.get(*f as usize))
        .flatten()
        .copied()
        .collect();
    for arg in args.iter_mut()
    {
        let ExprNode::Dup { value, lent } = &mut arg.node
        else
        {
            continue;
        };
        let Some(place) = root_of(value)
        else
        {
            continue;
        };
        let safe = match place
        {
            Place::Local(_) => true,
            Place::Global(g) => !written.contains(&g),
        };
        do_if!(safe && !kept.contains(&place), *lent = true);
    }
}

/// Lend the duplicate a loop starts from, such as the source of an each
/// loop, when the loop reads the duplicate through its cells alone and leaves
/// the source as it is: no write, no move, no whole read but another
/// duplicate, and no call that writes it. The duplicate then reads the
/// source's cells with a room of 0 until its drop as the loop ends.
///
/// # Arguments
///
/// * `s` - The statement, changed in place.
/// * `kept` - The places a lend, a range or an address reads.
/// * `writes` - The globals each function writes, at each depth.
fn lend_loop_dup(s: &mut Stmt, kept: &BTreeSet<Place>, writes: &[BTreeSet<u32>])
{
    let Stmt::Loop { init, cond, update, block, .. } = s
    else
    {
        return;
    };
    let inner = [update.as_slice(), block.as_slice()];
    let mut called = BTreeSet::new();
    do_for!(stmts in inner, callees(stmts, &mut called));
    let written: BTreeSet<u32> = called
        .iter()
        .filter_map(|f| writes.get(*f as usize))
        .flatten()
        .copied()
        .collect();
    let quiet = |place: Place| {
        let global = tern!(let Place::Global(g) = place, written.contains(&g), false);
        !global && !kept.contains(&place) && cells_alone(place, cond, &inner)
    };
    for set in init.iter_mut()
    {
        let Stmt::Set { place: dup, value, .. } = set
        else
        {
            continue;
        };
        let ExprNode::Dup { value: source, lent } = &mut value.node
        else
        {
            continue;
        };
        let walked = source.kind.is_seq() || source.kind.is_text();
        let from = root_of(source).filter(|_| walked);
        do_if!(let Some(from) = from, *lent |= quiet(from) && quiet(*dup));
    }
}

/// Whether a loop leaves a place as it is and reads it through its cells
/// alone: no statement writes it, and each read of it is the base of a cell,
/// a field or a character, or the value of a duplicate.
///
/// # Arguments
///
/// * `place` - The place.
/// * `cond` - The test of the loop.
/// * `inner` - The statements of the loop.
///
/// # Returns
///
/// `true` for such a place.
fn cells_alone(place: Place, cond: &Expr, inner: &[&[Stmt]]) -> bool
{
    let is_place = |e: &Expr| matches!(e.node, ExprNode::Read(p) if p == place);
    let (mut reads, mut through, mut writes) = (0, 0, 0);
    let mut note = |e: &Expr| match &e.node
    {
        ExprNode::Read(p) | ExprNode::AddrOf(p) if *p == place => reads += 1,
        ExprNode::CellAddr { list: base, .. }
        | ExprNode::FieldAddr { base, .. }
        | ExprNode::Chars { text: base, .. }
        | ExprNode::Dup { value: base, .. }
            if is_place(base) =>
        {
            through += 1
        }
        _ => (),
    };
    each_expr(cond, &mut note);
    for stmts in inner
    {
        each_stmt(stmts, &mut |s| {
            writes += written_places(s).iter().filter(|p| **p == place).count();
            do_for!(e in stmt_exprs(s), each_expr(e, &mut note));
        });
    }
    writes == 0 && reads == through
}

/// The place a value is read out of through fields and cells alone, with no
/// step through a pointer.
///
/// # Arguments
///
/// * `e` - The value, or the address of it.
///
/// # Returns
///
/// The place, or `None` for a value read through a pointer or made anew.
fn root_of(e: &Expr) -> Option<Place>
{
    match &e.node
    {
        ExprNode::Read(place) | ExprNode::AddrOf(place) => Some(*place),
        ExprNode::Load { addr } => root_of_addr(addr),
        _ => None,
    }
}

/// The place an address points into through fields and cells alone.
///
/// # Arguments
///
/// * `addr` - The address.
///
/// # Returns
///
/// The place, or `None` for an address read out of a pointer.
fn root_of_addr(addr: &Expr) -> Option<Place>
{
    match &addr.node
    {
        ExprNode::AddrOf(place) => Some(*place),
        ExprNode::FieldAddr { base, .. } | ExprNode::CellAddr { list: base, .. } =>
        {
            tern!(
                base.kind.is_ptr(),
                None,
                root_of(base).or_else(|| root_of_addr(base))
            )
        }
        _ => None,
    }
}

/// The globals each function writes, itself or through the functions it
/// calls: a set, a drop, an allocation, a resize or a free of one, a store
/// into one, a move out of one, or an address of one.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The global numbers, in function order.
fn global_writes(module: &Module) -> Vec<BTreeSet<u32>>
{
    let direct = |stmts: &[Stmt]| {
        let mut places = Vec::new();
        each_stmt(stmts, &mut |s| {
            places.extend(written_places(s));
            let mut note = |e: &Expr| match &e.node
            {
                ExprNode::Take { addr } => places.extend(places_in(addr)),
                ExprNode::AddrOf(place) => places.push(*place),
                _ => (),
            };
            do_for!(e in stmt_exprs(s), each_expr(e, &mut note));
        });
        let globals = places
            .into_iter()
            .filter_map(|p| tern!(let Place::Global(g) = p, Some(g), None));
        globals.collect::<BTreeSet<u32>>()
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
        for (f, called) in calls.iter().enumerate()
        {
            let more: BTreeSet<u32> =
                called.iter().flat_map(|&c| sets[c as usize].clone()).collect();
            let before = sets[f].len();
            sets[f].extend(more);
            changed |= sets[f].len() != before;
        }
    }
    sets
}

/// The places a statement writes as a whole: the slot it sets, the place it
/// drops, allocates, resizes or frees, and the places the address of a store
/// reads.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// The places.
fn written_places(s: &Stmt) -> Vec<Place>
{
    match s
    {
        Stmt::Set { place, .. } => vec![*place],
        Stmt::Drop { dest, .. }
        | Stmt::Free { dest, .. }
        | Stmt::Alloc { dest, .. }
        | Stmt::Resize { dest, .. }
        | Stmt::AddCell { dest, .. }
        | Stmt::RemoveCell { dest, .. }
        | Stmt::SetChar { dest, .. }
        | Stmt::AddText { dest, .. } => match dest
        {
            Dest::Slot(place) => vec![*place],
            Dest::Mem(addr) => places_in(addr),
        },
        Stmt::Store { addr, .. } => places_in(addr),
        _ => Vec::new(),
    }
}

/// Each place an expression reads or takes the address of.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The places.
fn places_in(e: &Expr) -> Vec<Place>
{
    let mut found = Vec::new();
    each_expr(e, &mut |sub| match sub.node
    {
        ExprNode::Read(place) | ExprNode::AddrOf(place) => found.push(place),
        _ => (),
    });
    found
}

/// Mark the list and struct parameters each function never writes, so it
/// reads them in the caller's storage through their address, and give a call
/// a duplicate of an argument the call could change while the function reads
/// it: a global the call, or a call inside its arguments, writes, or a place
/// whose address or a view of it is taken somewhere.
///
/// # Arguments
///
/// * `module` - The module, after [`lend_arguments`], changed in place.
pub(crate) fn share_params(module: &mut Module)
{
    let shared: Vec<Vec<u32>> = module.funcs.iter().map(unwritten_params).collect();
    do_for!((f, params) in module.funcs.iter_mut().zip(&shared), f.shared_params = params.clone());
    let writes = global_writes(module);
    let mut all = lent_in(&module.main);
    do_for!(f in &module.funcs, all.extend(lent_in(&f.block)));
    let globals: BTreeSet<Place> =
        all.into_iter().filter(|p| matches!(p, Place::Global(_))).collect();
    let with_globals = |mut own: BTreeSet<Place>| {
        own.extend(globals.iter().copied());
        own
    };
    let main = with_globals(lent_in(&module.main));
    let funcs: Vec<BTreeSet<Place>> =
        module.funcs.iter().map(|f| with_globals(lent_in(&f.block))).collect();
    let calls = Calls { writes: &writes, shared: &shared };
    duplicate_unstable(&mut module.main, &main, &calls);
    do_for!((f, kept) in module.funcs.iter_mut().zip(&funcs), duplicate_unstable(&mut f.block, kept, &calls));
}

/// What [`duplicate_unstable`] knows of the functions a call can reach.
struct Calls<'a>
{
    /// The globals each function writes, at each depth.
    writes: &'a [BTreeSet<u32>],
    /// The parameters each function reads in the caller's storage.
    shared: &'a [Vec<u32>],
}

/// The list and struct parameters a function never writes: no set, store,
/// move, drop, view or address of them, and no text, owner or borrow in
/// them, which the other rules of the language follow.
///
/// # Arguments
///
/// * `f` - The function.
///
/// # Returns
///
/// The parameter numbers.
fn unwritten_params(f: &Func) -> Vec<u32>
{
    let touched = touched_in(&f.block);
    let plain = |n: &u32| {
        let slot = &f.locals[*n as usize];
        let k = slot.kind;
        let held = k.holds_text() || k.holds_owner_ptr() || k.holds_borrow();
        k.is_aggregate() && !k.is_slice() && !held && !slot.addressed
    };
    (0..f.params)
        .filter(plain)
        .filter(|n| !touched.contains(&Place::Local(*n)))
        .collect()
}

/// The places a list of statements writes, moves out of, drops, views or
/// takes the address of, nested blocks included.
///
/// # Arguments
///
/// * `stmts` - The statements.
///
/// # Returns
///
/// The places.
fn touched_in(stmts: &[Stmt]) -> BTreeSet<Place>
{
    let mut found = BTreeSet::new();
    each_stmt(stmts, &mut |s| {
        found.extend(written_places(s));
        let mut note = |e: &Expr| match &e.node
        {
            ExprNode::Take { addr: inner }
            | ExprNode::MakeSlice { source: inner, .. }
            | ExprNode::Lend { operand: inner } => found.extend(places_in(inner)),
            ExprNode::AddrOf(place) =>
            {
                found.insert(*place);
            }
            _ => (),
        };
        do_for!(e in stmt_exprs(s), each_expr(e, &mut note));
    });
    found
}

/// Give each call in a list of statements a duplicate of each argument for
/// a parameter read in place that the call could change.
///
/// # Arguments
///
/// * `stmts` - The statements, changed in place.
/// * `kept` - The places a lend, a range or an address reads.
/// * `calls` - The writes and shared parameters of each function.
fn duplicate_unstable(stmts: &mut [Stmt], kept: &BTreeSet<Place>, calls: &Calls)
{
    each_stmt_mut(stmts, &mut |s| {
        do_if!(let Stmt::Call { func, args, .. } = s, duplicate_args(*func, args, kept, calls));
        each_expr_mut(s, &mut |e| {
            do_if!(let ExprNode::Call { func, args, .. } = &mut e.node, duplicate_args(*func, args, kept, calls));
        });
    });
}

/// Give one call a duplicate of each argument for a parameter read in place
/// that the call could change.
///
/// # Arguments
///
/// * `func` - The function called.
/// * `args` - The arguments, changed in place.
/// * `kept` - The places a lend, a range or an address reads.
/// * `calls` - The writes and shared parameters of each function.
fn duplicate_args(func: u32, args: &mut [Expr], kept: &BTreeSet<Place>, calls: &Calls)
{
    let Some(params) = calls.shared.get(func as usize).filter(|p| !p.is_empty())
    else
    {
        return;
    };
    let mut called = BTreeSet::from([func]);
    let mut note = |e: &Expr| do_if!(let ExprNode::Call { func, .. } = e.node, { called.insert(func); });
    do_for!(a in args.iter(), each_expr(a, &mut note));
    let written: BTreeSet<u32> = called
        .iter()
        .filter_map(|f| calls.writes.get(*f as usize))
        .flatten()
        .copied()
        .collect();
    for n in params
    {
        let Some(arg) = args.get_mut(*n as usize)
        else
        {
            continue;
        };
        let Some(place) = root_of(arg)
        else
        {
            continue;
        };
        let changed = matches!(place, Place::Global(g) if written.contains(&g));
        do_if!(changed || kept.contains(&place), *arg = plain_dup(arg));
    }
}

/// A duplicate of a list or a struct with no text in it: a blit into a slot
/// of its own.
///
/// # Arguments
///
/// * `e` - The value.
///
/// # Returns
///
/// The duplicate, of the same kind.
fn plain_dup(e: &Expr) -> Expr
{
    let node = ExprNode::Dup { value: Box::new(e.clone()), lent: false };
    Expr::builder().kind(e.kind).node(node).span(e.span).build()
}
