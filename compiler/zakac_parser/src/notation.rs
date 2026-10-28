use std::str::FromStr;

use zakac_ast::expr::BlankListLit;
use zakac_ast::expr::BoolLit;
use zakac_ast::expr::CallExpr;
use zakac_ast::expr::CharLit;
use zakac_ast::expr::Expr;
use zakac_ast::expr::GroupExpr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::InfixExpr;
use zakac_ast::expr::InfixOp;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::NullLit;
use zakac_ast::expr::PrefixExpr;
use zakac_ast::expr::PrefixOp;
use zakac_ast::expr::SizeOfExpr;
use zakac_ast::visit::Node;
use zakac_diagnostics::parser as diag;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::unquote_raw;

/// Binding power of the prefix operators; above each infix operator.
const PREFIX_BP: u8 = 120;

/// The infix operator a token spells, with its left and right binding power.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// The operator and powers, or `None` for another token.
fn infix_of(tok: &Token) -> Option<(InfixOp, u8, u8)>
{
    let TokenKind::Punct(punct) = tok.kind
    else
    {
        return None;
    };
    let op = InfixOp::from_str(punct.as_ref()).ok()?;
    let (lbp, rbp) = binding_power(op);
    Some((op, lbp, rbp))
}

/// Left and right binding power of an infix operator; operators with the
/// same C precedence share a row.
///
/// # Arguments
///
/// * `op` - The operator.
///
/// # Returns
///
/// The powers.
fn binding_power(op: InfixOp) -> (u8, u8)
{
    match op
    {
        InfixOp::Eq | InfixOp::Ne => (30, 31),
        InfixOp::Lt | InfixOp::Gt | InfixOp::Le | InfixOp::Ge => (40, 41),
        InfixOp::BitOr => (50, 51),
        InfixOp::BitXor => (60, 61),
        InfixOp::BitAnd => (70, 71),
        InfixOp::Shl | InfixOp::Shr => (80, 81),
        InfixOp::Add | InfixOp::Sub => (90, 91),
        InfixOp::Mul | InfixOp::Div | InfixOp::Rem => (100, 101),
    }
}

/// The prefix operator a token spells.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// The operator, or `None` for another token.
fn prefix_of(tok: &Token) -> Option<PrefixOp>
{
    match tok.kind
    {
        TokenKind::Punct(Punct::Minus) | TokenKind::Kw(Kw::Negate) =>
        {
            Some(PrefixOp::Neg)
        }
        TokenKind::Punct(Punct::Tilde) => Some(PrefixOp::BitNot),
        TokenKind::Punct(Punct::Bang) => Some(PrefixOp::Not),
        TokenKind::Punct(Punct::Star) => Some(PrefixOp::Deref),
        TokenKind::Punct(Punct::Amp) => Some(PrefixOp::AddrOf),
        _ => None,
    }
}

impl Parser<'_>
{
    /// Parse a notation expression.
    ///
    /// # Returns
    ///
    /// The expression, or a diagnostic.
    pub(crate) fn parse_notation(&mut self) -> PResult<Expr> { self.notation_bp(0) }

    /// Parse a notation expression whose operators bind at least `min`.
    ///
    /// # Arguments
    ///
    /// * `min` - The lowest left binding power to take.
    ///
    /// # Returns
    ///
    /// The expression, or a diagnostic.
    fn notation_bp(&mut self, min: u8) -> PResult<Expr>
    {
        let mut lhs = self.parse_prefix()?;
        while let Some((op, _, rbp)) =
            infix_of(&self.peek()).filter(|(_, l, _)| *l >= min)
        {
            self.bump();
            let rhs = self.notation_bp(rbp)?;
            lhs = self.infix(op, lhs, rhs)?;
        }
        Ok(lhs)
    }

    /// Build an infix node.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    ///
    /// # Returns
    ///
    /// The node, or a `chained-comparison` diagnostic.
    fn infix(&mut self, op: InfixOp, lhs: Expr, rhs: Expr) -> PResult<Expr>
    {
        let span = lhs.span().to(&rhs.span());
        let chained = op.is_cmp() && (lhs.is_cmp() || rhs.is_cmp());
        ret_if!(chained, Err(diag::chained_comparison(span)));
        let id = self.new_id();
        #[rustfmt::skip]
        let node = InfixExpr::builder().id(id).op(op).lhs(lhs).rhs(rhs).span(span).build();
        Ok(Expr::Infix(node))
    }

    /// Parse prefix operators and the atom after them.
    ///
    /// # Returns
    ///
    /// The expression, or a diagnostic.
    fn parse_prefix(&mut self) -> PResult<Expr>
    {
        let tok = self.peek();
        let Some(op) = prefix_of(&tok)
        else
        {
            return self.parse_atom();
        };
        self.bump();
        let rhs = self.notation_bp(PREFIX_BP)?;
        let span = tok.span.to(&rhs.span());
        let id = self.new_id();
        let node = PrefixExpr::builder().id(id).op(op).rhs(rhs).span(span).build();
        Ok(Expr::Prefix(node))
    }

    /// Parse an atom: a literal, a name, `( expr )`, a call expression,
    /// `ဗလာ` or `<kind> ၏ အရွယ်အစား`.
    ///
    /// # Returns
    ///
    /// The expression, or a diagnostic.
    fn parse_atom(&mut self) -> PResult<Expr>
    {
        let tok = self.peek();
        match tok.kind
        {
            TokenKind::Punct(Punct::LParen) => self.parse_group(),
            TokenKind::Ident | TokenKind::RawIdent | TokenKind::Kw(Kw::Result) =>
            {
                Ok(self.parse_name())
            }
            TokenKind::Kw(Kw::Function) => self.parse_call_expr(),
            TokenKind::Kw(Kw::True | Kw::False) => Ok(self.parse_bool()),
            TokenKind::Kw(Kw::Null) => Ok(self.parse_null()),
            TokenKind::Kw(Kw::BlankList) => Ok(self.parse_blank_list()),
            TokenKind::Char(value) => Ok(self.parse_char(value)),
            TokenKind::Number(_) | TokenKind::Kw(Kw::Base) if !self.at_size_of() =>
            {
                self.parse_literal()
            }
            TokenKind::Str(_) => Err(diag::str_outside_print(tok.span)),
            _ if self.starts_kind() =>
            {
                tern!(self.at_size_of(), self.parse_size_of(), self.parse_literal())
            }
            _ => Err(diag::expected_operand(tok.span, &self.found())),
        }
    }

    /// Parse `( expr )`.
    ///
    /// # Returns
    ///
    /// The group, or a diagnostic.
    fn parse_group(&mut self) -> PResult<Expr>
    {
        let open = self.bump();
        let inner = self.parse_expr()?;
        let close = self.expect_punct(Punct::RParen)?;
        let span = open.span.to(&close.span);
        let id = self.new_id();
        Ok(Expr::Group(GroupExpr::builder().id(id).inner(inner).span(span).build()))
    }

    /// Parse a variable, including the result variable `ရလဒ်`.
    ///
    /// # Returns
    ///
    /// The name.
    fn parse_name(&mut self) -> Expr
    {
        let tok = self.bump();
        let raw = tok.kind == TokenKind::RawIdent;
        let name = unquote_raw(raw, self.text_of(tok.span));
        let id = self.new_id();
        Expr::Name(NameExpr::builder().id(id).name(name).span(tok.span).build())
    }

    /// Parse `မှန်` or `မမှန်`.
    ///
    /// # Returns
    ///
    /// The literal, with the particle absorbed into it.
    fn parse_bool(&mut self) -> Expr
    {
        let tok = self.bump();
        let value = tok.kind == TokenKind::Kw(Kw::True);
        let id = self.new_id();
        #[rustfmt::skip]
        let lit = BoolLit::builder().id(id).value(value).particle(tok.particle).span(tok.span).build();
        Expr::Bool(lit)
    }

    /// Parse `ဗလာ`, which is the current token.
    ///
    /// # Returns
    ///
    /// The null literal.
    fn parse_null(&mut self) -> Expr
    {
        let tok = self.bump();
        let id = self.new_id();
        Expr::Null(NullLit::builder().id(id).span(tok.span).build())
    }

    /// Parse `စာရင်းအလွတ်`, which is the current token.
    ///
    /// # Returns
    ///
    /// The blank list.
    fn parse_blank_list(&mut self) -> Expr
    {
        let tok = self.bump();
        let id = self.new_id();
        Expr::BlankList(BlankListLit::builder().id(id).span(tok.span).build())
    }

    /// Parse `<kind> ၏ အရွယ်အစား`; [`Parser::at_size_of`] has passed.
    ///
    /// # Returns
    ///
    /// The size expression, or a diagnostic from the kind.
    fn parse_size_of(&mut self) -> PResult<Expr>
    {
        let kind = self.parse_kind()?;
        self.expect_punct(Punct::Ei)?;
        let end = self.expect(Kw::SizeOf)?;
        let span = kind.span.to(&end.span);
        let id = self.new_id();
        Ok(Expr::SizeOf(SizeOfExpr::builder().id(id).kind(kind).span(span).build()))
    }

    /// Parse a character literal.
    ///
    /// # Arguments
    ///
    /// * `value` - The decoded character.
    ///
    /// # Returns
    ///
    /// The literal.
    fn parse_char(&mut self, value: char) -> Expr
    {
        let tok = self.bump();
        let id = self.new_id();
        Expr::Char(CharLit::builder().id(id).value(value).span(tok.span).build())
    }

    /// Parse `လုပ်ဆောင်ချက် f ကို (args တို့ဖြင့်)? လုပ်ဆောင်သော ရလဒ်`.
    ///
    /// # Returns
    ///
    /// The call, or a diagnostic.
    fn parse_call_expr(&mut self) -> PResult<Expr>
    {
        let (start, callee, args) = self.parse_call_head()?;
        let end = self.expect(Kw::CallResult)?;
        Ok(self.call_expr(start.to(&end.span), callee, args))
    }

    /// Build a call expression node.
    ///
    /// # Arguments
    ///
    /// * `span` - The whole call.
    /// * `callee` - The function name.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// The expression.
    pub(crate) fn call_expr(
        &mut self, span: Span, callee: Ident, args: Vec<Expr>,
    ) -> Expr
    {
        let id = self.new_id();
        let node =
            CallExpr::builder().id(id).callee(callee).args(args).span(span).build();
        Expr::Call(node)
    }

    /// Parse `လုပ်ဆောင်ချက် f ကို (args တို့ဖြင့်)?`, shared between call
    /// expressions and call statements.
    ///
    /// # Returns
    ///
    /// The span of `လုပ်ဆောင်ချက်`, the function name and the arguments.
    pub(crate) fn parse_call_head(&mut self) -> PResult<(Span, Ident, Vec<Expr>)>
    {
        let start = self.bump().span;
        let callee = self.parse_ident()?;
        self.expect(Kw::Ko)?;
        let args = self.parse_call_args()?;
        Ok((start, callee, args))
    }

    /// Parse `expr (၊ expr)* တို့ဖြင့်`, or nothing when the call word comes
    /// next.
    ///
    /// # Returns
    ///
    /// The arguments, or a diagnostic.
    fn parse_call_args(&mut self) -> PResult<Vec<Expr>>
    {
        ret_if!(self.at(Kw::Call) || self.at(Kw::CallResult), Ok(Vec::new()));
        let mut args = vec![self.parse_expr()?];
        while self.eat_punct(Punct::ListSep)
        {
            args.push(self.parse_expr()?);
        }
        self.expect(Kw::ListWith)?;
        Ok(args)
    }
}
