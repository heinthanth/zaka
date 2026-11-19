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
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
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
