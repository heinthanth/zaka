use std::collections::BTreeSet;

use zakac_diagnostics::Diagnostic;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::digest::FnDigest;
use crate::flow::FlowResult;
use crate::flow::run_flow;
use crate::interval::CellProofs;
use crate::options::BorrowCheckOptions;
use crate::roots::Roots;
use crate::routine::Routine;
use crate::walk::each_expr;
use crate::walk::each_stmt;
use crate::walk::stmt_exprs;

/// The functions of a module in groups that call each other, callees
/// first: a group calls no function of a later group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Groups
{
    /// The groups; each holds function numbers in ascending order.
    pub(crate) order: Vec<Vec<u32>>,
}

/// Add the function each call in `stmts` names to `out`, nested
/// statements and calls inside expressions included.
///
/// # Arguments
///
/// * `stmts` - The statements.
/// * `out` - The function numbers found so far.
pub(crate) fn callees(stmts: &[Stmt], out: &mut BTreeSet<u32>)
{
    each_stmt(stmts, &mut |s| {
        out.extend(call_target(s));
        let mut add = |e: &Expr| out.extend(expr_call_target(e));
        do_for!(e in stmt_exprs(s), each_expr(e, &mut add));
    });
}

/// The function a call statement names.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// The function number, or `None` for another statement.
fn call_target(s: &Stmt) -> Option<u32>
{
    tern!(let Stmt::Call { func, .. } = s, Some(*func), None)
}

/// The function a call expression names.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The function number, or `None` for another expression.
fn expr_call_target(e: &Expr) -> Option<u32>
{
    tern!(let ExprNode::Call { func, .. } = &e.node, Some(*func), None)
}

/// The digest of each function, with what the flow of each found: the
/// groups go callees first, and the functions of a group that calls
/// itself run again until no digest of the group grows. Each digest
/// starts blank, so a call into the group reads what the rounds so far
/// found. The last round runs with the digests it leaves unchanged, so
/// its diagnostics and proofs are those of the final digests.
///
/// # Arguments
///
/// * `module` - The module.
/// * `global_roots` - The roots top-level code gives each global.
/// * `options` - How to report.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The digest of each function in module order, the diagnostics and
/// the cell addresses met with and without a proof.
pub(crate) fn digests(
    module: &Module, global_roots: &[Roots], options: &BorrowCheckOptions,
    platform: u16,
) -> (Vec<FnDigest>, Vec<Diagnostic>, CellProofs)
{
    let mut digests: Vec<FnDigest> = module.funcs.iter().map(FnDigest::blank).collect();
    let mut diags = Vec::new();
    let mut proofs = CellProofs::default();
    for group in sccs(module).order
    {
        let found =
            settle(module, &group, &mut digests, global_roots, options, platform);
        for result in found
        {
            diags.extend(result.diags);
            proofs.add(&result.proofs, &result.doubts);
        }
    }
    (digests, diags, proofs)
}

/// Run the flow of each function of a group until its digests stop
/// growing; once for a group that does not call itself.
///
/// # Arguments
///
/// * `module` - The module.
/// * `group` - The function numbers.
/// * `digests` - The digest of each function, changed in place.
/// * `global_roots` - The roots top-level code gives each global.
/// * `options` - How to report.
/// * `platform` - The bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// What the last round found for each function of the group.
fn settle(
    module: &Module, group: &[u32], digests: &mut [FnDigest], global_roots: &[Roots],
    options: &BorrowCheckOptions, platform: u16,
) -> Vec<FlowResult>
{
    let again = calls_within(module, group);
    loop
    {
        let mut grew = false;
        let mut found = Vec::new();
        for &f in group
        {
            let routine = Routine::func(&module.funcs[f as usize]);
            let result =
                run_flow(module, routine, digests, global_roots, options, platform);
            let next = digests[f as usize].join(&result.digest);
            grew |= next != digests[f as usize];
            digests[f as usize] = next;
            found.push(result);
        }
        ret_if!(!grew || !again, found);
    }
}

/// Whether a function of a group calls one of the group.
///
/// # Arguments
///
/// * `module` - The module.
/// * `group` - The function numbers.
///
/// # Returns
///
/// `true` when the group calls itself.
fn calls_within(module: &Module, group: &[u32]) -> bool
{
    let mut out = BTreeSet::new();
    do_for!(&f in group, callees(&module.funcs[f as usize].block, &mut out));
    out.iter().any(|f| group.contains(f))
}

/// Split the functions of a module into the parts of the call graph whose
/// functions reach each other, with Tarjan's method.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The groups, callees first.
pub(crate) fn sccs(module: &Module) -> Groups
{
    let edges: Vec<Vec<u32>> = module
        .funcs
        .iter()
        .map(|f| {
            let mut out = BTreeSet::new();
            callees(&f.block, &mut out);
            out.into_iter().collect()
        })
        .collect();
    let mut walk = Tarjan::new(&edges);
    (0..edges.len()).for_each(|n| walk.visit_new(n));
    Groups { order: walk.order }
}

/// The state of one run of Tarjan's method.
struct Tarjan<'e>
{
    /// The callees of each function.
    edges: &'e [Vec<u32>],
    /// The visit number of each function, once visited.
    number: Vec<Option<u32>>,
    /// The smallest visit number each function reaches on the stack.
    low: Vec<u32>,
    /// Whether each function is on the stack.
    on_stack: Vec<bool>,
    /// The functions visited and not placed in a group.
    stack: Vec<usize>,
    /// The next visit number.
    next: u32,
    /// The groups found, callees first.
    order: Vec<Vec<u32>>,
}

impl<'e> Tarjan<'e>
{
    /// A run over the graph with no function visited.
    ///
    /// # Arguments
    ///
    /// * `edges` - The callees of each function.
    ///
    /// # Returns
    ///
    /// The run.
    fn new(edges: &'e [Vec<u32>]) -> Self
    {
        let n = edges.len();
        Self {
            edges,
            number: vec![None; n],
            low: vec![0; n],
            on_stack: vec![false; n],
            stack: Vec::new(),
            next: 0,
            order: Vec::new(),
        }
    }

    /// Visit a function unless it was visited before.
    ///
    /// # Arguments
    ///
    /// * `n` - The function.
    fn visit_new(&mut self, n: usize)
    {
        ret_if!(self.number[n].is_some());
        self.visit(n);
    }

    /// Visit a function and its callees; close its group when it is the
    /// root of one.
    ///
    /// # Arguments
    ///
    /// * `n` - The function.
    fn visit(&mut self, n: usize)
    {
        self.number[n] = Some(self.next);
        self.low[n] = self.next;
        self.next += 1;
        self.stack.push(n);
        self.on_stack[n] = true;
        for &m in &self.edges[n]
        {
            self.follow(n, m as usize);
        }
        do_if!(Some(self.low[n]) == self.number[n], self.close(n));
    }

    /// Follow the call from `n` to `m`.
    ///
    /// # Arguments
    ///
    /// * `n` - The caller.
    /// * `m` - The callee.
    fn follow(&mut self, n: usize, m: usize)
    {
        let Some(seen) = self.number[m]
        else
        {
            self.visit(m);
            self.low[n] = self.low[n].min(self.low[m]);
            return;
        };
        do_if!(self.on_stack[m], self.low[n] = self.low[n].min(seen));
    }

    /// Pop the group whose root is `n` off the stack.
    ///
    /// # Arguments
    ///
    /// * `n` - The root of the group.
    fn close(&mut self, n: usize)
    {
        let at =
            self.stack.iter().rposition(|&m| m == n).expect("the root is on the stack");
        let mut group: Vec<u32> = self.stack.drain(at..).map(|m| m as u32).collect();
        do_for!(&m in &group, self.on_stack[m as usize] = false);
        group.sort_unstable();
        self.order.push(group);
    }
}

#[cfg(test)]
mod tests
{
    use zakac_ir_builder::ir::Func;
    use zakac_span::span::Span;

    use super::*;

    /// A function that calls each of `calls` in turn.
    fn func(name: &str, calls: &[u32]) -> Func
    {
        let call = |&n: &u32| Stmt::Call {
            func: n,
            name: format!("f{n}"),
            args: Vec::new(),
            span: Span::default(),
        };
        let block = calls.iter().map(call).collect();
        Func::builder().name(name).params(0).block(block).span(Span::default()).build()
    }

    #[test]
    fn groups_come_callees_first()
    {
        let funcs = vec![func("f", &[1]), func("g", &[2]), func("h", &[1])];
        let module = Module::builder().funcs(funcs).span(Span::default()).build();
        assert_eq!(sccs(&module).order, vec![vec![1, 2], vec![0]]);
    }

    #[test]
    fn separate_functions_form_their_own_groups()
    {
        let funcs = vec![func("f", &[]), func("g", &[0, 0]), func("h", &[2])];
        let module = Module::builder().funcs(funcs).span(Span::default()).build();
        assert_eq!(sccs(&module).order, vec![vec![0], vec![1], vec![2]]);
    }
}
