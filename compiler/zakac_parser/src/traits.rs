//! Traits, trait blocks and notes blocks.
//!
//! A trait names the functions each kind that has it has:
//! `<name> ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင်
//! အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။`, the function heads, then
//! `ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။`. A trait block gives a
//! kind a trait: `<kind> တွင် <trait> ဂုဏ်သတ္တိရှိစေရန် အောက်ပါလုပ်ဆောင်ချက်များ ကို
//! ထည့်သွင်းမည်။`, the functions, then `ထည့်သွင်းမည့် လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။`.
//! A notes block after the head of a generic function or a trait,
//! `ဤနေရာတွင် သတိပြုရန်မှာ … သတိပြုစရာများ ပြီးဆုံးပါပြီး။`, names local kinds
//! (`C သည် <kind> ဖြစ်သည်။`) and the traits a kind parameter needs
//! (`A တွင် <trait> ဂုဏ်သတ္တိ ရှိသည်။`).

use std::collections::HashMap;

use zakac_ast::expr::Ident;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::name::Name;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::BuiltinKindDecl;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::KindParam;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::TraitBlock;
use zakac_ast::stmt::TraitDecl;
use zakac_ast::visit::VisitorMut;
use zakac_ast::visit::walk_fn_mut;
use zakac_diagnostics::parser as diag;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::describe::kw_text;
use crate::fn_kind::FnForm;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::kw_of;

/// What a notes block gives: each local kind with the kind it names, and
/// each trait a kind parameter needs.
#[derive(Debug, Default)]
pub(crate) struct Notes
{
    /// Each local kind, in the order written.
    pub kinds: Vec<(Ident, Kind)>,
    /// Each kind parameter with a trait it needs.
    pub bounds: Vec<(Ident, Kind)>,
}

impl Parser<'_>
{
    /// Whether a trait starts here: a name, its kind parameters in the
    /// notation form when written, then `ဟူသည့်`.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    pub(crate) fn at_trait_decl(&self) -> bool
    {
        let named = matches!(self.peek().kind, TokenKind::Ident | TokenKind::RawIdent);
        let next = 1 + self.angle_len(1).unwrap_or(0);
        named && kw_of(&self.peek_n(next)) == Some(Kw::CalledRel)
    }

    /// Whether a trait block starts here: `ရှိစေရန်` stands in the statement
    /// before its full stop.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    pub(crate) fn at_trait_block(&self) -> bool
    {
        let open = |kind: &TokenKind| {
            *kind != TokenKind::Eof && *kind != TokenKind::Punct(Punct::FullStop)
        };
        let tokens = (0..).map(|n| self.peek_n(n)).take_while(|t| open(&t.kind));
        tokens.into_iter().any(|t| t.kind == TokenKind::Kw(Kw::ToHave))
    }

    /// Parse a trait: its name and kind parameters in either form, the head
    /// closer, a notes block of the traits it needs, the function heads and
    /// its closer.
    ///
    /// # Returns
    ///
    /// The trait, or a diagnostic. A trait inside a block is parsed and
    /// reported.
    pub(crate) fn parse_trait_decl(&mut self) -> PResult<Stmt>
    {
        let start = self.peek().span;
        let name = self.parse_ident()?;
        let mut params = self.parse_notation_params()?;
        self.expect(Kw::CalledRel)?;
        let exported = self.eat(Kw::Export);
        let builtin = self.eat_builtin_mark()?;
        self.expect(Kw::Trait)?;
        if params.is_empty()
        {
            self.expect(Kw::Thi)?;
            params = self.parse_word_params()?;
        }
        do_if!(params.is_empty(), _ = self.expect(Kw::GenericKind)?);
        self.expect(Kw::At)?;
        self.expect(Kw::Following)?;
        self.expect(Kw::Functions)?;
        self.expect(Kw::MustContain)?;
        self.expect_stop()?;
        let added = self.add_kind_names(&params);
        let rest = self.parse_trait_rest(&mut params);
        self.drop_kind_names(added);
        let (fns, end) = rest?;
        let span = start.to(&end);
        do_if!(self.depth > 0, self.diags.push(diag::nested_trait(span)));
        #[rustfmt::skip]
        let node = TraitDecl::builder().name(name).kind_params(params).fns(fns).span(span)
            .exported(exported).builtin(builtin).build();
        Ok(Stmt::Trait(node))
    }

    /// Parse the part of a trait after its head: a notes block when written,
    /// each function head, then the closer.
    ///
    /// # Arguments
    ///
    /// * `params` - The kind parameters, which take the traits of the notes.
    ///
    /// # Returns
    ///
    /// The function heads and the span of the full stop, or a diagnostic.
    fn parse_trait_rest(
        &mut self, params: &mut [KindParam],
    ) -> PResult<(Vec<FnDecl>, Span)>
    {
        if self.at(Kw::Here)
        {
            let notes = self.parse_notes()?;
            // A trait names no local kinds: each note gives a trait.
            let local = notes.kinds.first().map(|(n, _)| n.span);
            let found = |at: Span| {
                diag::expected_token(at, &kw_text(Kw::At), &kw_text(Kw::Thi))
            };
            do_if!(let Some(at) = local, return Err(found(at)));
            self.give_bounds(params, notes.bounds);
        }
        let mut fns = Vec::new();
        while !self.at(Kw::Trait) && !self.at_eof()
        {
            fns.push(self.parse_trait_fn_head()?);
        }
        self.expect(Kw::Trait)?;
        self.expect(Kw::At)?;
        self.expect(Kw::ItemOf)?;
        self.expect(Kw::Functions)?;
        self.expect(Kw::Done)?;
        let stop = self.expect_stop()?;
        Ok((fns, stop.span))
    }

    /// Parse the head of a function of a trait, which has no block:
    /// `လုပ်ဆောင်ချက် <name> သည် <params> လက်ခံပြီး <kind> ကို ပြန်ပေးမည်။`, with
    /// `လက်ခံမည်။` for no result, or the result alone.
    ///
    /// # Returns
    ///
    /// The function, or a diagnostic.
    fn parse_trait_fn_head(&mut self) -> PResult<FnDecl>
    {
        let start = self.expect(Kw::Function)?.span;
        let name = self.parse_ident()?;
        self.expect(Kw::Thi)?;
        let (params, end, _) =
            self.parse_params(&[Kw::Accepting, Kw::WillAccept], false)?;
        let bare = end == Some(Kw::WillAccept);
        let ret = tern!(bare, Default::default(), self.parse_ret_clause(true)?);
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        let block = BlockStmt::builder().span(stop.span).build();
        #[rustfmt::skip]
        let node = FnDecl::builder().name(name).params(params).maybe_ret(ret.kind)
            .owned_ret(ret.owned).block(block).span(span).build();
        Ok(node)
    }

    /// Parse a trait block: the kind, the trait, the head closer, each
    /// function, then the closer. The functions remain in the block until the
    /// parser of the file moves them out.
    ///
    /// # Returns
    ///
    /// The block, or a diagnostic. A block inside a block is parsed and
    /// reported.
    pub(crate) fn parse_trait_block(&mut self) -> PResult<Stmt>
    {
        let start = self.peek().span;
        let kind = self.parse_kind()?;
        self.expect(Kw::At)?;
        let bound = self.parse_trait_ref()?;
        self.expect(Kw::Trait)?;
        self.expect(Kw::ToHave)?;
        self.expect(Kw::Following)?;
        self.expect(Kw::Functions)?;
        self.expect(Kw::Ko)?;
        self.expect(Kw::Import)?;
        self.expect_stop()?;
        let mut fns = Vec::new();
        while !self.at(Kw::ToAddRel) && !self.at_eof()
        {
            fns.push(self.parse_block_fn()?);
        }
        self.expect(Kw::ToAddRel)?;
        self.expect(Kw::Functions)?;
        self.expect(Kw::Done)?;
        let span = start.to(&self.expect_stop()?.span);
        do_if!(self.depth > 0, self.diags.push(diag::nested_trait(span)));
        let fn_names = fns.iter().map(|f| f.name.clone()).collect();
        #[rustfmt::skip]
        let node = TraitBlock::builder().kind(kind).bound(bound).fns(fns).fn_names(fn_names)
            .span(span).build();
        Ok(Stmt::TraitBlock(node))
    }

    /// Parse one function of a trait block.
    ///
    /// # Returns
    ///
    /// The function, or a diagnostic when no function starts here.
    fn parse_block_fn(&mut self) -> PResult<FnDecl>
    {
        ret_if!(!self.at(Kw::Function), Err(self.err_expected("လုပ်ဆောင်ချက်")));
        let Stmt::Fn(decl) = self.parse_fn_decl()?
        else
        {
            return Err(self.err_expected("လုပ်ဆောင်ချက်"));
        };
        Ok(decl)
    }

    /// Read the mark of a declaration the compiler gives,
    /// `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော`, when it stands here.
    ///
    /// # Returns
    ///
    /// Whether it stood here, or a diagnostic for half of it.
    fn eat_builtin_mark(&mut self) -> PResult<bool>
    {
        ret_if!(!self.eat(Kw::InCompiler), Ok(false));
        self.expect(Kw::IncludedWith)?;
        Ok(true)
    }

    /// Parse the rest of the head of a function the compiler gives, after
    /// `လုပ်ဆောင်ချက် <name> သည်`, as an external function's:
    /// `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော လုပ်ဆောင်ချက် ဖြစ်သည်။`, or `… ဖြစ်၍` then
    /// the parameters ending in `လက်ခံမည်` or `လက်ခံပြီး` and the result clause,
    /// or the result clause alone. No block follows: the compiler gives the
    /// code.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `name` - The function name.
    /// * `kind_params` - The kind parameters of a generic function.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic.
    pub(crate) fn parse_builtin_fn(
        &mut self, start: Span, name: Ident, kind_params: Vec<KindParam>,
    ) -> PResult<Stmt>
    {
        self.eat_builtin_mark()?;
        self.expect(Kw::Function)?;
        let bare =
            self.expect_one_of(&[Kw::And, Kw::Is])?.kind == TokenKind::Kw(Kw::Is);
        let ends = [Kw::Accepting, Kw::WillAccept];
        let (params, end, _) =
            tern!(bare, (Vec::new(), None, false), self.parse_params(&ends, false)?);
        let no_ret = bare || end == Some(Kw::WillAccept);
        let ret = tern!(no_ret, Default::default(), self.parse_ret_clause(true)?);
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        do_if!(self.depth > 0, self.diags.push(diag::nested_function(span)));
        let block = BlockStmt::builder().span(stop.span).build();
        #[rustfmt::skip]
        let node = FnDecl::builder().name(name).kind_params(kind_params).params(params)
            .maybe_ret(ret.kind).owned_ret(ret.owned).block(block).builtin(true).span(span)
            .build();
        Ok(Stmt::Fn(node))
    }

    /// Whether a built-in kind of the prelude is declared here:
    /// `<kind> သည် ကွန်ပိုင်လာထဲတွင်`.
    ///
    /// # Returns
    ///
    /// `true` if one is.
    pub(crate) fn at_builtin_kind(&self) -> bool
    {
        let Some(n) = self.kind_len_at(0, FnForm::NoParams)
        else
        {
            return false;
        };
        kw_of(&self.peek_n(n)) == Some(Kw::Thi)
            && kw_of(&self.peek_n(n + 1)) == Some(Kw::InCompiler)
    }

    /// Parse a built-in kind of the prelude:
    /// `<kind> သည် ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော ဒေတာအမျိုးအစား ဖြစ်သည်။`.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic.
    pub(crate) fn parse_builtin_kind(&mut self) -> PResult<Stmt>
    {
        let start = self.peek().span;
        let kind = self.parse_kind()?;
        self.expect(Kw::Thi)?;
        self.eat_builtin_mark()?;
        self.expect(Kw::DataKindOpen)?;
        self.expect(Kw::Is)?;
        let span = start.to(&self.expect_stop()?.span);
        Ok(Stmt::BuiltinKind(BuiltinKindDecl::builder().kind(kind).span(span).build()))
    }

    /// Parse a trait as a constraint or a trait block names it: its name,
    /// with the kinds after the first in the word form before it
    /// (`<kinds> ကို သုံး၍ <name>`) or in the sign form after it
    /// (`<name><<kinds>>`).
    ///
    /// # Returns
    ///
    /// The trait as a named kind with its kinds, or a diagnostic.
    pub(crate) fn parse_trait_ref(&mut self) -> PResult<Kind>
    {
        let start = self.peek().span;
        let word = self.word_kind_args_len(0, Kw::Using).is_some();
        let mut args = tern!(word, self.parse_word_kind_args(Kw::Using)?, Vec::new());
        let name = self.parse_ident()?;
        if args.is_empty() && self.at_punct(Punct::Lt)
        {
            args = self.parse_sign_kind_args()?;
        }
        let span = start.to(&self.prev_span());
        #[rustfmt::skip]
        let kind = Kind::builder().base(BaseKind::Named).name(name).args(args).span(span)
            .build();
        Ok(kind)
    }

    /// Parse a notes block: `ဤနေရာတွင် သတိပြုရန်မှာ`, then each note, then
    /// `သတိပြုစရာများ ပြီးဆုံးပါပြီး။`. A local kind is a kind name for the rest
    /// of the declaration.
    ///
    /// # Returns
    ///
    /// The notes, or a diagnostic.
    pub(crate) fn parse_notes(&mut self) -> PResult<Notes>
    {
        self.expect(Kw::Here)?;
        self.expect(Kw::NoteThat)?;
        let mut notes = Notes::default();
        while !self.at(Kw::Notes) && !self.at_eof()
        {
            self.parse_note(&mut notes)?;
        }
        self.expect(Kw::Notes)?;
        self.expect(Kw::Done)?;
        self.expect_stop()?;
        Ok(notes)
    }

    /// Parse one note: `<name> သည် <kind> တစ်<classifier> ဖြစ်သည်။`, a local
    /// kind, or `<name> တွင် <trait> ဂုဏ်သတ္တိ ရှိသည်။`, with two or more traits
    /// as a list in words and `တို့` before `ရှိသည်`.
    ///
    /// # Arguments
    ///
    /// * `notes` - The notes so far.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic.
    fn parse_note(&mut self, notes: &mut Notes) -> PResult<()>
    {
        let name = self.parse_ident()?;
        if self.eat(Kw::Thi)
        {
            let kind = self.parse_slot_kind()?;
            self.expect(Kw::Is)?;
            self.expect_stop()?;
            self.structs.insert(name.name.to_string());
            notes.kinds.push((name, kind));
            return Ok(());
        }
        self.expect(Kw::At)?;
        let traits = self.parse_word_list(|p| {
            let bound = p.parse_trait_ref()?;
            p.expect(Kw::Trait)?;
            Ok(bound)
        })?;
        do_if!(traits.len() > 1, _ = self.expect(Kw::Plural)?);
        self.expect(Kw::Has)?;
        self.expect_stop()?;
        do_for!(bound in traits, notes.bounds.push((name.clone(), bound)));
        Ok(())
    }

    /// Give each kind parameter the traits the notes name for it; a note on
    /// another name is ZK0248.
    ///
    /// # Arguments
    ///
    /// * `params` - The kind parameters.
    /// * `bounds` - Each name with a trait.
    pub(crate) fn give_bounds(
        &mut self, params: &mut [KindParam], bounds: Vec<(Ident, Kind)>,
    )
    {
        for (name, bound) in bounds
        {
            let param = params.iter_mut().find(|p| p.name.name == name.name);
            let Some(param) = param
            else
            {
                self.diags.push(diag::note_names_no_param(name.span, &name.name));
                continue;
            };
            param.bounds.push(bound);
        }
    }
}

/// Put the kind each local kind of a notes block names in place of its
/// name, in each kind of a function; a local kind can name the ones before
/// it.
///
/// # Arguments
///
/// * `decl` - The function.
/// * `kinds` - The local kinds, in the order written.
pub(crate) fn expand_local_kinds(decl: &mut FnDecl, kinds: &[(Ident, Kind)])
{
    ret_if!(kinds.is_empty());
    let mut bound: HashMap<Name, Kind> = HashMap::new();
    for (name, kind) in kinds
    {
        let mut kind = kind.clone();
        kind.substitute(&bound);
        // The measure word belongs to the note, not to the kind it names.
        kind.classifier = None;
        bound.insert(name.name, kind);
    }
    walk_fn_mut(&mut Locals(&bound), decl);
}

/// The walker that puts local kinds in place.
struct Locals<'a>(&'a HashMap<Name, Kind>);

impl VisitorMut for Locals<'_>
{
    fn visit_kind_mut(&mut self, n: &mut Kind)
    {
        // A bare use of a local kind keeps its name, so it counts with `ခု`
        // as a kind parameter does.
        let named = n.base == BaseKind::Named && n.wraps.is_empty();
        let local = n.name.clone().filter(|i| named && self.0.contains_key(&i.name));
        n.substitute(self.0);
        do_if!(local.is_some(), n.alias = local);
    }
}
