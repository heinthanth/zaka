//! Compile-time conditionals, `အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် …`: the loader keeps
//! the arm whose condition holds for the target, spliced into the statements
//! around it, and drops the others before names and kinds are checked. A
//! condition reads the facts of the target, text, digits, comparisons, the
//! logic words and calls of comptime functions. A condition that calls one
//! waits for the check, which runs the call: its chain remains open, and a
//! later read of the program settles it with the truth the run gave.

use std::collections::HashMap;

use zakac_ast::expr::ConstTree;
use zakac_ast::expr::Expr;
use zakac_ast::expr::InfixExpr;
use zakac_ast::expr::InfixOp;
use zakac_ast::expr::LogicOp;
use zakac_ast::kind::BaseKind;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::IfStmt;
use zakac_ast::stmt::Stmt;
use zakac_ast::target::FactValue;
use zakac_ast::target::Target;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::VisitorMut;
use zakac_ast::visit::walk_expr;
use zakac_ast::visit::walk_stmt_mut;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser::TestFault;
use zakac_diagnostics::parser::bad_compile_time_test;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

/// Where a condition of a compile-time conditional stands, the same in
/// each read of the program: the name of its file and its offset there.
pub type CondPlace = (String, u32);

/// The truth of each condition that calls a comptime function, as an
/// earlier read of the program worked it out.
pub type Resolved = HashMap<CondPlace, bool>;

/// The file a settle reads: its name, where it starts, and the conditions
/// worked out so far.
pub(crate) struct Spot<'a>
{
    /// The name of the file.
    pub file: String,
    /// The position of the start of the file.
    pub base: u32,
    /// The truth of each condition worked out so far.
    pub resolved: &'a Resolved,
}

/// What a settle gives: the statements, the faults of the conditions,
/// whether a conditional was met, and each condition still open.
pub(crate) type Settled = (Vec<Stmt>, Vec<Diagnostic>, bool, Vec<(Span, CondPlace)>);

/// Settle each compile-time conditional among some statements, at each
/// depth.
///
/// # Arguments
///
/// * `stmts` - The statements.
/// * `target` - The target.
/// * `text_of` - The source text of a region, for reports.
/// * `spot` - The file, and the conditions worked out so far.
///
/// # Returns
///
/// The statements with each conditional replaced with the statements of the
/// arm it keeps, or left in place while a condition that calls a comptime
/// function is open; the diagnostics of the conditions; whether a
/// conditional was met; and each open condition with its place.
pub(crate) fn settle(
    stmts: Vec<Stmt>, target: &Target, text_of: &dyn Fn(Span) -> String, spot: &Spot,
) -> Settled
{
    #[rustfmt::skip]
    let mut settler = Settler { target, text_of, spot, diags: Vec::new(), met: false,
        open: Vec::new() };
    let stmts = settler.flatten(stmts);
    (stmts, settler.diags, settler.met, settler.open)
}

/// Whether a condition holds, with the calls of comptime functions in it
/// worked out to values.
///
/// # Arguments
///
/// * `cond` - The condition.
/// * `target` - The target.
/// * `text_of` - The source text of a region, for reports.
///
/// # Returns
///
/// The truth, or `ZK0255`.
pub fn truth(
    cond: &Expr, target: &Target, text_of: &dyn Fn(Span) -> String,
) -> Result<bool, Diagnostic>
{
    let resolved = Resolved::new();
    let spot = Spot { file: String::new(), base: 0, resolved: &resolved };
    #[rustfmt::skip]
    let settler = Settler { target, text_of, spot: &spot, diags: Vec::new(), met: false,
        open: Vec::new() };
    settler.truth(cond).map_err(|(span, f)| bad_compile_time_test(span, &f))
}

/// Whether a condition calls a function.
///
/// # Arguments
///
/// * `cond` - The condition.
///
/// # Returns
///
/// `true` when it holds a call.
pub fn has_call(cond: &Expr) -> bool
{
    /// The walker that finds a call.
    struct Calls(bool);
    impl Visitor for Calls
    {
        fn visit_expr(&mut self, n: &Expr)
        {
            self.0 |= matches!(n, Expr::Call(_));
            walk_expr(self, n);
        }
    }
    let mut calls = Calls(false);
    calls.visit_expr(cond);
    calls.0
}

/// The walker that settles compile-time conditionals.
struct Settler<'t>
{
    /// The target.
    target: &'t Target,
    /// The source text of a region.
    text_of: &'t dyn Fn(Span) -> String,
    /// The file, and the conditions worked out so far.
    spot: &'t Spot<'t>,
    /// The faults of the conditions.
    diags: Vec<Diagnostic>,
    /// Whether a compile-time conditional was met.
    met: bool,
    /// Each condition still open, with its place.
    open: Vec<(Span, CondPlace)>,
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
            match self.choose(n)
            {
                Ok(kept) => out.extend(self.flatten(kept)),
                Err(open) => out.push(Stmt::If(open)),
            }
        }
        out
    }

    /// The statements of the arm a compile-time conditional keeps: the first
    /// arm whose condition holds, else the else block. Each condition is
    /// settled, so each fault is reported. A condition that calls a function
    /// takes the truth an earlier read worked out; while it has none and no
    /// arm before it holds, the conditional remains open.
    ///
    /// # Arguments
    ///
    /// * `n` - The conditional.
    ///
    /// # Returns
    ///
    /// The statements, none when no arm holds and no else block is written;
    /// or the conditional, while it is open.
    fn choose(&mut self, n: IfStmt) -> Result<Vec<Stmt>, IfStmt>
    {
        let mut kept = None;
        let mut open = false;
        for (i, arm) in n.arms.iter().enumerate()
        {
            let decided = kept.is_some() || open;
            let holds = match self.called(&arm.cond)
            {
                Some(Some(truth)) => truth,
                Some(None) =>
                {
                    do_if!(!decided, self.open_cond(&arm.cond));
                    open |= !decided;
                    false
                }
                None => self.holds(&arm.cond),
            };
            do_if!(holds && !decided, kept = Some(i));
        }
        ret_if!(open, Err(n));
        let arm = kept.and_then(|i| n.arms.into_iter().nth(i)).map(|a| a.block.stmts);
        Ok(arm.or_else(|| n.else_block.map(|b| b.stmts)).unwrap_or_default())
    }

    /// The truth of a condition that calls a function, as an earlier read
    /// worked it out.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    ///
    /// # Returns
    ///
    /// `None` for a condition with no call; `Some(None)` while it is open.
    fn called(&self, cond: &Expr) -> Option<Option<bool>>
    {
        ret_if!(!has_call(cond), None);
        Some(self.spot.resolved.get(&self.place(cond)).copied())
    }

    /// Record a condition that remains open.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    fn open_cond(&mut self, cond: &Expr)
    {
        self.open.push((cond.span(), self.place(cond)))
    }

    /// The place of a condition, the same in each read of the program.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    ///
    /// # Returns
    ///
    /// The place.
    fn place(&self, cond: &Expr) -> CondPlace
    {
        let offset = cond.span().lo.as_u32().saturating_sub(self.spot.base);
        (self.spot.file.clone(), offset)
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
            Expr::Comptime(n) => match (&n.value, n.kind.base)
            {
                (ConstTree::Scalar(raw), BaseKind::Bool) => Ok(Val::Bool(*raw != 0)),
                (ConstTree::Scalar(raw), BaseKind::Int) => Ok(Val::Num(*raw)),
                (ConstTree::Text(t), _) => Ok(Val::Text(t.clone(), None)),
                _ => Err(form()),
            },
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
