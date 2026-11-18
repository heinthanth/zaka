//! The arguments-first call, `<args> (တို့)?ကို သုံး၍ ...`, and the short call
//! with no arguments, `<fn> မည်` and `<fn> သည့် ရလဒ်`.
//!
//! Each form ends as a statement or as a value. The long form names the
//! function as `<fn> ဟူသော လုပ်ဆောင်ချက်ကို`, then ends with `လုပ်ဆောင်မည်` (a statement)
//! or `လုပ်ဆောင်သော ရလဒ်(ကောင်း)?` (a value). The short form names the function
//! alone, then ends with `မည်` (a statement) or `သည့် ရလဒ်(ကောင်း)?` (a value).
//! Each builds the call node that the other call forms build.
//!
//! The preposition call, `<arg> <preposition> (<arg> <preposition>)* <fn>
//! <end>`, gives each argument with the preposition its parameter declares,
//! `<kind> <name>(<preposition>)`. The parser knows which functions declare
//! prepositions (`Parser::functions`), and those names alone end a preposition
//! call, and a value of another call keeps its meaning: `က ကို ခ သည့် ရလဒ် ဟု ထားမည်။`
//! still sets `က`.

use std::collections::HashMap;

use bon::Builder;
use zakac_ast::expr::CallExpr;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::Phrases;
use zakac_ast::expr::UsingParticle;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_diagnostics::resolve as rd;
use zakac_i18n::t;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex_at;
use zakac_lexer::spoken::written_form;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;
use zakac_span::pos::Pos;
use zakac_span::span::Span;

use crate::clause::Mode;
use crate::notation::Args;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::kw_of;
use crate::parser::unquote_raw;

/// What a call holds once its function is read.
#[derive(Builder)]
pub(crate) struct CallParts
{
    /// Where the call starts.
    pub(crate) start: Span,
    /// The function name.
    pub(crate) callee: Ident,
    /// The place that holds the function, in a long form that writes one.
    pub(crate) place: Option<Box<Expr>>,
    /// The arguments, with the edit flag of each.
    pub(crate) args: Args,
    /// The particle that closes the arguments of a call written arguments
    /// first.
    pub(crate) using: Option<UsingParticle>,
    /// The word after each argument of a preposition call, `None` where the
    /// argument has none.
    #[builder(default)]
    pub(crate) prepositions: Vec<Option<Ident>>,
    /// The starting and the ending phrase of a call written as a sentence.
    #[builder(default)]
    pub(crate) phrases: Phrases,
}

/// The words a call written as a sentence takes for one function: its
/// starting phrase, the preposition of each parameter and its ending phrase,
/// each with its words joined with one space, and the number of its
/// parameters.
#[derive(Debug, Clone, PartialEq, Eq, Default, Builder)]
pub struct CallWords
{
    /// The words before the first argument.
    pub start: Option<String>,
    /// The word after the argument of each parameter, `None` for a parameter
    /// with none; blank for a function with no parameters.
    #[builder(default)]
    pub prepositions: Vec<Option<String>>,
    /// The words right before the function name.
    pub end: Option<String>,
    /// The number of parameters.
    #[builder(default)]
    pub params: usize,
}

impl CallWords
{
    /// The words of the head of a function declaration.
    ///
    /// # Arguments
    ///
    /// * `decl` - The declaration.
    ///
    /// # Returns
    ///
    /// Its words.
    pub fn of_decl(decl: &FnDecl) -> CallWords
    {
        let word = |i: &Option<Ident>| i.as_ref().map(|i| i.name.to_string());
        let name = word;
        #[rustfmt::skip]
        return CallWords::builder().maybe_start(name(&decl.phrases.start))
            .prepositions(decl.params.iter().map(|p| word(&p.preposition)).collect())
            .maybe_end(name(&decl.phrases.end)).params(decl.params.len()).build();
    }

    /// Whether a call of the function can be written as a sentence: it has a
    /// phrase or prepositions.
    ///
    /// # Returns
    ///
    /// `true` when it has.
    pub fn is_sentence(&self) -> bool
    {
        let given = self.prepositions.iter().any(Option::is_some);
        self.start.is_some() || self.end.is_some() || given
    }

    /// Each phrase and preposition, the starting phrase first.
    ///
    /// # Returns
    ///
    /// The texts.
    pub fn texts(&self) -> impl Iterator<Item = &String>
    {
        let start = self.start.iter();
        start.chain(self.prepositions.iter().flatten()).chain(self.end.iter())
    }
}

/// The functions the parser knows, under their names: the words of each
/// function of the name. A function with no phrase and no preposition has
/// blank words.
pub type Functions = HashMap<String, Vec<CallWords>>;

/// Whether a token can be a word of a preposition: a name, `၏`, or a reserved
/// word that is no call end, no `ဟူသော`, and no word that starts an operand. A
/// closer (`သည်`, `ပါ`, `ဖြစ်ကာ`, `ပြီးလျှင်`, `လက်ခံပြီး`) can stand in a preposition
/// before another word of it, as `ပါ` in `ပါရှိသော`, and [`preposition_words`]
/// checks that.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// `true` for such a token.
fn is_preposition_word(tok: &Token) -> bool
{
    match tok.kind
    {
        TokenKind::Ident | TokenKind::Punct(Punct::Ei) => true,
        TokenKind::Kw(kw) => is_closer(kw) || !refused_in_preposition(kw),
        _ => false,
    }
}

/// Whether a reserved word closes a sentence or a part of it: `သည်`, `ပါ`,
/// `ဖြစ်ကာ`, `ပြီးလျှင်` or `လက်ခံပြီး`.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// `true` for a closer.
fn is_closer(kw: Kw) -> bool
{
    matches!(kw, Kw::Thi | Kw::Pa | Kw::Being | Kw::WhenDone | Kw::Accepting)
}

/// Whether a reserved word cannot stand in a preposition: a call end, a closer,
/// `ဟူသော`, or a word that starts an operand.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// `true` for such a word.
fn refused_in_preposition(kw: Kw) -> bool
{
    let closer = is_closer(kw);
    let end = matches!(
        kw,
        Kw::Will | Kw::WillDo | Kw::CallResult | Kw::CallGoodResult | Kw::Called
    );
    closer || end || kw == Kw::FnValue || in_notation(kw)
}

/// Whether a token is a particle a preposition can hold: not a closer, or `တွင်`
/// and `ဆီသို့`.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// `true` for such a token.
fn is_plain_particle(tok: &Token) -> bool
{
    let TokenKind::Kw(kw) = tok.kind
    else
    {
        return false;
    };
    (kw.is_particle() && !refused_in_preposition(kw))
        || matches!(kw, Kw::At | Kw::Toward)
}

/// The words of a preposition string as the lexer reads them, so a glued
/// preposition and a spaced one that lex the same are one preposition.
///
/// # Arguments
///
/// * `text` - The text inside the string.
///
/// # Returns
///
/// The words, or `None` when the text is no preposition: blank, lexed with an
/// error, with a word a preposition cannot hold, with `၏` after its first word,
/// or with a closer as its last word.
pub(crate) fn preposition_words(text: &str) -> Option<Vec<String>>
{
    let lexed = lex_at(text, Pos::default(), &LexerOptions::default());
    ret_if!(!lexed.diagnostics.is_empty(), None);
    let toks = lexed.tokens.iter().filter(|t| t.kind != TokenKind::Eof);
    let toks: Vec<&Token> = toks.collect();
    ret_if!(toks.is_empty() || !toks.iter().all(|t| is_preposition_word(t)), None);
    let late_ei = toks.iter().skip(1).any(|t| t.kind == TokenKind::Punct(Punct::Ei));
    ret_if!(late_ei, None);
    let closes = |t: &&&Token| matches!(t.kind, TokenKind::Kw(kw) if is_closer(kw));
    ret_if!(toks.last().is_some_and(|t| closes(&t)), None);
    let word = |t: &&Token| {
        let (lo, hi) = (t.span.lo - Pos::default(), t.span.hi - Pos::default());
        written_form(&text[lo as usize..hi as usize]).to_string()
    };
    Some(toks.iter().map(word).collect())
}

/// The preposition text that starts with `၏`, which the last parameter alone
/// can take.
pub(crate) const EI: &str = "၏";

/// Whether a notation argument of a preposition call can hold a reserved word:
/// a literal word, `အနုတ်`, or the end of a short call with no arguments.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// `true` for such a word.
fn in_notation(kw: Kw) -> bool
{
    matches!(
        kw,
        Kw::True
            | Kw::False
            | Kw::Null
            | Kw::BlankList
            | Kw::BlankTable
            | Kw::Negate
            | Kw::UsedResult
            | Kw::UsedGoodResult
    )
}

/// Whether a notation argument can hold a reserved word as a word of a size
/// or of a `၏` reading: the count word and `ဘိုက်စာ` of a size operand,
/// `၏ အရွယ်အစား`, the words of a kind, as in `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ၃ ခုစာ`,
/// and `၏ တန်ဖိုး`, `၏ လိပ်စာ` and `၏ အကွက် <start> မှစ၍ <length> ကွက်`. Such a
/// word can stand in a preposition too, unlike the words of [`in_notation`].
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// `true` for such a word.
fn in_size(kw: Kw) -> bool
{
    matches!(
        kw,
        Kw::Items
            | Kw::Octets
            | Kw::SizeOf
            | Kw::Address
            | Kw::Signed
            | Kw::Unsigned
            | Kw::Bit
            | Kw::Value
            | Kw::Cell
            | Kw::StartingAt
            | Kw::CellWord
    )
}

/// What the start of a clause gave: a value, or a whole call statement.
// The value, the common case, stays unboxed: a box would cost an allocation
// for each clause.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub(crate) enum Head
{
    /// A value; the clause goes on from it.
    Expr(Expr),
    /// A call statement, its full stop included.
    Stmt(Box<Stmt>),
}

impl Parser<'_>
{
    /// Whether the rest of an arguments-first call comes after its first
    /// argument: `ကို` or `တို့ကို`, then `သုံး၍`; or `၊` or `နှင့်` with those
    /// ahead, outside parentheses, before a word that no argument holds.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_using_rest(&self) -> bool
    {
        ret_true!(self.at_using_close(0));
        self.at_list_join() && self.word_list_ahead(1, Self::at_using_close)
    }

    /// Whether `ကို` or `တို့ကို` starts `n` tokens ahead, with `သုံး၍` after
    /// it.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_using_close(&self, n: usize) -> bool
    {
        let using = self.peek_n(n + 1).kind == TokenKind::Kw(Kw::Using);
        using && matches!(self.peek_n(n).kind, TokenKind::Kw(Kw::Ko | Kw::ListKo))
    }

    /// Whether the end of a short call comes next: `မည်`, `သည့် ရလဒ်` or
    /// `သည့် ရလဒ်ကောင်း`.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_short_end(&self) -> bool
    {
        matches!(
            kw_of(&self.peek()),
            Some(Kw::Will | Kw::UsedResult | Kw::UsedGoodResult)
        )
    }

    /// How far ahead the end of a preposition call is, when one starts `from`
    /// tokens ahead (the preposition after the first argument): the end of a
    /// call of a function that declares prepositions. A value end with a
    /// preposition after it can be an argument of a later call, so the scan
    /// goes on past it, and keeps it when no later end comes.
    ///
    /// # Arguments
    ///
    /// * `from` - Where the first preposition is.
    ///
    /// # Returns
    ///
    /// The distance to the first token of the end, or `None` with no
    /// preposition call here.
    pub(crate) fn preposition_end(&self, from: usize) -> Option<usize>
    {
        ret_if!(!self.starts_preposition(&self.peek_n(from)), None);
        self.sentence_end(from)
    }

    /// How far ahead the end of a call written as a sentence is, scanning
    /// from `from` tokens ahead: the end of a call of a function that declares
    /// phrases or prepositions. An argument with no preposition can touch the
    /// next one, so the scan starts at each token. A value end with a
    /// preposition or a `၊` after it can be an argument of a later call, so the
    /// scan goes on past it, and keeps it when no later end comes.
    ///
    /// # Arguments
    ///
    /// * `from` - Where the scan starts.
    ///
    /// # Returns
    ///
    /// The distance to the first token of the end, or `None` with no such call
    /// here.
    pub(crate) fn sentence_end(&self, from: usize) -> Option<usize>
    {
        let (mut n, mut depth, mut found) = (from, 0usize, None);
        loop
        {
            let end = tern!(depth == 0, self.call_end_at(n), None);
            if let Some((len, value)) = end
            {
                let next = self.peek_n(n + len);
                let goes_on = self.starts_preposition(&next)
                    || next.kind == TokenKind::Punct(Punct::ListSep);
                ret_if!(!value || !goes_on, Some(n));
                found = Some(n);
                n += len;
                continue;
            }
            let tok = self.peek_n(n);
            // `<number> ဟူသော <kind> တစ်<word>` is one literal with its kind.
            let with_kind = tok.kind == TokenKind::Kw(Kw::Called)
                && n > 0
                && matches!(self.peek_n(n - 1).kind, TokenKind::Number(_));
            ret_if!(!with_kind && self.stops_preposition_scan(&tok, depth), found);
            match tok.kind
            {
                TokenKind::Punct(Punct::LParen | Punct::LBracket) => depth += 1,
                TokenKind::Punct(Punct::RParen | Punct::RBracket) => depth -= 1,
                _ => (),
            }
            n += 1;
        }
    }

    /// The end of a call of a function that declares phrases or prepositions,
    /// `n` tokens ahead: `<fn> မည်`, `<fn> သည့် ရလဒ်(ကောင်း)`, or
    /// `<fn> ဟူသော လုပ်ဆောင်ချက်ကို` with `လုပ်ဆောင်မည်` or `လုပ်ဆောင်သော ရလဒ်(ကောင်း)`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// The number of tokens of the end and whether it gives a value, or
    /// `None`.
    pub(crate) fn call_end_at(&self, n: usize) -> Option<(usize, bool)>
    {
        let long = kw_of(&self.peek_n(n + 1)) == Some(Kw::FnValue)
            && kw_of(&self.peek_n(n + 2)) == Some(Kw::Ko);
        ret_if!(!self.is_sentence_fn(n), None);
        let word = kw_of(&self.peek_n(tern!(long, n + 3, n + 1)))?;
        let len = tern!(long, 4, 2);
        match word
        {
            Kw::WillDo if long => Some((len, false)),
            Kw::CallResult | Kw::CallGoodResult if long => Some((len, true)),
            Kw::Will if !long => Some((len, false)),
            Kw::UsedResult | Kw::UsedGoodResult if !long => Some((len, true)),
            _ => None,
        }
    }

    /// Whether the token `n` ahead names a function that declares phrases or
    /// prepositions:
    /// a name, a name in braces, or a reserved word that such a function
    /// was declared with in braces (`{စီ}`).
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` for such a name.
    fn is_sentence_fn(&self, n: usize) -> bool
    {
        let tok = self.peek_n(n);
        let named = matches!(
            tok.kind,
            TokenKind::Ident | TokenKind::RawIdent | TokenKind::Kw(_)
        );
        named && self.fn_words(&self.callee_text(&tok)).next().is_some()
    }

    /// The text of a token as the name of a function: a name in braces
    /// without its braces.
    ///
    /// # Arguments
    ///
    /// * `tok` - The token.
    ///
    /// # Returns
    ///
    /// The text.
    pub(crate) fn callee_text(&self, tok: &Token) -> String
    {
        let raw = tok.kind == TokenKind::RawIdent;
        unquote_raw(raw, self.text_of(tok.span)).to_string()
    }

    /// The words of the functions of a name that declare phrases or
    /// prepositions.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The words of each such function.
    pub(crate) fn fn_words<'a>(
        &'a self, name: &str,
    ) -> impl Iterator<Item = &'a CallWords>
    {
        let lists = self.functions.get(name).into_iter().flatten();
        lists.filter(|words| words.is_sentence())
    }

    /// Whether a token can start a preposition of a visible function, so a
    /// preposition call can go on from it.
    ///
    /// # Arguments
    ///
    /// * `tok` - The token.
    ///
    /// # Returns
    ///
    /// `true` for such a token.
    pub(crate) fn starts_preposition(&self, tok: &Token) -> bool
    {
        is_preposition_word(tok)
            && self.first_words.contains(written_form(self.text_of(tok.span)))
    }

    /// Whether the scan for the end of a preposition call stops at a token: the
    /// end of the sentence, a `)` or `]` that closes no group of the call, or,
    /// outside parentheses, a reserved word that is neither a word of a
    /// visible preposition, a particle, nor a word a notation argument can
    /// hold. A `၊` can part two arguments, so the scan goes past it.
    ///
    /// # Arguments
    ///
    /// * `tok` - The token.
    /// * `depth` - The number of groups of the call left open.
    ///
    /// # Returns
    ///
    /// `true` where the scan stops.
    fn stops_preposition_scan(&self, tok: &Token, depth: usize) -> bool
    {
        let deep = depth > 0;
        // A particle that is no preposition goes on too, so a call with a wrong
        // preposition still reads, and the check names the preposition
        // it expects.
        let in_preposition = || {
            self.preposition_words.contains(written_form(self.text_of(tok.span)))
                || is_plain_particle(tok)
        };
        match tok.kind
        {
            TokenKind::Punct(Punct::FullStop) | TokenKind::Eof => true,
            TokenKind::Punct(Punct::RParen | Punct::RBracket) => !deep,
            TokenKind::Kw(kw) =>
            {
                !deep && !in_preposition() && !in_notation(kw) && !in_size(kw)
            }
            _ => false,
        }
    }

    /// Read a preposition call after its first argument, and keep it when the
    /// number of its prepositions fits a function of its name; otherwise go
    /// back, so the sentence reads as it would with no preposition call,
    /// such as `က ကို ခ ၌ရှိသော ဖတ် သည့် ရလဒ် ဟု ထားမည်။` with `ဖတ်` taking one
    /// preposition.
    ///
    /// # Arguments
    ///
    /// * `first` - The first operand.
    /// * `end` - How far ahead the end is.
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, or the first operand as it was.
    pub(crate) fn attempt_preposition_call(
        &mut self, first: Expr, end: usize, mode: Mode,
    ) -> PResult<Head>
    {
        let (cur, diags, next_id) = (self.cur, self.diags.len(), self.next_id);
        let read = self.parse_preposition_call(first.clone(), end, mode, None);
        ret_if!(read.as_ref().is_ok_and(|head| self.fits_words(head)), read);
        self.cur = cur;
        self.diags.truncate(diags);
        self.next_id = next_id;
        Ok(Head::Expr(first))
    }

    /// Read a call written as a sentence that starts at the start of a
    /// clause with a phrase: a starting phrase and the rest of the call, or
    /// the phrases of a function with no parameters and the function, as
    /// `ဖန်သားပြင်ကို အားလုံး ရှင်း မည်`. When no such call reads and fits a
    /// function of its name, nothing is read.
    ///
    /// # Arguments
    ///
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, or `None` with no such call here.
    pub(crate) fn attempt_phrase_call(&mut self, mode: Mode) -> Option<Head>
    {
        let tok = self.peek();
        let word = written_form(self.text_of(tok.span));
        let opens = self.start_words.contains(word) || self.first_words.contains(word);
        ret_if!(!is_preposition_word(&tok) || !opens, None);
        let (cur, diags, next_id) = (self.cur, self.diags.len(), self.next_id);
        let bare = self.bare_phrase_call(mode);
        let read = tern!(bare.is_some(), bare, self.start_phrase_call(mode));
        let read = read.and_then(Result::ok).filter(|head| self.fits_words(head));
        ret_if!(read.is_some(), read);
        self.cur = cur;
        self.diags.truncate(diags);
        self.next_id = next_id;
        None
    }

    /// Read the call of a function with no parameters that the phrases ahead
    /// spell, the function name and a call end after them.
    ///
    /// # Arguments
    ///
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, or `None` when no such function fits the words ahead.
    fn bare_phrase_call(&mut self, mode: Mode) -> Option<PResult<Head>>
    {
        let all =
            self.functions.iter().flat_map(|(f, l)| l.iter().map(move |w| (f, w)));
        let bare = all.filter(|(_, w)| w.params == 0 && w.is_sentence());
        let spelled = bare.filter_map(|(f, w)| {
            let start = w.start.as_ref().map_or(Some(0), |m| self.spells_at(m, 0))?;
            let end = w.end.as_ref().map_or(Some(0), |m| self.spells_at(m, start))?;
            let named = self.callee_text(&self.peek_n(start + end)) == *f;
            let ends = named && self.call_end_at(start + end).is_some();
            ends.then_some((start + end, start, end))
        });
        let (_, start, end) = spelled.max()?;
        let start = (start > 0).then(|| self.take_words(start));
        let end = (end > 0).then(|| self.take_words(end));
        let phrases = Phrases::builder().maybe_start(start).maybe_end(end).build();
        let span = phrases.start.as_ref().or(phrases.end.as_ref()).map(|i| i.span)?;
        Some(self.finish_sentence_call(
            span,
            Args::default(),
            Vec::new(),
            phrases,
            mode,
        ))
    }

    /// Read a call that opens with the longest starting phrase the words
    /// ahead spell: the phrase, the first argument, then the rest as a
    /// preposition call.
    ///
    /// # Arguments
    ///
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, or `None` when no starting phrase is ahead.
    fn start_phrase_call(&mut self, mode: Mode) -> Option<PResult<Head>>
    {
        let all = self.functions.values().flatten();
        let starts = all.filter(|w| w.params > 0).filter_map(|w| w.start.as_ref());
        let len = starts.filter_map(|m| self.spells_at(m, 0)).max()?;
        let start = self.take_words(len);
        Some(self.read_after_start(start, mode))
    }

    /// Read the rest of a call after its starting phrase.
    ///
    /// # Arguments
    ///
    /// * `start` - The starting phrase.
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, or a diagnostic.
    fn read_after_start(&mut self, start: Ident, mode: Mode) -> PResult<Head>
    {
        let first = self.parse_notation()?;
        extract!(Head::Expr(first) = self.parse_short_after(first, Mode::Expr)?);
        let end = self.sentence_end(0);
        let end = end.ok_or_else(|| self.err_expected(&t!("parser.preposition")))?;
        self.parse_preposition_call(first, end, mode, Some(start))
    }

    /// Whether the number of arguments of a call written as a sentence, and
    /// the places of its prepositions, match a function of its name. The
    /// check of the names leaves a wrong phrase or preposition to the names
    /// and kinds stage, which names the words the function wants.
    ///
    /// # Arguments
    ///
    /// * `head` - The call read.
    ///
    /// # Returns
    ///
    /// `true` when the numbers match.
    fn fits_words(&self, head: &Head) -> bool
    {
        let stmt = tern!(let Head::Stmt(s) = head, Some(&**s), None);
        let (callee, args, written) = match (head, stmt)
        {
            (_, Some(Stmt::Call(n))) => (&n.callee, n.args.len(), &n.prepositions),
            (Head::Expr(Expr::Call(n)), _) =>
            {
                (&n.callee, n.args.len(), &n.prepositions)
            }
            _ => return false,
        };
        let places = |w: &CallWords| {
            let declared = w.prepositions.iter().map(Option::is_some);
            declared.eq(written.iter().map(Option::is_some))
        };
        let fits = |w: &&CallWords| w.params == args && places(w);
        self.fn_words(callee.name.as_str()).any(|w| fits(&w))
    }

    /// Parse the rest of a preposition call after its first argument: the
    /// preposition of each argument, the arguments after the first
    /// (notation expressions), then the function and the end, which
    /// [`Self::preposition_end`] found.
    ///
    /// # Arguments
    ///
    /// * `first` - The first argument.
    /// * `end` - How far ahead the end is.
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, or a diagnostic.
    pub(crate) fn parse_preposition_call(
        &mut self, first: Expr, end: usize, mode: Mode, start: Option<Ident>,
    ) -> PResult<Head>
    {
        let end_at = self.peek_n(end).span.lo;
        let words: Vec<CallWords> =
            self.fn_words(&self.callee_text(&self.peek_n(end))).cloned().collect();
        let ends: Vec<String> = words.iter().filter_map(|w| w.end.clone()).collect();
        let span = start.as_ref().map_or(first.span(), |s| s.span);
        let mut args = Args::default();
        args.push(first);
        let mut prepositions = Vec::new();
        let end = loop
        {
            let at = prepositions.len();
            prepositions.push(self.parse_arg_preposition(&words, at, end_at));
            let end = self.parse_end_phrase(&ends, end_at);
            do_if!(end.is_some() || self.peek().span.lo >= end_at, break end);
            // A `၊` parts two arguments, as a preposition does.
            self.eat_punct(Punct::ListSep);
            let arg = self.parse_notation()?;
            extract!(Head::Expr(arg) = self.parse_short_after(arg, Mode::Expr)?);
            args.push(arg);
        };
        ret_if!(
            self.peek().span.lo != end_at,
            Err(self.err_expected(&t!("parser.preposition")))
        );
        let phrases = Phrases::builder().maybe_start(start).maybe_end(end).build();
        self.finish_sentence_call(span, args, prepositions, phrases, mode)
    }

    /// Read the function and the end of a call written as a sentence, after
    /// its arguments, prepositions and phrases.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the call starts.
    /// * `args` - The arguments.
    /// * `prepositions` - The preposition after each argument.
    /// * `phrases` - The phrases written.
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, or a diagnostic.
    pub(crate) fn finish_sentence_call(
        &mut self, start: Span, args: Args, prepositions: Vec<Option<Ident>>,
        phrases: Phrases, mode: Mode,
    ) -> PResult<Head>
    {
        let callee_tok = self.bump();
        let name = self.callee_text(&callee_tok);
        let callee = Ident::builder().name(name).span(callee_tok.span).build();
        let long = self.eat(Kw::FnValue);
        do_if!(long, _ = self.expect(Kw::Ko)?);
        #[rustfmt::skip]
        let parts = CallParts::builder().start(start).callee(callee).args(args)
            .prepositions(prepositions).phrases(phrases).build();
        let head = self.finish_using(parts, long, mode)?;
        // A heap function of the prelude gives no value (ZK0315).
        let heap_value =
            |c: &CallExpr| self.prelude_heap.contains(c.callee.name.as_str());
        let value = tern!(let Head::Expr(Expr::Call(c)) = &head, Some(c), None);
        let heap = value.filter(|c| heap_value(c));
        do_if!(let Some(c) = heap, self.diags.push(rd::value_from_void_function(c.span, &c.callee.name)));
        Ok(head)
    }

    /// Read the ending phrase of a call when the words ahead spell one of
    /// `ends` and the function comes right after it.
    ///
    /// # Arguments
    ///
    /// * `ends` - The ending phrases of the functions the call can be of.
    /// * `end_at` - Where the function name starts.
    ///
    /// # Returns
    ///
    /// The phrase, or `None` with no such phrase ahead.
    fn parse_end_phrase(&mut self, ends: &[String], end_at: Pos) -> Option<Ident>
    {
        let reaches = |len: &usize| self.peek_n(*len).span.lo == end_at;
        let spelled = ends.iter().filter_map(|m| self.spells_at(m, 0));
        let len = spelled.filter(reaches).max()?;
        Some(self.take_words(len))
    }

    /// Read the preposition after an argument, each word in its written
    /// spelling: the longest of the prepositions the functions of the name give
    /// the parameter at `at` that the next words spell. Where a function gives
    /// one there, a reserved word that is no preposition of it still reads as
    /// one, so the check names the preposition it wants; a name, a literal or a
    /// group starts the next argument instead.
    ///
    /// # Arguments
    ///
    /// * `words` - The words of the functions the call can be of.
    /// * `at` - The place of the argument.
    /// * `end_at` - Where the function name starts.
    ///
    /// # Returns
    ///
    /// The preposition as a name (its words joined with one space), or `None`
    /// for an argument with none.
    fn parse_arg_preposition(
        &mut self, words: &[CallWords], at: usize, end_at: Pos,
    ) -> Option<Ident>
    {
        let given = |w: &CallWords| w.prepositions.get(at).cloned().flatten();
        let known: Vec<String> = words.iter().filter_map(given).collect();
        let spelled = known.iter().filter_map(|m| self.spells_at(m, 0)).max();
        ret_if!(let Some(len) = spelled, Some(self.take_words(len)));
        let tok = self.peek();
        let word = matches!(tok.kind, TokenKind::Kw(_)) && is_preposition_word(&tok);
        let wrong = !known.is_empty() && word && tok.span.lo < end_at;
        wrong.then(|| self.take_words(1))
    }

    /// Move past the next words, read as one preposition or phrase, each word
    /// in its written spelling.
    ///
    /// # Arguments
    ///
    /// * `len` - The number of words; at least one.
    ///
    /// # Returns
    ///
    /// The words joined with one space.
    pub(crate) fn take_words(&mut self, len: usize) -> Ident
    {
        let toks: Vec<Token> = (0..len).map(|_| self.bump()).collect();
        let words: Vec<&str> =
            toks.iter().map(|t| written_form(self.text_of(t.span))).collect();
        let span = toks[0].span.to(&toks[len - 1].span);
        Ident::builder().name(words.join(" ")).span(span).build()
    }

    /// The count of words a preposition or a phrase takes when the words from
    /// `from` tokens ahead spell it.
    ///
    /// # Arguments
    ///
    /// * `words` - The preposition or phrase, its words joined with one space.
    /// * `from` - How far ahead it starts.
    ///
    /// # Returns
    ///
    /// The number of its words, or `None` when the words ahead differ.
    pub(crate) fn spells_at(&self, words: &str, from: usize) -> Option<usize>
    {
        let words: Vec<&str> = words.split(' ').collect();
        let at = |(n, w): (usize, &&str)| {
            let tok = self.peek_n(from + n);
            is_preposition_word(&tok) && written_form(self.text_of(tok.span)) == *w
        };
        words.iter().enumerate().all(at).then_some(words.len())
    }

    /// Parse the rest of an arguments-first call after its first argument:
    /// the arguments as a list in words, then `ကို` after one or `တို့ကို`
    /// after two or more, `သုံး၍`, the function and the end. Each argument is
    /// a notation expression, so a sentence that gives a value goes inside
    /// parentheses.
    ///
    /// # Arguments
    ///
    /// * `first` - The first argument.
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, or a diagnostic.
    pub(crate) fn parse_using(&mut self, first: Expr, mode: Mode) -> PResult<Head>
    {
        let start = first.span();
        let args = self.parse_word_list_from(first, |p| {
            let arg = p.parse_notation()?;
            extract!(Head::Expr(arg) = p.parse_short_after(arg, Mode::Expr)?);
            Ok(arg)
        })?;
        let close = self.expect_list_word(args.len(), Kw::Ko, Kw::ListKo)?;
        let plural = close.kind == TokenKind::Kw(Kw::ListKo);
        let using = UsingParticle::builder().plural(plural).span(close.span).build();
        self.expect(Kw::Using)?;
        let long =
            self.at_punct(Punct::LParen) || kw_of(&self.peek_n(1)) == Some(Kw::FnValue);
        let (callee, place) =
            tern!(long, self.parse_long_callee()?, (self.parse_ident()?, None));
        let parts = CallParts::builder().start(start).callee(callee).maybe_place(place);
        let parts = parts.args(args).using(using);
        let parts = parts.build();
        self.finish_using(parts, long, mode)
    }

    /// Read an operand as a short call with no arguments when it is a name
    /// with the end of a short call after it.
    ///
    /// # Arguments
    ///
    /// * `operand` - The operand read.
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, the operand unchanged, or a diagnostic.
    pub(crate) fn parse_short_after(
        &mut self, operand: Expr, mode: Mode,
    ) -> PResult<Head>
    {
        match operand
        {
            Expr::Name(name) if self.at_short_end() =>
            {
                self.parse_bare_short(name, mode)
            }
            operand => Ok(Head::Expr(operand)),
        }
    }

    /// Parse the end of a short call with no arguments, after its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The function name, read as a variable.
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, or a diagnostic.
    fn parse_bare_short(&mut self, name: NameExpr, mode: Mode) -> PResult<Head>
    {
        let callee = Ident::builder().name(name.name).span(name.span).build();
        let parts =
            CallParts::builder().start(name.span).callee(callee).args(Args::default());
        let parts = parts.build();
        self.finish_using(parts, false, mode)
    }

    /// Parse the end of a call whose function is read: a statement end, read
    /// on to the full stop, or a value end.
    ///
    /// # Arguments
    ///
    /// * `parts` - What the call holds so far.
    /// * `long` - Whether the function came as `<fn> ဟူသော လုပ်ဆောင်ချက်ကို`.
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, or a diagnostic.
    pub(crate) fn finish_using(
        &mut self, parts: CallParts, long: bool, mode: Mode,
    ) -> PResult<Head>
    {
        let ends: &[Kw] = match (long, mode)
        {
            (true, Mode::Stmt) => &[Kw::WillDo, Kw::CallResult, Kw::CallGoodResult],
            (true, Mode::Expr) => &[Kw::CallResult, Kw::CallGoodResult],
            (false, Mode::Stmt) => &[Kw::Will, Kw::UsedResult, Kw::UsedGoodResult],
            (false, Mode::Expr) => &[Kw::UsedResult, Kw::UsedGoodResult],
        };
        let end = self.expect_one_of(ends)?;
        let kw = kw_of(&end);
        ret_if!(
            matches!(kw, Some(Kw::WillDo | Kw::Will)),
            self.finish_call_stmt(parts).map(|s| Head::Stmt(Box::new(s)))
        );
        let pass = matches!(kw, Some(Kw::CallGoodResult | Kw::UsedGoodResult));
        let CallParts { start, callee, place, args, using, prepositions, phrases } =
            parts;
        let call = self.call_expr(start.to(&end.span), callee, args, pass);
        let mut call = crate::notation::with_place(call, place);
        extract!(Expr::Call(node) = &mut call);
        node.using = using;
        node.prepositions = prepositions;
        node.phrases = phrases;
        Ok(Head::Expr(call))
    }
}
