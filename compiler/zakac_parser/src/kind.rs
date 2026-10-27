use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_diagnostics::parser as diag;
use zakac_i18n::t;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;
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
    /// as a kind. A bare `လိပ်စာ` starts one too.
    ///
    /// # Returns
    ///
    /// `true` if a kind starts here.
    pub(crate) fn starts_kind(&self) -> bool
    {
        let kw = kw_of(&self.peek());
        let starts =
            |kw| base_kind(kw).is_some() || sign_of(kw).is_some() || kw == Kw::Address;
        ret_true!(kw.is_some_and(starts));
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

    /// The number of tokens in the kind phrase at the cursor, its pointer
    /// levels included. Nothing is consumed.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no kind starts here.
    pub(crate) fn kind_len(&self) -> Option<usize>
    {
        let signed = kw_of(&self.peek()).and_then(sign_of).is_some();
        let sign = usize::from(signed);
        let n = sign + tern!(self.at_sized_kind(sign), 2, 0);
        let word = kw_of(&self.peek_n(n));
        let is_base = word.and_then(base_kind).is_some();
        let bare = n == 0 && word == Some(Kw::Address);
        ret_if!(!is_base && !bare, None);
        let lead = usize::from(is_base && self.at_address(n + 1));
        Some(n + 1 + lead + 2 * self.suffix_count(n + 1 + lead))
    }

    /// Whether `လိပ်စာ` is the token `n` ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    fn at_address(&self, n: usize) -> bool
    {
        kw_of(&self.peek_n(n)) == Some(Kw::Address)
    }

    /// The number of `၏ လိပ်စာ` pairs that start `n` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead the first pair would start.
    ///
    /// # Returns
    ///
    /// The count of pairs.
    fn suffix_count(&self, n: usize) -> usize
    {
        let at_pair = |i: usize| {
            self.peek_n(n + 2 * i).kind == TokenKind::Punct(Punct::Ei)
                && self.at_address(n + 2 * i + 1)
        };
        (0..).take_while(|i| at_pair(*i)).count()
    }

    /// Whether a kind phrase starts here and the reserved word `kw` follows
    /// it. Nothing is consumed.
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
        self.kind_len().is_some_and(|len| kw_of(&self.peek_n(len)) == Some(kw))
    }

    /// Whether `<kind> ၏ အရွယ်အစား` starts here. Nothing is consumed.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_size_of(&self) -> bool
    {
        self.kind_len().is_some_and(|len| {
            self.peek_n(len).kind == TokenKind::Punct(Punct::Ei)
                && kw_of(&self.peek_n(len + 1)) == Some(Kw::SizeOf)
        })
    }

    /// Parse a kind phrase with its pointer levels: one bare `လိပ်စာ` after
    /// the base word, then `၏ လိပ်စာ` pairs, as in `ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ`.
    /// The pairs can also start right after the base word.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic for a missing word, a sign word on a kind
    /// that is not an integer, or a bad bit count.
    pub(crate) fn parse_kind(&mut self) -> PResult<Kind>
    {
        ret_if!(self.at(Kw::Address), Ok(self.parse_opaque_kind()));
        let start = self.peek().span;
        let signed = kw_of(&self.peek()).and_then(sign_of);
        do_if!(signed.is_some(), self.advance());
        let bits = self.parse_bits();
        let base = self.parse_base_kind(bits.is_some())?;
        let span = start.to(&self.prev_span());
        let signed_float = signed.is_some() && base != BaseKind::Int;
        ret_if!(signed_float, Err(diag::signed_float(span)));
        self.check_bits(bits, base)?;
        let bits = bits.map(|(count, _)| count);
        let bare = self.at(Kw::Address);
        do_if!(bare, self.advance());
        let depth = self.parse_suffixes().saturating_add(u8::from(bare));
        let span = start.to(&self.prev_span());
        #[rustfmt::skip]
        return Ok(Kind::builder().base(base).maybe_bits(bits).maybe_signed(signed).depth(depth)
            .span(span).build());
    }

    /// Parse a bare `လိပ်စာ`, which is the current token, and its suffixes.
    ///
    /// # Returns
    ///
    /// The opaque kind; the bare word counts as one level.
    fn parse_opaque_kind(&mut self) -> Kind
    {
        let start = self.bump().span;
        let depth = self.parse_suffixes().saturating_add(1);
        let span = start.to(&self.prev_span());
        Kind::builder().base(BaseKind::Opaque).depth(depth).span(span).build()
    }

    /// Move past the `၏ လိပ်စာ` pairs that come next. A `၏` before another
    /// word ends the kind and is left in place.
    ///
    /// # Returns
    ///
    /// The number of pairs.
    fn parse_suffixes(&mut self) -> u8
    {
        let count = self.suffix_count(0);
        do_for!(_ in 0..2 * count, self.advance());
        u8::try_from(count).unwrap_or(u8::MAX)
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
