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
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::UsingParticle;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_i18n::t;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex_at;
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
    /// The word after each argument of a preposition call.
    #[builder(default)]
    pub(crate) prepositions: Vec<Ident>,
}

/// The prepositions of each function the parser knows, under its name: one list
/// for each function of the name, each preposition its words joined with one
/// space. A function with no prepositions has a blank list.
pub type Functions = HashMap<String, Vec<Vec<String>>>;

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
        text[lo as usize..hi as usize].to_string()
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
        let (mut n, mut depth, mut found) = (from, 0usize, None);
        loop
        {
            let end = tern!(depth == 0, self.call_end_at(n), None);
            if let Some((len, value)) = end
            {
                ret_if!(
                    !value || !self.starts_preposition(&self.peek_n(n + len)),
                    Some(n)
                );
                found = Some(n);
                n += len;
                continue;
            }
            let tok = self.peek_n(n);
            ret_if!(self.stops_preposition_scan(&tok, depth), found);
            match tok.kind
            {
                TokenKind::Punct(Punct::LParen | Punct::LBracket) => depth += 1,
                TokenKind::Punct(Punct::RParen | Punct::RBracket) => depth -= 1,
                _ => (),
            }
            n += 1;
        }
    }

    /// The end of a call of a function that declares prepositions, `n` tokens
    /// ahead: `<fn> မည်`, `<fn> သည့် ရလဒ်(ကောင်း)`, or
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
    fn call_end_at(&self, n: usize) -> Option<(usize, bool)>
    {
        let long = kw_of(&self.peek_n(n + 1)) == Some(Kw::FnValue)
            && kw_of(&self.peek_n(n + 2)) == Some(Kw::Ko);
        ret_if!(!self.is_preposition_fn(n), None);
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

    /// Whether the token `n` ahead names a function that declares prepositions:
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
    fn is_preposition_fn(&self, n: usize) -> bool
    {
        let tok = self.peek_n(n);
        let named = matches!(
            tok.kind,
            TokenKind::Ident | TokenKind::RawIdent | TokenKind::Kw(_)
        );
        named && self.fn_prepositions(&self.callee_text(&tok)).next().is_some()
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

    /// The preposition lists of the functions of a name that declare
    /// prepositions.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// Each list, one for each such function.
    fn fn_prepositions<'a>(
        &'a self, name: &str,
    ) -> impl Iterator<Item = &'a Vec<String>>
    {
        let lists = self.functions.get(name).into_iter().flatten();
        lists.filter(|prepositions| !prepositions.is_empty())
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
        is_preposition_word(tok) && self.first_words.contains(self.text_of(tok.span))
    }

    /// Whether the scan for the end of a preposition call stops at a token: the
    /// end of the sentence, a `)` or `]` that closes no group of the call, or,
    /// outside parentheses, a `၊` or a reserved word that is neither a word
    /// of a visible preposition, a particle, nor a word a notation argument can
    /// hold.
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
            self.preposition_words.contains(self.text_of(tok.span))
                || is_plain_particle(tok)
        };
        match tok.kind
        {
            TokenKind::Punct(Punct::FullStop) | TokenKind::Eof => true,
            TokenKind::Punct(Punct::RParen | Punct::RBracket) => !deep,
            TokenKind::Punct(Punct::ListSep) => !deep,
            TokenKind::Kw(kw) => !deep && !in_preposition() && !in_notation(kw),
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
        let read = self.parse_preposition_call(first.clone(), end, mode);
        ret_if!(read.as_ref().is_ok_and(|head| self.fits_prepositions(head)), read);
        self.cur = cur;
        self.diags.truncate(diags);
        self.next_id = next_id;
        Ok(Head::Expr(first))
    }

    /// Whether the number of prepositions of a preposition call matches the
    /// parameters of a function of its name.
    ///
    /// # Arguments
    ///
    /// * `head` - The call read.
    ///
    /// # Returns
    ///
    /// `true` when the numbers match.
    fn fits_prepositions(&self, head: &Head) -> bool
    {
        let stmt = tern!(let Head::Stmt(s) = head, Some(&**s), None);
        let (callee, count) = match (head, stmt)
        {
            (_, Some(Stmt::Call(n))) => (&n.callee, n.prepositions.len()),
            (Head::Expr(Expr::Call(n)), _) => (&n.callee, n.prepositions.len()),
            _ => return false,
        };
        self.fn_prepositions(callee.name.as_str())
            .any(|prepositions| prepositions.len() == count)
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
        &mut self, first: Expr, end: usize, mode: Mode,
    ) -> PResult<Head>
    {
        let callee_tok = self.peek_n(end);
        let end_at = callee_tok.span.lo;
        let name = self.callee_text(&callee_tok);
        let known: Vec<String> =
            self.fn_prepositions(&name).flatten().cloned().collect();
        let start = first.span();
        let mut args = Args::default();
        args.push(first);
        let mut prepositions = vec![self.parse_preposition(&known)?];
        while self.peek().span.lo < end_at
        {
            let arg = self.parse_notation()?;
            extract!(Head::Expr(arg) = self.parse_short_after(arg, Mode::Expr)?);
            args.push(arg);
            prepositions.push(self.parse_preposition(&known)?);
        }
        self.bump();
        let callee = Ident::builder().name(name).span(callee_tok.span).build();
        let long = self.eat(Kw::FnValue);
        do_if!(long, _ = self.expect(Kw::Ko)?);
        #[rustfmt::skip]
        let parts = CallParts::builder().start(start).callee(callee).args(args).prepositions(prepositions)
            .build();
        self.finish_using(parts, long, mode)
    }

    /// Read a preposition: the longest of the known prepositions that the next
    /// words spell, or else one word a preposition can hold, so a call with
    /// another preposition still reads and the check reports the
    /// preposition.
    ///
    /// # Arguments
    ///
    /// * `known` - The prepositions of the functions the call can be of.
    ///
    /// # Returns
    ///
    /// The preposition as a name (its words joined with one space), or an
    /// `expected-token` diagnostic.
    pub(crate) fn parse_preposition(&mut self, known: &[String]) -> PResult<Ident>
    {
        let spelled = known.iter().filter_map(|m| self.spells_preposition(m)).max();
        let len = spelled.unwrap_or(1);
        let first = self.peek();
        ret_if!(
            !is_preposition_word(&first),
            Err(self.err_expected(&t!("parser.preposition")))
        );
        let toks: Vec<Token> = (0..len).map(|n| self.peek_n(n)).collect();
        (0..len).for_each(|_| _ = self.bump());
        let words: Vec<&str> = toks.iter().map(|t| self.text_of(t.span)).collect();
        let span = first.span.to(&toks[len - 1].span);
        Ok(Ident::builder().name(words.join(" ")).span(span).build())
    }

    /// The count of words a preposition takes when the next words spell it.
    ///
    /// # Arguments
    ///
    /// * `preposition` - The preposition, its words joined with one space.
    ///
    /// # Returns
    ///
    /// The number of its words, or `None` when the next words differ.
    fn spells_preposition(&self, preposition: &str) -> Option<usize>
    {
        let words: Vec<&str> = preposition.split(' ').collect();
        let at = |(n, w): (usize, &&str)| self.text_of(self.peek_n(n).span) == *w;
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
        let CallParts { start, callee, place, args, using, prepositions } = parts;
        let call = self.call_expr(start.to(&end.span), callee, args, pass);
        let mut call = crate::notation::with_place(call, place);
        extract!(Expr::Call(node) = &mut call);
        node.using = using;
        node.prepositions = prepositions;
        Ok(Head::Expr(call))
    }
}
