use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::ownership::Invalidation;
use zakac_diagnostics::ownership::aliased_argument;
use zakac_diagnostics::ownership::invalidated_loan;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Func;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::cfg::NodeKind;
use crate::checks::loans::pointer_source;
use crate::checks::report;
use crate::checks::slot_name;
use crate::digest::FnDigest;
use crate::digest::Root;
use crate::flow::Flow;
use crate::flow::Loss;
use crate::liveness::node_exprs;
use crate::roots::Roots;
use crate::roots::roots_of;
use crate::roots::translate;
use crate::state::State;
use crate::walk::each_expr;
use crate::walk::sub_exprs;

/// Check a call once its arguments ran. No argument can reach a block
/// the callee can free through another argument (ZK0707). Each block the
/// callee can free through a borrow parameter or a global is checked
/// against the loans read after the call once the node ends; a move into
/// an owner parameter is checked as the move it is. A result path the
/// callee itself renewed points into the new block, so the slot the node
/// sets to that result holds no loan on it. A result read on the spot has
/// no slot, so it is checked here against each block the call gives back,
/// and so is each borrow the rest of the node holds across the call.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `func` - The function number.
/// * `args` - The roots of each argument before the call, see
///   [`crate::roots::value_roots`].
/// * `span` - The call.
/// * `state` - The facts once the arguments ran.
pub(crate) fn check_call(
    flow: &mut Flow, func: u32, args: &[Roots], span: Span, state: &State,
)
{
    ret_if!(!flow.recording());
    let Some((callee, digest)) = callee_digest(flow, func)
    else
    {
        return;
    };
    let freed = freed_per_param(callee, digest, args);
    check_aliasing(flow, &callee.name, &freed, args, span);
    let clash = global_clash(callee, digest, args);
    let clash = clash.or_else(|| read_clash(digest, args));
    let clash = clash.map(|(owner, n)| clash_report(flow, callee, owner, n, span));
    let returned = renewed_paths(callee, digest, args);
    let held = held_in_place(flow, digest, args, &returned, span);
    let given = given_paths(digest, &freed);
    let clash = clash.or_else(|| stale_report(flow, callee, &given, &held, span));
    let clash = clash.or_else(|| held_report(flow, callee, &given, span, state));
    let clash = clash.or_else(|| global_use_report(flow, &given, span, state));
    let lost = lost_paths(callee, digest, &freed);
    ret_if!(lost.is_empty() && clash.is_none());
    let loss = Loss::builder()
        .paths(lost)
        .what(Invalidation::Freed)
        .span(span)
        .returned(returned)
        .maybe_clash(clash)
        .build();
    flow.losses.push(loss);
}

/// The result paths of a call the node that runs reads on the spot, with
/// no slot to hold them: a call read through, handed to another call or
/// used in a test. A call set to a slot, under lends, conversions and
/// steps at most, is checked through the slot; a returned one through the
/// digest; the result of a call statement is dropped. The paths the
/// callee renewed are left out.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `digest` - The digest of the callee.
/// * `args` - The roots of each argument before the call.
/// * `returned` - The renewed result paths, see [`renewed_paths`].
/// * `span` - The call.
///
/// # Returns
///
/// The paths; none for a result a slot, a return or nothing takes.
fn held_in_place(
    flow: &Flow, digest: &FnDigest, args: &[Roots], returned: &Roots, span: Span,
) -> Roots
{
    ret_if!(!used_in_place(flow, span), Roots::new());
    let found = digest.result_roots.iter().flat_map(|r| translate(r, args));
    found.filter(|r| !returned.contains(r)).collect()
}

/// Whether the node that runs reads the result of a call on the spot.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `call` - The call.
///
/// # Returns
///
/// `false` for the value of a set of a slot or of a return, and for a call
/// statement; `true` for each other place of a call.
fn used_in_place(flow: &Flow, call: Span) -> bool
{
    let NodeKind::Leaf(s) = &flow.cfg.nodes[flow.node as usize].kind
    else
    {
        return true;
    };
    let value = match s
    {
        Stmt::Set { value, .. } | Stmt::Return { value: Some(value), .. } => value,
        Stmt::Call { span, .. } => return *span != call,
        _ => return true,
    };
    pointer_source(value).span != call
}

/// The report of a borrow the node that runs holds across a call that
/// gives its block back, see [`held_values`]: whatever the order the node
/// runs them in, since a value taken before the call can still be used
/// after it.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `callee` - The function.
/// * `given` - The paths the call can give back, see [`given_paths`].
/// * `span` - The call.
/// * `state` - The facts once the arguments ran.
///
/// # Returns
///
/// ZK0704 at the call, named after the owner and the value, or `None`
/// when no given path reaches a held value.
fn held_report(
    flow: &Flow, callee: &Func, given: &Roots, span: Span, state: &State,
) -> Option<Diagnostic>
{
    ret_if!(given.is_empty(), None);
    let hit = |v| {
        let roots = held_roots(flow, v, span, state);
        given.iter().find(|g| roots.iter().any(|r| g.reaches(r))).map(|g| (g, v))
    };
    let (hit, value) = held_values(flow, span).into_iter().find_map(hit)?;
    let owner = slot_name(flow, hit.place);
    let what = Invalidation::Freed.text();
    let name = value_name(flow, value).unwrap_or(&callee.name);
    Some(invalidated_loan(span, owner, &what, name, value.span))
}

/// The values of the node that runs a call can hold across it: each
/// expression outside the call, save three kinds. A value inside the call
/// goes into it, and the arguments are checked against each other. The
/// value a set takes is checked through the slot. A slot read after the
/// call is checked as a read that follows it, see
/// [`crate::checks::loans::check_invalidation`], save a read of the slot
/// the node sets: that read sees the old value, while the check after the
/// node sees the new one. The value a return takes counts: a callee that
/// renews the block hands its result on as fresh, so no caller checks it.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `span` - The call.
///
/// # Returns
///
/// The values, outer first.
fn held_values<'m>(flow: &Flow<'m>, span: Span) -> Vec<&'m Expr>
{
    let kind = &flow.cfg.nodes[flow.node as usize].kind;
    let (target, taken) = set_value(kind).unzip();
    let mut found = Vec::new();
    do_for!(e in node_exprs(kind), each_expr(e, &mut |v| found.push(v)));
    let later = |v: &Expr| {
        let read = tern!(let ExprNode::Read(p) = v.node, Some(p), None);
        read.is_some_and(|p| Some(p) != target) && v.span.lo >= span.hi
    };
    let kept = |v: &&Expr| {
        !span.contains(&v.span) && Some(pointer_source(v).span) != taken && !later(v)
    };
    found.into_iter().filter(kept).collect()
}

/// The report of a global borrow another call of the node reads, itself
/// or through its callees, while the call gives its block back. The order
/// the two calls run in is not followed, as for a held value; a call
/// inside the call runs before it. The global is read as the node found
/// it, so the old value of a global the node sets counts too.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `given` - The paths the call can give back, see [`given_paths`].
/// * `span` - The call.
/// * `state` - The facts once the arguments ran.
///
/// # Returns
///
/// ZK0704 at the call, named after the owner and the global, with the
/// other call as the use, or `None` when no given path reaches a global
/// such a call reads.
fn global_use_report(
    flow: &Flow, given: &Roots, span: Span, state: &State,
) -> Option<Diagnostic>
{
    ret_if!(given.is_empty(), None);
    let uses = |(func, at): (u32, Span)| {
        let globals = flow.global_uses.get(func as usize).into_iter().flatten();
        globals.map(move |&g| (Place::Global(g), at))
    };
    let hit = |(g, at): (Place, Span)| {
        let roots = state.ptr(g).map(|info| &info.roots)?;
        let found = given.iter().find(|r| roots.iter().any(|loan| r.reaches(loan)))?;
        Some((found.place, g, at))
    };
    let calls = node_calls(flow).into_iter().filter(|(_, at)| !span.contains(at));
    let (owner, global, at) = calls.flat_map(uses).find_map(hit)?;
    let (owner, global) = (slot_name(flow, owner), slot_name(flow, global));
    let what = Invalidation::Freed.text();
    Some(invalidated_loan(span, owner, &what, global, at))
}

/// The calls the node that runs makes: a call statement and each call in
/// its expressions.
///
/// # Arguments
///
/// * `flow` - The flow.
///
/// # Returns
///
/// The function and the span of each call.
fn node_calls(flow: &Flow) -> Vec<(u32, Span)>
{
    let kind = &flow.cfg.nodes[flow.node as usize].kind;
    let mut found = Vec::new();
    do_if!(let NodeKind::Leaf(Stmt::Call { func, span, .. }) = kind, found.push((*func, *span)));
    let mut add = |e: &Expr| do_if!(let ExprNode::Call { func, .. } = e.node, found.push((func, e.span)));
    do_for!(e in node_exprs(kind), each_expr(e, &mut add));
    found
}

/// The slot a set of the node writes, and the span of the value it
/// takes, through [`pointer_source`].
///
/// # Arguments
///
/// * `kind` - What the node runs.
///
/// # Returns
///
/// Both, or `None` for a node that is no set.
fn set_value(kind: &NodeKind) -> Option<(Place, Span)>
{
    let NodeKind::Leaf(Stmt::Set { place, value, .. }) = kind
    else
    {
        return None;
    };
    Some((*place, pointer_source(value).span))
}

/// The paths a value of the node can still point into once a call ran:
/// each root of a borrow value. A value around the call, or one that
/// starts after it, other than a slot read, runs after it: it lends, loads
/// or steps from what the call left, so the result of a call alone can
/// still point into the old block, through a global it reads.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `v` - The value, outside the call.
/// * `call` - The call.
/// * `state` - The facts.
///
/// # Returns
///
/// The paths; none for a value that holds no borrow, and for another
/// expression that runs after the call.
fn held_roots(flow: &Flow, v: &Expr, call: Span, state: &State) -> Roots
{
    let read = matches!(v.node, ExprNode::Read(_));
    let after = v.span.contains(&call) || (v.span.lo >= call.hi && !read);
    ret_if!(after, global_result(flow, v));
    tern!(v.kind.holds_borrow(), roots_of(v, state, &flow.ctx), Roots::new())
}

/// The global paths the result of a call can point into.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `v` - The expression.
///
/// # Returns
///
/// The paths; none for another expression and for a function with no
/// digest.
fn global_result(flow: &Flow, v: &Expr) -> Roots
{
    let ExprNode::Call { func, .. } = &v.node
    else
    {
        return Roots::new();
    };
    let Some((_, digest)) = callee_digest(flow, *func)
    else
    {
        return Roots::new();
    };
    digest.result_roots.iter().filter(|r| r.is_global()).cloned().collect()
}

/// The name of the slot or the function a value comes from, for a report.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `v` - The value.
///
/// # Returns
///
/// The name of the slot read, addressed or the function called, through
/// the first operand of each other expression; `None` when there is none.
fn value_name<'m>(flow: &Flow<'m>, v: &Expr) -> Option<&'m str>
{
    match &v.node
    {
        ExprNode::Read(place) | ExprNode::AddrOf(place) =>
        {
            Some(slot_name(flow, *place))
        }
        ExprNode::Call { func, .. } =>
        {
            flow.module.funcs.get(*func as usize).map(|f| f.name.as_str())
        }
        _ => value_name(flow, sub_exprs(v).first()?),
    }
}

/// The paths of the caller a call can free, resize or move through each
/// parameter, an owner one included, and through the globals.
///
/// # Arguments
///
/// * `digest` - The digest of the callee.
/// * `freed` - What the callee can free through each parameter.
///
/// # Returns
///
/// The paths.
fn given_paths(digest: &FnDigest, freed: &[Roots]) -> Roots
{
    let globals = digest.frees.iter().filter(|r| r.is_global());
    freed.iter().flatten().chain(globals).cloned().collect()
}

/// The report of a call result read on the spot that points into a block
/// the call gave back: ZK0704 at the call, named after the owner and the
/// function.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `callee` - The function.
/// * `given` - The paths the call can give back, see [`given_paths`].
/// * `held` - The result paths read on the spot, see [`held_in_place`].
/// * `span` - The call.
///
/// # Returns
///
/// The diagnostic, or `None` when no given path reaches a held one.
fn stale_report(
    flow: &Flow, callee: &Func, given: &Roots, held: &Roots, span: Span,
) -> Option<Diagnostic>
{
    let hit = given.iter().find(|g| held.iter().any(|r| g.reaches(r)))?;
    let owner = slot_name(flow, hit.place);
    let what = Invalidation::Freed.text();
    Some(invalidated_loan(span, owner, &what, &callee.name, span))
}

/// The result paths of a call that point into a block the callee
/// renewed: each path of the caller a renewed result path names, unless
/// the callee can leave it without a block, or another free of the call
/// reaches it there. At the call a parameter path and a global can name
/// one block, so a free that does not reach the renewed path in the
/// callee can still give that block back. The digest keeps the renewed
/// and the unfilled paths apart, each growing through the rounds over a
/// loop of calls, so the difference is taken here, from the final one.
/// A result path that is no renewal can name the same block of the
/// caller, as a global borrow taken before the free does: the result can
/// then point into the old block, so no path it reaches is exempt.
///
/// # Arguments
///
/// * `callee` - The function.
/// * `digest` - Its digest.
/// * `args` - The roots of each argument before the call.
///
/// # Returns
///
/// The paths.
fn renewed_paths(callee: &Func, digest: &FnDigest, args: &[Roots]) -> Roots
{
    let frees: Vec<&Root> = digest.frees.iter().filter(|f| lent(callee, f)).collect();
    let kept = |r: &Root, at: &Root| {
        let alien = |f: &&Root| {
            !f.reaches(r) && translate(f, args).iter().any(|l| l.reaches(at))
        };
        !frees.iter().any(alien)
    };
    let filled = |r: &&Root| !digest.unfilled.iter().any(|u| u.reaches(r));
    let renewed: Vec<&Root> = digest.renewed.iter().filter(filled).collect();
    let is_fresh = |(r, at): &(&Root, Root)| renewed.contains(r) && kept(r, at);
    let found = digest
        .result_roots
        .iter()
        .flat_map(|r| translate(r, args).into_iter().map(move |at| (r, at)));
    let (fresh, stale): (Vec<_>, Vec<_>) = found.partition(is_fresh);
    let old = |at: &Root| stale.iter().any(|(_, s)| s.overlaps(at));
    fresh.into_iter().map(|(_, at)| at).filter(|at| !old(at)).collect()
}

/// Whether a free of the callee is a loss of the call, not a move into an
/// owner parameter: a free through a global or a borrow parameter.
///
/// # Arguments
///
/// * `callee` - The function.
/// * `free` - The free path, in callee terms.
///
/// # Returns
///
/// `true` for a global path or a path from a borrow parameter.
fn lent(callee: &Func, free: &Root) -> bool
{
    matches!(free.place, Place::Local(n) if !owns(callee, n)) || free.is_global()
}

/// The paths of the caller a call can free, resize or move through its
/// borrow parameters and the globals: the blocks it can give back.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `func` - The function number.
/// * `args` - The roots of each argument before the call.
///
/// # Returns
///
/// The paths; none for a function with no digest.
pub(crate) fn given_back(flow: &Flow, func: u32, args: &[Roots]) -> Roots
{
    let Some((callee, digest)) = callee_digest(flow, func)
    else
    {
        return Roots::new();
    };
    lost_paths(callee, digest, &freed_per_param(callee, digest, args))
}

/// The paths of the caller a call can free, resize or move through each
/// parameter, an owner one included, and through the globals.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `func` - The function number.
/// * `args` - The roots of each argument before the call.
///
/// # Returns
///
/// The paths; none for a function with no digest.
pub(crate) fn all_given_back(flow: &Flow, func: u32, args: &[Roots]) -> Roots
{
    let Some((callee, digest)) = callee_digest(flow, func)
    else
    {
        return Roots::new();
    };
    given_paths(digest, &freed_per_param(callee, digest, args))
}

/// The function and its digest.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `func` - The function number.
///
/// # Returns
///
/// Both, or `None` for a function with no digest.
fn callee_digest<'m>(flow: &Flow<'m>, func: u32) -> Option<(&'m Func, &'m FnDigest)>
{
    let callee = flow.module.funcs.get(func as usize)?;
    // Function pointers do not exist so far. A call through one would take
    // the digest that frees each place its arguments reach and each global.
    let digest = flow.ctx.digests.get(func as usize)?;
    Some((callee, digest))
}

/// What the callee can free through each parameter.
///
/// # Arguments
///
/// * `callee` - The function.
/// * `digest` - Its digest.
/// * `args` - The roots of each argument.
///
/// # Returns
///
/// The paths, one set for each argument.
fn freed_per_param(callee: &Func, digest: &FnDigest, args: &[Roots]) -> Vec<Roots>
{
    let count = args.len() as u32;
    (0..count).map(|n| freed_through(callee, digest, n, args)).collect()
}

/// The paths a call frees through its borrow parameters and the globals;
/// a move into an owner parameter is a move of its own.
///
/// # Arguments
///
/// * `callee` - The function.
/// * `digest` - Its digest.
/// * `freed` - What the callee can free through each parameter.
///
/// # Returns
///
/// The paths.
fn lost_paths(callee: &Func, digest: &FnDigest, freed: &[Roots]) -> Roots
{
    let lent = (0..freed.len()).filter(|&n| !owns(callee, n as u32));
    let mut lost: Roots = lent.flat_map(|n| freed[n].clone()).collect();
    lost.extend(digest.frees.iter().filter(|r| r.is_global()).cloned());
    lost
}

/// Whether a parameter takes over what its argument owns: the callee can
/// free it, so the parameter invalidates at each call.
///
/// # Arguments
///
/// * `callee` - The function.
/// * `n` - The parameter number from 0.
///
/// # Returns
///
/// `true` for a parameter of a move kind.
fn owns(callee: &Func, n: u32) -> bool
{
    callee.locals.get(n as usize).is_some_and(|slot| slot.kind.is_move_kind())
}

/// The places of the caller the callee can free through one parameter:
/// its digest frees through the parameter, moved over to the arguments,
/// and the whole value an owner parameter takes.
///
/// # Arguments
///
/// * `callee` - The function.
/// * `digest` - Its digest.
/// * `n` - The parameter number from 0.
/// * `args` - The roots of each argument.
///
/// # Returns
///
/// The paths.
fn freed_through(callee: &Func, digest: &FnDigest, n: u32, args: &[Roots]) -> Roots
{
    let through = digest.frees.iter().filter(|r| r.place == Place::Local(n));
    let mut found: Roots = through.flat_map(|r| translate(r, args)).collect();
    do_if!(owns(callee, n), found.extend(args[n as usize].iter().cloned()));
    found
}

/// Report the first argument that reaches a place the callee can free
/// through another argument.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `name` - The function name.
/// * `freed` - What the callee can free through each parameter.
/// * `args` - The roots of each argument.
/// * `span` - The call.
fn check_aliasing(
    flow: &mut Flow, name: &str, freed: &[Roots], args: &[Roots], span: Span,
)
{
    let clash = |&(n, m): &(usize, usize)| n != m && overlap(&freed[n], &args[m]);
    let mut pairs = (0..freed.len()).flat_map(|n| (0..args.len()).map(move |m| (n, m)));
    let Some((n, m)) = pairs.find(clash)
    else
    {
        return;
    };
    report(flow, aliased_argument(span, name, n + 1, m + 1));
}

/// The first borrow argument that reaches a global block the callee can
/// free: the parameter is left pointing at freed space inside the call,
/// as a global owner works like a parameter the callee frees through. An
/// owner argument moves the block in, and the global it came from is
/// blank inside the call, so it cannot clash.
///
/// # Arguments
///
/// * `callee` - The function.
/// * `digest` - Its digest.
/// * `args` - The roots of each argument.
///
/// # Returns
///
/// The global and the argument number from 0, or `None`.
fn global_clash(
    callee: &Func, digest: &FnDigest, args: &[Roots],
) -> Option<(Place, usize)>
{
    let lent = |(m, roots): (usize, &Roots)| {
        tern!(owns(callee, m as u32), Roots::new(), roots.clone())
    };
    let lent: Vec<Roots> = args.iter().enumerate().map(lent).collect();
    let mut globals = digest.frees.iter().filter(|f| f.is_global());
    globals.find_map(|g| reaching(&lent, g).map(|m| (g.place, m)))
}

/// The first parameter the callee frees, resizes or moves through whose
/// block a global the callee can read points into: a read inside the
/// call can hit freed space. The order of the two inside the callee is
/// not known, so each pair counts. A read through a parameter of the
/// callee is checked at the calls of the routine that hands the global
/// in, once it is lifted there as a read of that global.
///
/// # Arguments
///
/// * `digest` - The digest of the callee.
/// * `args` - The roots of each argument.
///
/// # Returns
///
/// The global and the parameter number from 0, or `None`.
fn read_clash(digest: &FnDigest, args: &[Roots]) -> Option<(Place, usize)>
{
    let through = |n: usize| {
        let mine = digest.frees.iter().filter(|r| r.place == Place::Local(n as u32));
        mine.flat_map(|r| translate(r, args)).collect()
    };
    let freed: Vec<Roots> = (0..args.len()).map(through).collect();
    let mut reads = digest.reads.iter().filter(|r| r.is_global());
    reads.find_map(|r| reaching(&freed, r).map(|n| (r.place, n)))
}

/// The report of a block of the caller freed inside a call while a
/// parameter of the callee still points into it: ZK0704 at the call,
/// named after the owner and the parameter.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `callee` - The function.
/// * `owner` - The slot the freed path starts at.
/// * `param` - The parameter number from 0.
/// * `span` - The call.
///
/// # Returns
///
/// The diagnostic.
fn clash_report(
    flow: &Flow, callee: &Func, owner: Place, param: usize, span: Span,
) -> Diagnostic
{
    let owner = slot_name(flow, owner);
    let other = callee.locals.get(param).map_or("", |slot| slot.name.as_str());
    let what = Invalidation::Freed.text();
    invalidated_loan(span, owner, &what, other, span)
}

/// The first set of paths a path overlaps.
///
/// # Arguments
///
/// * `sets` - The sets, one for each argument or parameter.
/// * `path` - The path.
///
/// # Returns
///
/// The number of the set, or `None` when none shares a place with it.
fn reaching(sets: &[Roots], path: &Root) -> Option<usize>
{
    sets.iter().position(|set| set.iter().any(|r| r.overlaps(path)))
}

/// Note a path from a parameter or a global a read or a write goes
/// through, in whatever order against the frees of the routine: a call
/// checks a global one against the blocks its arguments reach, and moves
/// one from a parameter over to its argument, which can be a global.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `addr` - The address read or written through.
/// * `state` - The facts when it runs.
pub(crate) fn note_read(flow: &mut Flow, addr: &Expr, state: &State)
{
    ret_if!(!flow.recording() || flow.routine.is_main);
    let roots = roots_of(addr, state, &flow.ctx);
    let params = flow.routine.params;
    flow.reads.extend(roots.into_iter().filter(|r| r.is_outer(params)));
}

/// Note the paths a callee can read, moved over to the arguments, as
/// reads of the routine: a call reaches them too. A read through a
/// parameter whose argument points into a global becomes a read of that
/// global.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `func` - The function number.
/// * `args` - The roots of each argument before the call.
pub(crate) fn lift_reads(flow: &mut Flow, func: u32, args: &[Roots])
{
    ret_if!(!flow.recording() || flow.routine.is_main);
    let Some((_, digest)) = callee_digest(flow, func)
    else
    {
        return;
    };
    let params = flow.routine.params;
    let moved = digest.reads.iter().flat_map(|r| translate(r, args));
    flow.reads.extend(moved.filter(|r| r.is_outer(params)));
}

/// Note the paths of the caller a call leaves without a block: those the
/// callee frees and does not fill again, moved over to the arguments.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `func` - The function number.
/// * `args` - The roots of each argument before the call.
/// * `state` - The facts, changed in place.
pub(crate) fn note_unfilled(flow: &Flow, func: u32, args: &[Roots], state: &mut State)
{
    ret_if!(flow.routine.is_main);
    let Some((_, digest)) = callee_digest(flow, func)
    else
    {
        return;
    };
    let params = flow.routine.params;
    let moved = digest.unfilled.iter().flat_map(|u| translate(u, args));
    let outer = moved.filter(|r| r.is_outer(params));
    state.unfilled.extend(outer);
}

/// Whether two sets of paths share a place.
///
/// # Arguments
///
/// * `a` - One set.
/// * `b` - The other set.
///
/// # Returns
///
/// `true` when a path of one reaches a path of the other.
fn overlap(a: &Roots, b: &Roots) -> bool
{
    a.iter().any(|mine| b.iter().any(|theirs| mine.overlaps(theirs)))
}
