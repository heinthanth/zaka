use std::collections::HashSet;

use zakac_ast::expr::Expr;
use zakac_ast::expr::FieldInit;
use zakac_ast::expr::Ident;
use zakac_ast::expr::StructLit;
use zakac_i18n::t;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::parser::PResult;
use crate::parser::Parser;

/// The word between a struct name and its classifier: `တစ်`, "one".
const ONE: &str = "တစ်";

/// Whether a token can start the value of a field: a name, a literal, `(`,
/// or a prefix operator that cannot also stand between two operands.
///
/// # Arguments
///
/// * `kind` - The token kind.
///
/// # Returns
///
/// `true` if it can.
fn starts_value(kind: TokenKind) -> bool
{
    matches!(
        kind,
        TokenKind::Ident
            | TokenKind::RawIdent
            | TokenKind::Number(_)
            | TokenKind::Char(_)
            | TokenKind::Punct(
                Punct::LParen
                    | Punct::LBracket
                    | Punct::Minus
                    | Punct::Tilde
                    | Punct::Bang
            )
            | TokenKind::Kw(
                Kw::True
                    | Kw::False
                    | Kw::Null
                    | Kw::BlankList
                    | Kw::Function
                    | Kw::Base
                    | Kw::Negate
                    | Kw::Result
            )
    )
}

/// Build a name from part of a word.
///
/// # Arguments
///
/// * `text` - The part.
/// * `word` - The span of the whole word.
/// * `at` - Offset of the part in the word, in octets.
///
/// # Returns
///
/// The name.
fn part_ident(text: &str, word: Span, at: usize) -> Ident
{
    let lo = word.lo + at;
    Ident::builder().name(text.to_string()).span(Span::new(lo, lo + text.len())).build()
}

impl Parser<'_>
{
    /// Whether a struct literal starts here: a field name with a value right
    /// after it, and `ရှိသော` ahead before the end of the clause.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_struct_lit(&self) -> bool
    {
        let named = matches!(self.peek().kind, TokenKind::Ident | TokenKind::RawIdent);
        ret_if!(!named || !starts_value(self.peek_n(1).kind), false);
        self.close_ahead(2, |p, n| p.peek_n(n).kind == TokenKind::Kw(Kw::Having))
    }

    /// Parse `<field> <value> (၊ <field> <value>)* ရှိသော
    /// <struct>တစ်<classifier>`.
    ///
    /// # Returns
    ///
    /// The literal, or a diagnostic.
    pub(crate) fn parse_struct_lit(&mut self) -> PResult<Expr>
    {
        let start = self.peek().span;
        let mut fields = vec![self.parse_field_init()?];
        while self.eat_punct(Punct::ListSep)
        {
            fields.push(self.parse_field_init()?);
        }
        self.expect(Kw::Having)?;
        let (name, classifier) = self.parse_struct_count()?;
        let span = start.to(&classifier.span);
        let id = self.new_id();
        #[rustfmt::skip]
        let lit = StructLit::builder().id(id).name(name).classifier(classifier).fields(fields)
            .span(span).build();
        Ok(Expr::Struct(lit))
    }

    /// Whether a notation struct literal starts here: a struct name and `(`,
    /// or another name and `(<field>:` or `()`. The checker reports a name
    /// that no struct declaration gives.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_call_struct_lit(&self) -> bool
    {
        let open = self.peek_n(1).kind == TokenKind::Punct(Punct::LParen);
        let named = self.peek_n(3).kind == TokenKind::Punct(Punct::Colon)
            || self.peek_n(2).kind == TokenKind::Punct(Punct::RParen);
        self.peek().kind == TokenKind::Ident && open && (named || self.is_struct_at(0))
    }

    /// Parse `<struct>(<field>: <value> (၊ <field>: <value>)*)`, or
    /// `<struct>()` with each field at its default;
    /// [`Parser::at_call_struct_lit`] has passed.
    ///
    /// # Returns
    ///
    /// The literal, or a diagnostic.
    pub(crate) fn parse_call_struct_lit(&mut self) -> PResult<Expr>
    {
        let name = self.parse_ident()?;
        self.bump();
        let bare = self.at_punct(Punct::RParen);
        let fields = tern!(bare, Vec::new(), self.parse_named_fields()?);
        let close = self.expect_punct(Punct::RParen)?;
        let span = name.span.to(&close.span);
        let id = self.new_id();
        let lit = StructLit::builder().id(id).name(name).fields(fields).span(span);
        Ok(Expr::Struct(lit.build()))
    }

    /// Parse `<field>: <value>` pairs with `၊` between them.
    ///
    /// # Returns
    ///
    /// The pairs, or a diagnostic.
    fn parse_named_fields(&mut self) -> PResult<Vec<FieldInit>>
    {
        let mut fields = vec![self.parse_named_field()?];
        while self.eat_punct(Punct::ListSep)
        {
            fields.push(self.parse_named_field()?);
        }
        Ok(fields)
    }

    /// Parse one `<field>: <value>` pair.
    ///
    /// # Returns
    ///
    /// The pair, or a diagnostic.
    fn parse_named_field(&mut self) -> PResult<FieldInit>
    {
        let field = self.parse_ident()?;
        self.expect_punct(Punct::Colon)?;
        let value = self.parse_notation()?;
        Ok(FieldInit::builder().field(field).value(value).build())
    }

    /// Parse one `<field> <value>` pair.
    ///
    /// # Returns
    ///
    /// The pair, or a diagnostic.
    fn parse_field_init(&mut self) -> PResult<FieldInit>
    {
        let field = self.parse_ident()?;
        let value = self.parse_notation()?;
        Ok(FieldInit::builder().field(field).value(value).build())
    }

    /// Parse `<struct>တစ်<classifier>`, written as one word, or with a space
    /// before `တစ်`. The checker reports a name that no struct declaration
    /// gives.
    ///
    /// # Returns
    ///
    /// The struct name and the classifier, or an `expected-token` diagnostic.
    fn parse_struct_count(&mut self) -> PResult<(Ident, Ident)>
    {
        let spaced = self.is_spaced_count();
        let tok = self.peek_n(usize::from(spaced));
        let text = self.text_of(tok.span);
        let name_len = tern!(spaced, Some(0), name_len(text, &self.structs));
        let word = name_len.and_then(|n| text[n..].strip_prefix(ONE));
        let word = word.filter(|w| tok.kind == TokenKind::Ident && !w.is_empty());
        let (Some(name_len), Some(word)) = (name_len, word)
        else
        {
            return Err(self.err_expected(&t!("parser.struct_count")));
        };
        let name = tern!(
            spaced,
            self.parse_ident()?,
            part_ident(&text[..name_len], tok.span, 0)
        );
        self.bump();
        let classifier = part_ident(word, tok.span, text.len() - word.len());
        Ok((name, classifier))
    }

    /// Whether the struct name stands as a word of its own before
    /// `တစ်<classifier>`.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn is_spaced_count(&self) -> bool
    {
        ret_true!(self.is_struct_at(0));
        let (first, next) = (self.peek(), self.peek_n(1));
        let joined = name_len(self.text_of(first.span), &self.structs).is_some();
        let names = first.kind == TokenKind::Ident && next.kind == TokenKind::Ident;
        names && !joined && self.text_of(next.span).starts_with(ONE)
    }
}

/// The length of the struct name a word starts with, before `တစ်`: the
/// longest declared name that fits, else the text before the first `တစ်`.
///
/// # Arguments
///
/// * `text` - The word.
/// * `structs` - The declared struct names.
///
/// # Returns
///
/// The length in octets, or `None` when the word holds no `တစ်` after a name.
fn name_len(text: &str, structs: &HashSet<&str>) -> Option<usize>
{
    let starts = |s: &&&str| text.strip_prefix(**s).is_some_and(|r| r.starts_with(ONE));
    let declared = structs.iter().filter(starts).map(|s| s.len()).max();
    declared.or_else(|| text.find(ONE).filter(|at| *at > 0))
}
