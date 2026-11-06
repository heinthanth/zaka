//! The arguments-first call, `<args> (တို့)?ကို သုံး၍ ...`, and the short call
//! with no arguments, `<fn> မည်` and `<fn> သည့် ရလဒ်`.
//!
//! Each form ends as a statement or as a value. The long form names the
//! function as `လုပ်ဆောင်ချက် <fn> ကို`, then ends with `လုပ်ဆောင်မည်` (a statement) or
//! `လုပ်ဆောင်သော ရလဒ်(ကောင်း)?` (a value). The short form names the function
//! alone, then ends with `မည်` (a statement) or `သည့် ရလဒ်(ကောင်း)?` (a value).
//! Each builds the call node that the other call forms build.

use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::UsingMark;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::clause::Mode;
use crate::notation::Args;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::kw_of;

/// What a call holds once its function is read.
#[derive(Builder)]
struct CallParts
{
    /// Where the call starts.
    start: Span,
    /// The function name.
    callee: Ident,
    /// The arguments, with the mark of each.
    args: Args,
    /// The particle that closes the arguments of a call written arguments
    /// first.
    using: Option<UsingMark>,
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
    /// argument: `ကို` or `တို့ကို`, then `သုံး၍`; or `၊` with those ahead,
    /// outside parentheses. The mark `(လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)` can
    /// stand after the argument.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_using_rest(&self) -> bool
    {
        let n = self.mark_len();
        ret_true!(self.at_using_close(n));
        self.peek_n(n).kind == TokenKind::Punct(Punct::ListSep)
            && self.close_ahead(n + 1, Self::at_using_close)
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

    /// The number of tokens of the mark
    /// `(လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)` when it comes next.
    ///
    /// # Returns
    ///
    /// 3 for the mark, else 0.
    fn mark_len(&self) -> usize
    {
        let open = self.at_punct(Punct::LParen);
        tern!(open && kw_of(&self.peek_n(1)) == Some(Kw::EditArg), 3, 0)
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

    /// Parse the rest of an arguments-first call after its first argument:
    /// `(၊ <arg>)* (တို့)?ကို သုံး၍`, then the function and the end. Each
    /// argument is a notation expression, so a sentence that gives a value
    /// goes inside parentheses.
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
        let mut args = Args::default();
        args.0.push(first);
        args.1.push(self.parse_edit_mark()?);
        while self.eat_punct(Punct::ListSep)
        {
            let arg = self.parse_notation()?;
            extract!(Head::Expr(arg) = self.parse_short_after(arg, Mode::Expr)?);
            args.0.push(arg);
            args.1.push(self.parse_edit_mark()?);
        }
        let close = self.expect_one_of(&[Kw::Ko, Kw::ListKo])?;
        let plural = close.kind == TokenKind::Kw(Kw::ListKo);
        let using = UsingMark::builder().plural(plural).span(close.span).build();
        self.expect(Kw::Using)?;
        let long = self.eat(Kw::Function);
        let callee = self.parse_ident()?;
        do_if!(long, _ = self.expect(Kw::Ko)?);
        let parts =
            CallParts::builder().start(start).callee(callee).args(args).using(using);
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
    /// * `long` - Whether the function came as `လုပ်ဆောင်ချက် <fn> ကို`.
    /// * `mode` - Whether the call can be a statement.
    ///
    /// # Returns
    ///
    /// The call, or a diagnostic.
    fn finish_using(
        &mut self, parts: CallParts, long: bool, mode: Mode,
    ) -> PResult<Head>
    {
        let CallParts { start, callee, args, using } = parts;
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
            self.finish_call_stmt(start, callee, args, using).map(Head::Stmt)
        );
        let pass = matches!(kw, Some(Kw::CallGoodResult | Kw::UsedGoodResult));
        let mut call = self.call_expr(start.to(&end.span), callee, args, pass);
        extract!(Expr::Call(node) = &mut call);
        node.using = using;
        Ok(Head::Expr(call))
    }
}
