//! Function kinds: `<params> လက်ခံပြီး <kind> ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား`,
//! `<params> လက်ခံသော …`, `<kind> ရလဒ် ကို ပြန်ပေးသော …`, and
//! `လုပ်ဆောင်ချက် အမျိုးအစား` alone.
//!
//! The parameters are those of a header, each name optional, so in a list of
//! parameters or fields the parameters of the kind would run into the items
//! before it. A
//! function kind with parameters therefore stands bare where no list is open
//! (a declaration, a result, a conversion), and in parentheses inside a list,
//! inside another function kind among them. A kind with no parameters never
//! needs them.

use zakac_ast::expr::Expr;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::FnSig;
use zakac_ast::kind::Kind;
use zakac_ast::kind::SigParam;
use zakac_diagnostics::parser as diag;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::kw_of;

/// Which bare function kinds a kind phrase can be at its first token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FnForm
{
    /// Each form, a parameter list among them: no list is open.
    Full,
    /// The forms with no parameters: a list of parameters or fields is
    /// open, so a kind with parameters stands in parentheses.
    NoParams,
    /// `လုပ်ဆောင်ချက် အမျိုးအစား` alone: the result of a function kind, which
    /// starts where the kind around it does.
    NoFn,
}

impl Parser<'_>
{
    /// The bare function kinds a kind phrase can be here.
    ///
    /// # Returns
    ///
    /// [`FnForm::NoParams`] while a list is open, else [`FnForm::Full`].
    pub(crate) fn fn_form(&self) -> FnForm
    {
        tern!(self.in_list, FnForm::NoParams, FnForm::Full)
    }

    /// The token count of a bare function kind of `form` that starts `at`
    /// tokens ahead, its pointer and list levels left out.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    /// * `form` - The forms allowed.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when none starts there.
    pub(crate) fn fn_head_at(&self, at: usize, form: FnForm) -> Option<usize>
    {
        let closer = kw_of(&self.peek_n(at)) == Some(Kw::FnKind);
        ret_if!(closer, Some(1));
        ret_if!(form == FnForm::NoFn, None);
        let listed = tern!(form == FnForm::Full, self.listed_head_len(at), None);
        listed.or_else(|| self.ret_tail_len(at))
    }

    /// The token count of a function kind with parameters that starts `at`
    /// tokens ahead: the parameters, then `လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား` or
    /// `လက်ခံပြီး` and the result.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None`.
    fn listed_head_len(&self, at: usize) -> Option<usize>
    {
        let n = self.sig_params_len(at)?;
        match kw_of(&self.peek_n(at + n))?
        {
            Kw::AcceptingRel =>
            {
                let closed = kw_of(&self.peek_n(at + n + 1)) == Some(Kw::FnKind);
                closed.then_some(n + 2)
            }
            Kw::Accepting => self.ret_tail_len(at + n + 1).map(|r| n + 1 + r),
            _ => None,
        }
    }

    /// The token count of `<kind> ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား`
    /// that starts `at` tokens ahead. A result kind writes no `ရလဒ်` of its
    /// own, as in a header.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None`.
    fn ret_tail_len(&self, at: usize) -> Option<usize>
    {
        let kind = self.kind_len_at(at, FnForm::NoFn)?;
        let n = kind + usize::from(kw_of(&self.peek_n(at + kind)) == Some(Kw::Result));
        let tail = [Kw::Ko, Kw::ReturningRel, Kw::FnKind];
        let ends =
            (0..tail.len()).all(|i| kw_of(&self.peek_n(at + n + i)) == Some(tail[i]));
        ends.then_some(n + tail.len())
    }

    /// The token count of the parameters of a function kind that start `at`
    /// tokens ahead, their closer `ကို` or `တို့ကို` included; with marks no
    /// closer comes.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no parameter list starts there.
    fn sig_params_len(&self, at: usize) -> Option<usize>
    {
        let (mut n, mut marked) = (0, false);
        loop
        {
            let (len, mark) = self.sig_param_len(at + n)?;
            n += len;
            marked |= mark;
            let sep = self.peek_n(at + n).kind;
            match sep
            {
                TokenKind::Punct(Punct::ListSep) => n += 1,
                TokenKind::Kw(Kw::Hnint) =>
                {
                    let (len, mark) = self.sig_param_len(at + n + 1)?;
                    n += 1 + len;
                    marked |= mark;
                    break;
                }
                _ => break,
            }
        }
        let end = kw_of(&self.peek_n(at + n));
        let closer = matches!(end, Some(Kw::Ko | Kw::ListKo));
        let open = matches!(end, Some(Kw::Accepting | Kw::AcceptingRel));
        ret_if!(closer, Some(n + 1));
        (marked && open).then_some(n)
    }

    /// The token count of one parameter of a function kind that starts `at`
    /// tokens ahead: the edit mark, the kind, the name and its mark.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count and whether a mark came, or `None`.
    fn sig_param_len(&self, at: usize) -> Option<(usize, bool)>
    {
        let edit = usize::from(kw_of(&self.peek_n(at)) == Some(Kw::EditParam));
        let kind = self.kind_len_at(at + edit, FnForm::NoParams)?;
        let n = edit + kind;
        let named =
            matches!(self.peek_n(at + n).kind, TokenKind::Ident | TokenKind::RawIdent);
        ret_if!(!named, Some((n, false)));
        let open = self.peek_n(at + n + 1);
        let mark = open.kind == TokenKind::Punct(Punct::LParen) && open.joined;
        // A mark is one word in parentheses: `(<mark>)`.
        Some(tern!(mark, (n + 4, true), (n + 1, false)))
    }

    /// The token count of a function kind in parentheses that starts `at`
    /// tokens ahead, the parentheses included.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None`.
    pub(crate) fn paren_fn_len(&self, at: usize) -> Option<usize>
    {
        ret_if!(self.peek_n(at).kind != TokenKind::Punct(Punct::LParen), None);
        let inner = self.fn_head_at(at + 1, FnForm::Full)?;
        let close = self.peek_n(at + 1 + inner).kind == TokenKind::Punct(Punct::RParen);
        close.then_some(inner + 2)
    }

    /// Whether a function kind with parameters stands bare at the current
    /// token while a list is open, so it needs parentheses.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_bare_listed_kind(&self) -> bool
    {
        self.in_list && self.listed_head_len(0).is_some()
    }

    /// Parse a function kind with no levels, after the borrow marks, the
    /// count prefixes and the `(` before it: the parameters and the result
    /// up to `လုပ်ဆောင်ချက် အမျိုးအစား`. The kinds inside it are read with a
    /// list open.
    ///
    /// # Arguments
    ///
    /// * `form` - The forms allowed here.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic.
    pub(crate) fn parse_fn_head(&mut self, form: FnForm) -> PResult<Kind>
    {
        let start = self.peek().span;
        let with_params = form == FnForm::Full && self.listed_head_len(0).is_some();
        let outer = std::mem::replace(&mut self.in_list, true);
        let parts = self.parse_fn_parts(with_params);
        self.in_list = outer;
        let (params, ret) = parts?;
        let sig = FnSig::builder().params(params).maybe_ret(ret).build();
        let span = start.to(&self.prev_span());
        Ok(Kind::builder().base(BaseKind::Fn).sig(Box::new(sig)).span(span).build())
    }

    /// Parse the parameters and the result of a function kind, and the
    /// `လုပ်ဆောင်ချက် အမျိုးအစား` that closes it.
    ///
    /// # Arguments
    ///
    /// * `with_params` - Whether a parameter list comes first.
    ///
    /// # Returns
    ///
    /// The parameters and the result kind, or a diagnostic.
    fn parse_fn_parts(
        &mut self, with_params: bool,
    ) -> PResult<(Vec<SigParam>, Option<Kind>)>
    {
        ret_if!(self.eat(Kw::FnKind), Ok((Vec::new(), None)));
        let params = tern!(with_params, self.parse_sig_params()?, Vec::new());
        if with_params && self.eat(Kw::AcceptingRel)
        {
            self.expect(Kw::FnKind)?;
            return Ok((params, None));
        }
        do_if!(with_params, _ = self.expect(Kw::Accepting)?);
        let ret = self.parse_kind_with(FnForm::NoFn)?;
        do_if!(ret.base != BaseKind::Result, _ = self.expect(Kw::Result)?);
        self.expect(Kw::Ko)?;
        self.expect(Kw::ReturningRel)?;
        self.expect(Kw::FnKind)?;
        Ok((params, Some(ret)))
    }

    /// Parse the parameters of a function kind:
    /// `<param> (၊ <param>)* (နှင့် <param>)? (တို့ကို|ကို)`, with no closer
    /// when the parameters have marks.
    ///
    /// # Returns
    ///
    /// The parameters, or a diagnostic.
    fn parse_sig_params(&mut self) -> PResult<Vec<SigParam>>
    {
        let mut params = vec![self.parse_sig_param()?];
        while self.eat_punct(Punct::ListSep)
        {
            params.push(self.parse_sig_param()?);
        }
        do_if!(self.eat(Kw::Hnint), params.push(self.parse_sig_param()?));
        let marked = params.iter().filter(|p| p.mark.is_some()).count();
        let span = params[0].span.to(&params[params.len() - 1].span);
        do_if!(
            marked > 0 && marked < params.len(),
            self.diags.push(diag::mixed_marks(span))
        );
        let open = self.at(Kw::Accepting) || self.at(Kw::AcceptingRel);
        do_if!(marked == 0 || !open, _ = self.expect_one_of(&[Kw::ListKo, Kw::Ko])?);
        Ok(params)
    }

    /// Parse one parameter of a function kind: the edit mark, the kind, and
    /// the name and its mark when written.
    ///
    /// # Returns
    ///
    /// The parameter, or a diagnostic.
    fn parse_sig_param(&mut self) -> PResult<SigParam>
    {
        let start = self.peek().span;
        let edit = self.eat(Kw::EditParam);
        let kind = self.parse_kind_with(FnForm::NoParams)?;
        let named = matches!(self.peek().kind, TokenKind::Ident | TokenKind::RawIdent);
        let name = tern!(named, Some(self.parse_ident()?), None);
        let open = self.peek();
        let marked =
            named && open.kind == TokenKind::Punct(Punct::LParen) && open.joined;
        let mark = tern!(marked, Some(self.parse_param_mark()?), None);
        let span = start.to(&self.prev_span());
        #[rustfmt::skip]
        return Ok(SigParam::builder().edit(edit).kind(kind).maybe_name(name).maybe_mark(mark).span(span).build());
    }

    /// Register the name of a parameter or a variable whose kind is a
    /// function kind with marks, so a marked call through it can be read.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `kind` - Its kind.
    pub(crate) fn note_marked_value(&mut self, name: &str, kind: &Kind)
    {
        let Some(sig) = kind.sig.as_deref().filter(|_| kind.wraps.is_empty())
        else
        {
            return;
        };
        let count = sig.params.iter().filter(|p| p.mark.is_some()).count();
        ret_if!(count == 0);
        self.marked.entry(name.to_string()).or_default().push(count);
    }

    /// Register a variable that starts as the address of a function with
    /// marks, `ကိန်းရှင် <name> သည် လုပ်ဆောင်ချက် <fn> ၏ လိပ်စာ ဖြစ်ပါစေ။`, with the
    /// marks of that function, so a marked call through it can be read.
    ///
    /// # Arguments
    ///
    /// * `name` - The variable.
    /// * `init` - Its starting value.
    pub(crate) fn note_marked_addr(&mut self, name: &str, init: &Expr)
    {
        let Expr::FnAddr(addr) = init
        else
        {
            return;
        };
        let counts = self.marked.get(addr.name.name.as_str()).cloned();
        do_if!(let Some(counts) = counts, self.marked.entry(name.to_string()).or_default().extend(counts));
    }

    /// Report a function kind with parameters that stands bare while a list
    /// is open, then read it as if no list were open.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic.
    pub(crate) fn parse_bare_listed_kind(&mut self) -> PResult<Kind>
    {
        let outer = std::mem::replace(&mut self.in_list, false);
        let kind = self.parse_kind();
        self.in_list = outer;
        let kind = kind?;
        self.diags.push(diag::fn_kind_needs_parens(kind.span));
        Ok(kind)
    }
}
