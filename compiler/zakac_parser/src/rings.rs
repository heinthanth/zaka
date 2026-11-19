//! The statements of the ring collector. Each one leans on the reserved word
//! `မင်မိုရီ`; the other words are read by their text, so none of them is
//! reserved and each stays free to name a variable.
//!
//! - `မင်မိုရီရှိ မလိုအပ်သော အရာများကို ရှင်းလင်းမည်။` runs the collector now;
//! - `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းခြင်းကို ခေတ္တရပ်မည်။` pauses the collections a full buffer
//!   runs;
//! - `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းခြင်းကို ပြန်လည်စတင်မည်။` resumes them;
//! - `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းခြင်းကို <octets> ဘိုက် နှင့်အထက်မှ လုပ်ဆောင်မည်။` sets the octets
//!   in the buffer that run a collection.

use zakac_ast::stmt::RingsAct;
use zakac_ast::stmt::RingsStmt;
use zakac_ast::stmt::Stmt;
use zakac_lexer::token::Kw;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::describe::quote;
use crate::parser::PResult;
use crate::parser::Parser;

/// The word that opens the statements that pause, resume and tune the
/// collector.
const AUTO: &str = "အလိုအလျောက်";

/// The words after `မင်မိုရီ` that run the collector now.
const COLLECT: [&str; 4] = ["ရှိ", "မလိုအပ်သော", "အရာများကို", "ရှင်းလင်းမည်"];

/// The word after `အလိုအလျောက် မင်မိုရီ`.
const CLEARING: &str = "ရှင်းလင်းခြင်းကို";

/// The verb that pauses the collections.
const PAUSE: &str = "ခေတ္တရပ်မည်";

/// The verb that resumes them.
const RESUME: &str = "ပြန်လည်စတင်မည်";

/// The words after the octets of a threshold.
const THRESHOLD: [&str; 3] = ["ဘိုက်", "နှင့်အထက်မှ", "လုပ်ဆောင်မည်"];

impl Parser<'_>
{
    /// Whether `အလိုအလျောက် မင်မိုရီ` opens the statement.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_auto_rings(&self) -> bool
    {
        let first = self.peek();
        let named = first.kind == TokenKind::Ident && self.text_of(first.span) == AUTO;
        named && self.peek_n(1).kind == TokenKind::Kw(Kw::Mem)
    }

    /// Whether `မင်မိုရီရှိ` opens the statement; `မင်မိုရီတွင်` opens a heap
    /// statement.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_rings_collect(&self) -> bool
    {
        self.peek_n(1).kind == TokenKind::Kw(Kw::Located)
    }

    /// Parse `မင်မိုရီရှိ မလိုအပ်သော အရာများကို ရှင်းလင်းမည်။`.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    pub(crate) fn parse_rings_collect(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        self.eat_words(&COLLECT)?;
        self.finish_rings(RingsAct::Collect, None, start)
    }

    /// Parse a statement that opens with `အလိုအလျောက် မင်မိုရီ ရှင်းလင်းခြင်းကို`: a
    /// pause, a resume or a threshold.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    pub(crate) fn parse_auto_rings(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        self.bump();
        self.eat_words(&[CLEARING])?;
        let next = self.text_of(self.peek().span);
        ret_if!(
            PAUSE.starts_with(next),
            self.verb_rings(PAUSE, RingsAct::Pause, start)
        );
        ret_if!(
            RESUME.starts_with(next),
            self.verb_rings(RESUME, RingsAct::Resume, start)
        );
        let octets = self.parse_notation()?;
        self.eat_words(&THRESHOLD)?;
        self.finish_rings(RingsAct::Threshold, Some(octets), start)
    }

    /// Read the verb of a pause or a resume, and the full stop.
    ///
    /// # Arguments
    ///
    /// * `verb` - The verb.
    /// * `act` - What the statement does.
    /// * `start` - The region of the first word.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn verb_rings(&mut self, verb: &str, act: RingsAct, start: Span) -> PResult<Stmt>
    {
        self.eat_words(&[verb])?;
        self.finish_rings(act, None, start)
    }

    /// Read the full stop and build the statement.
    ///
    /// # Arguments
    ///
    /// * `act` - What the statement does.
    /// * `octets` - The octets of a threshold.
    /// * `start` - The region of the first word.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn finish_rings(
        &mut self, act: RingsAct, octets: Option<zakac_ast::expr::Expr>, start: Span,
    ) -> PResult<Stmt>
    {
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        Ok(Stmt::Rings(
            RingsStmt::builder().act(act).maybe_octets(octets).span(span).build(),
        ))
    }

    /// Read fixed words by their text. A word can stand in one token or in
    /// several, so `အရာများကို` and `အရာများ ကို` both read.
    ///
    /// # Arguments
    ///
    /// * `words` - The words, in order.
    ///
    /// # Returns
    ///
    /// Nothing, or an `expected-token` diagnostic at the first word that does
    /// not read.
    fn eat_words(&mut self, words: &[&str]) -> PResult<()>
    {
        for word in words
        {
            self.eat_word(word)?;
        }
        Ok(())
    }

    /// Read one fixed word, from as many tokens as spell it.
    ///
    /// # Arguments
    ///
    /// * `word` - The word.
    ///
    /// # Returns
    ///
    /// Nothing, or an `expected-token` diagnostic.
    fn eat_word(&mut self, word: &str) -> PResult<()>
    {
        let mut read = String::new();
        while read.len() < word.len()
        {
            let tok = self.peek();
            let next = format!("{read}{}", self.text_of(tok.span));
            let fits = tok.kind != TokenKind::Eof && word.starts_with(&next);
            ret_if!(!fits, Err(self.err_expected(&quote(word))));
            read = next;
            self.bump();
        }
        Ok(())
    }
}
