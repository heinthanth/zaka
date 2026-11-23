use zakac_ast::expr::Ident;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::ResultParts;
use zakac_ast::kind::TRAIT_HEAD;
use zakac_ast::kind::Wrap;
use zakac_ast::threads::ATOMIC_WORDS;
use zakac_ast::threads::PIPE_KIND;
use zakac_diagnostics::parser as diag;
use zakac_i18n::t;
use zakac_lexer::reserved::BOOL_LOAN_NAME;
use zakac_lexer::reserved::BOOL_NAME;
use zakac_lexer::reserved::CHAR_NAME;
use zakac_lexer::reserved::FLOAT_NAME;
use zakac_lexer::reserved::INT_NAME;
use zakac_lexer::reserved::LIST_NAME;
use zakac_lexer::reserved::TEXT_NAME;
use zakac_lexer::spelling::canonical;
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
pub(crate) enum Suffix
{
    /// `လိပ်စာ`
    Ptr,
    /// `စာရင်း`
    List,
    /// The part of a written-as hint after its first slot: the index of the
    /// hint.
    Hint(usize),
    /// The room of a bounded channel, `<n> <measure word>`, then the part of
    /// the hint of the channel kind after its first slot: the index of the
    /// hint.
    Room(usize),
}

/// Bit counts of integer kinds.
const INT_BITS: &[u16] = &[8, 16, 32, 64];
/// Bit counts of floating-point kinds.
const FLOAT_BITS: &[u16] = &[32, 64];

/// The base kind a kind name the compiler gives stands for.
///
/// # Arguments
///
/// * `text` - The name.
///
/// # Returns
///
/// The base kind, or `None` for another name.
pub fn builtin_kind(text: &str) -> Option<BaseKind>
{
    match text
    {
        INT_NAME => Some(BaseKind::Int),
        FLOAT_NAME => Some(BaseKind::Float),
        CHAR_NAME => Some(BaseKind::Char),
        TEXT_NAME => Some(BaseKind::Text),
        BOOL_NAME | BOOL_LOAN_NAME => Some(BaseKind::Bool),
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
    /// follow it, as in `၈ ဘစ် ကိန်းပြည့်`. A bare `လိပ်စာ` starts one too, and so
    /// does a count prefix such as `အကွက် ၃ ခုရှိသော`.
    ///
    /// # Returns
    ///
    /// `true` if a kind starts here.
    pub(crate) fn starts_kind(&self) -> bool
    {
        let kw = kw_of(&self.peek());
        let starts =
            |kw| sign_of(kw).is_some() || matches!(kw, Kw::Address | Kw::SuccessNews);
        ret_true!(kw.is_some_and(starts) || self.is_struct_at(0));
        ret_true!(self.builtin_at(0).is_some() || self.generic_builtin_at(0));
        ret_true!(self.hint_kind_at(0).is_some());
        ret_true!(kw == Some(Kw::FnKind) || self.paren_fn_len(0).is_some());
        self.at_sized_kind(0) || self.at_count_prefix(0)
    }

    /// How far ahead the `)` that closes the `(`
    /// `at` tokens ahead stands, within the sentence.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead the `(` stands.
    ///
    /// # Returns
    ///
    /// How far ahead, or `None` when the sentence ends first.
    pub(crate) fn close_paren_at(&self, at: usize) -> Option<usize>
    {
        let mut depth = 0usize;
        for n in at..
        {
            let tok = self.peek_n(n);
            ret_if!(ends_sentence(&tok), None);
            match tok.kind
            {
                TokenKind::Punct(Punct::LParen) => depth += 1,
                TokenKind::Punct(Punct::RParen) => depth -= 1,
                _ => (),
            }
            ret_if!(depth == 0, Some(n));
        }
        None
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
        let base = self.builtin_at(n + 2);
        is_number(&self.peek_n(n))
            && kw_of(&self.peek_n(n + 1)) == Some(Kw::Bit)
            && matches!(base, Some(BaseKind::Int | BaseKind::Float))
    }

    /// The base kind of the kind name the compiler gives that stands `n`
    /// tokens ahead. A name the file declares as a struct, an enumeration,
    /// an alias or a kind parameter hides the compiler's kind of that name.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// The base kind, or `None` when no such name stands there.
    pub(crate) fn builtin_at(&self, n: usize) -> Option<BaseKind>
    {
        let tok = self.peek_n(n);
        ret_if!(tok.kind != TokenKind::Ident, None);
        let text = self.text_of(tok.span);
        ret_if!(self.structs.contains(text), None);
        builtin_kind(text)
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
        ret_if!(
            self.at_atomic_words(at),
            self.kind_len_at(at + 2, form).map(|n| n + 2)
        );
        ret_if!(let Some(len) = self.word_generic_kind_len(at), Some(len));
        let levels = |core: usize| core + self.scan_suffixes(at + core, true).0;
        ret_if!(let Some(core) = self.fn_head_at(at, form), Some(levels(core)));
        let pre = self.count_prefixes_len(at);
        // A function kind in parentheses can be the good side of a result kind.
        ret_if!(let Some(core) = self.cell_kind_len(at + pre), Some(levels(pre + core)));
        let prefixed = (pre > 0).then(|| self.fn_head_at(at + pre, form)).flatten();
        let fn_core = self.paren_fn_len(at + pre).or(prefixed);
        ret_if!(let Some(core) = fn_core, Some(levels(pre + core)));
        self.plain_kind_len(at)
    }

    /// The number of tokens in an optional or a result kind that ends in
    /// `ဒေတာဆဲလ်`, starting `at` tokens ahead: `T ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`, or
    /// `T သို့မဟုတ် အမှားအယွင်းအနေဖြင့် E ရှိနိုင်သော ဒေတာဆဲလ်`, with
    /// `အောင်မြင်သည့်သတင်း` for T and `မအောင်မြင်သည့်သတင်း` for the error side
    /// when those hold no value.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no such kind starts there.
    fn cell_kind_len(&self, at: usize) -> Option<usize>
    {
        let kw = |n: usize| kw_of(&self.peek_n(n));
        let n = match kw(at)
        {
            Some(Kw::SuccessNews) => 1,
            _ => self.paren_fn_len(at).or_else(|| self.plain_kind_len(at))?,
        };
        let cell =
            |n: usize, word: Kw| kw(n) == Some(word) && kw(n + 1) == Some(Kw::DataCell);
        ret_if!(
            kw(at) != Some(Kw::SuccessNews) && cell(at + n, Kw::Perhaps),
            Some(n + 2)
        );
        ret_if!(kw(at + n) != Some(Kw::Or), None);
        let e = at + n + 1;
        let err = match kw(e)
        {
            Some(Kw::FailureNews) => 1,
            Some(Kw::AsFault) => 1 + self.plain_kind_len(e + 1)?,
            _ => return None,
        };
        cell(e + err, Kw::CanHold).then_some(n + 1 + err + 2)
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
        if let Some(core) = self.trait_value_len(at)
        {
            return Some(core + self.scan_suffixes(at + core, true).0);
        }
        if let Some((_, core)) = self.hint_kind_at(at)
        {
            return Some(core + self.scan_suffixes(at + core, true).0);
        }
        let pre = self.count_prefixes_len(at);
        let signed = kw_of(&self.peek_n(at + pre)).and_then(sign_of).is_some();
        let n = pre + usize::from(signed);
        let n = n + tern!(self.at_sized_kind(at + n), 2, 0);
        let word = kw_of(&self.peek_n(at + n));
        let named =
            n == pre && (self.is_struct_at(at + n) || self.generic_builtin_at(at + n));
        let is_base = self.builtin_at(at + n).is_some() || named;
        let bare = n == pre && word == Some(Kw::Address);
        ret_if!(!is_base && !bare, None);
        let angle = tern!(named, self.angle_len(at + n + 1).unwrap_or(0), 0);
        let (len, _) = self.scan_suffixes(at + n + 1 + angle, is_base);
        Some(n + 1 + angle + len)
    }

    /// The token count within which a trait value kind reaches its head noun,
    /// `အရာ`: the words of a hint of a trait and the kinds in its slots.
    const TRAIT_REACH: usize = 16;

    /// The number of tokens in a trait value kind with no levels, `<trait>
    /// အရာ`, that starts `at` tokens ahead: the trait as a hint, or as its
    /// name with its kinds in signs, then `အရာ`.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no trait value kind starts there.
    pub(crate) fn trait_value_len(&self, at: usize) -> Option<usize>
    {
        // A trait value ends in its head noun soon after it starts; most kinds
        // have none near, and skip the scan of the hints.
        ret_if!(!(1..=Self::TRAIT_REACH).any(|n| self.trait_head_at(at + n)), None);
        let mut len = self.trait_ref_len(at)?;
        while kw_of(&self.peek_n(at + len)) == Some(Kw::Hnint)
        {
            len += 1 + self.trait_ref_len(at + len + 1)?;
        }
        self.trait_head_at(at + len).then_some(len + 1)
    }

    /// Whether the head noun of a trait value, `အရာ`, stands `at` tokens
    /// ahead.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn trait_head_at(&self, at: usize) -> bool
    {
        let tok = self.peek_n(at);
        tok.kind == TokenKind::Ident && self.text_of(tok.span) == TRAIT_HEAD
    }

    /// The number of tokens of one trait of a trait value kind, `at` tokens
    /// ahead: a hint of a trait, or a name with its kinds in signs.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no trait stands there.
    fn trait_ref_len(&self, at: usize) -> Option<usize>
    {
        // The kind slot a trait's hint starts with stands at this same place,
        // where it is no trait value of its own.
        let here = self.cur.pos() + at;
        ret_if!(self.trait_value_at.get() == Some(here), None);
        let outer = self.trait_value_at.replace(Some(here));
        let hinted = self.hint_trait_at(at).map(|(_, len)| len);
        self.trait_value_at.set(outer);
        ret_if!(let Some(len) = hinted, Some(len));
        let named = self.peek_n(at).kind == TokenKind::Ident && !self.trait_head_at(at);
        ret_if!(!named || self.builtin_at(at).is_some(), None);
        Some(1 + self.angle_len(at + 1).unwrap_or(0))
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

    /// The suffix that starts `n` tokens ahead, and its token count:
    /// `၏ လိပ်စာ`, `၏ စာရင်း`, a bare `စာရင်း`, or a bare `လိပ်စာ` when the
    /// level before or the base allows one.
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
        let list = self.is_builtin_word_at(n + ei, LIST_NAME);
        ret_if!(list, Some((Suffix::List, ei + 1)));
        let hint = tern!(ei == 0, self.hint_level_at(n), None);
        ret_if!(let Some((idx, len)) = hint, Some((Suffix::Hint(idx), len)));
        let room = tern!(ei == 0, self.room_hint_at(n), None);
        ret_if!(let Some((idx, len)) = room, Some((Suffix::Room(idx), len + 2)));
        let ptr = kw_of(&self.peek_n(n + ei)) == Some(Kw::Address);
        (ptr && (ei == 1 || bare_ptr)).then_some((Suffix::Ptr, ei + 1))
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
    pub(crate) fn scan_suffixes(
        &self, n: usize, after_base: bool,
    ) -> (usize, Vec<Suffix>)
    {
        let (mut len, mut found, mut bare_ptr) = (0, Vec::new(), after_base);
        while let Some((suffix, more)) = self.suffix_at(n + len, bare_ptr)
        {
            len += more;
            found.push(suffix);
            bare_ptr = suffix != Suffix::Ptr;
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
    pub(crate) fn suffix_len(&self, n: usize) -> usize { self.scan_suffixes(n, true).0 }

    /// Whether `<kind> တစ်<classifier> ၏ အရွယ်အစား` starts here, its measure
    /// word written or not. Nothing is consumed.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_size_of(&self) -> bool
    {
        self.kind_len().is_some_and(|len| {
            let len = len + usize::from(self.is_count_word_at(len));
            self.peek_n(len).kind == TokenKind::Punct(Punct::Ei)
                && kw_of(&self.peek_n(len + 1)) == Some(Kw::SizeOf)
        })
    }

    /// Parse a kind phrase: count prefixes such as
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
        // `အက်တမ်မစ် ဖြစ်သော <kind>`: an atomic number of an integer kind.
        if self.at_atomic_words(0)
        {
            let first = self.bump();
            self.bump();
            let inner = self.parse_kind_with(form)?;
            let name = Ident::builder().name(ATOMIC_WORDS[0]).span(first.span).build();
            let span = start.to(&inner.span);
            #[rustfmt::skip]
            let kind = Kind::builder().base(BaseKind::Named).name(name).args(vec![inner])
                .span(span).build();
            return Ok(kind);
        }
        // `A နှင့် B တို့ကို အစားထိုးအသုံးပြုသည့် <name>`: the kind arguments of a
        // generic kind come first in the word form.
        if self.word_generic_kind_len(0).is_some()
        {
            let args = self.parse_word_kind_args(Kw::SubstitutedRel)?;
            let mut kind = self.parse_kind_with(form)?;
            kind.args = args;
            kind.span = start.to(&kind.span);
            return Ok(self.builtin_generic(kind));
        }
        if self.fn_head_at(0, form).is_some()
        {
            let mut kind = self.parse_fn_head(form)?;
            self.parse_levels(&mut kind, Vec::new(), true, start)?;
            return Ok(kind);
        }
        let pre = self.count_prefixes_len(0);
        let result = self.cell_kind_len(pre).is_some();
        let paren = self.paren_fn_len(pre).is_some();
        let prefixed = pre > 0 && self.fn_head_at(pre, form).is_some();
        let wrapped = !result && (paren || prefixed);
        ret_if!(wrapped, self.parse_wrapped_fn(paren, form, start));
        ret_if!(!result, self.parse_plain_kind());
        // Count prefixes before an optional or a result kind go to its list
        // levels innermost first: those of its data, then those after it.
        let mut counts = self.parse_counts()?;
        let mut kind = self.parse_cell_kind(start, &mut counts)?;
        self.parse_levels(&mut kind, counts, true, start)?;
        Ok(kind)
    }

    /// Parse a function kind after count prefixes, or in parentheses, then
    /// its levels.
    ///
    /// # Arguments
    ///
    /// * `paren` - Whether the kind stands in parentheses.
    /// * `form` - The bare function kinds allowed here.
    /// * `start` - Where the kind phrase starts.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic.
    fn parse_wrapped_fn(
        &mut self, paren: bool, form: FnForm, start: Span,
    ) -> PResult<Kind>
    {
        let counts = self.parse_counts()?;
        do_if!(paren, self.advance());
        let mut kind = self.parse_fn_head(tern!(paren, FnForm::Full, form))?;
        do_if!(paren, _ = self.expect_punct(Punct::RParen)?);
        self.parse_levels(&mut kind, counts, true, start)?;
        Ok(kind)
    }

    /// Parse an optional or a result kind that ends in `ဒေတာဆဲလ်`, after its
    /// count prefixes; [`Parser::cell_kind_len`] found one here.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the kind phrase starts.
    /// * `counts` - The count prefixes read before the kind, outermost first;
    ///   the list levels of its data take the innermost ones.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic.
    fn parse_cell_kind(&mut self, start: Span, counts: &mut Vec<Cells>)
    -> PResult<Kind>
    {
        let ok = match self.eat(Kw::SuccessNews)
        {
            true => None,
            false if self.paren_fn_len(0).is_some() =>
            {
                Some(self.parse_wrapped_fn(true, FnForm::Full, self.peek().span)?)
            }
            false => Some(self.parse_plain_kind_with(counts)?),
        };
        if self.eat(Kw::Perhaps)
        {
            self.expect(Kw::DataCell)?;
            let parts = ResultParts::builder().maybe_ok(ok).optional(true).build();
            let span = start.to(&self.prev_span());
            #[rustfmt::skip]
            return Ok(Kind::builder().base(BaseKind::Result).parts(Box::new(parts)).span(span).build());
        }
        self.expect(Kw::Or)?;
        let err = match self.eat(Kw::FailureNews)
        {
            true => None,
            false =>
            {
                self.expect(Kw::AsFault)?;
                Some(self.parse_plain_kind()?)
            }
        };
        self.expect(Kw::CanHold)?;
        self.expect(Kw::DataCell)?;
        Ok(result_kind(ok, err, start.to(&self.prev_span())))
    }

    /// Parse a kind phrase with no result sides: count prefixes, the base,
    /// then pointer and list levels.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic.
    fn parse_plain_kind(&mut self) -> PResult<Kind>
    {
        let mut counts = Vec::new();
        let kind = self.parse_plain_kind_with(&mut counts)?;
        ret_if!(!counts.is_empty(), Err(self.err_expected(&quote(LIST_NAME))));
        Ok(kind)
    }

    /// Parse a kind phrase with no result sides, as [`Self::parse_plain_kind`]
    /// does, whose list levels take count prefixes read before it too.
    ///
    /// # Arguments
    ///
    /// * `counts` - The count prefixes read before the phrase, outermost first;
    ///   its own come after them, and each `စာရင်း` takes the innermost one
    ///   left. Those its levels leave stay for the kind around it.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic.
    fn parse_plain_kind_with(&mut self, counts: &mut Vec<Cells>) -> PResult<Kind>
    {
        let start = self.peek().span;
        counts.extend(self.parse_counts()?);
        let mut kind = self.parse_core()?;
        let bare_ptr = kind.base != BaseKind::Opaque;
        self.take_levels(&mut kind, counts, bare_ptr, start)?;
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
    /// `Ok`, or a diagnostic for a count left for no list.
    fn parse_levels(
        &mut self, kind: &mut Kind, mut counts: Vec<Cells>, bare_ptr: bool, start: Span,
    ) -> PResult<()>
    {
        self.take_levels(kind, &mut counts, bare_ptr, start)?;
        ret_if!(!counts.is_empty(), Err(self.err_expected(&quote(LIST_NAME))));
        Ok(())
    }

    /// Read the pointer, list and hint levels after a base or a result kind,
    /// and give each list level the innermost count prefix left, leaving the
    /// rest. A hint level makes the kind so far the first slot of its hint.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind so far, which gets the levels.
    /// * `counts` - The count prefixes left, outermost first.
    /// * `bare_ptr` - Whether a pointer word without `၏` can follow the kind.
    /// * `start` - Where the kind phrase starts.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic from a slot of a hint.
    fn take_levels(
        &mut self, kind: &mut Kind, counts: &mut Vec<Cells>, mut bare_ptr: bool,
        start: Span,
    ) -> PResult<()>
    {
        while let Some((suffix, len)) = self.suffix_at(0, bare_ptr)
        {
            bare_ptr = suffix != Suffix::Ptr;
            if let Suffix::Room(idx) = suffix
            {
                let room = self.parse_room()?;
                *kind =
                    self.parse_hint(idx, Some(kind.clone()), start, self.fn_form())?;
                kind.room = Some(room);
                continue;
            }
            let Suffix::Hint(idx) = suffix
            else
            {
                do_for!(_ in 0..len, self.advance());
                kind.wraps.push(level_wrap(suffix, counts));
                continue;
            };
            *kind = self.parse_hint(idx, Some(kind.clone()), start, self.fn_form())?;
        }
        kind.span = start.to(&self.prev_span());
        Ok(())
    }

    /// Whether the words of an atomic number kind stand `n` tokens ahead:
    /// `အက်တမ်မစ် ဖြစ်သော`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if both stand there.
    fn at_atomic_words(&self, n: usize) -> bool
    {
        self.is_word_at(n, ATOMIC_WORDS[0]) && self.is_word_at(n + 1, ATOMIC_WORDS[1])
    }

    /// The hint of the channel kind after the room of a bounded channel,
    /// `<n> <measure word>`, written `n` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead the room starts.
    ///
    /// # Returns
    ///
    /// The index of the hint and the token count after the room.
    fn room_hint_at(&self, n: usize) -> Option<(usize, usize)>
    {
        let room =
            is_number(&self.peek_n(n)) && self.peek_n(n + 1).kind == TokenKind::Ident;
        ret_if!(!room, None);
        let (idx, len) = self.hint_level_at(n + 2)?;
        let pipe = canonical(&self.hints[idx].name) == canonical(PIPE_KIND);
        pipe.then_some((idx, len))
    }

    /// Parse the room of a bounded channel, `<n> <measure word>`, in digits.
    ///
    /// # Returns
    ///
    /// The room, or ZK0227 for a room that is not a whole number above zero.
    fn parse_room(&mut self) -> PResult<u32>
    {
        let tok = self.bump();
        self.bump();
        let text = self.text_of(tok.span);
        let room = decimal_value(text).and_then(|v| u32::try_from(v).ok());
        room.filter(|r| *r > 0).ok_or_else(|| diag::bad_list_count(tok.span, text))
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

    /// Parse the core of a kind: a bare `လိပ်စာ`, or a base
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
            _ => None,
        };
        if let Some(wrap) = opaque
        {
            self.advance();
            let kind = Kind::builder().base(BaseKind::Opaque).wraps(vec![wrap]);
            return Ok(kind.span(start).build());
        }
        // Where a kind must stand, each name but a kind name the compiler
        // gives is read as a struct name; the check reports one that no
        // struct declares.
        if let Some((idx, _)) = self.hint_kind_at(0)
        {
            return self.parse_hint(idx, None, start, self.fn_form());
        }
        if self.trait_value_len(0).is_some()
        {
            let mut kind = self.parse_value_trait()?;
            while self.eat(Kw::Hnint)
            {
                let next = self.parse_value_trait()?;
                kind.also.push(next);
            }
            self.advance();
            kind.base = BaseKind::TraitValue;
            kind.span = start.to(&self.prev_span());
            return Ok(kind);
        }
        if self.peek().kind == TokenKind::Ident && self.builtin_at(0).is_none()
        {
            let name = self.parse_ident()?;
            // `<name><A၊ B>`: the kind arguments of a generic kind.
            let generic = self.angle_len(0).is_some();
            let args = tern!(generic, self.parse_sign_kind_args()?, Vec::new());
            let span = start.to(&self.prev_span());
            let kind = Kind::builder().base(BaseKind::Named).span(span).args(args);
            return Ok(self.builtin_generic(kind.name(name).build()));
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

    /// Parse one trait of a trait value kind, with the kind slot its hint
    /// starts with read as a plain kind.
    ///
    /// # Returns
    ///
    /// The trait as a named kind with its kinds, or a diagnostic.
    fn parse_value_trait(&mut self) -> PResult<Kind>
    {
        let outer = self.trait_value_at.replace(Some(self.cur.pos()));
        let trait_ref = self.parse_trait_ref();
        self.trait_value_at.set(outer);
        trait_ref
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
        let base = self.builtin_at(0);
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
fn sized_kind_words() -> String { one_of(&[quote(INT_NAME), quote(FLOAT_NAME)]) }

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

/// The wrap of a pointer or list level.
///
/// # Arguments
///
/// * `suffix` - The level: [`Suffix::Ptr`] or [`Suffix::List`].
/// * `counts` - The count prefixes left; a list level takes the innermost.
///
/// # Returns
///
/// The wrap. A written count makes a list of that count of cells; with none,
/// the list is a plain list.
fn level_wrap(suffix: Suffix, counts: &mut Vec<Cells>) -> Wrap
{
    match suffix
    {
        Suffix::List => Wrap::List(counts.pop().unwrap_or(Cells::Inferred)),
        _ => Wrap::Ptr,
    }
}
