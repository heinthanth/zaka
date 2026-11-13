use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser as diag;
use zakac_lexer::spelling::canonical;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::clause::whole_word;
use crate::fn_kind::FnForm;
use crate::parser::PResult;
use crate::parser::Parser;
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
    /// The kind with its measure word, when written.
    pub(crate) kind: Option<Kind>,
    /// The initial value, when written.
    pub(crate) init: Option<Expr>,
}

impl Parser<'_>
{
    /// Parse what follows `သည်` in a declaration or a generic loop:
    /// `<kind> တစ်<classifier>`, then `ဖြစ်၍ တန်ဖိုးသည် <expr>` for a value;
    /// or a value alone, whose kind the checker takes from it. A size such as
    /// `ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား` is a value alone.
    ///
    /// # Returns
    ///
    /// The head, or a `ZK0234` diagnostic for a kind written without
    /// `တစ်<classifier>`.
    pub(crate) fn parse_decl_head(&mut self) -> PResult<DeclHead>
    {
        ret_if!(self.at_joined_count(), self.counted_head());
        let named = self.named_kind_len().filter(|len| self.is_count_word_at(*len));
        let Some(len) = self.kind_len().or(named)
        else
        {
            return self.value_head();
        };
        let counted = self.is_count_word_at(len);
        ret_if!(counted && !self.size_at(len + 1), self.counted_head());
        ret_if!(counted || self.value_goes_on(len), self.value_head());
        Err(self.needs_word(len))
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
        let kind = self.parse_slot_kind()?;
        let init = tern!(self.at(Kw::And), Some(self.parse_value_clause()?), None);
        Ok(DeclHead::builder().kind(kind).maybe_init(init).build())
    }

    /// Whether `၏ အရွယ်အစား` starts `n` tokens ahead, so the kind before it
    /// is measured, not declared.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn size_at(&self, n: usize) -> bool
    {
        self.peek_n(n).kind == TokenKind::Punct(Punct::Ei)
            && kw_of(&self.peek_n(n + 1)) == Some(Kw::SizeOf)
    }

    /// Parse a kind in a slot and the measure word after it,
    /// `<kind> တစ်<classifier>`, or a struct name joined to its word, as in
    /// `လူတစ်ယောက်`.
    ///
    /// # Returns
    ///
    /// The kind with its measure word, or a `ZK0234` diagnostic for a kind
    /// with no word after it.
    pub(crate) fn parse_slot_kind(&mut self) -> PResult<Kind>
    {
        self.parse_slot_kind_with(self.fn_form())
    }

    /// Parse a kind in a slot whose first token can start a bare function
    /// kind of `form`, and the measure word after it; see
    /// [`Self::parse_slot_kind`].
    ///
    /// # Arguments
    ///
    /// * `form` - The bare function kinds the kind can be.
    ///
    /// # Returns
    ///
    /// The kind with its measure word, or a diagnostic.
    pub(crate) fn parse_slot_kind_with(&mut self, form: FnForm) -> PResult<Kind>
    {
        if self.at_joined_count()
        {
            let (name, classifier) = self.parse_struct_count()?;
            let span = name.span;
            #[rustfmt::skip]
            return Ok(Kind::builder().base(BaseKind::Named).name(name).classifier(classifier)
                .span(span).build());
        }
        let start = self.cur;
        let mut kind = self.parse_kind_with(form)?;
        if !self.is_count_word_at(0)
        {
            let len = self.cur.pos() - start.pos();
            self.cur = start;
            return Err(self.needs_word(len));
        }
        kind.classifier = Some(self.parse_count_word());
        Ok(kind)
    }

    /// The token count of a kind in a slot that starts `at` tokens ahead,
    /// its measure word included.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    /// * `form` - The bare function kinds the kind can be.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no kind with its word starts there.
    pub(crate) fn slot_kind_len_at(&self, at: usize, form: FnForm) -> Option<usize>
    {
        ret_if!(self.joined_count_at(at), Some(1));
        let len = self.kind_len_at(at, form)?;
        self.is_count_word_at(at + len).then_some(len + 1)
    }

    /// Whether a kind starts here and the reserved word `kw` follows it, with
    /// or without its measure word between them. Nothing is consumed.
    ///
    /// # Arguments
    ///
    /// * `kw` - The word that must follow the kind.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn slot_then(&self, kw: Kw) -> bool
    {
        let Some(len) = self.kind_len()
        else
        {
            return false;
        };
        let len = len + usize::from(self.is_count_word_at(len));
        kw_of(&self.peek_n(len)) == Some(kw)
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
    pub(crate) fn is_count_word_at(&self, n: usize) -> bool
    {
        let tok = self.peek_n(n);
        let text = self.text_of(tok.span);
        // `တစ်ခုလုံး` after a name is "the whole of" a value, not its count word.
        ret_false!(whole_word(text).is_some());
        let word = text.strip_prefix(ONE);
        tok.kind == TokenKind::Ident && word.is_some_and(|w| !w.is_empty())
    }

    /// Whether a declared struct name joined to its count word is here.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    fn at_joined_count(&self) -> bool { self.joined_count_at(0) }

    /// Whether a declared struct name joined to its count word stands `n`
    /// tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn joined_count_at(&self, n: usize) -> bool
    {
        let tok = self.peek_n(n);
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
    pub(crate) fn parse_count_word(&mut self) -> Ident
    {
        let tok = self.bump();
        let text = self.text_of(tok.span);
        part_ident(&text[ONE.len()..], tok.span, ONE.len())
    }

    /// Whether the kind phrase of `len` tokens starts a value instead: a size
    /// whose word is missing, which the size reports, or a struct literal in
    /// the sign form, with or without `::<…>`.
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
        let marks = [Punct::Ei, Punct::LParen, Punct::PathSep].map(TokenKind::Punct);
        marks.contains(&self.peek_n(len).kind)
    }

    /// The report of a kind written without its measure word.
    ///
    /// # Arguments
    ///
    /// * `len` - The token count of the kind phrase, at the cursor.
    ///
    /// # Returns
    ///
    /// The `ZK0234` diagnostic, with the measure word the kind takes.
    pub(crate) fn needs_word(&self, len: usize) -> Diagnostic
    {
        let span = self.peek().span.to(&self.peek_n(len.max(1) - 1).span);
        let text = self.text_of(span);
        let alone = len == 1;
        let char_kind = alone && kw_of(&self.peek()) == Some(Kw::CharKind);
        let own =
            self.classifiers.get(canonical(text).as_ref()).copied().filter(|_| alone);
        let word = own.unwrap_or(tern!(char_kind, CHARACTER, GENERAL));
        diag::decl_needs_classifier(span, text, word)
    }
}
