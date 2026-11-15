//! Compile-time conditionals, `အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် …`: the loader keeps
//! the arm whose condition holds for the target, spliced into the statements
//! around it, and drops the others before names and kinds are checked. A
//! condition reads the facts of the target, text, digits, comparisons and
//! the logic words alone.

use zakac_ast::expr::Expr;
use zakac_ast::expr::InfixExpr;
use zakac_ast::expr::InfixOp;
use zakac_ast::expr::LogicOp;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::IfStmt;
use zakac_ast::stmt::Stmt;
use zakac_ast::target::FactValue;
use zakac_ast::target::Target;
use zakac_ast::visit::Node;
use zakac_ast::visit::VisitorMut;
use zakac_ast::visit::walk_stmt_mut;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser::TestFault;
use zakac_diagnostics::parser::bad_compile_time_test;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

/// Settle each compile-time conditional among some statements, at each
/// depth.
///
/// # Arguments
///
/// * `stmts` - The statements.
/// * `target` - The target.
/// * `text_of` - The source text of a region, for reports.
///
/// # Returns
///
/// The statements with each conditional replaced with the statements of the
/// arm it keeps, the diagnostics of the conditions, and whether a
/// conditional was met.
pub(crate) fn settle(
    stmts: Vec<Stmt>, target: &Target, text_of: &dyn Fn(Span) -> String,
) -> (Vec<Stmt>, Vec<Diagnostic>, bool)
{
    let mut settler = Settler { target, text_of, diags: Vec::new(), met: false };
    let stmts = settler.flatten(stmts);
    (stmts, settler.diags, settler.met)
}

/// The walker that settles compile-time conditionals.
struct Settler<'t>
{
    /// The target.
    target: &'t Target,
    /// The source text of a region.
    text_of: &'t dyn Fn(Span) -> String,
    /// The faults of the conditions.
    diags: Vec<Diagnostic>,
    /// Whether a compile-time conditional was met.
    met: bool,
}

/// A value of a compile-time condition.
enum Val
{
    /// Text: from a fact, with its name and each text it can hold, or a
    /// literal.
    Text(String, Option<(String, &'static [&'static str])>),
    /// A number.
    Num(u128),
    /// True or false.
    Bool(bool),
}

/// The reason a condition has no value, at its place.
type Fault = (Span, TestFault);

impl Settler<'_>
{
    /// Replace each compile-time conditional among some statements with the
    /// statements of the arm it keeps, and settle the blocks inside each
    /// statement kept.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    ///
    /// # Returns
    ///
    /// The statements.
    fn flatten(&mut self, stmts: Vec<Stmt>) -> Vec<Stmt>
    {
        let mut out = Vec::new();
        for mut stmt in stmts
        {
            let Stmt::If(n) = stmt
            else
            {
                walk_stmt_mut(self, &mut stmt);
                out.push(stmt);
                continue;
            };
            if !n.comptime
            {
                let mut kept = Stmt::If(n);
                walk_stmt_mut(self, &mut kept);
                out.push(kept);
                continue;
            }
            self.met = true;
            let kept = self.choose(n);
            out.extend(self.flatten(kept));
        }
        out
    }

    /// The statements of the arm a compile-time conditional keeps: the first
    /// arm whose condition holds, else the else block. Each condition is
    /// settled, so each fault is reported.
    ///
    /// # Arguments
    ///
    /// * `n` - The conditional.
    ///
    /// # Returns
    ///
    /// The statements; none when no arm holds and no else block is written.
    fn choose(&mut self, n: IfStmt) -> Vec<Stmt>
    {
        let holds: Vec<bool> = n.arms.iter().map(|a| self.holds(&a.cond)).collect();
        let arms = n.arms.into_iter().zip(holds);
        let kept = arms.filter(|(_, h)| *h).map(|(a, _)| a.block.stmts).next();
        kept.or_else(|| n.else_block.map(|b| b.stmts)).unwrap_or_default()
    }

    /// Whether a condition holds for the target; a fault is reported, and the
    /// condition counts as false.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    ///
    /// # Returns
    ///
    /// `true` when it holds.
    fn holds(&mut self, cond: &Expr) -> bool
    {
        let fault = |e: &Expr| (e.span(), TestFault::Form((self.text_of)(e.span())));
        let value = self.eval(cond).and_then(|v| match v
        {
            Val::Bool(b) => Ok(b),
            _ => Err(fault(cond)),
        });
        value.unwrap_or_else(|(span, f)| {
            self.diags.push(bad_compile_time_test(span, &f));
            false
        })
    }

    /// The value of a part of a condition.
    ///
    /// # Arguments
    ///
    /// * `e` - The part.
    ///
    /// # Returns
    ///
    /// The value, or the fault.
    fn eval(&self, e: &Expr) -> Result<Val, Fault>
    {
        let form = || (e.span(), TestFault::Form((self.text_of)(e.span())));
        match e
        {
            Expr::Group(n) => self.eval(&n.inner),
            Expr::Bool(n) => Ok(Val::Bool(n.value)),
            Expr::Int(n) => Ok(Val::Num(n.value)),
            Expr::Str(n) =>
            {
                n.plain().map(|t| Val::Text(t.to_string(), None)).ok_or_else(form)
            }
            Expr::Name(n) => self.fact(&n.name, n.span),
            Expr::Not(n) => self.truth(&n.inner).map(|b| Val::Bool(!b)),
            Expr::Logic(n) =>
            {
                let (lhs, rhs) = (self.truth(&n.lhs)?, self.truth(&n.rhs)?);
                Ok(Val::Bool(logic(n.op, lhs, rhs)))
            }
            Expr::Infix(n) if n.op.is_cmp() => self.compare(n),
            _ => Err(form()),
        }
    }

    /// The truth of a part of a condition.
    ///
    /// # Arguments
    ///
    /// * `e` - The part.
    ///
    /// # Returns
    ///
    /// The truth, or the fault for a part that is no truth.
    fn truth(&self, e: &Expr) -> Result<bool, Fault>
    {
        match self.eval(e)?
        {
            Val::Bool(b) => Ok(b),
            _ => Err((e.span(), TestFault::Form((self.text_of)(e.span())))),
        }
    }

    /// The value of a fact of the target.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `span` - The name where it stands.
    ///
    /// # Returns
    ///
    /// The value, or the fault for a name that is no fact.
    fn fact(&self, name: &str, span: Span) -> Result<Val, Fault>
    {
        match self.target.fact(name)
        {
            Some(FactValue::Text(v, all)) =>
            {
                Ok(Val::Text(v.to_string(), Some((name.to_string(), all))))
            }
            Some(FactValue::Bits(b)) => Ok(Val::Num(u128::from(b))),
            None => Err((span, TestFault::Name(name.to_string()))),
        }
    }

    /// The truth of a comparison: two texts with `==` or `!=`, a text the
    /// fact on the other side can hold; or two numbers with each comparison.
    ///
    /// # Arguments
    ///
    /// * `n` - The comparison.
    ///
    /// # Returns
    ///
    /// The truth, or the fault.
    fn compare(&self, n: &InfixExpr) -> Result<Val, Fault>
    {
        let (lhs, rhs) = (self.eval(&n.lhs)?, self.eval(&n.rhs)?);
        let mixed = (n.span, TestFault::Mixed);
        let equal = matches!(n.op, InfixOp::Eq | InfixOp::Ne);
        let same = match (lhs, rhs)
        {
            (Val::Text(a, fa), Val::Text(b, fb)) if equal =>
            {
                known_value(&fa, &b, n.rhs.span())?;
                known_value(&fb, &a, n.lhs.span())?;
                a == b
            }
            (Val::Bool(a), Val::Bool(b)) if equal => a == b,
            (Val::Num(a), Val::Num(b)) => return Ok(Val::Bool(ordered(n.op, a, b))),
            _ => return Err(mixed),
        };
        Ok(Val::Bool(same == (n.op == InfixOp::Eq)))
    }
}

impl VisitorMut for Settler<'_>
{
    fn visit_block_mut(&mut self, n: &mut BlockStmt)
    {
        n.stmts = self.flatten(std::mem::take(&mut n.stmts));
    }
}

/// Check that text compared with a fact is a text the fact can hold.
///
/// # Arguments
///
/// * `fact` - The fact on one side, with each text it can hold, when one stands
///   there.
/// * `text` - The text on the other side.
/// * `span` - The other side.
///
/// # Returns
///
/// `Ok`, or the fault for a text the fact never holds.
fn known_value(
    fact: &Option<(String, &'static [&'static str])>, text: &str, span: Span,
) -> Result<(), Fault>
{
    let Some((name, all)) = fact
    else
    {
        return Ok(());
    };
    ret_if!(all.contains(&text), Ok(()));
    let list = all.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(", ");
    Err((span, TestFault::Value(name.clone(), text.to_string(), list)))
}

/// The truth of a comparison of two numbers.
///
/// # Arguments
///
/// * `op` - The comparison.
/// * `a` - The left number.
/// * `b` - The right number.
///
/// # Returns
///
/// The truth.
fn ordered(op: InfixOp, a: u128, b: u128) -> bool
{
    match op
    {
        InfixOp::Eq => a == b,
        InfixOp::Ne => a != b,
        InfixOp::Lt => a < b,
        InfixOp::Gt => a > b,
        InfixOp::Le => a <= b,
        _ => a >= b,
    }
}

/// The truth of `နှင့်` or `သို့မဟုတ်` on two truths.
///
/// # Arguments
///
/// * `op` - The word.
/// * `a` - The left truth.
/// * `b` - The right truth.
///
/// # Returns
///
/// The truth.
fn logic(op: LogicOp, a: bool, b: bool) -> bool
{
    match op
    {
        LogicOp::And => a && b,
        LogicOp::Or => a || b,
    }
}
