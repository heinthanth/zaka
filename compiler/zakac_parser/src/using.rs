//! The arguments-first call, `<args> (တို့)?ကို သုံး၍ ...`, and the short call
//! with no arguments, `<fn> မည်` and `<fn> သည့် ရလဒ်`.
//!
//! Each form ends as a statement or as a value. The long form names the
//! function as `<fn> ဟူသော လုပ်ဆောင်ချက်ကို`, then ends with `လုပ်ဆောင်မည်` (a statement)
//! or `လုပ်ဆောင်သော ရလဒ်(ကောင်း)?` (a value). The short form names the function
//! alone, then ends with `မည်` (a statement) or `သည့် ရလဒ်(ကောင်း)?` (a value).
//! Each builds the call node that the other call forms build.
//!
//! The marked call, `<arg> <mark> (<arg> <mark>)* <fn> <end>`, gives each
//! argument with the mark its parameter declares, `<kind> <name>(<mark>)`. The
//! parser knows which functions declare marks (`Parser::marked`), and those
//! names alone end a marked call, and a value of another call keeps its
//! meaning: `က ကို ခ သည့် ရလဒ် ဟု ထားမည်။` still sets `က`.

use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::UsingMark;
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
    pub(crate) using: Option<UsingMark>,
    /// The word after each argument of a marked call.
    #[builder(default)]
    pub(crate) marks: Vec<Ident>,
}

/// Whether a token can be a mark: a name, or a particle other than the
/// endings and closers `သည်`, `ပါ`, `ဖြစ်ကာ`, `ပြီးလျှင်` and `လက်ခံပြီး`, or
/// `တွင်` and `ဆီသို့`.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// `true` for such a token.
pub(crate) fn is_mark(tok: &Token) -> bool
{
    let TokenKind::Kw(kw) = tok.kind
    else
    {
        return matches!(tok.kind, TokenKind::Ident | TokenKind::RawIdent);
    };
    let closer =
        matches!(kw, Kw::Thi | Kw::Pa | Kw::Being | Kw::WhenDone | Kw::Accepting);
    (kw.is_particle() && !closer) || matches!(kw, Kw::At | Kw::Toward)
}

/// Whether the text of a mark string can be a mark: it reads as one token
/// that [`is_mark`] takes.
///
/// # Arguments
///
/// * `text` - The text inside the string.
///
/// # Returns
///
/// `true` for such a text.
pub(crate) fn is_mark_text(text: &str) -> bool
{
    let lexed = lex_at(text, Pos::default(), &LexerOptions::default());
    let [tok, end] = lexed.tokens.as_slice()
    else
    {
        return false;
    };
    lexed.diagnostics.is_empty() && end.kind == TokenKind::Eof && is_mark(tok)
}

/// Whether the scan for the end of a marked call stops at a token: the end
/// of the sentence, a `)` that closes no group of the call, or, outside
/// parentheses, a `၊` or a reserved word that is neither a mark nor a word
/// a notation argument can hold.
///
/// # Arguments
///
/// * `tok` - The token.
/// * `depth` - The number of groups of the call left open.
///
/// # Returns
///
/// `true` where the scan stops.
fn stops_marked_scan(tok: &Token, depth: usize) -> bool
{
    let deep = depth > 0;
    match tok.kind
    {
        TokenKind::Punct(Punct::FullStop) | TokenKind::Eof => true,
        TokenKind::Punct(Punct::RParen) => !deep,
        TokenKind::Punct(Punct::ListSep) => !deep,
        TokenKind::Kw(kw) => !deep && !is_mark(tok) && !in_notation(kw),
        _ => false,
    }
}

/// Whether a notation argument of a marked call can hold a reserved word:
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
#[derive(Debug)]
pub(crate) enum Head
{
    /// A value; the clause goes on from it.
    Expr(Expr),
    /// A call statement, its full stop included.
    Stmt(Stmt),
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

    /// How far ahead the end of a marked call is, when one starts `from`
    /// tokens ahead (the mark after the first argument): the end of a call of
    /// a function that declares marks. A value end with a mark after it can be
    /// an argument of a later call, so the scan goes on past it, and keeps it
    /// when no later end comes.
    ///
    /// # Arguments
    ///
    /// * `from` - Where the first mark is.
    ///
    /// # Returns
    ///
    /// The distance to the first token of the end, or `None` with no marked
    /// call here.
    pub(crate) fn marked_end(&self, from: usize) -> Option<usize>
    {
        ret_if!(!is_mark(&self.peek_n(from)), None);
        let (mut n, mut depth, mut found) = (from, 0usize, None);
        loop
        {
            let end = tern!(depth == 0, self.call_end_at(n), None);
            if let Some((len, value)) = end
            {
                ret_if!(!value || !is_mark(&self.peek_n(n + len)), Some(n));
                found = Some(n);
                n += len;
                continue;
            }
            let tok = self.peek_n(n);
            ret_if!(stops_marked_scan(&tok, depth), found);
            match tok.kind
            {
                TokenKind::Punct(Punct::LParen) => depth += 1,
                TokenKind::Punct(Punct::RParen) => depth -= 1,
                _ => (),
            }
            n += 1;
        }
    }

    /// The end of a call of a function that declares marks, `n` tokens
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
        ret_if!(!self.is_marked_fn(n), None);
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

    /// Whether the token `n` ahead names a function that declares marks.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` for such a name.
    fn is_marked_fn(&self, n: usize) -> bool
    {
        let tok = self.peek_n(n);
        tok.kind == TokenKind::Ident && self.marked.contains_key(self.text_of(tok.span))
    }

    /// Read a marked call after its first argument, and keep it when the
    /// number of its marks fits a function of its name; otherwise go back,
    /// so the sentence reads as it would with no marked call, such as
    /// `က ကို ခ ၌ရှိသော ဖတ် သည့် ရလဒ် ဟု ထားမည်။` with `ဖတ်` taking one mark.
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
    pub(crate) fn attempt_marked(
        &mut self, first: Expr, end: usize, mode: Mode,
    ) -> PResult<Head>
    {
        let (cur, diags, next_id) = (self.cur, self.diags.len(), self.next_id);
        let read = self.parse_marked(first.clone(), end, mode);
        ret_if!(read.as_ref().is_ok_and(|head| self.fits_marks(head)), read);
        self.cur = cur;
        self.diags.truncate(diags);
        self.next_id = next_id;
        Ok(Head::Expr(first))
    }

    /// Whether the number of marks of a marked call matches the parameters
    /// of a function of its name.
    ///
    /// # Arguments
    ///
    /// * `head` - The call read.
    ///
    /// # Returns
    ///
    /// `true` when the numbers match.
    fn fits_marks(&self, head: &Head) -> bool
    {
        let (callee, count) = match head
        {
            Head::Stmt(Stmt::Call(n)) => (&n.callee, n.marks.len()),
            Head::Expr(Expr::Call(n)) => (&n.callee, n.marks.len()),
            _ => return false,
        };
        self.marked
            .get(callee.name.as_str())
            .is_some_and(|counts| counts.contains(&count))
    }

    /// Parse the rest of a marked call after its first argument: the mark of
    /// each argument, the arguments after the first (notation expressions),
    /// then the function and the end, which [`Self::marked_end`] found.
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
    pub(crate) fn parse_marked(
        &mut self, first: Expr, end: usize, mode: Mode,
    ) -> PResult<Head>
    {
        let end_at = self.peek_n(end).span.lo;
        let start = first.span();
        let mut args = Args::default();
        args.push(first);
        let mut marks = vec![self.parse_mark()?];
        while self.peek().span.lo < end_at
        {
            let arg = self.parse_notation()?;
            extract!(Head::Expr(arg) = self.parse_short_after(arg, Mode::Expr)?);
            args.push(arg);
            marks.push(self.parse_mark()?);
        }
        let callee = self.parse_ident()?;
        let long = self.eat(Kw::FnValue);
        do_if!(long, _ = self.expect(Kw::Ko)?);
        #[rustfmt::skip]
        let parts = CallParts::builder().start(start).callee(callee).args(args).marks(marks)
            .build();
        self.finish_using(parts, long, mode)
    }

    /// Read a mark: a name, or a particle a mark can be.
    ///
    /// # Returns
    ///
    /// The mark as a name, or an `expected-token` diagnostic.
    pub(crate) fn parse_mark(&mut self) -> PResult<Ident>
    {
        let tok = self.peek();
        ret_if!(!is_mark(&tok), Err(self.err_expected(&t!("parser.mark"))));
        self.bump();
        let text = unquote_raw(tok.kind == TokenKind::RawIdent, self.text_of(tok.span));
        Ok(Ident::builder().name(text).span(tok.span).build())
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
        let using = UsingMark::builder().plural(plural).span(close.span).build();
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
            self.finish_call_stmt(parts).map(Head::Stmt)
        );
        let pass = matches!(kw, Some(Kw::CallGoodResult | Kw::UsedGoodResult));
        let CallParts { start, callee, place, args, using, marks } = parts;
        let call = self.call_expr(start.to(&end.span), callee, args, pass);
        let mut call = crate::notation::with_place(call, place);
        extract!(Expr::Call(node) = &mut call);
        node.using = using;
        node.marks = marks;
        Ok(Head::Expr(call))
    }
}
