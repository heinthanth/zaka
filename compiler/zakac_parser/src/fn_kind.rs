//! Function kinds: `<params> လက်ခံပြီး <kind> တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက်
//! အမျိုးအစား`, `<params> လက်ခံသော …`, `<kind> တစ်ခု ကို ပြန်ပေးသော …`, and
//! `လုပ်ဆောင်ချက် အမျိုးအစား` alone. Each parameter is
//! `<kind> တစ်<classifier> ကို [prepositions] [လည်းကောင်း]`, with no name.
//!
//! In a list of parameters or fields the parameters of the kind would run
//! into the items around it, so a function kind with parameters stands bare
//! where no list is open (a declaration, a result, a conversion), and in
//! parentheses inside a list, inside another function kind among them. A
//! kind with no parameters never needs them.

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

use crate::describe::kw_text;
use crate::describe::quote;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::kw_of;

/// The parameters and the result kind of a function kind.
type FnParts = (Vec<SigParam>, Option<Kind>);

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

    /// The token count of `<kind> တစ်<classifier> ကို ပြန်ပေးသော လုပ်ဆောင်ချက်
    /// အမျိုးအစား` that starts `at` tokens ahead.
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
        let n = self.slot_kind_len_at(at, FnForm::NoFn)?;
        let tail = [Kw::Ko, Kw::ReturningRel, Kw::FnKind];
        let ends =
            (0..tail.len()).all(|i| kw_of(&self.peek_n(at + n + i)) == Some(tail[i]));
        ends.then_some(n + tail.len())
    }

    /// The token count of the parameters of a function kind that start `at`
    /// tokens ahead, up to `လက်ခံပြီး` or `လက်ခံသော`: items that each read
    /// `<kind> တစ်<classifier> ကို [ဒေတာ ပြောင်းလဲရန် | ဒေတာ ထည့်ပေးရန်]
    /// [အပိုင်အနေဖြင့်] [ဝိဘတ် "<preposition>" ကို သုံး၍] [လည်းကောင်း]`. An item with no
    /// `လည်းကောင်း` ends the list.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None`.
    fn sig_params_len(&self, at: usize) -> Option<usize>
    {
        let mut n = 0;
        loop
        {
            let (len, more) = self.sig_item_len(at + n)?;
            n += len;
            let end = kw_of(&self.peek_n(at + n));
            ret_if!(matches!(end, Some(Kw::Accepting | Kw::AcceptingRel)), Some(n));
            ret_if!(!more, None);
        }
    }

    /// The token count of one parameter of a function kind that starts `at`
    /// tokens ahead, and whether `လည်းကောင်း` closes it.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count and the closer, or `None`.
    fn sig_item_len(&self, at: usize) -> Option<(usize, bool)>
    {
        let is = |n: usize, kw: Kw| kw_of(&self.peek_n(at + n)) == Some(kw);
        let mut n = self.slot_kind_len_at(at, FnForm::NoParams)?;
        ret_if!(!is(n, Kw::Ko), None);
        n += 1;
        n += self.markers_len(at + n);
        let more = is(n, Kw::AsWellAs);
        Some((n + usize::from(more), more))
    }

    /// The token count of the prepositions of a parameter that start `at`
    /// tokens ahead, after its `ကို`.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count; 0 for none.
    pub(crate) fn markers_len(&self, at: usize) -> usize
    {
        let is = |n: usize, kw: Kw| kw_of(&self.peek_n(at + n)) == Some(kw);
        let mut n = usize::from(is(0, Kw::DataToChange) || is(0, Kw::DataToFill));
        n += usize::from(is(n, Kw::AsOwned));
        // A preposition is four tokens: `ဝိဘတ်`, the string, `ကို` and `သုံး၍`.
        n + tern!(is(n, Kw::PrepositionWord), 4, 0)
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
        // Each function kind ends in `လုပ်ဆောင်ချက် အမျိုးအစား`, so a group whose
        // last word is another is no kind, and its inside goes unscanned: a
        // scan of each nested group would cost more at each level.
        let close = self.close_paren_at(at)?;
        ret_if!(kw_of(&self.peek_n(close - 1)) != Some(Kw::FnKind), None);
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
    /// The parameters and the result kind with its measure word, or a
    /// diagnostic.
    fn parse_fn_parts(&mut self, with_params: bool) -> PResult<FnParts>
    {
        ret_if!(self.eat(Kw::FnKind), Ok((Vec::new(), None)));
        let params = tern!(with_params, self.parse_sig_params()?, Vec::new());
        if with_params && self.eat(Kw::AcceptingRel)
        {
            self.expect(Kw::FnKind)?;
            return Ok((params, None));
        }
        do_if!(with_params, _ = self.expect(Kw::Accepting)?);
        let ret = self.parse_slot_kind_with(FnForm::NoFn)?;
        self.expect(Kw::Ko)?;
        self.expect(Kw::ReturningRel)?;
        self.expect(Kw::FnKind)?;
        Ok((params, Some(ret)))
    }

    /// Parse the parameters of a function kind, up to the `လက်ခံ…` after
    /// them. With more than one, each ends in `လည်းကောင်း`; either each has a
    /// preposition or none has.
    ///
    /// # Returns
    ///
    /// The parameters, or a diagnostic.
    fn parse_sig_params(&mut self) -> PResult<Vec<SigParam>>
    {
        let mut params = Vec::new();
        let mut closed = Vec::new();
        loop
        {
            params.push(self.parse_sig_item()?);
            closed.push(self.eat(Kw::AsWellAs));
            do_if!(self.at(Kw::Accepting) || self.at(Kw::AcceptingRel), break);
            ret_if!(!closed[closed.len() - 1], Err(self.err_as_well_as()));
        }
        let open = params.len() > 1 && closed.contains(&false);
        ret_if!(open, Err(self.err_as_well_as()));
        let given = params.iter().filter(|p| p.preposition.is_some()).count();
        let span = params[0].span.to(&params[params.len() - 1].span);
        do_if!(
            given > 0 && given < params.len(),
            self.diags.push(diag::mixed_prepositions(span))
        );
        Ok(params)
    }

    /// The report of a parameter in a list of several that `လည်းကောင်း` does
    /// not close.
    ///
    /// # Returns
    ///
    /// The `expected-token` diagnostic.
    pub(crate) fn err_as_well_as(&self) -> zakac_diagnostics::Diagnostic
    {
        self.err_expected(&quote(&kw_text(Kw::AsWellAs)))
    }

    /// Parse one parameter of a function kind: its kind with the measure
    /// word, `ကို` and its marks.
    ///
    /// # Returns
    ///
    /// The parameter, or a diagnostic.
    fn parse_sig_item(&mut self) -> PResult<SigParam>
    {
        let start = self.peek().span;
        let kind = self.parse_slot_kind_with(FnForm::NoParams)?;
        self.expect(Kw::Ko)?;
        let fill = self.peek_n(0).kind == TokenKind::Kw(Kw::DataToFill);
        let (owned, edit, _, preposition) = self.parse_item_markers(fill)?;
        do_if!(fill, self.diags.push(diag::fill_in_fn_kind(self.prev_span())));
        let span = start.to(&self.prev_span());
        #[rustfmt::skip]
        return Ok(SigParam::builder().owned(owned).edit(edit).kind(kind).maybe_preposition(preposition)
            .span(span).build());
    }

    /// Register the name of a parameter or a variable whose kind is a
    /// function kind with prepositions, so a preposition call through it can be
    /// read.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `kind` - Its kind.
    pub(crate) fn note_preposition_value(&mut self, name: &str, kind: &Kind)
    {
        let Some(sig) = kind.sig.as_deref().filter(|_| kind.wraps.is_empty())
        else
        {
            return;
        };
        let prepositions = sig.params.iter().filter_map(|p| p.preposition.as_ref());
        let prepositions: Vec<String> =
            prepositions.map(|m| m.name.to_string()).collect();
        ret_if!(prepositions.is_empty());
        self.add_function(name, vec![prepositions]);
    }

    /// Register a variable that starts as the address of a function with
    /// prepositions, `ကိန်းရှင် <name> သည် <fn> ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။`, with the
    /// prepositions of that function, so a preposition call through it can be
    /// read.
    ///
    /// # Arguments
    ///
    /// * `name` - The variable.
    /// * `init` - Its starting value.
    pub(crate) fn note_preposition_addr(&mut self, name: &str, init: &Expr)
    {
        let Expr::FnAddr(addr) = init
        else
        {
            return;
        };
        let lists = self.functions.get(addr.name.name.as_str()).cloned();
        do_if!(let Some(lists) = lists, self.add_function(name, lists));
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
        let kind = self.parse_slot_kind();
        self.in_list = outer;
        let kind = kind?;
        self.diags.push(diag::fn_kind_needs_parens(kind.span));
        Ok(kind)
    }
}
