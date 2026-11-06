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
        Expr::Verb(n) if matches!(n.op, VerbOp::IsGood | VerbOp::IsFault) =>
        {
            let name = n.args.first().and_then(result_var);
            name.map_or_else(Vec::new, |name| {
                vec![(name, (n.op == VerbOp::IsGood) == holds)]
            })
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

/// The variables a statement can change by itself, the statements of its
/// blocks left out.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// The names.
pub(crate) fn changed_in_stmt(s: &Stmt) -> HashSet<Name>
{
    let mut found = Changed { names: HashSet::new(), deep: false };
    walk_stmt(&mut found, s);
    found.names
}

/// The variables the statements of a loop can change, at each depth.
///
/// # Arguments
///
/// * `stmts` - The statements.
///
/// # Returns
///
/// The names.
pub(crate) fn changed_in(stmts: &[Stmt]) -> HashSet<Name>
{
    let mut found = Changed { names: HashSet::new(), deep: true };
    do_for!(s in stmts, found.visit_stmt(s));
    found.names
}

/// Gathers the variables that statements can change: an assignment target,
/// an argument with the edit mark, and a place whose address is taken.
struct Changed
{
    /// The names found.
    names: HashSet<Name>,
    /// Whether the statements of blocks count too.
    deep: bool,
}

impl Changed
{
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

impl Visitor for Changed
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
        do_for!((a, _) in n.args.iter().zip(&n.edits).filter(|(_, e)| **e), self.note(a));
        walk_call_stmt(self, n);
    }

    fn visit_call_expr(&mut self, n: &CallExpr)
    {
        do_for!((a, _) in n.args.iter().zip(&n.edits).filter(|(_, e)| **e), self.note(a));
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
