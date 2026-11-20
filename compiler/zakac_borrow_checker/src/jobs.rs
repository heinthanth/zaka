//! The globals rule of jobs: a mutable global belongs to the main thread, so
//! a function a job or a parallel map runs, itself or through each function it
//! calls, reads and writes none (ZK0726). Constants are free to read.

use std::collections::BTreeSet;

use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::ownership as od;
use zakac_ir_builder::ir::Access;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::RECORD_HEAD;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

use crate::callgraph::callees;
use crate::last_use::written_places;
use crate::values::taken_fns;
use crate::walk::each_expr;
use crate::walk::each_stmt;
use crate::walk::stmt_exprs;

/// Report each function a job runs that reaches a mutable global, at its
/// first use of one.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The diagnostics.
pub(crate) fn check_jobs(module: &Module) -> Vec<Diagnostic>
{
    let spawned = spawned_fns(module);
    ret_if!(spawned.is_empty(), Vec::new());
    let reached = reached_fns(module, spawned);
    reached.into_iter().filter_map(|f| global_use(module, f)).collect()
}

/// Whether calls of an export of a C lib can run on several threads at once
/// with no lock: neither it nor each function it reaches uses a mutable
/// global, calls C, calls through a function value, or starts a job or a
/// parallel map.
///
/// # Arguments
///
/// * `module` - The module.
/// * `f` - The function number of the export.
///
/// # Returns
///
/// `true` when they can.
pub fn runs_free(module: &Module, f: u32) -> bool
{
    let reached = reached_fns(module, BTreeSet::from([f]));
    let alone = |g: &u32| !needs_lock(&module.funcs[*g as usize].block);
    reached.iter().all(|g| alone(g) && global_use(module, *g).is_none())
}

/// Which functions C can call from a thread of its own: those that, itself
/// or through each function it calls, uses no mutable global; for the thunk
/// of a closure, each capture is also a duplicate of a value jobs can use.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// For each function, whether it can.
pub fn crossing_fns(module: &Module) -> Vec<bool>
{
    let mut crosses = global_free_fns(module);
    // The rule of captures belongs to the closure alone, not to its callers.
    for func in &module.funcs
    {
        each_stmt(&func.block, &mut |s| {
            let mut note = |e: &Expr| {
                let ExprNode::Closure { func, record, .. } = &e.node
                else
                {
                    return;
                };
                let copies = record.kind.pointee().is_some_and(copied_captures);
                do_if!(!copies, crosses[*func as usize] = false);
            };
            do_for!(e in stmt_exprs(s), each_expr(e, &mut note));
        });
    }
    crosses
}

/// Whether each capture of a closure record is a duplicate of a value another
/// thread can read: no box, which the function around shares, and no
/// function value, pointer or handle at each depth of the value.
///
/// # Arguments
///
/// * `record` - The kind of the record.
///
/// # Returns
///
/// `true` when each is.
fn copied_captures(record: DataKind) -> bool
{
    let fields = record.as_struct().map(|s| s.fields());
    let mut captures = fields.unwrap_or_default().iter().skip(RECORD_HEAD);
    captures.all(|(_, k)| crossing_value(*k))
}

/// Whether a value can be read from another thread once copied: it holds no
/// box, function value, pointer or handle, at each depth.
///
/// # Arguments
///
/// * `kind` - The kind of the value.
///
/// # Returns
///
/// `true` when it can.
fn crossing_value(kind: DataKind) -> bool
{
    let kind = kind.repr();
    let shared = kind.is_fn() || kind.is_ptr() || kind.as_external().is_some();
    ret_false!(shared || kind.is_box());
    let cell = kind.cell_kind().filter(|_| !kind.is_text());
    let fields = kind.as_struct().map(|s| s.fields()).unwrap_or_default();
    cell.is_none_or(crossing_value) && fields.iter().all(|(_, k)| crossing_value(*k))
}

/// The functions that use no mutable global, in their own statements or in
/// those of each function reached from them.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// For each function, whether it uses none.
fn global_free_fns(module: &Module) -> Vec<bool>
{
    let taken = taken_fns(module);
    let count = module.funcs.len();
    let mut callers: Vec<Vec<u32>> = vec![Vec::new(); count];
    for (f, func) in module.funcs.iter().enumerate()
    {
        let mut called = BTreeSet::new();
        callees(&func.block, &taken, &mut called);
        do_for!(g in called, callers[g as usize].push(f as u32));
    }
    let mut crosses = vec![true; count];
    let uses = (0..count as u32).filter(|f| global_use(module, *f).is_some());
    let mut next: Vec<u32> = uses.collect();
    while let Some(f) = next.pop()
    {
        let seen = !std::mem::replace(&mut crosses[f as usize], false);
        do_if!(!seen, next.extend(&callers[f as usize]));
    }
    crosses
}

/// Whether statements call C, call through a function value, or start a job
/// or a parallel map.
///
/// # Arguments
///
/// * `block` - The statements.
///
/// # Returns
///
/// `true` when they do.
fn needs_lock(block: &[Stmt]) -> bool
{
    let mut found = false;
    each_stmt(block, &mut |s| {
        let par = matches!(s, Stmt::Loop { par: Some(_), .. });
        found |= par || matches!(s, Stmt::Foreign { .. } | Stmt::CallValue { .. });
        let mut note = |e: &Expr| {
            let node = &e.node;
            found |=
                matches!(node, ExprNode::Foreign { .. } | ExprNode::CallValue { .. });
            found |= matches!(node, ExprNode::Spawn { .. });
        };
        do_for!(e in stmt_exprs(s), each_expr(e, &mut note));
    });
    found
}

/// The functions some call starts a job in, and those a parallel map runs.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The function numbers.
fn spawned_fns(module: &Module) -> BTreeSet<u32>
{
    let mut found = BTreeSet::new();
    let mut note = |e: &Expr| {
        let ExprNode::Spawn { func, .. } = &e.node
        else
        {
            return;
        };
        found.insert(*func);
    };
    let mut maps = BTreeSet::new();
    for f in &module.funcs
    {
        each_stmt(&f.block, &mut |s| {
            // A parallel map runs its function on other threads too.
            do_if!(let Stmt::Loop { par: Some(p), .. } = s, _ = maps.insert(p.func));
            do_for!(e in stmt_exprs(s), each_expr(e, &mut note));
        });
    }
    found.extend(maps);
    found
}

/// The functions a job can run: those it starts in, and each function they
/// call, through function values too.
///
/// # Arguments
///
/// * `module` - The module.
/// * `start` - The functions jobs start in.
///
/// # Returns
///
/// The function numbers.
fn reached_fns(module: &Module, start: BTreeSet<u32>) -> BTreeSet<u32>
{
    let taken = taken_fns(module);
    let mut reached = BTreeSet::new();
    let mut next: Vec<u32> = start.into_iter().collect();
    while let Some(f) = next.pop()
    {
        if !reached.insert(f)
        {
            continue;
        }
        let mut called = BTreeSet::new();
        callees(&module.funcs[f as usize].block, &taken, &mut called);
        next.extend(called.difference(&reached));
    }
    reached
}

/// The first use of a mutable global in a function, as its diagnostic.
///
/// # Arguments
///
/// * `module` - The module.
/// * `f` - The function number.
///
/// # Returns
///
/// The diagnostic, or `None` when the function uses no mutable global.
fn global_use(module: &Module, f: u32) -> Option<Diagnostic>
{
    let mutable = |p: &Place| match p
    {
        Place::Global(g) => module
            .globals
            .get(*g as usize)
            .filter(|s| s.access != Access::Const)
            .map(|_| *g),
        Place::Local(_) => None,
    };
    let mut first: Option<(u32, Span)> = None;
    let func = &module.funcs[f as usize];
    each_stmt(&func.block, &mut |s: &Stmt| {
        let written = written_places(s).iter().find_map(mutable).map(|g| (g, s.span()));
        first = first.or(written);
        let mut note = |e: &Expr| {
            let place = match &e.node
            {
                ExprNode::Read(p) | ExprNode::AddrOf(p) => Some(*p),
                _ => None,
            };
            first = first.or(place.as_ref().and_then(mutable).map(|g| (g, e.span)));
        };
        do_for!(e in stmt_exprs(s), each_expr(e, &mut note));
    });
    let (g, span) = first?;
    let name = &module.globals[g as usize].name;
    Some(od::global_in_job(span, &func.name, name))
}
