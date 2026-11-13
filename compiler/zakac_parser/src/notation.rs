use std::str::FromStr;

use zakac_ast::expr::BlankListLit;
use zakac_ast::expr::BoolLit;
use zakac_ast::expr::CallExpr;
use zakac_ast::expr::CharLit;
use zakac_ast::expr::Expr;
use zakac_ast::expr::FnAddrExpr;
use zakac_ast::expr::GroupExpr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::InfixExpr;
use zakac_ast::expr::InfixOp;
use zakac_ast::expr::ListLit;
use zakac_ast::expr::MemberExpr;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::NullLit;
use zakac_ast::expr::PrefixExpr;
use zakac_ast::expr::PrefixOp;
use zakac_ast::expr::SizeOfExpr;
use zakac_ast::expr::TableLit;
use zakac_ast::expr::TableRow;
use zakac_ast::expr::VerbExpr;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_diagnostics::parser as diag;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Particle;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::clause::ClauseOut;
use crate::clause::Mode;
use crate::clause::as_verb;
use crate::describe::quote;
use crate::parser::MAX_NEST;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::kw_of;
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
pub(crate) fn infix_of(tok: &Token) -> Option<(InfixOp, u8, u8)>
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

/// The arguments of a call.
pub(crate) type Args = Vec<Expr>;

/// What the head of a long call holds: where it starts, the function name,
/// the place that holds the function when one is written, and the arguments.
pub(crate) type CallHead = (Span, Ident, Option<Box<Expr>>, Args);

/// Set the place of the function of a call expression.
///
/// # Arguments
///
/// * `call` - The call expression.
/// * `place` - The place that holds the function, when one is written.
///
/// # Returns
///
/// The call.
pub(crate) fn with_place(mut call: Expr, place: Option<Box<Expr>>) -> Expr
{
    do_if!(let Expr::Call(node) = &mut call, node.place = place);
    call
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
        ret_if!(self.nest >= MAX_NEST, Err(self.err_too_deep()));
        self.nest += 1;
        let parsed = self.notation_rest(min);
        self.nest -= 1;
        parsed
    }

    /// Parse an expression of operators once its depth is counted; see
    /// [`Self::notation_bp`].
    ///
    /// # Arguments
    ///
    /// * `min` - The lowest left binding power to take.
    ///
    /// # Returns
    ///
    /// The expression, or a diagnostic.
    fn notation_rest(&mut self, min: u8) -> PResult<Expr>
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
            let atom = self.parse_atom()?;
            return self.data_reads(atom);
        };
        self.bump();
        let rhs = self.notation_bp(PREFIX_BP)?;
        let span = tok.span.to(&rhs.span());
        let id = self.new_id();
        let node = PrefixExpr::builder().id(id).op(op).rhs(rhs).span(span).build();
        Ok(Expr::Prefix(node))
    }

    /// Parse an atom: a literal, a name, `( expr )`, `[a၊ b]`,
    /// `<struct>(<field>: <value>)`, a call expression in the word form or
    /// the sign form `f(a၊ b)`, `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ` or
    /// `<kind> တစ်<classifier> ၏ အရွယ်အစား`.
    ///
    /// # Returns
    ///
    /// The expression, or a diagnostic.
    fn parse_atom(&mut self) -> PResult<Expr>
    {
        let tok = self.peek();
        match tok.kind
        {
            TokenKind::Punct(Punct::LParen) if self.at_long_call() =>
            {
                self.parse_call_expr()
            }
            TokenKind::Punct(Punct::LParen) => self.parse_group(),
            TokenKind::Punct(Punct::LBracket) => self.parse_bracket_list(),
            TokenKind::Ident if self.at_call_struct_lit() =>
            {
                self.parse_call_struct_lit()
            }
            TokenKind::Ident | TokenKind::RawIdent if self.at_sign_call() =>
            {
                self.parse_sign_call()
            }
            TokenKind::Ident if self.is_struct_at(0) && self.at_size_of() =>
            {
                self.parse_size_of()
            }
            TokenKind::Ident
                if self.peek_n(1).kind == TokenKind::Kw(Kw::Among)
                    && !self.at_data_read(1)
                    && !self.at_row_read(1) =>
            {
                self.parse_member()
            }
            TokenKind::Ident | TokenKind::RawIdent if self.at_long_call() =>
            {
                self.parse_call_expr()
            }
            TokenKind::Ident | TokenKind::RawIdent if self.at_fn_value() =>
            {
                self.parse_fn_value()
            }
            TokenKind::Ident | TokenKind::RawIdent | TokenKind::Kw(Kw::Result) =>
            {
                Ok(self.parse_name())
            }
            TokenKind::Kw(Kw::True | Kw::False) => Ok(self.parse_bool()),
            TokenKind::Kw(Kw::Null | Kw::NoData | Kw::Initial) => Ok(self.parse_null()),
            TokenKind::Kw(Kw::BlankList) => self.parse_blank_list(),
            TokenKind::Kw(Kw::BlankTable) =>
            {
                let tok = self.bump();
                self.table_rest(tok.span, Vec::new())
            }
            TokenKind::Char(value) => Ok(self.parse_char(value)),
            TokenKind::Number(_) | TokenKind::Kw(Kw::Base) if !self.at_size_of() =>
            {
                self.parse_literal()
            }
            TokenKind::Str(idx) => self.parse_str(idx),
            _ if self.at_size_of() => self.parse_size_of(),
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
        let inner = self.outside_word_list(Self::parse_expr)?;
        let close = self.expect_punct(Punct::RParen)?;
        let span = open.span.to(&close.span);
        let id = self.new_id();
        Ok(Expr::Group(GroupExpr::builder().id(id).inner(inner).span(span).build()))
    }

    /// Parse `[a၊ b၊ c]` or `[]`, and its kind when
    /// `ဟူသော <kind> တစ်<classifier>` follows.
    ///
    /// # Returns
    ///
    /// The literal, or a diagnostic.
    fn parse_bracket_list(&mut self) -> PResult<Expr>
    {
        let open = self.bump();
        if self.at_punct(Punct::RBracket)
        {
            self.bump();
            return self.blank_list_rest(open.span, true);
        }
        if self.eat_punct(Punct::Colon)
        {
            self.expect_punct(Punct::RBracket)?;
            return self.table_rest(open.span, Vec::new());
        }
        let first = self.parse_notation()?;
        ret_if!(
            self.at_punct(Punct::Colon),
            self.parse_bracket_table(open.span, first)
        );
        let mut cells = vec![first];
        while self.eat_punct(Punct::ListSep)
        {
            cells.push(self.parse_notation()?);
        }
        self.expect_punct(Punct::RBracket)?;
        let kind = tern!(self.at_literal_kind(), Some(self.parse_list_kind()?), None);
        let span = open.span.to(&self.prev_span());
        let id = self.new_id();
        #[rustfmt::skip]
        return Ok(Expr::List(ListLit::builder().id(id).cells(cells).maybe_kind(kind).span(span).build()));
    }

    /// Parse the rest of `[<index>: <value>၊ …]` after its first index.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the literal starts.
    /// * `first` - The first index.
    ///
    /// # Returns
    ///
    /// The literal, or a diagnostic.
    fn parse_bracket_table(&mut self, start: Span, first: Expr) -> PResult<Expr>
    {
        let mut rows = Vec::new();
        let mut index = first;
        loop
        {
            self.expect_punct(Punct::Colon)?;
            let value = self.parse_notation()?;
            rows.push(TableRow::builder().index(index).value(value).build());
            ret_if!(!self.eat_punct(Punct::ListSep), {
                self.expect_punct(Punct::RBracket)?;
                self.table_rest(start, rows)
            });
            index = self.parse_notation()?;
        }
    }

    /// Finish a ဇယား literal once its rows are read: its kind when
    /// `ဟူသော <kind> တစ်<classifier>` follows.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the literal starts.
    /// * `rows` - The rows.
    ///
    /// # Returns
    ///
    /// The literal, or a `bad-literal-kind` diagnostic for a kind that is no
    /// ဇယား.
    pub(crate) fn table_rest(
        &mut self, start: Span, rows: Vec<TableRow>,
    ) -> PResult<Expr>
    {
        let kind = match self.at_literal_kind()
        {
            true =>
            {
                self.bump();
                let kind = self.parse_slot_kind()?;
                let table = kind.base == BaseKind::Table && kind.wraps.is_empty();
                ret_if!(!table, Err(diag::bad_literal_kind(kind.span)));
                Some(kind)
            }
            false => None,
        };
        let span = start.to(&self.prev_span());
        let id = self.new_id();
        #[rustfmt::skip]
        return Ok(Expr::Table(TableLit::builder().id(id).rows(rows).maybe_kind(kind).span(span).build()));
    }

    /// Parse `ဟူသော <kind> တစ်<classifier>` after a list literal, a list kind.
    ///
    /// # Returns
    ///
    /// The kind, or a `bad-literal-kind` diagnostic for a kind that is not a
    /// list.
    pub(crate) fn parse_list_kind(&mut self) -> PResult<Kind>
    {
        self.bump();
        let kind = self.parse_slot_kind()?;
        let list = matches!(kind.wraps.last(), Some(Wrap::List(_)));
        ret_if!(!list, Err(diag::bad_literal_kind(kind.span)));
        Ok(kind)
    }

    /// Parse `<enum>များထဲမှ <member>`, one member of an enumeration.
    ///
    /// # Returns
    ///
    /// The member, or a diagnostic.
    fn parse_member(&mut self) -> PResult<Expr>
    {
        let tok = self.bump();
        let qualifier =
            Ident::builder().name(self.text_of(tok.span)).span(tok.span).build();
        self.bump();
        let member = self.parse_ident()?;
        let span = tok.span.to(&member.span);
        let id = self.new_id();
        #[rustfmt::skip]
        let node = MemberExpr::builder().id(id).qualifier(qualifier).member(member).span(span).build();
        Ok(Expr::Member(node))
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

    /// Parse `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, `အလွတ်` or `ကနဦးတန်ဖိုး`, which is the current
    /// token.
    ///
    /// # Returns
    ///
    /// The literal.
    fn parse_null(&mut self) -> Expr
    {
        let tok = self.bump();
        let id = self.new_id();
        let blank = tok.kind == TokenKind::Kw(Kw::NoData);
        let initial = tok.kind == TokenKind::Kw(Kw::Initial);
        #[rustfmt::skip]
        let lit = NullLit::builder().id(id).blank(blank).initial(initial).span(tok.span).build();
        Expr::Null(lit)
    }

    /// The read of an optional or a result that starts `n` tokens ahead:
    /// `ထဲမှ ဒေတာ`, or `ထဲမှ အမှားအယွင်း ဒေတာ` for the error value.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead `ထဲမှ` stands.
    ///
    /// # Returns
    ///
    /// The operation, or `None` when no such read starts there.
    pub(crate) fn data_read_at(&self, n: usize) -> Option<VerbOp>
    {
        ret_if!(self.peek_n(n).kind != TokenKind::Kw(Kw::Among), None);
        match kw_of(&self.peek_n(n + 1))
        {
            Some(Kw::Data) => Some(VerbOp::Unwrap),
            Some(Kw::FaultData) => Some(VerbOp::UnwrapFault),
            _ => None,
        }
    }

    /// Whether the row of a ဇယား starts `n` tokens ahead: `ထဲမှ အညွှန်း`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead `ထဲမှ` stands.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    pub(crate) fn at_row_read(&self, n: usize) -> bool
    {
        self.peek_n(n).kind == TokenKind::Kw(Kw::Among)
            && kw_of(&self.peek_n(n + 1)) == Some(Kw::IndexWord)
    }

    /// Whether the fallback of a read of an optional comes next:
    /// `၊ မရှိလျှင်`.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_fallback(&self) -> bool
    {
        self.at_punct(Punct::ListSep)
            && kw_of(&self.peek_n(1)) == Some(Kw::NotLocated)
            && kw_of(&self.peek_n(2)) == Some(Kw::Then)
    }

    /// Parse `ထဲမှ အညွှန်း <index>` after a ဇယား, and the `၏ တန်ဖိုး` that
    /// reads the value of the row when it follows. The index is a notation
    /// expression, so it ends before the `၏`.
    ///
    /// # Arguments
    ///
    /// * `table` - The ဇယား.
    ///
    /// # Returns
    ///
    /// The row, or the value of the row, or a diagnostic.
    pub(crate) fn parse_row(&mut self, table: Expr) -> PResult<Expr>
    {
        self.bump();
        self.bump();
        let index = self.parse_notation()?;
        let value =
            self.at_punct(Punct::Ei) && kw_of(&self.peek_n(1)) == Some(Kw::Value);
        let (op, end, particle) = match value
        {
            true =>
            {
                self.bump();
                let tok = self.bump();
                (VerbOp::RowValue, tok.span, tok.particle)
            }
            false => (VerbOp::Row, index.span(), Particle::None),
        };
        let span = table.span().to(&end);
        let id = self.new_id();
        #[rustfmt::skip]
        let node = VerbExpr::builder().id(id).op(op).args(vec![table, index]).particle(particle)
            .is_cmp(false).span(span).build();
        Ok(Expr::Verb(node))
    }

    /// Parse `၊ မရှိလျှင် <value>` after the read of an optional: the data, or
    /// the value when the optional holds none. The value is one operand, a
    /// prefix operator and its operand included, as a read binds tighter than
    /// an operator: `<x> ထဲမှ ဒေတာ၊ မရှိလျှင် ၀ + ၁` adds 1 to either.
    ///
    /// # Arguments
    ///
    /// * `optional` - The optional read.
    ///
    /// # Returns
    ///
    /// The read with its fallback, or a diagnostic.
    pub(crate) fn parse_fallback(&mut self, optional: Expr) -> PResult<Expr>
    {
        do_for!(_ in 0..3, self.bump());
        let fallback = self.parse_prefix()?;
        let span = optional.span().to(&fallback.span());
        let id = self.new_id();
        #[rustfmt::skip]
        let node = VerbExpr::builder().id(id).op(VerbOp::OrElse).args(vec![optional, fallback])
            .particle(Particle::None).is_cmp(false).span(span).build();
        Ok(Expr::Verb(node))
    }

    /// Whether a read of an optional or a result starts `n` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead `ထဲမှ` stands.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    fn at_data_read(&self, n: usize) -> bool { self.data_read_at(n).is_some() }

    /// Wrap a value in each read of its data that follows it: `ထဲမှ ဒေတာ`
    /// or `ထဲမှ အမှားအယွင်း ဒေတာ`. A read binds tighter than an operator.
    ///
    /// # Arguments
    ///
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// The value under its reads.
    fn data_reads(&mut self, mut value: Expr) -> PResult<Expr>
    {
        loop
        {
            if self.at_row_read(0)
            {
                value = self.parse_row(value)?;
                continue;
            }
            let Some(op) = self.data_read_at(0)
            else
            {
                return Ok(value);
            };
            self.bump();
            let last = self.bump();
            let span = value.span().to(&last.span);
            let fallback = op == VerbOp::Unwrap && self.at_fallback();
            value = match fallback
            {
                true => self.parse_fallback(value)?,
                false => self.dup_expr(op, value, span, last.particle),
            };
        }
    }

    /// Parse `စာရင်းအလွတ်`, which is the current token, and its kind when
    /// `ဟူသော <kind> တစ်<classifier>` follows.
    ///
    /// # Returns
    ///
    /// The blank list, or a diagnostic.
    fn parse_blank_list(&mut self) -> PResult<Expr>
    {
        let tok = self.bump();
        self.blank_list_rest(tok.span, false)
    }

    /// Finish a blank list once its words are read: its kind when
    /// `ဟူသော <kind> တစ်<classifier>` follows.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the blank list starts.
    /// * `brackets` - Whether it is written `[]`.
    ///
    /// # Returns
    ///
    /// The blank list, or a diagnostic.
    fn blank_list_rest(&mut self, start: Span, brackets: bool) -> PResult<Expr>
    {
        let kind = tern!(self.at_literal_kind(), Some(self.parse_list_kind()?), None);
        let span = start.to(&self.prev_span());
        let id = self.new_id();
        #[rustfmt::skip]
        let node = BlankListLit::builder().id(id).span(span).brackets(brackets).maybe_kind(kind).build();
        Ok(Expr::BlankList(node))
    }

    /// Parse `<kind> တစ်<classifier> ၏ အရွယ်အစား`; [`Parser::at_size_of`] has
    /// passed.
    ///
    /// # Returns
    ///
    /// The size expression, or a diagnostic from the kind.
    fn parse_size_of(&mut self) -> PResult<Expr>
    {
        let start = self.peek().span;
        let kind = self.parse_slot_kind()?;
        self.expect_punct(Punct::Ei)?;
        let end = self.expect(Kw::SizeOf)?;
        let span = start.to(&end.span);
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

    /// Parse `f ဟူသော လုပ်ဆောင်ချက်ကို (args တို့ဖြင့်)? လုပ်ဆောင်သော ရလဒ်`, or
    /// `... ရလဒ်ကောင်း`, which passes a failure on.
    ///
    /// # Returns
    ///
    /// The call, or a diagnostic.
    fn parse_call_expr(&mut self) -> PResult<Expr>
    {
        let (start, callee, place, args) = self.parse_call_head()?;
        let end = self.expect_one_of(&[Kw::CallResult, Kw::CallGoodResult])?;
        let pass = end.kind == TokenKind::Kw(Kw::CallGoodResult);
        let call = self.call_expr(start.to(&end.span), callee, args, pass);
        Ok(with_place(call, place))
    }

    /// Whether the function of a long call starts here: a name or a place
    /// in parentheses, then `ဟူသော လုပ်ဆောင်ချက်ကို` (or `မည့်` for `ဟူသော`). A
    /// verb right after the `ကို` makes the function value the object of that
    /// verb instead, as in `f ဟူသော လုပ်ဆောင်ချက်ကို ပြန်ပေးမည်။`.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_long_call(&self) -> bool
    {
        let named = matches!(self.peek().kind, TokenKind::Ident | TokenKind::RawIdent);
        let after = tern!(named, Some(1), self.group_end());
        let Some(n) = after
        else
        {
            return false;
        };
        let head = [n, n + 1].map(|at| kw_of(&self.peek_n(at)));
        ret_if!(head != [Some(Kw::FnValue), Some(Kw::Ko)], false);
        let next = self.peek_n(n + 2);
        let object = kw_of(&next) == Some(Kw::Outside) || as_verb(&next).is_some();
        !object
    }

    /// How far ahead the token after the `)` that closes the group opening
    /// here is.
    ///
    /// # Returns
    ///
    /// The distance, or `None` when no group opens here or it does not close
    /// before the end of the sentence.
    fn group_end(&self) -> Option<usize>
    {
        ret_if!(!self.at_punct(Punct::LParen), None);
        let mut depth = 0usize;
        for n in 0..
        {
            match self.peek_n(n).kind
            {
                TokenKind::Punct(Punct::LParen) => depth += 1,
                TokenKind::Punct(Punct::RParen) if depth == 1 => return Some(n + 1),
                TokenKind::Punct(Punct::RParen) => depth -= 1,
                TokenKind::Punct(Punct::FullStop) | TokenKind::Eof => return None,
                _ => (),
            }
        }
        None
    }

    /// Whether a function value starts here: `<name> ဟူသော လုပ်ဆောင်ချက်`.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_fn_value(&self) -> bool
    {
        kw_of(&self.peek_n(1)) == Some(Kw::FnValue)
    }

    /// Parse a function value: `<name> ဟူသော လုပ်ဆောင်ချက်`, "the function
    /// called <name>".
    ///
    /// # Returns
    ///
    /// The expression, or a diagnostic.
    pub(crate) fn parse_fn_value(&mut self) -> PResult<Expr>
    {
        let name = self.parse_ident()?;
        let end = self.bump().span;
        let id = self.new_id();
        let span = name.span.to(&end);
        Ok(Expr::FnAddr(FnAddrExpr::builder().id(id).name(name).span(span).build()))
    }

    /// Whether the sign form of a call starts here: a name with `(` right
    /// after it, no space between. With a space, `f (a)` is a name, then a
    /// group.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_sign_call(&self) -> bool
    {
        let open = self.peek_n(1);
        open.kind == TokenKind::Punct(Punct::LParen) && open.joined
    }

    /// Parse the sign form of a call, `f(a၊ b)` or `f()`: the same node as
    /// `f ဟူသော လုပ်ဆောင်ချက်ကို a နှင့် b တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်`. A `?` right after
    /// the `)` passes a failure on, as `... ရလဒ်ကောင်း` does.
    ///
    /// # Returns
    ///
    /// The call, or a diagnostic.
    fn parse_sign_call(&mut self) -> PResult<Expr>
    {
        let callee = self.parse_ident()?;
        let mut call = self.parse_sign_tail(callee.clone(), None)?;
        // `f(a)(b)`: a `(` right after a call calls the function value it
        // gives.
        while self.at_sign_open()
        {
            call = self.parse_sign_tail(callee.clone(), Some(Box::new(call)))?;
        }
        Ok(call)
    }

    /// Whether a `(` touches the token before it, so a sign-form call goes on.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_sign_open(&self) -> bool
    {
        let open = self.peek();
        open.kind == TokenKind::Punct(Punct::LParen) && open.joined
    }

    /// Parse the arguments of a sign-form call, `(a၊ b)` or `()`, and the `?`
    /// that passes a failure on.
    ///
    /// # Arguments
    ///
    /// * `callee` - The function name: the first name of the chain.
    /// * `value` - The call whose result is the function, for `f(a)(b)`.
    ///
    /// # Returns
    ///
    /// The call, or a diagnostic.
    fn parse_sign_tail(
        &mut self, callee: Ident, value: Option<Box<Expr>>,
    ) -> PResult<Expr>
    {
        let start = value.as_ref().map_or(callee.span, |v| v.span());
        self.bump();
        let bare = self.at_punct(Punct::RParen);
        let args = tern!(
            bare,
            Args::default(),
            self.outside_word_list(Self::parse_sign_args)?
        );
        let close = self.expect_punct(Punct::RParen)?;
        let mark = self.peek();
        let pass = mark.kind == TokenKind::Punct(Punct::Question) && mark.joined;
        let end = tern!(pass, self.bump().span, close.span);
        let call = self.call_expr(start.to(&end), callee, args, pass);
        Ok(with_place(call, value))
    }

    /// Parse `expr (၊ expr)*` inside the parentheses of a sign-form call.
    ///
    /// # Returns
    ///
    /// The arguments with their marks, or a diagnostic.
    fn parse_sign_args(&mut self) -> PResult<Args>
    {
        let mut args = Args::default();
        self.parse_arg(&mut args)?;
        while self.eat_punct(Punct::ListSep)
        {
            self.parse_arg(&mut args)?;
        }
        Ok(args)
    }

    /// Parse one argument.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments so far, to add to.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic.
    fn parse_arg(&mut self, args: &mut Args) -> PResult<()>
    {
        args.push(self.parse_expr()?);
        Ok(())
    }

    /// Build a call expression node; [`with_place`] adds the place of its
    /// function, when one is written.
    ///
    /// # Arguments
    ///
    /// * `span` - The whole call.
    /// * `callee` - The function name.
    /// * `args` - The arguments, with the mark of each.
    /// * `pass` - Whether a failure of the call passes on.
    ///
    /// # Returns
    ///
    /// The expression.
    pub(crate) fn call_expr(
        &mut self, span: Span, callee: Ident, args: Args, pass: bool,
    ) -> Expr
    {
        let id = self.new_id();
        #[rustfmt::skip]
        let node = CallExpr::builder().id(id).callee(callee).args(args).pass(pass)
            .span(span).build();
        Expr::Call(node)
    }

    /// Parse `f ဟူသော လုပ်ဆောင်ချက်ကို (args တို့ဖြင့်)?`, shared between call
    /// expressions and call statements.
    ///
    /// # Returns
    ///
    /// The span of the function, the function name, the place that holds it
    /// and the arguments with their marks.
    pub(crate) fn parse_call_head(&mut self) -> PResult<CallHead>
    {
        let start = self.peek().span;
        let (callee, place) = self.parse_long_callee()?;
        let args = self.parse_call_args()?;
        Ok((start, callee, place, args))
    }

    /// Parse the function of a long call form with the words after it,
    /// `ဟူသော လုပ်ဆောင်ချက်ကို`: a name, or a place that holds a function value
    /// in parentheses, such as `(ဂရစ် ၏ အကွက် i)`.
    ///
    /// # Returns
    ///
    /// The name, the first name of the place, and the place when one is
    /// written; or a diagnostic.
    pub(crate) fn parse_long_callee(&mut self) -> PResult<(Ident, Option<Box<Expr>>)>
    {
        let callee = self.parse_callee()?;
        self.expect(Kw::FnValue)?;
        self.expect(Kw::Ko)?;
        Ok(callee)
    }

    /// Parse a name, or a place in parentheses that starts with a name.
    ///
    /// # Returns
    ///
    /// The name, the first name of the place, and the place when it is more
    /// than the name; or a diagnostic.
    fn parse_callee(&mut self) -> PResult<(Ident, Option<Box<Expr>>)>
    {
        let open = self.eat_punct(Punct::LParen);
        let callee = self.parse_ident()?;
        ret_if!(!open, Ok((callee, None)));
        let id = self.new_id();
        let name = NameExpr::builder().id(id).name(callee.name).span(callee.span);
        let place = Expr::Name(name.build());
        let read =
            self.outside_word_list(|p| p.parse_clause_from(place, Mode::Expr))?;
        self.expect_punct(Punct::RParen)?;
        let ClauseOut::Expr(place) = read
        else
        {
            return Err(self.err_expected(&quote(")")));
        };
        let bare = matches!(place, Expr::Name(_));
        Ok((callee, tern!(bare, None, Some(Box::new(place)))))
    }

    /// Parse the arguments of a long call as a list in words, then `ဖြင့်`
    /// after one or `တို့ဖြင့်` after two or more; or nothing when the call word
    /// comes next. An argument with a `နှင့်` of its own stands in parentheses.
    ///
    /// # Returns
    ///
    /// The arguments with their marks, or a diagnostic.
    fn parse_call_args(&mut self) -> PResult<Args>
    {
        let ends = [Kw::WillDo, Kw::CallResult, Kw::CallGoodResult];
        ret_if!(ends.iter().any(|kw| self.at(*kw)), Ok(Args::default()));
        let args = self.parse_word_list(Self::parse_expr)?;
        self.expect_list_word(args.len(), Kw::Hpjint, Kw::ListWith)?;
        Ok(args)
    }
}
