use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_diagnostics::parser as diag;
use zakac_i18n::t;
use zakac_lexer::token::Kw;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::describe::kw_text;
use crate::describe::one_of;
use crate::describe::quote;
use crate::literal::decimal_value;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::is_number;
use crate::parser::kw_of;

/// Bit counts of integer kinds.
const INT_BITS: &[u16] = &[8, 16, 32, 64, 128];
/// Bit counts of floating-point kinds.
const FLOAT_BITS: &[u16] = &[16, 32, 64, 128];

/// The base kind a kind word names.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// The base kind, or `None` for another word.
fn base_kind(kw: Kw) -> Option<BaseKind>
{
    match kw
    {
        Kw::IntKind => Some(BaseKind::Int),
        Kw::FloatKind => Some(BaseKind::Float),
        Kw::CharKind => Some(BaseKind::Char),
        Kw::BoolKind | Kw::BoolKindLoan => Some(BaseKind::Bool),
        _ => None,
    }
}

/// The sign a sign word names.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// `Some(true)` for `လက္ခဏာရှိ`, `Some(false)` for `လက္ခဏာမဲ့`.
fn sign_of(kw: Kw) -> Option<bool>
{
    match kw
    {
        Kw::Signed => Some(true),
        Kw::Unsigned => Some(false),
        _ => None,
    }
}

impl Parser<'_>
{
    /// Whether a kind phrase starts at the current token.
    ///
    /// A number starts one when `ဘစ်` and an integer or float kind word
    /// follow it, so the distance in `ညာသို့ ၂ ဘစ် ရွေ့သောရလဒ်` is not read
    /// as a kind.
    ///
    /// # Returns
    ///
    /// `true` if a kind starts here.
    pub(crate) fn starts_kind(&self) -> bool
    {
        let kw = kw_of(&self.peek());
        ret_true!(
            kw.is_some_and(|kw| base_kind(kw).is_some() || sign_of(kw).is_some())
        );
        self.at_sized_kind(0)
    }

    /// Whether `N ဘစ် (ကိန်းပြည့်|ဒဿမကိန်း)` starts `n` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_sized_kind(&self, n: usize) -> bool
    {
        let base = kw_of(&self.peek_n(n + 2));
        is_number(&self.peek_n(n))
            && kw_of(&self.peek_n(n + 1)) == Some(Kw::Bit)
            && matches!(base, Some(Kw::IntKind | Kw::FloatKind))
    }

    /// Whether a kind phrase starts here and the reserved word `kw` follows
    /// it. Nothing is consumed; the scan reads at most five tokens.
    ///
    /// # Arguments
    ///
    /// * `kw` - The word that must follow the kind.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn kind_then(&self, kw: Kw) -> bool
    {
        let signed = kw_of(&self.peek()).and_then(sign_of).is_some();
        let sign = usize::from(signed);
        let n = sign + tern!(self.at_sized_kind(sign), 2, 0);
        let is_base = kw_of(&self.peek_n(n)).and_then(base_kind).is_some();
        is_base && kw_of(&self.peek_n(n + 1)) == Some(kw)
    }

    /// Parse a kind phrase.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic for a missing word, a sign word on a kind
    /// that is not an integer, or a bad bit count.
    pub(crate) fn parse_kind(&mut self) -> PResult<Kind>
    {
        let start = self.peek().span;
        let signed = kw_of(&self.peek()).and_then(sign_of);
        do_if!(signed.is_some(), self.advance());
        let bits = self.parse_bits();
        let base = self.parse_base_kind(bits.is_some())?;
        let span = start.to(&self.prev_span());
        ret_if!(
            signed.is_some() && base != BaseKind::Int,
            Err(diag::signed_float(span))
        );
        self.check_bits(bits, base)?;
        let bits = bits.map(|(count, _)| count);
        #[rustfmt::skip]
        return Ok(Kind::builder().base(base).maybe_bits(bits).maybe_signed(signed).span(span).build());
    }

    /// Parse `N ဘစ်` when it comes next.
    ///
    /// # Returns
    ///
    /// The bit count (0 when it is no plain decimal number) and its span.
    fn parse_bits(&mut self) -> Option<(u16, Span)>
    {
        let sized = is_number(&self.peek()) && kw_of(&self.peek_n(1)) == Some(Kw::Bit);
        ret_if!(!sized, None);
        let tok = self.bump();
        self.bump();
        let value = decimal_value(self.text_of(tok.span));
        let count = value.and_then(|v| u16::try_from(v).ok()).unwrap_or(0);
        Some((count, tok.span))
    }

    /// Parse the base kind word.
    ///
    /// # Arguments
    ///
    /// * `sized` - Whether a bit count came before; then the word must be
    ///   `ကိန်းပြည့်` or `ဒဿမကိန်း`.
    ///
    /// # Returns
    ///
    /// The base kind, or an `expected-token` diagnostic.
    fn parse_base_kind(&mut self, sized: bool) -> PResult<BaseKind>
    {
        let base = kw_of(&self.peek()).and_then(base_kind);
        let numeric = matches!(base, Some(BaseKind::Int | BaseKind::Float));
        ret_if!(sized && !numeric, Err(self.err_expected(&sized_kind_words())));
        let Some(base) = base
        else
        {
            return Err(self.err_expected(&t!("parser.kind")));
        };
        self.bump();
        Ok(base)
    }

    /// Check a bit count against the base kind.
    ///
    /// # Arguments
    ///
    /// * `bits` - The bit count and its span, when written.
    /// * `base` - The base kind; `ကိန်းပြည့်` or `ဒဿမကိန်း` when `bits` is set.
    ///
    /// # Returns
    ///
    /// `Ok` for an allowed count, else an `invalid-bit-count` diagnostic.
    fn check_bits(&self, bits: Option<(u16, Span)>, base: BaseKind) -> PResult<()>
    {
        let Some((count, span)) = bits
        else
        {
            return Ok(());
        };
        let allowed = tern!(base == BaseKind::Int, INT_BITS, FLOAT_BITS);
        ret_if!(allowed.contains(&count), Ok(()));
        let list: Vec<String> = allowed.iter().map(u16::to_string).collect();
        Err(diag::invalid_bit_count(span, self.text_of(span), &list.join(", ")))
    }
}

/// The kind words allowed after a bit count, for a message.
///
/// # Returns
///
/// `"ကိန်းပြည့်" or "ဒဿမကိန်း"`.
fn sized_kind_words() -> String
{
    one_of(&[quote(&kw_text(Kw::IntKind)), quote(&kw_text(Kw::FloatKind))])
}
