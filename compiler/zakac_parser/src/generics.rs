use zakac_ast::expr::Expr;
use zakac_ast::expr::FnAddrExpr;
use zakac_ast::kind::Kind;
use zakac_ast::stmt::KindParam;
use zakac_ast::stmt::Stmt;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::describe::kw_text;
use crate::describe::quote;
use crate::fn_kind::FnForm;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::kw_of;
use crate::using::Head;

/// The number of `<…>` groups a token closes: one for `>`, two for `>>`.
///
/// # Arguments
///
/// * `kind` - The token kind.
///
/// # Returns
///
/// The count; 0 for each other token.
fn closers(kind: TokenKind) -> usize
{
    match kind
    {
        TokenKind::Punct(Punct::Gt) => 1,
        TokenKind::Punct(Punct::Shr) => 2,
        _ => 0,
    }
}

/// The token count of a `<â¦>` group that starts `at` tokens ahead, both
/// marks included; a `>>` that closes it and the group around it counts for
/// both.
///
/// # Arguments
///
/// * `at` - Where the `<` stands.
/// * `kind_at` - The kind of the token at a place.
///
/// # Returns
///
/// The token count, or `None` when no group starts there or the sentence
/// ends first.
pub(crate) fn angle_len_with(
    at: usize, kind_at: impl Fn(usize) -> TokenKind,
) -> Option<usize>
{
    ret_if!(kind_at(at) != TokenKind::Punct(Punct::Lt), None);
    let mut depth = 0usize;
    for n in at..
    {
        let kind = kind_at(n);
        let ends = matches!(kind, TokenKind::Eof | TokenKind::Punct(Punct::FullStop));
        ret_if!(ends, None);
        if kind == TokenKind::Punct(Punct::Lt)
        {
            depth += 1;
            continue;
        }
        let shut = closers(kind);
        ret_if!(shut > 0 && shut >= depth, Some(n + 1 - at));
        depth -= shut;
    }
    None
}

impl Parser<'_>
{
    /// The token count of a `<…>` group that starts `at` tokens ahead, both
    /// marks included; a `>>` that closes it and the group around it counts
    /// for both.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead the `<` stands.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no group starts there or the sentence
    /// ends first.
    pub(crate) fn angle_len(&self, at: usize) -> Option<usize>
    {
        angle_len_with(at, |n| self.peek_n(n).kind)
    }

    /// Move past the mark that closes a `<…>` group: `>`, or one half of a
    /// `>>`, whose second half closes the group around it.
    ///
    /// # Returns
    ///
    /// `Ok`, or an `expected-token` diagnostic.
    fn expect_angle_close(&mut self) -> PResult<()>
    {
        if self.half_close
        {
            self.half_close = false;
            self.advance();
            return Ok(());
        }
        if self.at_punct(Punct::Shr)
        {
            self.half_close = true;
            return Ok(());
        }
        self.expect_punct(Punct::Gt)?;
        Ok(())
    }

    /// Parse the kind parameters of a declaration in the notation form,
    /// `<T၊ U: <trait> နှင့် <trait>>`, after its name; nothing when no `<`
    /// follows the name.
    ///
    /// # Returns
    ///
    /// The parameters, or a diagnostic.
    pub(crate) fn parse_notation_params(&mut self) -> PResult<Vec<KindParam>>
    {
        ret_if!(!self.at_punct(Punct::Lt), Ok(Vec::new()));
        self.advance();
        let mut params = vec![self.parse_kind_param()?];
        while self.eat_punct(Punct::ListSep)
        {
            params.push(self.parse_kind_param()?);
        }
        self.expect_angle_close()?;
        Ok(params)
    }

    /// Parse one kind parameter in the notation form: its name, and after a
    /// `:` the traits it must have, with `နှင့်` between them.
    ///
    /// # Returns
    ///
    /// The parameter, or a diagnostic.
    fn parse_kind_param(&mut self) -> PResult<KindParam>
    {
        let name = self.parse_ident()?;
        let mut bounds = Vec::new();
        if self.eat_punct(Punct::Colon)
        {
            bounds.push(self.parse_trait_ref()?);
            while self.eat(Kw::Hnint)
            {
                bounds.push(self.parse_trait_ref()?);
            }
        }
        let span = name.span.to(&self.prev_span());
        Ok(KindParam::builder().name(name).bounds(bounds).span(span).build())
    }

    /// Parse the kind parameters of a declaration in the word form: each
    /// `ယေဘုယျဒေတာအမျိုးအစား <name>` as an item of a list in words, then `ကို`
    /// or `တို့ကို`, `အစားထိုးသုံးနိုင်၍` and `၎င်း`; nothing when the list does
    /// not start here.
    ///
    /// # Returns
    ///
    /// The parameters, or a diagnostic.
    pub(crate) fn parse_word_params(&mut self) -> PResult<Vec<KindParam>>
    {
        ret_if!(!self.at(Kw::GenericKind), Ok(Vec::new()));
        let params = self.parse_word_list(|p| {
            let start = p.expect(Kw::GenericKind)?.span;
            let name = p.parse_ident()?;
            let span = start.to(&name.span);
            Ok(KindParam::builder().name(name).span(span).build())
        })?;
        self.expect_list_word(params.len(), Kw::Ko, Kw::ListKo)?;
        self.expect(Kw::Substitutable)?;
        self.expect(Kw::Itself)?;
        Ok(params)
    }

    /// Parse the kind arguments in the notation form, `<<kind>၊ …>`. The
    /// kinds take no measure word; a word form with a list of its own stands
    /// in parentheses.
    ///
    /// # Returns
    ///
    /// The kinds, or a diagnostic.
    pub(crate) fn parse_sign_kind_args(&mut self) -> PResult<Vec<Kind>>
    {
        self.expect_punct(Punct::Lt)?;
        let mut args = vec![self.parse_kind_arg()?];
        while !self.half_close && self.eat_punct(Punct::ListSep)
        {
            args.push(self.parse_kind_arg()?);
        }
        self.expect_angle_close()?;
        Ok(args)
    }

    /// Parse one kind argument: a kind, or a kind in parentheses.
    ///
    /// # Returns
    ///
    /// The kind, or a diagnostic.
    fn parse_kind_arg(&mut self) -> PResult<Kind>
    {
        let grouped = self.at_punct(Punct::LParen) && self.paren_fn_len(0).is_none();
        ret_if!(!grouped, self.outside_word_list(|p| p.parse_kind_with(FnForm::Full)));
        self.advance();
        let kind = self.outside_word_list(|p| p.parse_kind_with(FnForm::Full))?;
        self.expect_punct(Punct::RParen)?;
        Ok(kind)
    }

    /// The token count of the kind arguments in the word form that start `at`
    /// tokens ahead: kinds as a list in words, then `ကို` or `တို့ကို`, then
    /// `closer`. A kind in the list is no word form itself.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    /// * `closer` - `အစားထိုးအသုံးပြုသည့်` or `အစားထိုးအသုံးပြု၍`.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no such list starts there.
    pub(crate) fn word_kind_args_len(&self, at: usize, closer: Kw) -> Option<usize>
    {
        ret_if!(self.in_kind_args || self.scanning_kind_args.get(), None);
        self.scanning_kind_args.set(true);
        let len = self.word_kind_args_scan(at, closer);
        self.scanning_kind_args.set(false);
        len
    }

    /// The scan of [`Parser::word_kind_args_len`], with the word forms of
    /// the kinds in the list turned off.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    /// * `closer` - The word after `ကို` or `တို့ကို`.
    ///
    /// # Returns
    ///
    /// The token count, or `None`.
    fn word_kind_args_scan(&self, at: usize, closer: Kw) -> Option<usize>
    {
        let mut n = at;
        loop
        {
            n += self.kind_len_at(n, FnForm::NoParams)?;
            match self.peek_n(n).kind
            {
                TokenKind::Punct(Punct::ListSep) | TokenKind::Kw(Kw::Hnint) => n += 1,
                TokenKind::Kw(Kw::Ko | Kw::ListKo)
                    if kw_of(&self.peek_n(n + 1)) == Some(closer) =>
                {
                    return Some(n + 2 - at);
                }
                _ => return None,
            }
        }
    }

    /// The token count of a generic kind in the word form that starts `at`
    /// tokens ahead: its kind arguments, then a struct name and the levels
    /// after it.
    ///
    /// # Arguments
    ///
    /// * `at` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when none starts there.
    pub(crate) fn word_generic_kind_len(&self, at: usize) -> Option<usize>
    {
        let args = self.word_kind_args_len(at, Kw::SubstitutedRel)?;
        ret_if!(!self.is_struct_at(at + args), None);
        let (levels, _) = self.scan_suffixes(at + args + 1, true);
        Some(args + 1 + levels)
    }

    /// Parse the kind arguments in the word form: the kinds as a list in
    /// words, then `ကို` or `တို့ကို`, then `closer`.
    ///
    /// # Arguments
    ///
    /// * `closer` - `အစားထိုးအသုံးပြုသည့်` or `အစားထိုးအသုံးပြု၍`.
    ///
    /// # Returns
    ///
    /// The kinds, or a diagnostic.
    pub(crate) fn parse_word_kind_args(&mut self, closer: Kw) -> PResult<Vec<Kind>>
    {
        let outer = std::mem::replace(&mut self.in_kind_args, true);
        let args = self.parse_word_list(|p| p.parse_kind_with(FnForm::NoParams));
        self.in_kind_args = outer;
        let args = args?;
        self.expect_list_word(args.len(), Kw::Ko, Kw::ListKo)?;
        self.expect(closer)?;
        Ok(args)
    }

    /// Make the kind parameters of a declaration names of kinds while its
    /// parts are parsed, as struct names are.
    ///
    /// # Arguments
    ///
    /// * `params` - The kind parameters.
    ///
    /// # Returns
    ///
    /// The names added, for [`Parser::drop_kind_names`].
    pub(crate) fn add_kind_names(&mut self, params: &[KindParam]) -> Vec<String>
    {
        let names = params.iter().map(|p| p.name.name.to_string());
        let fresh: Vec<String> = names.filter(|n| !self.structs.contains(n)).collect();
        self.structs.extend(fresh.iter().cloned());
        fresh
    }

    /// Take back the names [`Parser::add_kind_names`] added.
    ///
    /// # Arguments
    ///
    /// * `names` - The names.
    pub(crate) fn drop_kind_names(&mut self, names: Vec<String>)
    {
        names.iter().for_each(|n| _ = self.structs.remove(n));
    }

    /// Whether the kind arguments of a call or a function value in the
    /// notation form start here: a name, then `::<`.
    ///
    /// # Returns
    ///
    /// `true` when the kind arguments start here.
    pub(crate) fn at_turbofish(&self) -> bool
    {
        let named = matches!(self.peek().kind, TokenKind::Ident | TokenKind::RawIdent);
        let sep = self.peek_n(1).kind == TokenKind::Punct(Punct::PathSep);
        named && sep && self.peek_n(2).kind == TokenKind::Punct(Punct::Lt)
    }
}

impl Parser<'_>
{
    /// Parse a name with its kind arguments in the notation form, `f::<A၊ B>`:
    /// a call when a `(` touches the `>`, a struct literal when the name is a
    /// struct's or a field and `:` follow, else a function value.
    ///
    /// # Returns
    ///
    /// The expression, or a diagnostic.
    pub(crate) fn parse_turbofish(&mut self) -> PResult<Expr>
    {
        let name = self.parse_ident()?;
        self.advance();
        let kind_args = self.parse_sign_kind_args()?;
        let open = self.peek();
        let called = open.kind == TokenKind::Punct(Punct::LParen) && open.joined;
        if !called
        {
            let span = name.span.to(&self.prev_span());
            let id = self.new_id();
            #[rustfmt::skip]
            let value = FnAddrExpr::builder().id(id).name(name).kind_args(kind_args).span(span);
            return Ok(Expr::FnAddr(value.build()));
        }
        let field = self.peek_n(2).kind == TokenKind::Punct(Punct::Colon);
        let lit = field || self.structs.contains(name.name.as_str());
        ret_if!(lit, self.call_struct_lit_rest(name, kind_args));
        let call = self.parse_sign_tail(name.clone(), None)?;
        let mut call = with_kind_args(call, kind_args, None);
        while self.at_sign_open()
        {
            call = self.parse_sign_tail(name.clone(), Some(Box::new(call)))?;
        }
        Ok(call)
    }

    /// The word that closes kind arguments in the word form standing before a
    /// call or a function value here: `အစားထိုးအသုံးပြုသည့်` before
    /// `<name> ဟူသော လုပ်ဆောင်ချက်`, or `အစားထိုးအသုံးပြု၍` before the arguments of
    /// a call written arguments first.
    ///
    /// # Returns
    ///
    /// The word, or `None` when no such list starts here.
    pub(crate) fn word_call_kind_args(&self) -> Option<Kw>
    {
        if let Some(len) = self.word_kind_args_len(0, Kw::SubstitutedRel)
        {
            let function = kw_of(&self.peek_n(len + 1)) == Some(Kw::FnValue);
            return function.then_some(Kw::SubstitutedRel);
        }
        self.word_kind_args_len(0, Kw::Substituting).map(|_| Kw::Substituting)
    }

    /// Give kind arguments in the word form to the call or the function value
    /// a clause starts with.
    ///
    /// # Arguments
    ///
    /// * `head` - What the clause starts with.
    /// * `kind_args` - The kind arguments.
    /// * `start` - Where the kind arguments start.
    ///
    /// # Returns
    ///
    /// The head with the kind arguments, or a diagnostic when it is no call
    /// and no function value.
    pub(crate) fn give_kind_args(
        &self, head: Head, kind_args: Vec<Kind>, start: Span,
    ) -> PResult<Head>
    {
        match head
        {
            Head::Expr(e @ (Expr::Call(_) | Expr::FnAddr(_))) =>
            {
                Ok(Head::Expr(with_kind_args(e, kind_args, Some(start))))
            }
            Head::Stmt(mut stmt) if matches!(*stmt, Stmt::Call(_)) =>
            {
                extract!(Stmt::Call(s) = &mut *stmt);
                s.kind_args = kind_args;
                s.span = start.to(&s.span);
                Ok(Head::Stmt(stmt))
            }
            _ => Err(self.err_expected(&quote(&kw_text(Kw::FnValue)))),
        }
    }
}

/// Give kind arguments to a call or a function value.
///
/// # Arguments
///
/// * `expr` - The call or the function value.
/// * `kind_args` - The kind arguments.
/// * `start` - Where the kind arguments start, when the word form puts them
///   first.
///
/// # Returns
///
/// The expression with the kind arguments; each other expression as it is.
fn with_kind_args(mut expr: Expr, kind_args: Vec<Kind>, start: Option<Span>) -> Expr
{
    match &mut expr
    {
        Expr::Call(c) =>
        {
            c.kind_args = kind_args;
            c.span = start.map_or(c.span, |s| s.to(&c.span));
        }
        Expr::FnAddr(f) =>
        {
            f.kind_args = kind_args;
            f.span = start.map_or(f.span, |s| s.to(&f.span));
        }
        _ => (),
    }
    expr
}
