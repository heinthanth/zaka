//! The phrase `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော`: the mark of a shared struct
//! whose objects jobs can use, and the name of the trait of the kinds jobs can
//! use. Its words are read through their text, apart or glued.

use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::VerbExpr;
use zakac_ast::jobs::JOBS_WORDS;
use zakac_ast::jobs::USABLE_IN_JOBS;
use zakac_ast::stmt::EachPart;
use zakac_ast::stmt::EachStmt;
use zakac_ast::stmt::Stmt;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::parser::PResult;
use crate::parser::Parser;

/// The tokens the phrase takes from `at` on: 2 for the words apart, 1 for
/// the words glued, or `None` when the phrase does not stand there.
///
/// # Arguments
///
/// * `tokens` - The tokens.
/// * `at` - Where the phrase would start.
/// * `text_of` - The text of a token.
///
/// # Returns
///
/// The count of tokens, or `None`.
pub(crate) fn jobs_phrase_at(
    tokens: &[Token], at: usize, text_of: &dyn Fn(&Token) -> String,
) -> Option<usize>
{
    let word = |n: usize| {
        tokens
            .get(n)
            .filter(|t| t.kind == TokenKind::Ident)
            .map(text_of)
            .unwrap_or_default()
    };
    let first = word(at);
    let glued = format!("{}{}", JOBS_WORDS[0], JOBS_WORDS[1]);
    (first == glued).then_some(1).or_else(|| {
        (first == JOBS_WORDS[0] && word(at + 1) == JOBS_WORDS[1]).then_some(2)
    })
}

/// The first token past the marks of a kind declaration from `at` on:
/// `အများသုံး`, `မျှဝေနိုင်သော` and the phrase, in each order.
///
/// # Arguments
///
/// * `tokens` - The tokens.
/// * `at` - The first token after `ဟူသော`.
/// * `text_of` - The text of a token.
///
/// # Returns
///
/// The index of the token past the marks.
pub(crate) fn past_marks(
    tokens: &[Token], mut at: usize, text_of: &dyn Fn(&Token) -> String,
) -> usize
{
    let mark = |n: usize| {
        let kw = tokens.get(n).map(|t| t.kind);
        let plain = matches!(kw, Some(TokenKind::Kw(Kw::Export | Kw::Shareable)));
        tern!(plain, Some(1), jobs_phrase_at(tokens, n, text_of))
    };
    while let Some(len) = mark(at)
    {
        at += len;
    }
    at
}

/// The word before the call end that runs a call as a job.
pub(crate) const PARALLEL: &str = "အပြိုင်";

/// The word after `လုပ်ဆောင်သော` in the expression that gives the job.
pub(crate) const JOB_NOUN: &str = "လုပ်ငန်း";

/// The words after `မှ` that receive from a channel, `ရရှိသော` or
/// `လက်ခံရရှိသော`.
const RECEIVED: [&str; 2] = ["ရရှိသော", "လက်ခံရရှိသော"];

/// The word after them in the expression of a receive.
const DATA_WORD: &str = "ဒေတာ";

/// The word after them in the loop over each value that comes, alone or
/// glued to its `ကို`.
const EACH_DATA: [&str; 2] = ["ဒေတာတိုင်း", "ဒေတာတိုင်းကို"];

impl Parser<'_>
{
    /// Whether the end of a parallel call stands at the cursor:
    /// `အပြိုင် လုပ်ဆောင်မည်` (a statement) or `အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း` (the
    /// expression of the job).
    ///
    /// # Returns
    ///
    /// `Some(true)` for the statement, `Some(false)` for the expression,
    /// `None` when neither stands there.
    pub(crate) fn parallel_end(&self) -> Option<bool> { self.parallel_end_at(0) }

    /// [`Parser::parallel_end`] from the token `at` ahead on.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// `Some(true)` for the statement, `Some(false)` for the expression,
    /// `None` when neither stands there.
    pub(crate) fn parallel_end_at(&self, at: usize) -> Option<bool>
    {
        let word = |n: usize, text: &str| {
            let tok = self.peek_n(at + n);
            tok.kind == TokenKind::Ident && self.text_of(tok.span) == text
        };
        ret_if!(!word(0, PARALLEL), None);
        let next = self.peek_n(at + 1).kind;
        ret_if!(next == TokenKind::Kw(Kw::WillDo), Some(true));
        (next == TokenKind::Kw(Kw::CallHead) && word(2, JOB_NOUN)).then_some(false)
    }

    /// Whether the end of a long call stands `n` tokens ahead: `လုပ်ဆောင်မည်`,
    /// `လုပ်ဆောင်သော ရလဒ်`, `လုပ်ဆောင်သော ရလဒ်ကောင်း`, or the end of a parallel call.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    pub(crate) fn call_word_at(&self, n: usize) -> bool
    {
        let ends = [Kw::WillDo, Kw::CallResult, Kw::CallGoodResult];
        let kw = self.peek_n(n).kind;
        ends.iter().any(|e| kw == TokenKind::Kw(*e))
            || self.parallel_end_at(n).is_some()
    }

    /// The token count of `မှ ရရှိသော <word>` from the token `n` ahead on,
    /// when `<word>` is one of `words`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead `မှ` would stand.
    /// * `words` - The words that can end it.
    ///
    /// # Returns
    ///
    /// The index of the word in `words`, or `None`.
    fn received_at(&self, n: usize, words: &[&str]) -> Option<usize>
    {
        ret_if!(self.peek_n(n).kind != TokenKind::Kw(Kw::From), None);
        ret_if!(!RECEIVED.iter().any(|w| self.is_word_at(n + 1, w)), None);
        words.iter().position(|w| self.is_word_at(n + 2, w))
    }

    /// Whether a receive comes next: `မှ ရရှိသော ဒေတာ`.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    pub(crate) fn at_receive(&self) -> bool
    {
        self.received_at(0, &[DATA_WORD]).is_some()
    }

    /// Parse `မှ ရရှိသော ဒေတာ` after a channel: the next value of it.
    ///
    /// # Arguments
    ///
    /// * `pipe` - The channel.
    ///
    /// # Returns
    ///
    /// The receive.
    pub(crate) fn parse_receive(&mut self, pipe: Expr) -> Expr
    {
        do_for!(_ in 0..2, self.bump());
        let last = self.bump();
        let span = pipe.span().to(&last.span);
        let id = self.new_id();
        #[rustfmt::skip]
        let node = VerbExpr::builder().id(id).op(VerbOp::Receive).args(vec![pipe])
            .particle(last.particle).is_cmp(false).span(span).build();
        Expr::Verb(node)
    }

    /// Whether the head of a loop over each value of a channel comes next:
    /// `မှ ရရှိသော ဒေတာတိုင်း ကို`.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_receive_each(&self) -> bool
    {
        let at = self.received_at(0, &EACH_DATA);
        at == Some(1) || (at == Some(0) && self.peek_n(3).kind == TokenKind::Kw(Kw::Ko))
    }

    /// Parse `မှ ရရှိသော ဒေတာတိုင်း ကို <name> အနေဖြင့် ယူ၍ <block>` after the
    /// channel a loop takes each value of.
    ///
    /// # Arguments
    ///
    /// * `pipe` - The channel.
    ///
    /// # Returns
    ///
    /// The loop, or a diagnostic.
    pub(crate) fn parse_receive_each(&mut self, pipe: Expr) -> PResult<Stmt>
    {
        let glued = self.received_at(0, &EACH_DATA) == Some(1);
        do_for!(_ in 0..3, self.bump());
        do_if!(!glued, _ = self.bump());
        let var = self.parse_ident()?;
        self.expect(Kw::CountAs)?;
        self.expect(Kw::Taking)?;
        let block = self.parse_branch(true)?;
        let span = pipe.span().to(&block.span);
        #[rustfmt::skip]
        let node = EachStmt::builder().source(pipe).part(EachPart::Received).var(var)
            .block(block).span(span).build();
        Ok(Stmt::Each(node))
    }

    /// The tokens the phrase takes from the token `n` ahead on.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// The count of tokens, or `None` when the phrase does not stand there.
    pub(crate) fn jobs_phrase_ahead(&self, n: usize) -> Option<usize>
    {
        let tokens: Vec<Token> = (n..n + 2).map(|k| self.peek_n(k)).collect();
        jobs_phrase_at(&tokens, 0, &|t: &Token| self.text_of(t.span).to_string())
    }

    /// Whether the phrase stands at the cursor, as the name of a trait.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_jobs_trait(&self) -> bool { self.jobs_phrase_ahead(0).is_some() }

    /// Read the phrase as the name of the built-in trait of the kinds jobs
    /// can use.
    ///
    /// # Returns
    ///
    /// The name, over the words of the phrase.
    pub(crate) fn jobs_trait_name(&mut self) -> Ident
    {
        let start = self.peek().span;
        self.eat_jobs_phrase();
        Ident::builder().name(USABLE_IN_JOBS).span(start.to(&self.prev_span())).build()
    }

    /// Move past the phrase when it stands at the cursor.
    ///
    /// # Returns
    ///
    /// `true` when it stood there.
    pub(crate) fn eat_jobs_phrase(&mut self) -> bool
    {
        let Some(count) = self.jobs_phrase_ahead(0)
        else
        {
            return false;
        };
        (0..count).for_each(|_| _ = self.bump());
        true
    }
}
