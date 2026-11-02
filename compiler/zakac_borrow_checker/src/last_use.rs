use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::ownership::duplicate_made;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

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
/// global read in a function, which a later call can read; a place that a
/// lend, a range or an address reads, so a move never ends a live view; or a
/// place the same statement reads again.
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
    let main = moves_in(module, &Routine::main(module), &lent_globals);
    let funcs: Vec<HashSet<DupAt>> = module
        .funcs
        .iter()
        .map(|f| moves_in(module, &Routine::func(f), &lent_globals))
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
///
/// # Returns
///
/// The duplicates, each with the span of its read and its place.
fn moves_in(
    module: &Module, routine: &Routine, lent_globals: &BTreeSet<Place>,
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
        let mut judge = |e: &Expr| {
            let Some(place) = dup_of(e)
            else
            {
                return;
            };
            let once = read.iter().filter(|(p, _)| *p == place).count() == 1;
            let kept = lent.contains(&place) || lent_globals.contains(&place);
            let ok =
                once && movable(routine, place) && !kept && !out.contains_key(&place);
            verdicts.entry((e.span, place)).and_modify(|v| *v &= ok).or_insert(ok);
        };
        do_for!(e in &exprs, each_expr(e, &mut judge));
    }
    verdicts.into_iter().filter(|(_, ok)| *ok).map(|(at, _)| at).collect()
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
/// nested blocks included.
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
    let mut note = |e: &Expr| match &e.node
    {
        ExprNode::AddrOf(place) =>
        {
            found.insert(*place);
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
    let ExprNode::Dup { value } = &e.node
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
            let ExprNode::Dup { value } = &e.node
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
