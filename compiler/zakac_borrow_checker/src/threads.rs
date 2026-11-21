//! The globals rule of threads: a mutable global belongs to the main thread, so
//! a function a thread or a parallel map runs, itself or through each function
//! it calls, reads and writes none (ZK0726). Constants are free to read.

use std::collections::BTreeSet;

use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::ownership as od;
use zakac_ir_builder::ir::Access;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Func;
use zakac_ir_builder::ir::HostFn;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::RECORD_HEAD;
use zakac_ir_builder::ir::Stmt;
use zakac_ir_builder::ir::is_round;
use zakac_ir_builder::ir::shown_name;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

use crate::callgraph::callees;
use crate::last_use::written_places;
use crate::values::taken_fns;
use crate::walk::each_expr;
use crate::walk::each_stmt;
use crate::walk::stmt_exprs;

/// Report each function a thread runs that reaches a mutable global, at its
/// first use of one.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The diagnostics.
pub(crate) fn check_threads(module: &Module) -> Vec<Diagnostic>
{
    let spawned = spawned_fns(module);
    ret_if!(spawned.is_empty(), Vec::new());
    let reached = reached_fns(module, spawned);
    reached.into_iter().filter_map(|f| global_use(module, f)).collect()
}

/// The functions whose threads can run as light tasks, small stacks that a pool
/// of OS threads takes turns on: neither the function nor each function it
/// reaches, through function values too, calls C, reads standard input,
/// calls itself again or keeps more than 16 KiB of locals. A thread written
/// `သီးသန့် အပြိုင်` runs on an OS thread of its own whatever this gives.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The function numbers, among those some call starts a thread in.
pub fn light_fns(module: &Module) -> BTreeSet<u32>
{
    let taken = taken_fns(module);
    let calls: Vec<BTreeSet<u32>> = module
        .funcs
        .iter()
        .map(|f| {
            let mut called = BTreeSet::new();
            callees(&f.block, &taken, &mut called);
            called
        })
        .collect();
    let pins = module.funcs.iter().map(|f| pins_thread(&f.block) || large_frame(f));
    let pins: Vec<bool> = pins.collect();
    let light = |f: &u32| {
        let reached = reach(&calls, *f);
        let pinned = pins[*f as usize] || reached.iter().any(|g| pins[*g as usize]);
        let again = |g: &u32| reach(&calls, *g).contains(g);
        !pinned && !again(f) && !reached.iter().any(again)
    };
    spawn_targets(module).into_iter().filter(light).collect()
}

/// The functions a function reaches through one call or more: the function
/// itself is among them when it calls itself again.
///
/// # Arguments
///
/// * `calls` - The functions each function calls.
/// * `start` - The function.
///
/// # Returns
///
/// The function numbers.
fn reach(calls: &[BTreeSet<u32>], start: u32) -> BTreeSet<u32>
{
    let mut reached = BTreeSet::new();
    let mut next: Vec<u32> = calls[start as usize].iter().copied().collect();
    while let Some(f) = next.pop()
    {
        do_if!(reached.insert(f), next.extend(&calls[f as usize]));
    }
    reached
}

/// The most octets of locals a function of a light task can have: below the
/// guard of the stack of a task, so a frame never steps past the guard.
const LIGHT_FRAME: u64 = 16 * 1024;

/// Whether a function keeps more locals than a light task can hold in one
/// frame.
///
/// # Arguments
///
/// * `func` - The function.
///
/// # Returns
///
/// `true` when it does.
fn large_frame(func: &Func) -> bool
{
    // Sizes at 64 bits, the widest platform: a 32-bit one sends more threads
    // to OS threads, never fewer.
    let octets: u64 = func.locals.iter().map(|s| u64::from(s.kind.size(64))).sum();
    octets > LIGHT_FRAME
}

/// Whether statements need an OS thread of their own: a call of C, which can
/// keep state for each OS thread or hold it in a call that blocks, or a read of
/// standard input, which can wait with no end.
///
/// # Arguments
///
/// * `block` - The statements.
///
/// # Returns
///
/// `true` when one stands there.
fn pins_thread(block: &[Stmt]) -> bool
{
    let mut found = false;
    each_stmt(block, &mut |s| {
        found |= matches!(s, Stmt::Foreign { .. });
        let mut note = |e: &Expr| {
            let read = matches!(e.node, ExprNode::Host { func: HostFn::ReadLine, .. });
            found |= read || matches!(e.node, ExprNode::Foreign { .. });
        };
        do_for!(e in stmt_exprs(s), each_expr(e, &mut note));
    });
    found
}

/// The functions some call starts a thread in.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The function numbers.
fn spawn_targets(module: &Module) -> BTreeSet<u32>
{
    let mut found = BTreeSet::new();
    let mut note = |e: &Expr| {
        do_if!(let ExprNode::Spawn { func, .. } = &e.node, _ = found.insert(*func));
    };
    let bodies = module.funcs.iter().map(|f| &f.block).chain([&module.main]);
    for block in bodies
    {
        each_stmt(block, &mut |s| do_for!(e in stmt_exprs(s), each_expr(e, &mut note)));
    }
    found
}

/// Whether calls of an export of a C lib can run on several OS threads at once
/// with no lock: neither it nor each function it reaches uses a mutable
/// global, calls C, calls through a function value, or starts a thread or a
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

/// Whether a program starts a thread or runs a parallel loop or map, so its
/// loops pass a safe point of the ring collector at each round.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// `true` when it does.
pub fn starts_threads(module: &Module) -> bool
{
    let (mut loops, mut spawns) = (false, false);
    let bodies = module.funcs.iter().map(|f| &f.block).chain([&module.main]);
    for block in bodies
    {
        each_stmt(block, &mut |s| {
            loops |= matches!(s, Stmt::Loop { par: Some(_), .. });
            let mut note =
                |e: &Expr| spawns |= matches!(e.node, ExprNode::Spawn { .. });
            do_for!(e in stmt_exprs(s), each_expr(e, &mut note));
        });
    }
    loops || spawns
}

/// Which functions C can call from an OS thread of its own: those that, itself
/// or through each function it calls, uses no mutable global; for the thunk
/// of a closure, each capture is also a duplicate of a value threads can use.
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
                let copies = record.kind.pointee();
                let copies =
                    copies.is_some_and(|r| copied_captures(r, &module.threads_structs));
                do_if!(!copies, crosses[*func as usize] = false);
            };
            do_for!(e in stmt_exprs(s), each_expr(e, &mut note));
        });
    }
    crosses
}

/// Whether each capture of a closure record is a duplicate of a value another
/// thread can read: no box, which the function around shares, and no
/// function value, pointer or handle at each depth of the value, save an
/// object of a struct marked as usable in threads.
///
/// # Arguments
///
/// * `record` - The kind of the record.
/// * `threads` - The structs marked as usable in threads.
///
/// # Returns
///
/// `true` when each is.
fn copied_captures(record: DataKind, threads: &BTreeSet<String>) -> bool
{
    let fields = record.as_struct().map(|s| s.fields());
    let mut captures = fields.unwrap_or_default().iter().skip(RECORD_HEAD);
    captures.all(|(_, k)| crossing_value(*k, threads))
}

/// Whether a value can be read from another thread once copied: it holds no
/// box, function value, pointer or handle, at each depth, save an object of a
/// struct marked as usable in threads, whose fields each thread reaches under
/// its lock. The box of a captured variable a statement writes is no such
/// object: a variable that holds an object is a box itself, so its box holds a
/// box.
///
/// # Arguments
///
/// * `kind` - The kind of the value.
/// * `threads` - The structs marked as usable in threads.
///
/// # Returns
///
/// `true` when it can.
fn crossing_value(kind: DataKind, threads: &BTreeSet<String>) -> bool
{
    let kind = kind.repr();
    let base = |name: &str| name.split('<').next().unwrap_or_default().to_string();
    ret_true!(kind.as_object().is_some_and(|s| threads.contains(&base(&s.name))));
    let shared = kind.is_fn() || kind.is_ptr() || kind.as_external().is_some();
    ret_false!(shared || kind.is_box());
    let cell = kind.cell_kind().filter(|_| !kind.is_text());
    let fields = kind.as_struct().map(|s| s.fields()).unwrap_or_default();
    let inner = |k: DataKind| crossing_value(k, threads);
    cell.is_none_or(inner) && fields.iter().all(|(_, k)| inner(*k))
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

/// Whether statements call C, call through a function value, or start a thread
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

/// The functions some call starts a thread in, and those a parallel map or the
/// rounds of a parallel loop run.
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
            do_if!(let Stmt::Loop { par: Some(p), .. } = s, _ = maps.insert(p.func()));
            do_for!(e in stmt_exprs(s), each_expr(e, &mut note));
        });
    }
    found.extend(maps);
    found
}

/// The functions a thread can run: those it starts in, and each function they
/// call, through function values too.
///
/// # Arguments
///
/// * `module` - The module.
/// * `start` - The functions threads start in.
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
    ret_if!(is_round(&func.name), Some(od::global_in_round(span, name)));
    Some(od::global_in_thread(span, shown_name(&func.name), name))
}
