//! The phrase `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော`: the mark of a shared struct
//! whose objects jobs can use, and the name of the trait of the kinds jobs can
//! use. Its words are read through their text, apart or glued.

use zakac_ast::expr::Ident;
use zakac_ast::jobs::JOBS_WORDS;
use zakac_ast::jobs::USABLE_IN_JOBS;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::tern;

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

impl Parser<'_>
{
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
