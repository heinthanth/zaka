use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::ResultParts;
use zakac_ast::kind::Wrap;
use zakac_diagnostics::parser as diag;
use zakac_i18n::t;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
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
use crate::fn_kind::FnForm;
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
    /// `တစ်စိတ်တစ်ပိုင်း`, after a `စာရင်း`
    Part,
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
        Kw::TextKind => Some(BaseKind::Text),
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
pub(crate) fn sign_of(kw: Kw) -> Option<bool>
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
                || matches!(
                    kw,
                    Kw::Address | Kw::RawAddress | Kw::Borrowed | Kw::SuccessNews
                )
        };
        ret_true!(kw.is_some_and(starts) || self.is_struct_at(0));
        ret_true!(kw == Some(Kw::FnKind) || self.paren_fn_len(0).is_some());
        let edit_first = kw == Some(Kw::EditParam) && !self.in_list;
        ret_true!(edit_first && self.fn_head_at(0, FnForm::Full).is_some());
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
        self.kind_len_at(0, self.fn_form())
    }

    /// The number of tokens in the kind phrase that starts `at` tokens
    /// ahead, its levels included. Nothing is consumed.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    /// * `form` - The bare function kinds the phrase can be.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no kind starts there.
    pub(crate) fn kind_len_at(&self, at: usize, form: FnForm) -> Option<usize>
    {
        let levels = |core: usize| core + self.scan_suffixes(at + core, true, false).0;
        ret_if!(let Some(core) = self.fn_head_at(at, form), Some(levels(core)));
        let marks = self.count_at(at, Kw::Borrowed);
        let pre = marks + self.count_prefixes_len(at + marks);
        // A function kind in parentheses can be the good side of a result kind.
        ret_if!(let Some(core) = self.result_core_len(at + pre), Some(levels(pre + core)));
        let prefixed = (pre > 0).then(|| self.fn_head_at(at + pre, form)).flatten();
        let fn_core = self.paren_fn_len(at + pre).or(prefixed);
        ret_if!(let Some(core) = fn_core, Some(levels(pre + core)));
        self.plain_kind_len(at)
    }

    /// The number of tokens in the two sides of a result kind that starts
    /// `at` tokens ahead, its pointer and list levels left out.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no result kind starts there.
    fn result_core_len(&self, at: usize) -> Option<usize>
    {
        if kw_of(&self.peek_n(at)) == Some(Kw::SuccessNews)
        {
            ret_if!(kw_of(&self.peek_n(at + 1)) != Some(Kw::Or), None);
            return self.err_side_len(at + 2).map(|len| 2 + len);
        }
        let n = self.paren_fn_len(at).or_else(|| self.plain_kind_len(at))?;
        let tail = match (kw_of(&self.peek_n(at + n)), kw_of(&self.peek_n(at + n + 1)))
        {
            (Some(Kw::Result), Some(Kw::Or)) => 2,
            (Some(Kw::Or), _) if self.ends_with_at(at, n, Kw::Result) => 1,
            _ => return None,
        };
        self.err_side_len(at + n + tail).map(|len| n + tail + len)
    }

    /// The number of tokens in the error side of a result kind that starts
    /// `n` tokens ahead: `မအောင်မြင်သည့်သတင်း`, or a kind and `အမှားအယွင်း`,
    /// which a struct name that ends in `အမှားအယွင်း` can leave out.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no error side starts there.
    fn err_side_len(&self, n: usize) -> Option<usize>
    {
        ret_if!(kw_of(&self.peek_n(n)) == Some(Kw::FailureNews), Some(1));
        let len = self.plain_kind_len(n)?;
        ret_if!(kw_of(&self.peek_n(n + len)) == Some(Kw::Fault), Some(len + 1));
        // A struct name that ends in `အမှားအယွင်း` stands alone, and the levels
        // after it belong to the whole result kind.
        Some(tern!(self.ends_with_at(n, 1, Kw::Fault), 1, len))
    }

    /// Whether the kind of `len` tokens `n` tokens ahead is a struct name
    /// that ends in the word of `kw`, so a result kind can leave that word
    /// out after it.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead the kind starts.
    /// * `len` - Its token count.
    /// * `kw` - `ရလဒ်` or `အမှားအယွင်း`.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn ends_with_at(&self, n: usize, len: usize, kw: Kw) -> bool
    {
        let tok = self.peek_n(n);
        len == 1
            && tok.kind == TokenKind::Ident
            && self.text_of(tok.span).ends_with(&kw_text(kw))
    }

    /// The number of tokens in the kind phrase with no result sides that
    /// starts `at` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no kind starts there.
    fn plain_kind_len(&self, at: usize) -> Option<usize>
    {
        let marks = self.count_at(at, Kw::Borrowed);
        let pre = marks + self.count_prefixes_len(at + marks);
        let signed = kw_of(&self.peek_n(at + pre)).and_then(sign_of).is_some();
        let n = pre + usize::from(signed);
        let n = n + tern!(self.at_sized_kind(at + n), 2, 0);
        let word = kw_of(&self.peek_n(at + n));
        let named = n == pre && self.is_struct_at(at + n);
        let is_base = word.and_then(base_kind).is_some() || named;
        let bare = n == pre && matches!(word, Some(Kw::Address | Kw::RawAddress));
        ret_if!(!is_base && !bare, None);
        let text = word.and_then(base_kind).is_some_and(BaseKind::is_text);
        let (len, _) = self.scan_suffixes(at + n + 1, is_base, text);
        Some(n + 1 + len)
    }

    /// Whether `အကွက် <count> ခုရှိသော|ရှိသော` starts `n` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_count_prefix(&self, n: usize) -> bool { self.count_prefix_len(n).is_some() }

    /// The number of tokens in the count prefix that starts `n` tokens
    /// ahead: `အကွက်`, a count, then the `ခုရှိသော` or `ရှိသော` that closes it.
    /// A count can hold kinds with counts of their own, as in a size, so each
    /// `အကွက်` that opens one inside it waits for its own `ခုရှိသော`; the full
    /// stop ends the search.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no count prefix starts there.
    fn count_prefix_len(&self, n: usize) -> Option<usize>
    {
        ret_if!(kw_of(&self.peek_n(n)) != Some(Kw::Cell), None);
        let mut depth = 1usize;
        for at in n + 1..
        {
            let tok = self.peek_n(at);
            ret_if!(ends_sentence(&tok), None);
            // `အကွက်` after `၏` or `ခုမြောက်` reads a cell; elsewhere it opens a
            // count of its own.
            let prev = self.peek_n(at - 1);
            let read = prev.kind == TokenKind::Punct(Punct::Ei)
                || kw_of(&prev) == Some(Kw::Ordinal);
            match kw_of(&tok)
            {
                Some(Kw::Cell) if !read => depth += 1,
                Some(Kw::Having) => depth -= 1,
                _ => (),
            }
            ret_if!(depth == 0, (at > n + 1).then_some(at - n + 1));
        }
        None
    }

    /// The number of tokens in the run of count prefixes `from` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `from` - How far ahead the first prefix would start.
    ///
    /// # Returns
    ///
    /// The token count; 0 when no prefix starts there.
    fn count_prefixes_len(&self, from: usize) -> usize
    {
        let next = |at: &usize| self.count_prefix_len(*at).map(|len| at + len);
        std::iter::successors(Some(from), next).last().unwrap_or(from) - from
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
    /// `၏ လိပ်စာ`, `၏ လိပ်စာအကြမ်း`, `၏ စာရင်း`, a bare `စာရင်း`, a bare
    /// `လိပ်စာ` or `လိပ်စာအကြမ်း` when `after_list` or the base allows one, or
    /// `တစ်စိတ်တစ်ပိုင်း` right after a `စာရင်း`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    /// * `bare_ptr` - Whether a pointer word without `၏` is a suffix here:
    ///   right after the base word or a list level.
    /// * `after_list` - Whether the suffix before is a `စာရင်း`.
    ///
    /// # Returns
    ///
    /// The suffix and its length, or `None` when no suffix starts there.
    fn suffix_at(
        &self, n: usize, bare_ptr: bool, after_list: bool,
    ) -> Option<(Suffix, usize)>
    {
        let ei = usize::from(self.peek_n(n).kind == TokenKind::Punct(Punct::Ei));
        let suffix = match kw_of(&self.peek_n(n + ei))?
        {
            Kw::Address if ei == 1 || bare_ptr => Suffix::Ptr,
            Kw::RawAddress if ei == 1 || bare_ptr => Suffix::RawPtr,
            Kw::List => Suffix::List,
            Kw::Part if ei == 0 && after_list => Suffix::Part,
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
    /// * `after_text` - Whether that base word is `စာသား`, so a bare
    ///   `တစ်စိတ်တစ်ပိုင်း` is a suffix: the view of text.
    ///
    /// # Returns
    ///
    /// The token count of the suffixes, and the suffixes, innermost first.
    fn scan_suffixes(
        &self, n: usize, after_base: bool, after_text: bool,
    ) -> (usize, Vec<Suffix>)
    {
        let (mut len, mut found, mut bare_ptr) = (0, Vec::new(), after_base);
        let mut after_list = after_text;
        while let Some((suffix, more)) = self.suffix_at(n + len, bare_ptr, after_list)
        {
            len += more;
            found.push(suffix);
            after_list = suffix == Suffix::List;
            bare_ptr = matches!(suffix, Suffix::List | Suffix::Part);
        }
        (len, found)
    }

    /// The token count of the pointer and list words that start `n` tokens
    /// ahead, right after a base word.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead the first word would start.
    ///
    /// # Returns
    ///
    /// The token count.
    pub(crate) fn suffix_len(&self, n: usize) -> usize
    {
        self.scan_suffixes(n, true, false).0
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
    /// left, or none; a `စာရင်း` with `တစ်စိတ်တစ်ပိုင်း` after it is a slice
    /// level and takes none.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic for a missing word, a bad count, a count
    /// left for a slice, a sign word on a kind that is not an integer, or a
    /// bad bit count.
    pub(crate) fn parse_kind(&mut self) -> PResult<Kind>
    {
        self.parse_kind_with(self.fn_form())
    }

    /// Parse a kind phrase whose first token can start a bare function kind
    /// of `form`; see [`Self::parse_kind`].
    ///
    /// # Arguments
    ///
    /// * `form` - The bare function kinds the phrase can be.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic.
    pub(crate) fn parse_kind_with(&mut self, form: FnForm) -> PResult<Kind>
    {
        let start = self.peek().span;
        if self.fn_head_at(0, form).is_some()
        {
            let mut kind = self.parse_fn_head(form)?;
            self.parse_levels(&mut kind, Vec::new(), true, start)?;
            return Ok(kind);
        }
        let marks = self.count_at(0, Kw::Borrowed);
        let pre = marks + self.count_prefixes_len(marks);
        let result = self.result_core_len(pre).is_some();
        let paren = self.paren_fn_len(pre).is_some();
        let prefixed = pre > 0 && self.fn_head_at(pre, form).is_some();
        let wrapped = !result && (paren || prefixed);
        ret_if!(wrapped, self.parse_wrapped_fn(marks, paren, form, start));
        ret_if!(!result, self.parse_plain_kind());
        // Borrow marks and count prefixes before a result kind belong to the
        // levels after it.
        do_for!(_ in 0..marks, self.advance());
        let counts = self.parse_counts()?;
        let mut kind = if self.eat(Kw::SuccessNews)
        {
            self.expect(Kw::Or)?;
            self.parse_err_side(start, None)?
        }
        else
        {
            let ok = tern!(
                self.paren_fn_len(0).is_some(),
                self.parse_wrapped_fn(0, true, FnForm::Full, self.peek().span)?,
                self.parse_plain_kind()?
            );
            do_if!(self.at(Kw::Result), self.advance());
            self.expect(Kw::Or)?;
            self.parse_err_side(start, Some(ok))?
        };
        kind.borrowed = marks > 0;
        kind.extra_borrows = u8::try_from(marks.saturating_sub(1)).unwrap_or(u8::MAX);
        self.parse_levels(&mut kind, counts, true, start)?;
        Ok(kind)
    }

    /// Parse a function kind after borrow marks and count prefixes, or in
    /// parentheses, then its levels.
    ///
    /// # Arguments
    ///
    /// * `marks` - The number of `ငှားထားသော` markers before it.
    /// * `paren` - Whether the kind stands in parentheses.
    /// * `form` - The bare function kinds allowed here.
    /// * `start` - Where the kind phrase starts.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic.
    fn parse_wrapped_fn(
        &mut self, marks: usize, paren: bool, form: FnForm, start: Span,
    ) -> PResult<Kind>
    {
        do_for!(_ in 0..marks, self.advance());
        let counts = self.parse_counts()?;
        do_if!(paren, self.advance());
        let mut kind = self.parse_fn_head(tern!(paren, FnForm::Full, form))?;
        do_if!(paren, _ = self.expect_punct(Punct::RParen)?);
        kind.borrowed = marks > 0;
        kind.extra_borrows = u8::try_from(marks.saturating_sub(1)).unwrap_or(u8::MAX);
        self.parse_levels(&mut kind, counts, true, start)?;
        Ok(kind)
    }

    /// Parse the error side of a result kind and build the kind:
    /// `မအောင်မြင်သည့်သတင်း`, or a kind and `အမှားအယွင်း`, which a struct name
    /// that ends in `အမှားအယွင်း` can leave out.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the result kind starts.
    /// * `ok` - The good side; `None` for `အောင်မြင်သည့်သတင်း`.
    ///
    /// # Returns
    ///
    /// The result kind, or a diagnostic.
    pub(crate) fn parse_err_side(
        &mut self, start: Span, ok: Option<Kind>,
    ) -> PResult<Kind>
    {
        let err = if self.eat(Kw::FailureNews)
        {
            None
        }
        else
        {
            // A struct name that ends in `အမှားအယွင်း` and has no `အမှားအယွင်း`
            // after its kind phrase stands alone; the levels after it belong
            // to the whole result kind.
            let full = self.plain_kind_len(0).unwrap_or(1);
            let fault_after = kw_of(&self.peek_n(full)) == Some(Kw::Fault);
            let sugar = !fault_after && self.ends_with_at(0, 1, Kw::Fault);
            let err = tern!(sugar, self.parse_core()?, self.parse_plain_kind()?);
            do_if!(!sugar, _ = self.expect(Kw::Fault)?);
            Some(err)
        };
        Ok(result_kind(ok, err, start.to(&self.prev_span())))
    }

    /// Parse a kind phrase with no result sides: `ငှားထားသော` markers, count
    /// prefixes, the base, then pointer and list levels.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic.
    fn parse_plain_kind(&mut self) -> PResult<Kind>
    {
        let start = self.peek().span;
        let marks = std::iter::from_fn(|| self.eat(Kw::Borrowed).then_some(())).count();
        let counts = self.parse_counts()?;
        let mut kind = self.parse_core()?;
        kind.borrowed = marks > 0;
        kind.extra_borrows = u8::try_from(marks.saturating_sub(1)).unwrap_or(u8::MAX);
        let bare_ptr = kind.base != BaseKind::Opaque;
        self.parse_levels(&mut kind, counts, bare_ptr, start)?;
        Ok(kind)
    }

    /// Read the pointer and list levels after a base or a result kind, and
    /// give each `စာရင်း` the innermost count prefix left.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind so far, which gets the levels.
    /// * `counts` - The count prefixes read before the kind, outermost first.
    /// * `bare_ptr` - Whether a pointer word without `၏` can follow the kind.
    /// * `start` - Where the kind phrase starts.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic for a count left for a slice or for no list.
    fn parse_levels(
        &mut self, kind: &mut Kind, mut counts: Vec<Cells>, bare_ptr: bool, start: Span,
    ) -> PResult<()>
    {
        let text = kind.base.is_text();
        let (len, suffixes) = self.scan_suffixes(0, bare_ptr, text);
        do_for!(_ in 0..len, self.advance());
        let view = suffixes.first() == Some(&Suffix::Part);
        let sliced =
            suffixes.iter().skip(usize::from(view)).any(|s| *s == Suffix::Part);
        let mut rest = suffixes.into_iter().peekable();
        do_if!(view, _ = rest.next(), kind.wraps.push(Wrap::View));
        while let Some(suffix) = rest.next()
        {
            let part =
                suffix == Suffix::List && rest.next_if_eq(&Suffix::Part).is_some();
            kind.wraps.push(match suffix
            {
                Suffix::Ptr => Wrap::Ptr,
                Suffix::RawPtr => Wrap::RawPtr,
                // A written count makes a list of that count of cells; with none,
                // `စာရင်း` is a plain list.
                Suffix::List if !part =>
                {
                    Wrap::List(counts.pop().unwrap_or(Cells::Inferred))
                }
                Suffix::List | Suffix::Part => Wrap::Slice,
            });
        }
        kind.span = start.to(&self.prev_span());
        ret_if!(sliced && !counts.is_empty(), Err(diag::slice_count(kind.span)));
        ret_if!(!counts.is_empty(), Err(self.err_expected(&quote(&kw_text(Kw::List)))));
        Ok(())
    }

    /// Parse the count prefixes `အကွက် <count> ခုရှိသော` that come next.
    ///
    /// # Returns
    ///
    /// The counts, outermost first, or a diagnostic for a count in digits
    /// that is not a whole number above zero.
    fn parse_counts(&mut self) -> PResult<Vec<Cells>>
    {
        let mut counts = Vec::new();
        while self.eat(Kw::Cell)
        {
            counts.push(self.parse_count()?);
            self.expect(Kw::Having)?;
        }
        Ok(counts)
    }

    /// Parse one count: digits alone, or an expression that the check works
    /// out, such as `N * ၂` or a size.
    ///
    /// # Returns
    ///
    /// The count, or a diagnostic for digits that are not a whole number
    /// above zero.
    fn parse_count(&mut self) -> PResult<Cells>
    {
        let digits =
            is_number(&self.peek()) && kw_of(&self.peek_n(1)) == Some(Kw::Having);
        if !digits
        {
            let outer = std::mem::replace(&mut self.in_count, true);
            let count = self.parse_expr();
            self.in_count = outer;
            return Ok(Cells::Expr(Box::new(count?)));
        }
        let tok = self.bump();
        let text = self.text_of(tok.span);
        let count = decimal_value(text).and_then(|v| u32::try_from(v).ok());
        let Some(count) = count.filter(|c| *c > 0)
        else
        {
            return Err(diag::bad_list_count(tok.span, text));
        };
        Ok(Cells::Fixed(count))
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
        // Where a kind must stand, each name is read as a struct name; the
        // check reports one that no struct declares.
        if self.peek().kind == TokenKind::Ident
        {
            let name = self.parse_ident()?;
            let kind = Kind::builder().base(BaseKind::Named).span(start);
            return Ok(kind.name(name).build());
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

/// Whether a token ends a sentence: the full stop, or the end of the file.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// `true` if it does.
fn ends_sentence(tok: &Token) -> bool
{
    matches!(tok.kind, TokenKind::Eof | TokenKind::Punct(Punct::FullStop))
}

/// A result kind with its two sides.
///
/// # Arguments
///
/// * `ok` - The good side; `None` when it holds no value.
/// * `err` - The error side; `None` when it holds no value.
/// * `span` - The whole phrase.
///
/// # Returns
///
/// The kind.
pub(crate) fn result_kind(ok: Option<Kind>, err: Option<Kind>, span: Span) -> Kind
{
    let parts = ResultParts::builder().maybe_ok(ok).maybe_err(err).build();
    Kind::builder().base(BaseKind::Result).parts(Box::new(parts)).span(span).build()
}
