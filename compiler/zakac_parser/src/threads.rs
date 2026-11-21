//! The phrase `အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော`: the mark of a shared struct
//! whose objects threads can use, and the name of the trait of the kinds
//! threads can use. Its words are read through their text, apart or glued.

use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::VerbExpr;
use zakac_ast::stmt::EachPart;
use zakac_ast::stmt::EachStmt;
use zakac_ast::stmt::LockStmt;
use zakac_ast::stmt::Stmt;
use zakac_ast::threads::THREADS_WORDS;
use zakac_ast::threads::USABLE_IN_THREADS;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
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
pub(crate) fn threads_phrase_at(
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
    let glued = format!("{}{}", THREADS_WORDS[0], THREADS_WORDS[1]);
    (first == glued).then_some(1).or_else(|| {
        (first == THREADS_WORDS[0] && word(at + 1) == THREADS_WORDS[1]).then_some(2)
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
        tern!(plain, Some(1), threads_phrase_at(tokens, n, text_of))
    };
    while let Some(len) = mark(at)
    {
        at += len;
    }
    at
}

/// The word before the call end that runs a call as a thread.
pub(crate) const PARALLEL: &str = "အပြိုင်";

/// The word before `အပြိုင်` that puts a thread on an OS thread of its own.
const APART: &str = "သီးသန့်";

/// The word after `လုပ်ဆောင်သော` in the expression that gives the thread.
pub(crate) const THREAD_NOUN: &str = "လုပ်ငန်း";

/// The words that take the lock of an object after its `ကို`, glued to `ယူ၍`
/// or apart from it.
const LOCK_WORDS: [&str; 2] = ["ခေတ္တ", "ခဏ"];

/// The word that opens the count of threads of a parallel loop or map,
/// `အပြိုင်လုပ်ငန်း <n> ခုခွဲ၍`.
const THREAD_WORD: &str = "အပြိုင်လုပ်ငန်း";

/// The word that ends the count of threads.
const SPLIT_WORD: &str = "ခုခွဲ၍";

/// The word that ends a parallel map, after `လုပ်ဆောင်သော`.
const MAP_RESULTS: &str = "ရလဒ်များ";

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
    /// expression of the thread).
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
        let at = at + usize::from(self.apart_at(at));
        let word = |n: usize, text: &str| {
            let tok = self.peek_n(at + n);
            tok.kind == TokenKind::Ident && self.text_of(tok.span) == text
        };
        ret_if!(!word(0, PARALLEL), None);
        let next = self.peek_n(at + 1).kind;
        ret_if!(next == TokenKind::Kw(Kw::WillDo), Some(true));
        (next == TokenKind::Kw(Kw::CallHead) && word(2, THREAD_NOUN)).then_some(false)
    }

    /// Whether `သီးသန့် အပြိုင်` stands `at` tokens ahead: the end of a parallel
    /// call whose thread runs on an OS thread of its own.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn apart_at(&self, at: usize) -> bool
    {
        self.is_word_at(at, APART) && self.is_word_at(at + 1, PARALLEL)
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

    /// Move past `အပြိုင်` when it comes next, the mark of a parallel loop
    /// before `ရေတွက်၍` or `ယူ၍`.
    ///
    /// # Returns
    ///
    /// `true` when it came.
    pub(crate) fn eat_parallel(&mut self) -> bool
    {
        let next = self.peek_n(1).kind;
        let loop_word = matches!(
            next,
            TokenKind::Kw(Kw::Taking | Kw::Counting | Kw::Increasing | Kw::Decreasing)
        );
        ret_if!(!self.is_word_at(0, PARALLEL) || !loop_word, false);
        self.bump();
        true
    }

    /// Parse the count of threads of a parallel loop or map,
    /// `အပြိုင်လုပ်ငန်း <n> ခုခွဲ၍`, which stands in place of `အပြိုင်`; the word
    /// `ခုခွဲ၍` reads glued or apart.
    ///
    /// # Returns
    ///
    /// The count, `None` when the phrase does not come next, or a diagnostic.
    pub(crate) fn parse_threads(&mut self) -> PResult<Option<Expr>>
    {
        ret_if!(!self.is_word_at(0, THREAD_WORD), Ok(None));
        self.bump();
        let count = self.parse_notation()?;
        self.eat_words(&[SPLIT_WORD])?;
        Ok(Some(count))
    }

    /// Whether a parallel map comes next: `၏ အကွက်တိုင်း ကို <fn> ဟူသော
    /// လုပ်ဆောင်ချက် ဖြင့် အပြိုင် လုပ်ဆောင်သော ရလဒ်များ`.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    pub(crate) fn at_par_map(&self) -> bool
    {
        let kw = |n: usize, k: Kw| self.peek_n(n).kind == TokenKind::Kw(k);
        self.peek().kind == TokenKind::Punct(Punct::Ei)
            && kw(1, Kw::EachCell)
            && kw(2, Kw::Ko)
            && self.peek_n(3).kind == TokenKind::Ident
            && kw(4, Kw::FnValue)
            && kw(5, Kw::Hpjint)
            && (self.is_word_at(6, THREAD_WORD) || self.par_map_end_at(6))
    }

    /// Whether `အပြိုင် လုပ်ဆောင်သော ရလဒ်များ` stands `n` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - Where `အပြိုင်` stands.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn par_map_end_at(&self, n: usize) -> bool
    {
        let call_head = self.peek_n(n + 1).kind == TokenKind::Kw(Kw::CallHead);
        self.is_word_at(n, PARALLEL) && call_head && self.is_word_at(n + 2, MAP_RESULTS)
    }

    /// Parse a parallel map after the list it maps.
    ///
    /// # Arguments
    ///
    /// * `list` - The list.
    ///
    /// # Returns
    ///
    /// The map, or a diagnostic.
    pub(crate) fn parse_par_map(&mut self, list: Expr) -> PResult<Expr>
    {
        do_for!(_ in 0..3, _ = self.bump());
        let func = self.parse_ident()?;
        do_for!(_ in 0..2, _ = self.bump());
        let threads = self.parse_threads()?;
        do_if!(threads.is_none(), _ = self.bump());
        self.expect(Kw::CallHead)?;
        let last = self.bump();
        let span = list.span().to(&last.span);
        let id = self.new_id();
        let fn_id = self.new_id();
        let name = Expr::Name(
            NameExpr::builder().id(fn_id).name(func.name).span(func.span).build(),
        );
        #[rustfmt::skip]
        let args = [list, name].into_iter().chain(threads).collect();
        let node = VerbExpr::builder()
            .id(id)
            .op(VerbOp::ParMap)
            .args(args)
            .particle(last.particle)
            .is_cmp(false)
            .span(span)
            .build();
        Ok(Expr::Verb(node))
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

    /// Parse `မှ ရရှိသော ဒေတာတိုင်း ကို <name> အနေဖြင့် ယူ၍ <block>` after the
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

    /// The token count of the head of a lock block at the cursor:
    /// `<obj> ကို (ခေတ္တ|ခဏ)ယူ၍`, the last word glued or apart.
    ///
    /// # Returns
    ///
    /// The count, or `None` when no head stands there.
    fn lock_head(&self) -> Option<usize>
    {
        ret_if!(self.peek().kind != TokenKind::Ident, None);
        ret_if!(self.peek_n(1).kind != TokenKind::Kw(Kw::Ko), None);
        let glued = LOCK_WORDS.iter().any(|w| self.is_word_at(2, &format!("{w}ယူ၍")));
        ret_if!(glued, Some(3));
        let apart = LOCK_WORDS.iter().any(|w| self.is_word_at(2, w));
        (apart && self.peek_n(3).kind == TokenKind::Kw(Kw::Taking)).then_some(4)
    }

    /// Whether a lock block starts at the cursor.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    pub(crate) fn at_lock(&self) -> bool { self.lock_head().is_some() }

    /// Parse a lock block: `<obj> ကို (ခေတ္တ|ခဏ)ယူ၍ <stmt>`, or with a block.
    ///
    /// # Returns
    ///
    /// The block, or a diagnostic.
    pub(crate) fn parse_lock(&mut self) -> PResult<Stmt>
    {
        let head = self.lock_head().unwrap_or(3);
        let obj = self.parse_ident()?;
        do_for!(_ in 1..head, _ = self.bump());
        let block = self.parse_branch(true)?;
        let span = obj.span.to(&block.span);
        Ok(Stmt::Lock(LockStmt::builder().obj(obj).block(block).span(span).build()))
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
    pub(crate) fn threads_phrase_ahead(&self, n: usize) -> Option<usize>
    {
        let tokens: Vec<Token> = (n..n + 2).map(|k| self.peek_n(k)).collect();
        threads_phrase_at(&tokens, 0, &|t: &Token| self.text_of(t.span).to_string())
    }

    /// Whether the phrase stands at the cursor, as the name of a trait.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_threads_trait(&self) -> bool
    {
        self.threads_phrase_ahead(0).is_some()
    }

    /// Read the phrase as the name of the built-in trait of the kinds threads
    /// can use.
    ///
    /// # Returns
    ///
    /// The name, over the words of the phrase.
    pub(crate) fn threads_trait_name(&mut self) -> Ident
    {
        let start = self.peek().span;
        self.eat_threads_phrase();
        Ident::builder()
            .name(USABLE_IN_THREADS)
            .span(start.to(&self.prev_span()))
            .build()
    }

    /// Move past the phrase when it stands at the cursor.
    ///
    /// # Returns
    ///
    /// `true` when it stood there.
    pub(crate) fn eat_threads_phrase(&mut self) -> bool
    {
        let Some(count) = self.threads_phrase_ahead(0)
        else
        {
            return false;
        };
        (0..count).for_each(|_| _ = self.bump());
        true
    }
}
