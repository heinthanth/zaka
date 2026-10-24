use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::expr::Expr;
use crate::expr::IntLit;
use crate::expr::VerbExpr;
use crate::kind::BaseKind;
use crate::kind::Kind;
use crate::stmt::BlockStmt;
use crate::stmt::CountedStmt;
use crate::stmt::DeclStmt;
use crate::stmt::FnDecl;
use crate::stmt::GenericStmt;
use crate::stmt::IfStmt;
use crate::stmt::Param;
use crate::stmt::PrintArg;
use crate::stmt::Program;
use crate::stmt::Stmt;
use crate::verb::Dir;
use crate::verb::VerbOp;

/// Conversion to the S-expression form.
pub trait ToSexpr
{
    /// The S-expression form of the node.
    ///
    /// # Returns
    ///
    /// The text.
    fn to_sexpr(&self) -> String;
}

impl ToSexpr for Program
{
    fn to_sexpr(&self) -> String
    {
        list("program", self.stmts.iter().map(ToSexpr::to_sexpr))
    }
}

impl ToSexpr for BlockStmt
{
    fn to_sexpr(&self) -> String
    {
        list("block", self.stmts.iter().map(ToSexpr::to_sexpr))
    }
}

impl ToSexpr for Kind
{
    fn to_sexpr(&self) -> String
    {
        let bits = self.bits.map_or("size".to_string(), |b| b.to_string());
        let sign = tern!(self.signed == Some(false), 'u', 'i');
        match self.base
        {
            BaseKind::Int => format!("{sign}{bits}"),
            BaseKind::Float => format!("f{}", self.bits.unwrap_or(64)),
            BaseKind::Char | BaseKind::Bool => self.base.as_ref().to_string(),
        }
    }
}

impl ToSexpr for Expr
{
    fn to_sexpr(&self) -> String
    {
        match self
        {
            Expr::Int(n) => int_sexpr(n),
            Expr::Float(n) => format!("{:?}", n.value),
            Expr::Char(n) => format!("'{}'", n.value),
            Expr::Bool(n) => n.value.to_string(),
            Expr::Name(n) => n.name.clone(),
            Expr::Group(n) => format!("[{}]", n.inner.to_sexpr()),
            Expr::Prefix(n) => list(n.op.as_ref(), [n.rhs.to_sexpr()]),
            Expr::Infix(n) => list(n.op.as_ref(), [n.lhs.to_sexpr(), n.rhs.to_sexpr()]),
            Expr::Not(n) => list("not", [n.inner.to_sexpr()]),
            Expr::Logic(n) => list(n.op.as_ref(), [n.lhs.to_sexpr(), n.rhs.to_sexpr()]),
            Expr::Verb(n) => verb_sexpr(n),
            Expr::Call(n) => list("call", call_parts(&n.callee.name, &n.args)),
        }
    }
}

impl ToSexpr for Stmt
{
    fn to_sexpr(&self) -> String
    {
        match self
        {
            Stmt::Decl(s) => decl_sexpr(s),
            Stmt::Assign(s) => list("set", [s.target.name.clone(), s.value.to_sexpr()]),
            Stmt::Print(s) => list("print", [print_arg(&s.value)]),
            Stmt::Block(s) => s.to_sexpr(),
            Stmt::If(s) => if_sexpr(s),
            Stmt::While(s) => list("while", [s.cond.to_sexpr(), s.block.to_sexpr()]),
            Stmt::Counted(s) => counted_sexpr(s),
            Stmt::Generic(s) => generic_sexpr(s),
            Stmt::Break(_) => "(break)".into(),
            Stmt::Continue(_) => "(continue)".into(),
            Stmt::Fn(s) => fn_sexpr(s),
            Stmt::Return(s) => list("return", s.value.iter().map(ToSexpr::to_sexpr)),
            Stmt::Call(s) => list("call", call_parts(&s.callee.name, &s.args)),
        }
    }
}

/// `(head part part)`; just `(head)` without parts.
///
/// # Arguments
///
/// * `head` - The first word.
/// * `parts` - The rest, in S-expression form.
///
/// # Returns
///
/// The list.
fn list(head: &str, parts: impl IntoIterator<Item = String>) -> String
{
    let parts: Vec<String> = parts.into_iter().collect();
    ret_if!(parts.is_empty(), format!("({head})"));
    format!("({head} {})", parts.join(" "))
}

/// An integer literal with its kind prefix as a suffix: `18u8`.
///
/// # Arguments
///
/// * `n` - The literal.
///
/// # Returns
///
/// The text.
fn int_sexpr(n: &IntLit) -> String
{
    let kind = n.kind.map_or(String::new(), |k| k.to_sexpr());
    format!("{}{kind}", n.value)
}

/// `(op arg arg kind)` for a verb clause.
///
/// # Arguments
///
/// * `n` - The clause.
///
/// # Returns
///
/// The text.
fn verb_sexpr(n: &VerbExpr) -> String
{
    let args = n.args.iter().map(ToSexpr::to_sexpr);
    list(verb_name(n), args.chain(n.kind_arg.iter().map(ToSexpr::to_sexpr)))
}

/// The callee name followed with the arguments.
///
/// # Arguments
///
/// * `callee` - The function name.
/// * `args` - The arguments.
///
/// # Returns
///
/// The parts of a call list.
fn call_parts(callee: &str, args: &[Expr]) -> Vec<String>
{
    std::iter::once(callee.to_string())
        .chain(args.iter().map(ToSexpr::to_sexpr))
        .collect()
}

/// The operand of a print statement; a string literal is quoted.
///
/// # Arguments
///
/// * `arg` - The operand.
///
/// # Returns
///
/// The text.
fn print_arg(arg: &PrintArg) -> String
{
    match arg
    {
        PrintArg::Expr(e) => e.to_sexpr(),
        PrintArg::Str(s) => format!("\"{}\"", s.lit.value),
    }
}

/// `(var name kind init)` or `(const name kind init)`.
///
/// # Arguments
///
/// * `s` - The declaration.
///
/// # Returns
///
/// The text.
fn decl_sexpr(s: &DeclStmt) -> String
{
    let head = tern!(s.is_const, "const", "var");
    let header = [s.name.name.clone(), s.kind.to_sexpr()];
    list(head, header.into_iter().chain(s.init.iter().map(ToSexpr::to_sexpr)))
}

/// `(if cond {..} cond {..} else {..})`.
///
/// # Arguments
///
/// * `s` - The conditional.
///
/// # Returns
///
/// The text.
fn if_sexpr(s: &IfStmt) -> String
{
    let arms =
        s.arms.iter().flat_map(|arm| [arm.cond.to_sexpr(), arm.block.to_sexpr()]);
    let else_part =
        s.else_block.iter().flat_map(|b| ["else".to_string(), b.to_sexpr()]);
    list("if", arms.chain(else_part))
}

/// `(count var from to step N mode {..})`.
///
/// # Arguments
///
/// * `s` - The counted loop.
///
/// # Returns
///
/// The text.
fn counted_sexpr(s: &CountedStmt) -> String
{
    let bounds = [s.var.name.clone(), s.from.to_sexpr(), s.to.to_sexpr()];
    let step = s.step.iter().flat_map(|e| ["step".to_string(), e.to_sexpr()]);
    let tail = [s.mode.as_ref().to_string(), s.block.to_sexpr()];
    list("count", bounds.into_iter().chain(step).chain(tail))
}

/// `(for var kind init cond {..} then update)`.
///
/// # Arguments
///
/// * `s` - The generic loop.
///
/// # Returns
///
/// The text.
fn generic_sexpr(s: &GenericStmt) -> String
{
    let header = [s.var.name.clone(), s.kind.to_sexpr()];
    let init = s.init.iter().map(ToSexpr::to_sexpr);
    let loop_part = [s.cond.to_sexpr(), s.block.to_sexpr()];
    let update = s.update.iter().flat_map(|u| ["then".to_string(), u.to_sexpr()]);
    list("for", header.into_iter().chain(init).chain(loop_part).chain(update))
}

/// `(fn name [kind param, kind param] ret {..})`; `-` for no result.
///
/// # Arguments
///
/// * `s` - The function.
///
/// # Returns
///
/// The text.
fn fn_sexpr(s: &FnDecl) -> String
{
    let params: Vec<String> = s.params.iter().map(param_sexpr).collect();
    let ret = s.ret.map_or("-".to_string(), |k| k.to_sexpr());
    let parts = [s.name.name.clone(), format!("[{}]", params.join(", ")), ret];
    list("fn", parts.into_iter().chain([s.block.to_sexpr()]))
}

/// `kind name` for a parameter.
///
/// # Arguments
///
/// * `p` - The parameter.
///
/// # Returns
///
/// The text.
fn param_sexpr(p: &Param) -> String { format!("{} {}", p.kind.to_sexpr(), p.name.name) }

/// Name of a verb operation; a shift takes the name of its direction.
///
/// # Arguments
///
/// * `n` - The verb clause.
///
/// # Returns
///
/// The name.
fn verb_name(n: &VerbExpr) -> &str
{
    ret_if!(n.op != VerbOp::Shift, n.op.as_ref());
    tern!(n.dir == Some(Dir::Left), "shl", "shr")
}
