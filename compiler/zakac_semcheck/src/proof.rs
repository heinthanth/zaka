//! What a test of a result proves: inside the arm of `R သည် ရလဒ်ကောင်း`,
//! the result variable `R` holds a good result, and so on. While nothing
//! can change `R`, a conversion to the part the test ruled out would stop
//! the run each time it ran, so the check reports it.

use std::collections::HashSet;

use zakac_ast::expr::CallExpr;
use zakac_ast::expr::Expr;
use zakac_ast::expr::LogicOp;
use zakac_ast::expr::PrefixExpr;
use zakac_ast::expr::PrefixOp;
use zakac_ast::expr::VerbExpr;
use zakac_ast::name::Name;
use zakac_ast::stmt::AssignStmt;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::Stmt;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_assign;
use zakac_ast::visit::walk_block;
use zakac_ast::visit::walk_call_expr;
use zakac_ast::visit::walk_call_stmt;
use zakac_ast::visit::walk_prefix;
use zakac_ast::visit::walk_stmt;
use zakac_ast::visit::walk_verb;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_span::span::Span;

/// A result variable and the part it holds: `true` for a good result.
pub(crate) type Fact = (Name, bool);

/// The facts a condition proves when it holds, or when it does not.
///
/// # Arguments
///
/// * `cond` - The condition.
/// * `holds` - Whether the condition holds.
///
/// # Returns
///
/// The facts.
pub(crate) fn facts_when(cond: &Expr, holds: bool) -> Vec<Fact>
{
    match cond
    {
        Expr::Group(n) => facts_when(&n.inner, holds),
        Expr::Not(n) => facts_when(&n.inner, !holds),
        Expr::Logic(n) if (n.op == LogicOp::And) == holds =>
        {
            let mut facts = facts_when(&n.lhs, holds);
            facts.extend(facts_when(&n.rhs, holds));
            facts
        }
        Expr::Verb(n)
            if matches!(
                n.op,
                VerbOp::IsGood | VerbOp::IsFault | VerbOp::HasData | VerbOp::LacksData
            ) =>
        {
            let name = n.args.first().and_then(result_var);
            let good = matches!(n.op, VerbOp::IsGood | VerbOp::HasData);
            name.map_or_else(Vec::new, |name| vec![(name, good == holds)])
        }
        _ => Vec::new(),
    }
}

/// The variable a value names, through parentheses.
///
/// # Arguments
///
/// * `e` - The value.
///
/// # Returns
///
/// The name, or `None` for a value that is no plain variable.
pub(crate) fn result_var(e: &Expr) -> Option<Name>
{
    match e
    {
        Expr::Group(n) => result_var(&n.inner),
        Expr::Name(n) => Some(n.name),
        _ => None,
    }
}

/// Whether a block leaves at its end: its last statement returns,
/// breaks or goes on with the next round.
///
/// # Arguments
///
/// * `block` - The block.
///
/// # Returns
///
/// `true` if it does.
pub(crate) fn leaves_at_end(block: &BlockStmt) -> bool
{
    matches!(
        block.stmts.last(),
        Some(Stmt::Return(_) | Stmt::Break(_) | Stmt::Continue(_))
    )
}

/// The variables a statement can change on its own, the statements of its
/// blocks left out.
///
/// # Arguments
///
/// * `s` - The statement.
/// * `edits` - Whether each parameter of a function can be changed, under its
///   name; `None` for a name that is no function.
///
/// # Returns
///
/// The names.
pub(crate) fn changed_in_stmt(s: &Stmt, edits: &Edits<'_>) -> HashSet<Name>
{
    let mut found = Changed { names: HashSet::new(), deep: false, edits };
    walk_stmt(&mut found, s);
    found.names
}

/// The variables the statements of a loop can change, at each depth.
///
/// # Arguments
///
/// * `stmts` - The statements.
/// * `edits` - Whether each parameter of a function can be changed, under its
///   name; `None` for a name that is no function.
///
/// # Returns
///
/// The names.
pub(crate) fn changed_in(stmts: &[Stmt], edits: &Edits<'_>) -> HashSet<Name>
{
    let mut found = Changed { names: HashSet::new(), deep: true, edits };
    do_for!(s in stmts, found.visit_stmt(s));
    found.names
}

/// Whether each parameter of a function can be changed, under the
/// function name; `None` for a name that is no function.
pub(crate) type Edits<'e> = dyn Fn(&str) -> Option<Vec<bool>> + 'e;

/// Gathers the variables that statements can change: an assignment target,
/// an argument for a parameter marked `ဒေတာ ပြောင်းလဲရန်`, and a place whose
/// address is taken.
struct Changed<'e>
{
    /// The names found.
    names: HashSet<Name>,
    /// Whether the statements of blocks count too.
    deep: bool,
    /// Whether each parameter of a function can be changed.
    edits: &'e Edits<'e>,
}

impl Changed<'_>
{
    /// Note each argument of a call whose parameter can be changed; each
    /// argument of a call through a function value, whose parameters are
    /// not known here.
    ///
    /// # Arguments
    ///
    /// * `callee` - The function name.
    /// * `args` - The arguments.
    fn note_args(&mut self, callee: &str, args: &[Expr])
    {
        let edits = (self.edits)(callee);
        let edit = |n: usize| {
            edits.as_ref().is_none_or(|e| e.get(n).copied().unwrap_or(false))
        };
        do_for!(a in args.iter().enumerate().filter(|(n, _)| edit(*n)).map(|(_, a)| a), self.note(a));
    }

    /// Note the variable a place belongs to.
    ///
    /// # Arguments
    ///
    /// * `place` - The place.
    fn note(&mut self, place: &Expr)
    {
        do_if!(let Some(name) = root(place), _ = self.names.insert(name));
    }
}

impl Visitor for Changed<'_>
{
    fn visit_block(&mut self, n: &BlockStmt) { do_if!(self.deep, walk_block(self, n)) }

    fn visit_fn(&mut self, _: &FnDecl) {}

    fn visit_assign(&mut self, n: &AssignStmt)
    {
        self.note(&n.target);
        walk_assign(self, n);
    }

    fn visit_call_stmt(&mut self, n: &CallStmt)
    {
        self.note_args(&n.callee.name, &n.args);
        walk_call_stmt(self, n);
    }

    fn visit_call_expr(&mut self, n: &CallExpr)
    {
        self.note_args(&n.callee.name, &n.args);
        walk_call_expr(self, n);
    }

    fn visit_verb(&mut self, n: &VerbExpr)
    {
        let address = n.op == VerbOp::AddrOf;
        do_if!(let Some(a) = n.args.first().filter(|_| address), self.note(a));
        walk_verb(self, n);
    }

    fn visit_prefix(&mut self, n: &PrefixExpr)
    {
        do_if!(n.op == PrefixOp::AddrOf, self.note(&n.rhs));
        walk_prefix(self, n);
    }
}

/// The variable a place belongs to: the name at the root of its fields and
/// cells.
///
/// # Arguments
///
/// * `place` - The place.
///
/// # Returns
///
/// The name, or `None` for a place behind an address.
fn root(place: &Expr) -> Option<Name>
{
    match place
    {
        Expr::Group(n) => root(&n.inner),
        Expr::Name(n) => Some(n.name),
        Expr::Field(n) => root(&n.base),
        Expr::Index(n) => root(&n.list),
        _ => None,
    }
}

/// Whether a run of statements gives back on each path: its last statement
/// is a return, a block that gives back, an `if` with an else whose arms each
/// give back, or a switch with a default, or one that names each member of
/// its enumeration, whose arms each give back.
///
/// # Arguments
///
/// * `stmts` - The statements.
/// * `full` - The switches that name each member of their enumeration.
///
/// # Returns
///
/// `true` when no path reaches the end of the run.
pub(crate) fn returns_on_each_path(stmts: &[Stmt], full: &HashSet<Span>) -> bool
{
    let Some(last) = stmts.last()
    else
    {
        return false;
    };
    let gives = |b: &BlockStmt| returns_on_each_path(&b.stmts, full);
    match last
    {
        Stmt::Return(_) => true,
        Stmt::Block(b) => gives(b),
        Stmt::If(n) =>
        {
            n.else_block.as_ref().is_some_and(gives)
                && n.arms.iter().all(|a| gives(&a.block))
        }
        Stmt::Switch(n) =>
        {
            let default =
                n.default.as_ref().map_or(full.contains(&n.span), |d| gives(&d.block));
            default && n.arms.iter().all(|a| gives(&a.block))
        }
        // A loop whose test is the constant true and that no break leaves never
        // ends, so no path runs past it.
        Stmt::While(n) => is_true(&n.cond) && !breaks_out(&n.block.stmts),
        Stmt::Generic(n) => is_true(&n.cond) && !breaks_out(&n.block.stmts),
        _ => false,
    }
}

/// Whether an expression is the constant true, perhaps in parentheses.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// `true` for `မှန်`.
fn is_true(e: &Expr) -> bool
{
    match e
    {
        Expr::Group(n) => is_true(&n.inner),
        Expr::Bool(b) => b.value,
        _ => false,
    }
}

/// Whether a break among statements leaves the loop that holds them. A break
/// inside a loop nested in them leaves that loop alone, and one inside a
/// nested function belongs to that function.
///
/// # Arguments
///
/// * `stmts` - The statements of the loop block.
///
/// # Returns
///
/// `true` when one does.
fn breaks_out(stmts: &[Stmt]) -> bool
{
    let block = |b: &BlockStmt| breaks_out(&b.stmts);
    stmts.iter().any(|s| match s
    {
        Stmt::Break(_) => true,
        Stmt::Block(b) => block(b),
        Stmt::If(n) =>
        {
            n.arms.iter().any(|a| block(&a.block))
                || n.else_block.as_ref().is_some_and(block)
        }
        Stmt::Switch(n) =>
        {
            n.arms.iter().any(|a| block(&a.block))
                || n.default.as_ref().is_some_and(|d| block(&d.block))
        }
        _ => false,
    })
}
