use zakac_ast::expr::Expr;
use zakac_ast::expr::LogicExpr;
use zakac_ast::expr::LogicOp;
use zakac_ast::expr::NextCtx;
use zakac_ast::expr::NotExpr;
use zakac_ast::visit::Node;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::extract;

use crate::clause::ClauseOut;
use crate::clause::Mode;
use crate::parser::PResult;
use crate::parser::Parser;

/// Binding power of postfix `မဟုတ်`; above `ဖြစ်၍` and `သို့မဟုတ်`.
const NOT_BP: u8 = 25;

/// The logical operator a token spells, with its left and right binding
/// power.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// The operator and powers, or `None` for another token.
fn logic_of(tok: &Token) -> Option<(LogicOp, u8, u8)>
{
    match tok.kind
    {
        TokenKind::Kw(Kw::Or) | TokenKind::Punct(Punct::OrOr) =>
        {
            Some((LogicOp::Or, 10, 11))
        }
        TokenKind::Kw(Kw::And) | TokenKind::Punct(Punct::AndAnd) =>
        {
            Some((LogicOp::And, 20, 21))
        }
        _ => None,
    }
}

/// What a token means for the particle before it.
///
/// # Arguments
///
/// * `tok` - The token after a verb, literal or `မဟုတ်`.
///
/// # Returns
///
/// The context.
fn next_ctx(tok: &Token) -> NextCtx
{
    match tok.kind
    {
        TokenKind::Kw(Kw::Or) | TokenKind::Punct(Punct::OrOr) => NextCtx::Or,
        TokenKind::Kw(Kw::Then) => NextCtx::Then,
        TokenKind::Kw(Kw::ThenIs) => NextCtx::ThenIs,
        TokenKind::Kw(Kw::While) => NextCtx::While,
        TokenKind::Kw(Kw::WhileIs) => NextCtx::WhileIs,
        TokenKind::Kw(Kw::And) | TokenKind::Punct(Punct::AndAnd) => NextCtx::And,
        TokenKind::Punct(Punct::RParen) => NextCtx::Close,
        _ => NextCtx::Other,
    }
}

/// Record what follows an expression that ends in a particle-bearing word.
///
/// # Arguments
///
/// * `expr` - The expression.
/// * `ctx` - What follows it.
fn mark_next(expr: &mut Expr, ctx: NextCtx)
{
    match expr
    {
        Expr::Verb(e) => e.next = ctx,
        Expr::Bool(e) => e.next = ctx,
        Expr::Not(e) => e.next = ctx,
        _ =>
        {}
    }
}

impl Parser<'_>
{
    /// Parse a full expression: clauses joined with logical words.
    ///
    /// # Returns
    ///
    /// The expression, or a diagnostic.
    pub(crate) fn parse_expr(&mut self) -> PResult<Expr>
    {
        let first = self.parse_clause_expr()?;
        self.parse_logical_rest(first, 0)
    }

    /// Parse a clause that must produce a value.
    ///
    /// # Returns
    ///
    /// The expression, or a diagnostic.
    pub(crate) fn parse_clause_expr(&mut self) -> PResult<Expr>
    {
        extract!(ClauseOut::Expr(expr) = self.parse_clause(Mode::Expr)?);
        Ok(expr)
    }

    /// Continue a logical expression from a parsed left clause.
    ///
    /// # Arguments
    ///
    /// * `lhs` - The left clause.
    /// * `min` - The lowest left binding power to take.
    ///
    /// # Returns
    ///
    /// The expression, or a diagnostic.
    pub(crate) fn parse_logical_rest(&mut self, mut lhs: Expr, min: u8)
    -> PResult<Expr>
    {
        loop
        {
            let tok = self.peek();
            mark_next(&mut lhs, next_ctx(&tok));
            if tok.kind == TokenKind::Kw(Kw::Not) && NOT_BP >= min
            {
                lhs = self.not_expr(lhs);
                continue;
            }
            let Some((op, _, rbp)) = logic_of(&tok).filter(|(_, l, _)| *l >= min)
            else
            {
                return Ok(lhs);
            };
            self.bump();
            let first = self.parse_clause_expr()?;
            let rhs = self.parse_logical_rest(first, rbp)?;
            lhs = self.logic_expr(op, lhs, rhs);
        }
    }

    /// Wrap a clause in postfix `မဟုတ်`, which is the current token.
    ///
    /// # Arguments
    ///
    /// * `inner` - The negated clause.
    ///
    /// # Returns
    ///
    /// The node.
    fn not_expr(&mut self, inner: Expr) -> Expr
    {
        let tok = self.bump();
        let span = inner.span().to(&tok.span);
        let id = self.new_id();
        #[rustfmt::skip]
        let node = NotExpr::builder().id(id).inner(inner).particle(tok.particle).span(span).build();
        Expr::Not(node)
    }

    /// Build a logical node.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `lhs` - The left clause.
    /// * `rhs` - The right clause.
    ///
    /// # Returns
    ///
    /// The node.
    fn logic_expr(&mut self, op: LogicOp, lhs: Expr, rhs: Expr) -> Expr
    {
        let span = lhs.span().to(&rhs.span());
        let id = self.new_id();
        let node =
            LogicExpr::builder().id(id).op(op).lhs(lhs).rhs(rhs).span(span).build();
        Expr::Logic(node)
    }
}
