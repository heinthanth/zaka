use zakac_ast::expr::Expr;
use zakac_ast::expr::PrefixOp;
use zakac_ast::verb::VerbOp;

/// The innermost place of an expression.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The variable itself, the dereference or cell node itself (a place of the
/// pointee or cell kind), the place inside parentheses, or `None` for an
/// expression that is no place.
pub(crate) fn place_of(e: &Expr) -> Option<&Expr>
{
    match e
    {
        Expr::Name(_) => Some(e),
        Expr::Group(n) => place_of(&n.inner),
        Expr::Prefix(n) if n.op == PrefixOp::Deref => Some(e),
        Expr::Verb(n) if n.op == VerbOp::Deref => Some(e),
        Expr::Index(_) => Some(e),
        _ => None,
    }
}
