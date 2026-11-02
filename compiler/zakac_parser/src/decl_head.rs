use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser as diag;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::kind::sign_of;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::is_number;
use crate::parser::kw_of;
use crate::struct_lit::ONE;
use crate::struct_lit::name_len;
use crate::struct_lit::part_ident;

/// The classifier of each kind that declares none.
const GENERAL: &str = "ခု";

/// The classifier of `စာလုံး`.
const CHARACTER: &str = "လုံး";

/// What follows `သည်` in a declaration or a generic loop.
#[derive(Debug, Clone, PartialEq, Builder)]
pub(crate) struct DeclHead
{
    /// The kind, when written.
    pub(crate) kind: Option<Kind>,
    /// The classifier after `တစ်`, written with the kind.
    pub(crate) classifier: Option<Ident>,
    /// The initial value, when written.
    pub(crate) init: Option<Expr>,
}

impl Parser<'_>
{
    /// Parse what follows `သည်` in a declaration or a generic loop:
    /// `<kind> တစ်<classifier>`, then `ဖြစ်၍ တန်ဖိုးသည် <expr>` for a value;
    /// or a value alone, whose kind the checker takes from it. A kind-marked
    /// literal such as `ကိန်းပြည့် ၁၂၇` is a value alone.
    ///
    /// # Returns
    ///
    /// The head, or a `ZK0234` diagnostic for a kind written without
    /// `တစ်<classifier>`.
    pub(crate) fn parse_decl_head(&mut self) -> PResult<DeclHead>
    {
        ret_if!(self.at_joined_count(), self.joined_head());
        let named = self.named_kind_len().filter(|len| self.is_count_word_at(*len));
        let Some(len) = self.kind_len().or(named)
        else
        {
            return self.value_head();
        };
        ret_if!(self.is_count_word_at(len), self.counted_head());
        ret_if!(self.value_goes_on(len), self.value_head());
        Err(self.old_form(len))
    }

    /// The token count of a kind phrase that starts with a name no struct
    /// declaration gives, such as `လုပ် လိပ်စာ`. With a count word after it,
    /// it is a kind all the same, and the checker reports the name.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no name starts here.
    fn named_kind_len(&self) -> Option<usize>
    {
        ret_if!(self.peek().kind != TokenKind::Ident, None);
        Some(1 + self.suffix_len(1))
    }

    /// Parse a value alone.
    ///
    /// # Returns
    ///
    /// The head with no kind.
    fn value_head(&mut self) -> PResult<DeclHead>
    {
        let init = self.parse_clause_expr()?;
        Ok(DeclHead::builder().init(init).build())
    }

    /// Parse `<kind> တစ်<classifier>`, then the value when one is written.
    ///
    /// # Returns
    ///
    /// The head.
    fn counted_head(&mut self) -> PResult<DeclHead>
    {
        let kind = self.parse_kind()?;
        let classifier = self.parse_count_word();
        let init = tern!(self.at(Kw::And), Some(self.parse_value_clause()?), None);
        Ok(DeclHead::builder()
            .kind(kind)
            .classifier(classifier)
            .maybe_init(init)
            .build())
    }

    /// Parse a struct name joined to its count word, as in `လူတစ်ယောက်`,
    /// then the value when one is written.
    ///
    /// # Returns
    ///
    /// The head.
    fn joined_head(&mut self) -> PResult<DeclHead>
    {
        let (name, classifier) = self.parse_struct_count()?;
        let span = name.span;
        let kind = Kind::builder().base(BaseKind::Named).name(name).span(span).build();
        let init = tern!(self.at(Kw::And), Some(self.parse_value_clause()?), None);
        Ok(DeclHead::builder()
            .kind(kind)
            .classifier(classifier)
            .maybe_init(init)
            .build())
    }

    /// Whether the token `n` ahead is a count word: `တစ်` and a classifier
    /// in one word, such as `တစ်ခု`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    fn is_count_word_at(&self, n: usize) -> bool
    {
        let tok = self.peek_n(n);
        let word = self.text_of(tok.span).strip_prefix(ONE);
        tok.kind == TokenKind::Ident && word.is_some_and(|w| !w.is_empty())
    }

    /// Whether a declared struct name joined to its count word is here.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    fn at_joined_count(&self) -> bool
    {
        let tok = self.peek();
        let text = self.text_of(tok.span);
        let Some(n) =
            name_len(text, &self.structs).filter(|_| tok.kind == TokenKind::Ident)
        else
        {
            return false;
        };
        let word = text[n..].strip_prefix(ONE).is_some_and(|w| !w.is_empty());
        word && self.structs.contains(&text[..n])
    }

    /// Parse a count word: the classifier after its `တစ်`.
    ///
    /// # Returns
    ///
    /// The classifier.
    fn parse_count_word(&mut self) -> Ident
    {
        let tok = self.bump();
        let text = self.text_of(tok.span);
        part_ident(&text[ONE.len()..], tok.span, ONE.len())
    }

    /// Whether the kind phrase of `len` tokens starts a value instead: a
    /// literal marked with a plain number kind, a size such as `ကိန်းပြည့် ၏
    /// အရွယ်အစား`, or a struct literal in the sign form.
    ///
    /// # Arguments
    ///
    /// * `len` - The token count of the kind phrase.
    ///
    /// # Returns
    ///
    /// `true` if the value goes on past the kind.
    fn value_goes_on(&self, len: usize) -> bool
    {
        let tok = self.peek_n(len);
        let marks = [Punct::Ei, Punct::LParen].map(TokenKind::Punct);
        let literal = is_number(&tok) && self.is_number_kind(len);
        literal || marks.contains(&tok.kind)
    }

    /// Whether the kind phrase of `len` tokens is a plain number kind, the
    /// one kind that can mark a literal: sign words, a bit count and the base
    /// word.
    ///
    /// # Arguments
    ///
    /// * `len` - The token count of the kind phrase.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    fn is_number_kind(&self, len: usize) -> bool
    {
        let word = |kw: Kw| {
            sign_of(kw).is_some() || matches!(kw, Kw::IntKind | Kw::FloatKind | Kw::Bit)
        };
        let tokens = (0..len).map(|n| self.peek_n(n));
        tokens.into_iter().all(|t| is_number(&t) || kw_of(&t).is_some_and(word))
    }

    /// The report of a kind written without its count word.
    ///
    /// # Arguments
    ///
    /// * `len` - The token count of the kind phrase.
    ///
    /// # Returns
    ///
    /// The `ZK0234` diagnostic, with the count word the kind takes.
    fn old_form(&self, len: usize) -> Diagnostic
    {
        let span = self.peek().span.to(&self.peek_n(len - 1).span);
        let text = self.text_of(span);
        let alone = len == 1;
        let char_kind = alone && kw_of(&self.peek()) == Some(Kw::CharKind);
        let own = self.classifiers.get(text).copied().filter(|_| alone);
        let word = own.unwrap_or(tern!(char_kind, CHARACTER, GENERAL));
        diag::decl_needs_classifier(span, text, word)
    }
}
