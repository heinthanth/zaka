use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
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

/// A level after the base of a kind, before a count is matched to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Suffix
{
    /// `လိပ်စာ`
    Ptr,
    /// `လိပ်စာအကြမ်း`
    RawPtr,
    /// `စာရင်း`
    List,
}

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
    /// as a kind. A bare `လိပ်စာ` or `လိပ်စာအကြမ်း` starts one too, and so
    /// does a `ငှားထားသော` marker or a count prefix such as `အကွက် ၃ ခုရှိသော`.
    ///
    /// # Returns
    ///
    /// `true` if a kind starts here.
    pub(crate) fn starts_kind(&self) -> bool
    {
        let kw = kw_of(&self.peek());
        let starts = |kw| {
            base_kind(kw).is_some()
                || sign_of(kw).is_some()
                || matches!(kw, Kw::Address | Kw::RawAddress | Kw::Borrowed)
        };
        ret_true!(kw.is_some_and(starts));
        self.at_sized_kind(0) || self.at_count_prefix(0)
    }

    /// Whether `N ဘစ် (ကိန်းပြည့်|ဒဿမကိန်း)` starts `n` tokens ahead.
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

    /// The number of tokens in the kind phrase at the cursor, its borrow
    /// markers, count prefixes, pointer levels and list levels included.
    /// Nothing is consumed.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no kind starts here.
    pub(crate) fn kind_len(&self) -> Option<usize>
    {
        let marks = self.count_at(0, Kw::Borrowed);
        let pre = marks + 3 * self.count_prefixes(marks);
        let signed = kw_of(&self.peek_n(pre)).and_then(sign_of).is_some();
        let n = pre + usize::from(signed);
        let n = n + tern!(self.at_sized_kind(n), 2, 0);
        let word = kw_of(&self.peek_n(n));
        let is_base = word.and_then(base_kind).is_some();
        let bare = n == pre && matches!(word, Some(Kw::Address | Kw::RawAddress));
        ret_if!(!is_base && !bare, None);
        let (len, _) = self.scan_suffixes(n + 1, is_base);
        Some(n + 1 + len)
    }

    /// Whether `အကွက် <number> ခုရှိသော|ရှိသော` starts `n` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_count_prefix(&self, n: usize) -> bool
    {
        kw_of(&self.peek_n(n)) == Some(Kw::Cell)
            && is_number(&self.peek_n(n + 1))
            && kw_of(&self.peek_n(n + 2)) == Some(Kw::Having)
    }

    /// The number of count prefixes `from` tokens ahead; each takes three
    /// tokens.
    ///
    /// # Arguments
    ///
    /// * `from` - How far ahead the first prefix would start.
    ///
    /// # Returns
    ///
    /// The count of prefixes.
    fn count_prefixes(&self, from: usize) -> usize
    {
        (0..).take_while(|i| self.at_count_prefix(from + 3 * i)).count()
    }

    /// The length of the run of the reserved word `kw` that starts `n` tokens
    /// ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead the run would start.
    /// * `kw` - The word.
    ///
    /// # Returns
    ///
    /// The number of `kw` tokens in a row.
    fn count_at(&self, n: usize, kw: Kw) -> usize
    {
        (0..).take_while(|i| kw_of(&self.peek_n(n + i)) == Some(kw)).count()
    }

    /// The suffix that starts `n` tokens ahead, and its token count:
    /// `၏ လိပ်စာ`, `၏ လိပ်စာအကြမ်း`, `၏ စာရင်း`, a bare `စာရင်း`, or a bare
    /// `လိပ်စာ` or `လိပ်စာအကြမ်း` when `bare_ptr` allows one.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    /// * `bare_ptr` - Whether a pointer word without `၏` is a suffix here:
    ///   right after the base word or a list level.
    ///
    /// # Returns
    ///
    /// The suffix and its length, or `None` when no suffix starts there.
    fn suffix_at(&self, n: usize, bare_ptr: bool) -> Option<(Suffix, usize)>
    {
        let ei = usize::from(self.peek_n(n).kind == TokenKind::Punct(Punct::Ei));
        let suffix = match kw_of(&self.peek_n(n + ei))?
        {
            Kw::Address if ei == 1 || bare_ptr => Suffix::Ptr,
            Kw::RawAddress if ei == 1 || bare_ptr => Suffix::RawPtr,
            Kw::List => Suffix::List,
            _ => return None,
        };
        Some((suffix, ei + 1))
    }

    /// The suffixes that start `n` tokens ahead. A `၏` before another word
    /// ends them.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead the first suffix would start.
    /// * `after_base` - Whether the base word comes right before, so a bare
    ///   `လိပ်စာ` is a suffix.
    ///
    /// # Returns
    ///
    /// The token count of the suffixes, and the suffixes, innermost first.
    fn scan_suffixes(&self, n: usize, after_base: bool) -> (usize, Vec<Suffix>)
    {
        let (mut len, mut found, mut bare_ptr) = (0, Vec::new(), after_base);
        while let Some((suffix, more)) = self.suffix_at(n + len, bare_ptr)
        {
            len += more;
            found.push(suffix);
            bare_ptr = suffix == Suffix::List;
        }
        (len, found)
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

    /// Parse a kind phrase: `ငှားထားသော` markers, count prefixes such as
    /// `အကွက် ၃ ခုရှိသော`, the base, then pointer and list levels. A bare `လိပ်စာ` can
    /// follow the base word or a list level; `၏ လိပ်စာ` and `[၏] စာရင်း` can
    /// follow each level. Each `စာရင်း` takes the innermost count prefix
    /// left, or none.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic for a missing word, a bad count, a sign word
    /// on a kind that is not an integer, or a bad bit count.
    pub(crate) fn parse_kind(&mut self) -> PResult<Kind>
    {
        let start = self.peek().span;
        let marks = std::iter::from_fn(|| self.eat(Kw::Borrowed).then_some(())).count();
        let mut counts = self.parse_counts()?;
        let mut kind = self.parse_core()?;
        kind.borrowed = marks > 0;
        kind.extra_borrows = u8::try_from(marks.saturating_sub(1)).unwrap_or(u8::MAX);
        let (len, suffixes) = self.scan_suffixes(0, kind.base != BaseKind::Opaque);
        do_for!(_ in 0..len, self.advance());
        let wrap = |suffix| match suffix
        {
            Suffix::Ptr => Wrap::Ptr,
            Suffix::RawPtr => Wrap::RawPtr,
            Suffix::List =>
            {
                Wrap::List(counts.pop().map_or(Cells::Inferred, Cells::Fixed))
            }
        };
        kind.wraps.extend(suffixes.into_iter().map(wrap));
        ret_if!(!counts.is_empty(), Err(self.err_expected(&quote(&kw_text(Kw::List)))));
        kind.span = start.to(&self.prev_span());
        Ok(kind)
    }

    /// Parse the count prefixes `အကွက် <n> ခုရှိသော` that come next.
    ///
    /// # Returns
    ///
    /// The counts, outermost first, or a diagnostic for a count that is not
    /// a whole number above zero.
    fn parse_counts(&mut self) -> PResult<Vec<u32>>
    {
        let mut counts = Vec::new();
        while self.eat(Kw::Cell)
        {
            let tok = self.bump();
            let text = self.text_of(tok.span);
            let count = decimal_value(text).and_then(|v| u32::try_from(v).ok());
            let Some(count) = count.filter(|c| *c > 0)
            else
            {
                return Err(diag::bad_list_count(tok.span, text));
            };
            counts.push(count);
            self.expect(Kw::Having)?;
        }
        Ok(counts)
    }

    /// Parse the core of a kind: a bare `လိပ်စာ` or `လိပ်စာအကြမ်း`, or a base
    /// word with its sign and bit count.
    ///
    /// # Returns
    ///
    /// The kind without its suffixes, or a diagnostic.
    fn parse_core(&mut self) -> PResult<Kind>
    {
        let start = self.peek().span;
        let opaque = match kw_of(&self.peek())
        {
            Some(Kw::Address) => Some(Wrap::Ptr),
            Some(Kw::RawAddress) => Some(Wrap::RawPtr),
            _ => None,
        };
        if let Some(wrap) = opaque
        {
            self.advance();
            let kind = Kind::builder().base(BaseKind::Opaque).wraps(vec![wrap]);
            return Ok(kind.span(start).build());
        }
        let signed = kw_of(&self.peek()).and_then(sign_of);
        do_if!(signed.is_some(), self.advance());
        let bits = self.parse_bits();
        let base = self.parse_base_kind(bits.is_some())?;
        let span = start.to(&self.prev_span());
        let signed_float = signed.is_some() && base != BaseKind::Int;
        ret_if!(signed_float, Err(diag::signed_float(span)));
        self.check_bits(bits, base)?;
        let bits = bits.map(|(count, _)| count);
        Ok(Kind::builder()
            .base(base)
            .maybe_bits(bits)
            .maybe_signed(signed)
            .span(span)
            .build())
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
    ///   `ကိန်းပြည့်` or `ဒဿမကိန်း`.
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
    /// * `base` - The base kind; `ကိန်းပြည့်` or `ဒဿမကိန်း` when `bits` is set.
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
/// `"ကိန်းပြည့်" or "ဒဿမကိန်း"`.
fn sized_kind_words() -> String
{
    one_of(&[quote(&kw_text(Kw::IntKind)), quote(&kw_text(Kw::FloatKind))])
}
