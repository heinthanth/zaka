use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::Phrases;
use zakac_ast::expr::VerbExpr;
use zakac_ast::kind::BASE_FIELD;
use zakac_ast::kind::BASED_ON;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::stmt::AliasDecl;
use zakac_ast::stmt::AssertStmt;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::BreakStmt;
use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::ContinueStmt;
use zakac_ast::stmt::CountMode;
use zakac_ast::stmt::CountedStmt;
use zakac_ast::stmt::DeclStmt;
use zakac_ast::stmt::DeferStmt;
use zakac_ast::stmt::EachPart;
use zakac_ast::stmt::EachStmt;
use zakac_ast::stmt::EnumDecl;
use zakac_ast::stmt::EnumMember;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::GenericStmt;
use zakac_ast::stmt::IfArm;
use zakac_ast::stmt::IfStmt;
use zakac_ast::stmt::ImportStmt;
use zakac_ast::stmt::KindParam;
use zakac_ast::stmt::Param;
use zakac_ast::stmt::ReturnStmt;
use zakac_ast::stmt::START_NAME;
use zakac_ast::stmt::ShareStmt;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::StrArg;
use zakac_ast::stmt::StructDecl;
use zakac_ast::stmt::SwitchArm;
use zakac_ast::stmt::SwitchDefault;
use zakac_ast::stmt::SwitchStmt;
use zakac_ast::stmt::TraitDecl;
use zakac_ast::stmt::WhileStmt;
use zakac_ast::threads::USABLE_IN_THREADS;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_diagnostics::parser as diag;
use zakac_i18n::t;
use zakac_lexer::spelling::DATA;
use zakac_lexer::spelling::canonical;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::clause::ClauseOut;
use crate::clause::Mode;
use crate::decl_head::DeclHead;
use crate::describe::kw_text;
use crate::describe::quote;
use crate::fn_kind::FnForm;
use crate::parser::MAX_NEST;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::fn_head_at;
use crate::parser::kw_of;
use crate::threads::past_marks;
use crate::traits::expand_local_kinds;
use crate::using::CallParts;
use crate::using::EI;
use crate::using::preposition_words;

/// The markers of a parameter item: owned, changed, written alone, and its
/// preposition.
pub(crate) type ItemMarkers = (bool, bool, bool, Option<Ident>);

/// A parameter clause: the parameters, the word that ended it, and whether it
/// ends in `နှင့် အခြားတန်ဖိုးများ`, or `အခြားတန်ဖိုးများနှင့်အတူ` in the item form.
pub(crate) type ParamClause = (Vec<Param>, Option<Kw>, bool);

/// The result clause of a function header.
#[derive(Default)]
pub(crate) struct RetClause
{
    /// The result kind with its measure word, `None` when the header gives
    /// none.
    pub kind: Option<Kind>,
    /// Whether the result is the program's own, `အပိုင်အနေဖြင့်`.
    pub owned: bool,
}

/// The count word `တစ်ခု` ("one") in `လုပ်ဆောင်ချက် တစ်ခု`.
const ONE_THING: &str = "တစ်ခု";

/// What a function that code outside can call writes before the phrase that
/// marks it: its C name.
const C_NAME_HEAD: &str = "\"<C name>\" အမည်ဖြင့်";

/// The mark of a struct that a lib gives to C.
const USABLE_MARK: &str = "ပြင်ပမှ ယူသုံးနိုင်သော";

/// The mark of a kind from outside the program.
const OUTSIDE_MARK: &str = "ပြင်ပမှ လာသော";

/// The mark of a struct whose values copies share.
const SHARED_MARK: &str = "မျှဝေနိုင်သော";

/// The first word of the kind parameters in words.
const GENERIC_MARK: &str = "ယေဘုယျဒေတာအမျိုးအစား";

/// The mark of a struct, after `ဒေတာအစုအဝေးသည်`.
#[derive(Debug, Clone, PartialEq)]
enum StructMark
{
    /// No mark.
    None,
    /// `"<C name>" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်သော`, with the C name.
    CName(StrArg),
    /// `ပြင်ပမှ လာသော`: a kind from outside the program.
    External,
    /// `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော`: a struct the compiler gives.
    Builtin,
}

/// The field that holds a struct a struct is based on: the name of the struct
/// after [`BASE_FIELD`].
///
/// # Arguments
///
/// * `base` - The struct, as written.
///
/// # Returns
///
/// The field.
fn base_field(base: &Kind) -> Param
{
    let written = base.name.as_ref().map_or("", |n| n.name.as_str());
    let name = Ident::builder()
        .name(format!("{BASE_FIELD}{written}").as_str())
        .span(base.span)
        .build();
    Param::builder().kind(base.clone()).name(name).span(base.span).build()
}

/// The words of a mark of a struct, for a message.
///
/// # Arguments
///
/// * `mark` - The mark.
///
/// # Returns
///
/// The words.
fn struct_mark_text(mark: &StructMark) -> String
{
    match mark
    {
        StructMark::CName(_) => USABLE_MARK.to_string(),
        StructMark::External => OUTSIDE_MARK.to_string(),
        _ => kw_text(Kw::InCompiler),
    }
}

/// The direction a counted-loop word names.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// The mode, or `None` for another word.
fn count_mode(kw: Kw) -> Option<CountMode>
{
    match kw
    {
        Kw::Counting => Some(CountMode::Count),
        Kw::Increasing => Some(CountMode::Increase),
        Kw::Decreasing => Some(CountMode::Decrease),
        _ => None,
    }
}

/// Take the default of each function of a trait out of it, as a generic
/// function of the file: the kind parameters of the trait, the first one
/// needing the trait itself. The trait keeps the head.
///
/// # Arguments
///
/// * `decl` - The trait.
///
/// # Returns
///
/// The generic functions.
fn hoist_defaults(decl: &mut TraitDecl) -> Vec<FnDecl>
{
    let params = &decl.kind_params;
    let span = decl.name.span;
    let arg = |p: &KindParam| {
        Kind::builder().base(BaseKind::Named).name(p.name.clone()).span(p.span).build()
    };
    let others = params.iter().skip(1).map(arg).collect();
    #[rustfmt::skip]
    let itself = Kind::builder().base(BaseKind::Named).name(decl.name.clone()).args(others)
        .span(span).build();
    let mut kind_params = params.clone();
    do_if!(let Some(first) = kind_params.first_mut(), first.bounds.push(itself));
    let exported = decl.exported;
    let with_block = decl.fns.iter_mut().filter(|f| f.has_default);
    let hoist = |f: &mut FnDecl| {
        let blank = BlockStmt::builder().span(f.block.span).build();
        let block = std::mem::replace(&mut f.block, blank);
        FnDecl { kind_params: kind_params.clone(), block, exported, ..f.clone() }
    };
    with_block.map(hoist).collect()
}

/// Add a top-level statement: the functions of a trait block are functions
/// of the file, and so is the default of each function of a trait, as a
/// generic function of the kinds that have the trait.
///
/// # Arguments
///
/// * `stmts` - The statements so far.
/// * `stmt` - The statement.
fn push_top(stmts: &mut Vec<Stmt>, mut stmt: Stmt)
{
    let fns = match &mut stmt
    {
        Stmt::TraitBlock(b) => std::mem::take(&mut b.fns),
        Stmt::Trait(t) => hoist_defaults(t),
        _ => Vec::new(),
    };
    stmts.push(stmt);
    stmts.extend(fns.into_iter().map(Stmt::Fn));
}

impl Parser<'_>
{
    /// Parse statements up to the end of the input.
    ///
    /// # Returns
    ///
    /// The statements that parsed; errors are in the diagnostics.
    pub(crate) fn parse_program(&mut self) -> Vec<Stmt>
    {
        let mut stmts = Vec::new();
        while !self.at_eof()
        {
            let stmt = self.parse_stmt_recover();
            self.imports_closed |=
                !matches!(stmt, Some(Stmt::Import(_) | Stmt::Share(_)));
            do_if!(let Some(stmt) = stmt, push_top(&mut stmts, stmt));
        }
        stmts
    }

    /// Parse one statement; a call of a heap function of the core is its
    /// heap statement.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    pub(crate) fn parse_stmt(&mut self) -> PResult<Stmt>
    {
        self.parse_stmt_node().map(|stmt| self.heap_node(stmt))
    }

    /// Parse one statement, chosen from its first token.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn parse_stmt_node(&mut self) -> PResult<Stmt>
    {
        ret_if!(self.at_kind_decl(), self.parse_kind_decl());
        ret_if!(self.at_test_decl(), self.parse_test_decl());
        ret_if!(self.at_fn_decl(), self.parse_fn_decl());
        ret_if!(self.at_builtin_kind(), self.parse_builtin_kind());
        ret_if!(self.at_trait_decl(), self.parse_trait_decl());
        ret_if!(self.at_trait_block(), self.parse_trait_block());
        ret_if!(self.at_auto_rings(), self.parse_auto_rings());
        ret_if!(self.at_lock(), self.parse_lock());
        ret_if!(self.at_select(), self.parse_select());
        let tok = self.peek();
        let Some(kw) = kw_of(&tok)
        else
        {
            return self.parse_expr_headed();
        };
        match kw
        {
            Kw::If => self.parse_if(),
            Kw::AtDevTime => self.parse_debug_assert(),
            Kw::WhenTasksDone | Kw::WhenTasksFail => self.parse_defer(),
            Kw::ElseIf | Kw::Else =>
            {
                Err(diag::else_without_if(tok.span, self.text_of(tok.span)))
            }
            Kw::Var | Kw::Const => self.parse_decl_or_loop(),
            Kw::Program => self.parse_fn_decl(),
            Kw::DataKindOpen => self.parse_alias_decl(),
            Kw::Export => self.parse_exported(),
            Kw::Unsafe | Kw::Comptime if self.at_marked_start() =>
            {
                self.parse_marked_start()
            }
            Kw::BlockOpen | Kw::UnsafeBlockOpen => self.parse_block().map(Stmt::Block),
            Kw::Loop => self.parse_break(),
            Kw::Mem if self.at_rings_collect() => self.parse_rings_collect(),
            Kw::NextIteration => self.parse_continue(),
            Kw::WillReturn | Kw::MarkSucceeded => self.parse_bare_return(),
            Kw::MarkFailed => self.parse_mark_failed(),
            Kw::WhenDone => Err(diag::update_outside_loop(tok.span)),
            Kw::BlockClose | Kw::ProgramClose =>
            {
                Err(diag::unmatched_block_close(tok.span))
            }
            _ => self.parse_expr_headed(),
        }
    }

    /// Parse a statement that starts with an expression: a clause closed
    /// with a statement verb, a while loop or a counted loop.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn parse_expr_headed(&mut self) -> PResult<Stmt>
    {
        let is_str = matches!(self.peek().kind, TokenKind::Str(_));
        let from = self.peek_n(1).kind == TokenKind::Kw(Kw::From);
        // A string with `မှ` after it opens an import, unless it is the first
        // argument of a preposition call.
        ret_if!(
            is_str && from && self.preposition_end(1).is_none(),
            self.parse_import()
        );
        let whole =
            matches!(self.peek_n(2).kind, TokenKind::Kw(Kw::Import | Kw::Reshare));
        let ko = self.peek_n(1).kind == TokenKind::Kw(Kw::Ko);
        ret_if!(is_str && ko && whole, self.parse_whole_import());
        ret_if!(self.at_share(), self.parse_share());
        let clause = self.parse_clause(Mode::Stmt)?;
        self.finish_expr_headed(clause)
    }

    /// Finish a statement that starts with a parsed clause: a while loop or
    /// a counted loop.
    ///
    /// # Arguments
    ///
    /// * `clause` - The clause.
    ///
    /// # Returns
    ///
    /// The statement, or an `expected-stmt-verb` diagnostic.
    fn finish_expr_headed(&mut self, clause: ClauseOut) -> PResult<Stmt>
    {
        let first = match clause
        {
            ClauseOut::Stmt(stmt) => return Ok(*stmt),
            ClauseOut::Switch(pair) =>
            {
                let (subject, case) = *pair;
                return self.parse_switch(subject, case);
            }
            ClauseOut::Expr(Expr::Verb(test))
                if test.op == VerbOp::IsVariant && self.at(Kw::Then) =>
            {
                return self.parse_variant_switch(test);
            }
            ClauseOut::Expr(first) => first,
        };
        let cond = self.parse_logical_rest(first, 0)?;
        let start = cond.span();
        ret_if!(self.at(Kw::MustBe), self.parse_assert_tail(cond, false, start));
        let is_while = self.at(Kw::While) || self.at(Kw::WhileIs);
        ret_if!(is_while, self.parse_while_tail(cond));
        ret_if!(self.at_receive_each(), self.parse_receive_each(cond));
        ret_if!(self.at(Kw::From), self.parse_counted_tail(cond));
        ret_if!(self.at_each_head(), self.parse_each_tail(cond));
        Err(diag::expected_stmt_verb(self.peek().span, &self.found()))
    }

    /// Parse `ပရိုဂရမ်ရေးသားချိန်တွင် <cond> ဖြစ်ရမည်`, a debug assert, with its
    /// message and full stop.
    ///
    /// # Returns
    ///
    /// The assert, or a diagnostic.
    fn parse_debug_assert(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        let cond = self.parse_expr()?;
        self.parse_assert_tail(cond, true, start)
    }

    /// Parse `လုပ်ဆောင်စရာများပြီးသည့် အခါတွင် <statement>` or the mark before a block:
    /// code that runs when the block around it ends.
    ///
    /// # Returns
    ///
    /// The marked code, or a diagnostic.
    fn parse_defer(&mut self) -> PResult<Stmt>
    {
        let mark = self.bump();
        let failure = mark.kind == TokenKind::Kw(Kw::WhenTasksFail);
        let block = self.parse_branch(true)?;
        let span = mark.span.to(&block.span);
        #[rustfmt::skip]
        let node = DeferStmt::builder().block(block).failure(failure).span(span).build();
        Ok(Stmt::Defer(node))
    }

    /// Parse the rest of an assert after its condition: `ဖြစ်ရမည်`, then
    /// `၊ မဟုတ်ပါက <text> ဟု ရပ်မည်` for a message, then the full stop.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    /// * `debug` - Whether `ပရိုဂရမ်ရေးသားချိန်တွင်` opened the assert.
    /// * `start` - Where the assert starts.
    ///
    /// # Returns
    ///
    /// The assert, or a diagnostic.
    fn parse_assert_tail(
        &mut self, cond: Expr, debug: bool, start: Span,
    ) -> PResult<Stmt>
    {
        self.expect(Kw::MustBe)?;
        let message = self.parse_assert_message()?;
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        #[rustfmt::skip]
        let node = AssertStmt::builder().cond(cond).debug(debug).maybe_message(message)
            .span(span).build();
        Ok(Stmt::Assert(node))
    }

    /// Parse `၊ မဟုတ်ပါက <text> ဟု ရပ်မည်`, the message of an assert, when it
    /// stands next.
    ///
    /// # Returns
    ///
    /// The text, `None` with no message, or a diagnostic.
    fn parse_assert_message(&mut self) -> PResult<Option<Expr>>
    {
        let comma = self.peek().kind == TokenKind::Punct(Punct::ListSep);
        let if_not = self.peek_n(1).kind == TokenKind::Kw(Kw::IfNot);
        ret_if!(!(comma && if_not), Ok(None));
        self.advance();
        self.advance();
        let message = self.parse_operand()?;
        self.expect(Kw::Hu)?;
        self.expect(Kw::Stop)?;
        Ok(Some(message))
    }

    /// Parse `"<file>" မှ <name> [(၊ <name>)* နှင့် <name>] (တို့ကို|ကို) ထည့်သွင်းမည်။`. An
    /// import after another statement or inside a block is parsed and
    /// reported.
    ///
    /// # Returns
    ///
    /// The import, or a diagnostic.
    fn parse_import(&mut self) -> PResult<Stmt>
    {
        let tok = self.bump();
        let TokenKind::Str(idx) = tok.kind
        else
        {
            unreachable!("the current token is a string");
        };
        self.bump();
        let names = self.parse_word_list(Self::parse_ident)?;
        self.expect_list_word(names.len(), Kw::Ko, Kw::ListKo)?;
        self.expect(Kw::Import)?;
        let stop = self.expect_stop()?;
        let span = tok.span.to(&stop.span);
        self.check_import_place(span);
        let lit = self.strings[idx as usize].clone();
        let path = StrArg::builder().lit(lit).span(tok.span).build();
        Ok(Stmt::Import(
            ImportStmt::builder().path(path).names(names).span(span).build(),
        ))
    }

    /// Parse `"<file>" ကို ထည့်သွင်းမည်။`, which imports each name a file
    /// exports, or `"<file>" ကို ပြန်လည် မျှဝေမည်။`, which shares each of
    /// them again and brings none.
    ///
    /// # Returns
    ///
    /// The import, or a diagnostic.
    fn parse_whole_import(&mut self) -> PResult<Stmt>
    {
        let tok = self.bump();
        let TokenKind::Str(idx) = tok.kind
        else
        {
            unreachable!("the current token is a string");
        };
        self.bump();
        let share = self.bump().kind == TokenKind::Kw(Kw::Reshare);
        let stop = self.expect_stop()?;
        let span = tok.span.to(&stop.span);
        self.check_import_place(span);
        let lit = self.strings[idx as usize].clone();
        let path = StrArg::builder().lit(lit).span(tok.span).build();
        #[rustfmt::skip]
        let node = ImportStmt::builder().path(path).names(Vec::new()).whole(!share)
            .share(share).span(span).build();
        Ok(Stmt::Import(node))
    }

    /// Whether the sentence ahead ends in `အများသုံးနိုင်ရန် ပြန်လည် မျှဝေမည်။`,
    /// a share of imported names.
    ///
    /// # Returns
    ///
    /// `true` when it does.
    fn at_share(&self) -> bool
    {
        let ends = |t: &TokenKind| {
            matches!(t, TokenKind::Eof | TokenKind::Punct(Punct::FullStop))
        };
        let Some(stop) = (0..).find(|n| ends(&self.peek_n(*n).kind))
        else
        {
            return false;
        };
        let kind = |n: usize| self.peek_n(n).kind;
        stop >= 3
            && kind(stop - 1) == TokenKind::Kw(Kw::Reshare)
            && kind(stop - 2) == TokenKind::Kw(Kw::ToShare)
    }

    /// Parse `<name>၊ ... နှင့် <name> တို့ကို အများသုံးနိုင်ရန် ပြန်လည် မျှဝေမည်။`,
    /// or with one name, `<name> ကို ...`: a share of imported names.
    ///
    /// # Returns
    ///
    /// The share, or a diagnostic.
    fn parse_share(&mut self) -> PResult<Stmt>
    {
        let first = self.peek().span;
        let names = self.parse_word_list(Self::parse_ident)?;
        self.expect_list_word(names.len(), Kw::Ko, Kw::ListKo)?;
        self.expect(Kw::ToShare)?;
        self.expect(Kw::Reshare)?;
        let stop = self.expect_stop()?;
        let span = first.to(&stop.span);
        self.check_import_place(span);
        Ok(Stmt::Share(ShareStmt::builder().names(names).span(span).build()))
    }

    /// Report an import or a share after another statement or inside a
    /// block: each comes first in its file.
    ///
    /// # Arguments
    ///
    /// * `span` - The import or the share.
    fn check_import_place(&mut self, span: Span)
    {
        let late = self.nested() || self.imports_closed;
        do_if!(late, self.diags.push(diag::import_not_first(span)));
    }

    /// Parse a declaration after the export mark `အများသုံးနိုင်သော` and mark it
    /// exported: a variable, a constant or a kind alias. A function, a struct
    /// and an enumeration hold the mark after `ဟူသော`. The mark before another
    /// statement, or before a declaration inside a block, is reported.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic.
    fn parse_exported(&mut self) -> PResult<Stmt>
    {
        let mark = self.bump().span;
        let mut stmt = self.parse_stmt()?;
        let flag = match &mut stmt
        {
            Stmt::Alias(n) => Some((&mut n.exported, &mut n.span)),
            Stmt::Decl(n) => Some((&mut n.exported, &mut n.span)),
            _ => None,
        };
        let top = !self.nested();
        let Some((exported, span)) = flag.filter(|_| top)
        else
        {
            return Err(diag::export_not_on_decl(mark));
        };
        *exported = true;
        *span = mark.to(span);
        Ok(stmt)
    }

    /// Whether the marks of a function head stand before the start block here:
    /// `<marks> ပရိုဂရမ်သည်`, which reads well but marks no function.
    ///
    /// # Returns
    ///
    /// `true` when the marks stand there.
    fn at_marked_start(&self) -> bool
    {
        let sep =
            |n: usize| self.at_punct_n(n, Punct::ListSep) && self.fn_mark_at(n + 1);
        let after = (0..).find(|n| !self.fn_mark_at(*n) && !sep(*n)).unwrap_or(0);
        kw_of(&self.peek_n(after)) == Some(Kw::Program)
    }

    /// Parse the start block after the marks of a function head, and report
    /// each mark: ZK0229 for `အများသုံးနိုင်သော`, ZK0257 for `စိတ်မချရသော` and ZK0256
    /// for `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော`.
    ///
    /// # Returns
    ///
    /// The start block, or a diagnostic.
    fn parse_marked_start(&mut self) -> PResult<Stmt>
    {
        let mut marks = Vec::new();
        while self.fn_mark_at(0)
        {
            marks.push(self.bump());
            self.mark_gap(|p, n| p.fn_mark_at(n))?;
        }
        let stmt = self.parse_fn_decl()?;
        let reports = marks.iter().map(|m| match kw_of(m)
        {
            Some(Kw::Export) => diag::export_not_on_decl(m.span),
            Some(Kw::Unsafe) => diag::unsafe_start(m.span),
            _ => diag::comptime_mark_misplaced(m.span),
        });
        self.diags.extend(reports);
        Ok(stmt)
    }

    /// Parse `နေသရွေ့ <block>` after the condition of a while loop.
    ///
    /// # Arguments
    ///
    /// * `cond` - The condition.
    ///
    /// # Returns
    ///
    /// The loop, or a diagnostic.
    fn parse_while_tail(&mut self, cond: Expr) -> PResult<Stmt>
    {
        self.bump();
        let block = self.parse_branch(true)?;
        let span = cond.span().to(&block.span);
        Ok(Stmt::While(WhileStmt::builder().cond(cond).block(block).span(span).build()))
    }

    /// Parse `မှ <to> အထိ (<step> စီ)? <name> အနေဖြင့် <mode> <block>` after
    /// the first bound of a counted loop.
    ///
    /// # Arguments
    ///
    /// * `from` - The first bound.
    ///
    /// # Returns
    ///
    /// The loop, or a diagnostic.
    fn parse_counted_tail(&mut self, from: Expr) -> PResult<Stmt>
    {
        self.bump();
        let to = self.parse_expr()?;
        self.expect(Kw::To)?;
        let (step, var) = self.parse_step_and_var()?;
        self.expect(Kw::CountAs)?;
        let threads = self.parse_threads()?;
        let parallel = threads.is_some() || self.eat_parallel();
        let mode = kw_of(&self.peek()).and_then(count_mode);
        let names = [Kw::Counting, Kw::Increasing, Kw::Decreasing];
        let mode = mode.ok_or_else(|| self.err_one_of(&names))?;
        self.bump();
        let block = self.parse_branch(true)?;
        let span = from.span().to(&block.span);
        #[rustfmt::skip]
        let node = CountedStmt::builder().var(var).from(from).to(to).maybe_step(step).mode(mode)
            .block(block).parallel(parallel).maybe_threads(threads).span(span).build();
        Ok(Stmt::Counted(node))
    }

    /// Whether `၏ <part>တိုင်း` comes next: the head of an each loop over the
    /// cells of a list, the characters of text, or the rows of a ဇယား.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_each_head(&self) -> bool
    {
        let word = kw_of(&self.peek_n(1));
        self.at_punct(Punct::Ei) && word.and_then(each_part).is_some()
    }

    /// Parse `၏ <part>တိုင်း ကို [<place> နှင့်] <name> အနေဖြင့် ယူ၍ <block>`
    /// after the list, the text or the ဇယား an each loop walks.
    ///
    /// # Arguments
    ///
    /// * `source` - The list or the text.
    ///
    /// # Returns
    ///
    /// The loop, or a diagnostic.
    fn parse_each_tail(&mut self, source: Expr) -> PResult<Stmt>
    {
        self.bump();
        let part = kw_of(&self.bump()).and_then(each_part).unwrap_or_default();
        self.expect(Kw::Ko)?;
        let first = self.parse_ident()?;
        let (place, var) = match self.eat(Kw::Hnint)
        {
            true => (Some(first), self.parse_ident()?),
            false => (None, first),
        };
        self.expect(Kw::CountAs)?;
        let threads = self.parse_threads()?;
        let parallel = threads.is_some() || self.eat_parallel();
        self.expect(Kw::Taking)?;
        let block = self.parse_branch(true)?;
        let span = source.span().to(&block.span);
        #[rustfmt::skip]
        let node = EachStmt::builder().source(source).part(part).maybe_place(place).var(var)
            .block(block).parallel(parallel).maybe_threads(threads).span(span).build();
        Ok(Stmt::Each(node))
    }

    /// Parse `(<step> စီ)? <name>` in a counted loop. Both start with an
    /// expression, so the word after it decides.
    ///
    /// # Returns
    ///
    /// The step, when written, and the loop variable.
    fn parse_step_and_var(&mut self) -> PResult<(Option<Expr>, Ident)>
    {
        let expr = self.parse_expr()?;
        ret_if!(self.eat(Kw::Step), Ok((Some(expr), self.parse_ident()?)));
        let Expr::Name(name) = expr
        else
        {
            return Err(self.err_expected(&quote(&kw_text(Kw::Step))));
        };
        Ok((None, Ident::builder().name(name.name).span(name.span).build()))
    }

    /// An `expected-token` diagnostic that lists reserved words.
    ///
    /// # Arguments
    ///
    /// * `kws` - The words the grammar allows.
    ///
    /// # Returns
    ///
    /// The diagnostic.
    fn err_one_of(&self, kws: &[Kw]) -> zakac_diagnostics::Diagnostic
    {
        let names: Vec<String> = kws.iter().map(|kw| quote(&kw_text(*kw))).collect();
        self.err_expected(&crate::describe::one_of(&names))
    }

    /// Parse a conditional with its else-if arms and else block.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn parse_if(&mut self) -> PResult<Stmt>
    {
        let comptime = self.peek_n(1).kind == TokenKind::Kw(Kw::AtCompileTime);
        // The branches of a compile-time conditional at the top level hold
        // top-level statements: declarations, imports, the start block.
        let lift = comptime && !self.nested();
        do_if!(lift, self.top += 1);
        let parsed = self.parse_if_chain(comptime);
        do_if!(lift, self.top -= 1);
        parsed
    }

    /// Parse the arms and the else block of a conditional.
    ///
    /// # Arguments
    ///
    /// * `comptime` - Whether `ကွန်ပိုင်းလုပ်ချိန်တွင်` follows `အကယ်၍`.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn parse_if_chain(&mut self, comptime: bool) -> PResult<Stmt>
    {
        let start = self.peek().span;
        let mut arms = vec![self.parse_if_arm()?];
        while self.at(Kw::ElseIf)
        {
            arms.push(self.parse_if_arm()?);
        }
        let else_block = match self.eat(Kw::Else)
        {
            true => Some(self.parse_branch(false)?),
            false => None,
        };
        let span = start.to(&self.prev_span());
        #[rustfmt::skip]
        let node = IfStmt::builder().arms(arms).maybe_else_block(else_block).comptime(comptime)
            .span(span).build();
        Ok(Stmt::If(node))
    }

    /// Parse `(အကယ်၍|ထိုသို့မဟုတ်ဘဲ) <cond> (လျှင်|ဖြစ်လျှင်) <block>`, with
    /// `ကွန်ပိုင်းလုပ်ချိန်တွင်` after `အကယ်၍` for a compile-time conditional.
    ///
    /// # Returns
    ///
    /// The arm, or a diagnostic.
    fn parse_if_arm(&mut self) -> PResult<IfArm>
    {
        let start = self.bump().span;
        self.eat(Kw::AtCompileTime);
        let cond = self.parse_expr()?;
        self.expect_one_of(&[Kw::Then, Kw::ThenIs])?;
        let block = self.parse_branch(false)?;
        let span = start.to(&block.span);
        Ok(IfArm::builder().cond(cond).block(block).span(span).build())
    }

    /// Parse a block, or the single statement that stands in for one.
    ///
    /// # Arguments
    ///
    /// * `allow_if` - Whether the single statement can be a conditional; inside
    ///   a conditional arm it cannot, so each else arm belongs to one clear
    ///   conditional.
    ///
    /// # Returns
    ///
    /// The block, or a diagnostic.
    pub(crate) fn parse_branch(&mut self, allow_if: bool) -> PResult<BlockStmt>
    {
        ret_if!(
            self.at(Kw::BlockOpen) || self.at(Kw::UnsafeBlockOpen),
            self.parse_block()
        );
        let tok = self.peek();
        let nested_if = !allow_if && self.at(Kw::If);
        ret_if!(nested_if, Err(diag::nested_single_line_if(tok.span)));
        self.depth += 1;
        let stmt = self.parse_stmt();
        self.depth -= 1;
        let stmt = stmt?;
        let span = stmt.span();
        // Marked code alone in a branch would end the branch as it ran.
        ret_if!(matches!(stmt, Stmt::Defer(_)), Err(diag::defer_alone(span)));
        Ok(BlockStmt::builder().stmts(vec![stmt]).single(true).span(span).build())
    }

    /// Parse `အောက်ပါတို့ကို လုပ်ဆောင်မည်။ <stmt>* လုပ်ဆောင်စရာများ ပြီးပါပြီ။`.
    ///
    /// # Returns
    ///
    /// The block, or a diagnostic.
    pub(crate) fn parse_block(&mut self) -> PResult<BlockStmt>
    {
        ret_if!(self.depth >= MAX_NEST, Err(self.err_too_deep()));
        let open = self.expect_one_of(&[Kw::BlockOpen, Kw::UnsafeBlockOpen])?;
        let unchecked = open.kind == TokenKind::Kw(Kw::UnsafeBlockOpen);
        self.expect_stop()?;
        let stmts = self.parse_stmts_to_close();
        let span = self.parse_block_close(open.span)?;
        #[rustfmt::skip]
        return Ok(BlockStmt::builder().stmts(stmts).unchecked(unchecked).span(span).build());
    }

    /// Parse statements until a block closer or the end of the input.
    ///
    /// # Returns
    ///
    /// The statements that parsed.
    fn parse_stmts_to_close(&mut self) -> Vec<Stmt>
    {
        self.depth += 1;
        // The block of a compile-time conditional at the top level holds
        // top-level statements.
        let top = !self.nested();
        let mut stmts = Vec::new();
        while !self.at_close() && !self.at_eof()
        {
            let Some(stmt) = self.parse_stmt_recover()
            else
            {
                continue;
            };
            tern!(top, push_top(&mut stmts, stmt), stmts.push(stmt));
        }
        self.depth -= 1;
        stmts
    }

    /// Whether the next token closes a block: a block closer, or the closer
    /// of the start block.
    ///
    /// # Returns
    ///
    /// `true` at either closer.
    pub(crate) fn at_close(&self) -> bool
    {
        self.at(Kw::BlockClose) || self.at(Kw::ProgramClose)
    }

    /// Parse the block closer and its full stop.
    ///
    /// # Arguments
    ///
    /// * `start` - The span of the block opener.
    ///
    /// # Returns
    ///
    /// The span of the whole block, or a diagnostic.
    fn parse_block_close(&mut self, start: Span) -> PResult<Span>
    {
        self.parse_close(start, Kw::BlockClose)
    }

    /// Parse a closer and its full stop.
    ///
    /// # Arguments
    ///
    /// * `start` - The span of the opener.
    /// * `closer` - The closer the block takes.
    ///
    /// # Returns
    ///
    /// The span from the opener to the full stop, or a diagnostic.
    fn parse_close(&mut self, start: Span, closer: Kw) -> PResult<Span>
    {
        self.expect(closer)?;
        let stop = self.expect_stop()?;
        Ok(start.to(&stop.span))
    }

    /// Parse a declaration or a generic loop; both start with
    /// `(ကိန်းရှင်|ကိန်းသေ) <name> သည် <head>`, where the head is
    /// `<kind> တစ်<classifier>`, with `ဖြစ်၍ တန်ဖိုးသည် <expr>` for a value,
    /// or a value alone.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn parse_decl_or_loop(&mut self) -> PResult<Stmt>
    {
        let word = self.bump();
        let is_const = word.kind == TokenKind::Kw(Kw::Const);
        let at_name = self.cur;
        let name = self.parse_ident()?;
        if self.at(Kw::Into)
        {
            self.cur = at_name;
            return self.parse_decl_into(word.span, is_const, name);
        }
        self.expect(Kw::Thi)?;
        ret_if!(
            is_const && self.at(Kw::InCompiler),
            self.parse_builtin_const(word.span, name)
        );
        let head = self.parse_decl_head()?;
        // A written kind gives the prepositions of a call through the name; the
        // starting value gives them when no kind is written.
        match (&head.kind, &head.init)
        {
            (Some(kind), _) => self.note_preposition_value(&name.name, kind),
            (None, Some(init)) => self.note_preposition_addr(&name.name, init),
            (None, None) => (),
        }
        if self.at(Kw::And)
        {
            ret_if!(is_const, Err(diag::const_loop(word.span)));
            return self.parse_generic_tail(word.span, name, head);
        }
        self.expect_one_of(&[Kw::Let, Kw::And])?;
        let stop = self.expect_stop()?;
        let span = word.span.to(&stop.span);
        #[rustfmt::skip]
        let node = DeclStmt::builder().name(name).maybe_kind(head.kind)
            .is_const(is_const).maybe_init(head.init).span(span).build();
        Ok(Stmt::Decl(node))
    }

    /// Parse the rest of a constant the compiler gives, after `ကိန်းသေ <name>
    /// သည်`: `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော <kind> တစ်<word> ဖြစ်သည်။`, with no
    /// value; the loader writes the value of the target in.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic.
    fn parse_builtin_const(&mut self, start: Span, name: Ident) -> PResult<Stmt>
    {
        self.eat_builtin_mark()?;
        let kind = self.parse_slot_kind()?;
        self.expect(Kw::Is)?;
        let span = start.to(&self.expect_stop()?.span);
        #[rustfmt::skip]
        let node = DeclStmt::builder().name(name).kind(kind).is_const(true).builtin(true)
            .span(span).build();
        Ok(Stmt::Decl(node))
    }

    /// Parse `<name> ထဲသို့ <value> ကို ကူးထည့်မည်။` or `... အကုန်ကူးထည့်မည်။` after
    /// `ကိန်းရှင်` or `ကိန်းသေ`: a declaration whose
    /// kind is the kind of the value. The clause engine reads it from the
    /// name as an assignment.
    ///
    /// # Arguments
    ///
    /// * `start` - The span of `ကိန်းရှင်` or `ကိန်းသေ`.
    /// * `is_const` - `true` for `ကိန်းသေ`.
    /// * `name` - The declared name; the cursor is back at it.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic.
    fn parse_decl_into(
        &mut self, start: Span, is_const: bool, name: Ident,
    ) -> PResult<Stmt>
    {
        let ClauseOut::Stmt(stmt) = self.parse_clause(Mode::Stmt)?
        else
        {
            return Err(diag::expected_stmt_verb(self.peek().span, &self.found()));
        };
        let Stmt::Assign(assign) = *stmt
        else
        {
            return Err(diag::expected_stmt_verb(stmt.span(), "ထည့်မည်"));
        };
        let span = start.to(&assign.span);
        #[rustfmt::skip]
        let node = DeclStmt::builder().name(name).is_const(is_const).init(assign.value)
            .span(span).build();
        Ok(Stmt::Decl(node))
    }

    /// Parse `ဖြစ်၍ တန်ဖိုးသည် <expr>`; `ဖြစ်၍` is the current token.
    ///
    /// # Returns
    ///
    /// The value, or a diagnostic.
    pub(crate) fn parse_value_clause(&mut self) -> PResult<Expr>
    {
        self.advance();
        self.expect(Kw::Value)?;
        self.expect(Kw::Thi)?;
        self.parse_clause_expr()
    }

    /// Parse `ဖြစ်၍|ဖြစ်ပြီး|ဖြစ်ကာ <cond> နေသရွေ့ <block>` after the header of a
    /// generic loop. The block can end with a `ပြီးလျှင်` update.
    ///
    /// # Arguments
    ///
    /// * `start` - The span of `ကိန်းရှင်`.
    /// * `var` - The loop variable.
    /// * `head` - Its kind, classifier and initial value.
    ///
    /// # Returns
    ///
    /// The loop, or a diagnostic.
    fn parse_generic_tail(
        &mut self, start: Span, var: Ident, head: DeclHead,
    ) -> PResult<Stmt>
    {
        self.bump();
        let cond = self.parse_expr()?;
        self.expect_one_of(&[Kw::While, Kw::WhileIs])?;
        let open = self.expect_one_of(&[Kw::BlockOpen, Kw::UnsafeBlockOpen])?;
        let unchecked = open.kind == TokenKind::Kw(Kw::UnsafeBlockOpen);
        self.expect_stop()?;
        let (stmts, update) = self.parse_loop_stmts();
        let block_span = self.parse_block_close(open.span)?;
        #[rustfmt::skip]
        let block = BlockStmt::builder().stmts(stmts).unchecked(unchecked).span(block_span).build();
        let span = start.to(&block_span);
        #[rustfmt::skip]
        let node = GenericStmt::builder().var(var).maybe_kind(head.kind)
            .maybe_init(head.init).cond(cond)
            .maybe_update(update.map(Box::new)).block(block).span(span).build();
        Ok(Stmt::Generic(node))
    }

    /// Parse the statements of a generic loop and its update.
    ///
    /// # Returns
    ///
    /// The statements and the update, when written.
    fn parse_loop_stmts(&mut self) -> (Vec<Stmt>, Option<Stmt>)
    {
        self.depth += 1;
        let (mut stmts, mut update) = (Vec::new(), None);
        while !self.at_close() && !self.at_eof()
        {
            let is_update = self.at(Kw::WhenDone);
            let start = self.peek().span;
            do_if!(is_update, self.advance());
            let stmt = self.parse_stmt_recover();
            if !is_update
            {
                stmts.extend(stmt);
                continue;
            }
            update = stmt.or(update);
            self.check_update_last(start);
        }
        self.depth -= 1;
        (stmts, update)
    }

    /// Report an update that is not followed with the block closer.
    ///
    /// # Arguments
    ///
    /// * `start` - The span of `ပြီးလျှင်`.
    fn check_update_last(&mut self, start: Span)
    {
        ret_if!(self.at_close() || self.at_eof());
        let span = start.to(&self.prev_span());
        self.diags.push(diag::update_not_last(span));
    }

    /// Finish a call statement after its end word: the pass-on
    /// `၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေး(ပါ|မည်)` when written, then the
    /// full stop.
    ///
    /// # Arguments
    ///
    /// * `parts` - What the call holds.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    pub(crate) fn finish_call_stmt(&mut self, parts: CallParts) -> PResult<Stmt>
    {
        let CallParts { start, callee, place, args, using, prepositions, phrases } =
            parts;
        let pass = self.at_punct(Punct::ListSep)
            && kw_of(&self.peek_n(1)) == Some(Kw::PassFault);
        do_if!(pass, {
            self.bump();
            self.bump();
        });
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        #[rustfmt::skip]
        let node = CallStmt::builder().callee(callee).maybe_place(place).args(args).pass(pass)
            .maybe_using(using).prepositions(prepositions).phrases(phrases).span(span).build();
        Ok(Stmt::Call(node))
    }

    /// Parse a function declaration:
    /// `f ဟူသော လုပ်ဆောင်ချက် သည် <params>? အောက်ပါတို့ကို လုပ်ဆောင်မည်။`, or with
    /// a result, `... အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> ရလဒ် ကို ပြန်ပေးမည်။`. The
    /// marks `အများသုံးနိုင်သော`, `စိတ်မချရသော` and `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော`
    /// stand between `ဟူသော` and `လုပ်ဆောင်ချက်`, as on a struct. The start block
    /// has the same parts after `ပရိုဂရမ်သည်`, with no name.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic. A start block inside a block is
    /// parsed and reported.
    pub(crate) fn parse_fn_decl(&mut self) -> PResult<Stmt>
    {
        let start = self.peek().span;
        if self.eat(Kw::Program)
        {
            let name = Ident::builder().name(START_NAME).span(start).build();
            self.expect(Kw::Thi)?;
            return self.parse_fn_rest(start, name, true, Vec::new());
        }
        let top = !self.nested();
        let name = self.parse_ident()?;
        let mut kind_params = self.parse_notation_params()?;
        self.expect(Kw::Called)?;
        let mut marks = Vec::new();
        while self.fn_mark_at(0)
        {
            marks.push(self.bump());
            self.mark_gap(|p, n| p.fn_mark_at(n))?;
        }
        self.expect(Kw::Function)?;
        self.expect(Kw::Thi)?;
        let c_name = self.parse_c_name()?;
        let word_kinds = kind_params.is_empty() && self.at(Kw::GenericKind);
        if word_kinds
        {
            kind_params = self.parse_word_params()?;
            self.expect(Kw::Thi)?;
        }
        // The compiler mark stands right after `သည်`, before the kind parameters
        // in words; after them it is no mark, and the head goes on as one with
        // statements.
        let builtin = self.at(Kw::InCompiler) && !word_kinds;
        // The phrases stand right before the parameters: here in a head with
        // statements, after `ဖြစ်၍` in a head with no block.
        let blockless = builtin || self.at(Kw::External) || self.at_lib();
        let phrases = tern!(blockless, Phrases::default(), self.parse_head_phrases()?);
        let mut stmt = self.parse_fn_kinds(start, name, kind_params, builtin)?;
        let Stmt::Fn(node) = &mut stmt
        else
        {
            return Ok(stmt);
        };
        marks.iter().try_for_each(|m| mark_fn(node, kw_of(m), m.span, top))?;
        do_if!(!phrases.is_empty(), node.phrases = phrases);
        node.c_name = c_name;
        self.check_phrases(node);
        Ok(stmt)
    }

    /// Parse the head of a function that code outside can call, when one
    /// starts here: `"<C name>" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍`. The
    /// phrase with no C name before it is an error.
    ///
    /// # Returns
    ///
    /// The C name, `None` when no such head starts here, or a diagnostic.
    fn parse_c_name(&mut self) -> PResult<Option<StrArg>>
    {
        ret_if!(self.at(Kw::CallableOutside), Err(self.err_expected(C_NAME_HEAD)));
        let TokenKind::Str(idx) = self.peek().kind
        else
        {
            return Ok(None);
        };
        ret_if!(self.peek_n(1).kind != TokenKind::Kw(Kw::UnderName), Ok(None));
        let tok = self.bump();
        self.advance();
        self.expect(Kw::CallableOutside)?;
        self.parse_one_function()?;
        self.expect(Kw::And)?;
        let lit = self.strings[idx as usize].clone();
        Ok(Some(StrArg::builder().lit(lit).span(tok.span).build()))
    }

    /// Parse the rest of a function declaration after its head: an external
    /// function, a function the compiler gives, or one with statements.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `name` - The function name.
    /// * `kind_params` - The kind parameters of a generic function.
    /// * `builtin` - Whether the compiler mark stands in its place, right after
    ///   `သည်`.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic.
    fn parse_fn_kinds(
        &mut self, start: Span, name: Ident, kind_params: Vec<KindParam>, builtin: bool,
    ) -> PResult<Stmt>
    {
        let external = self.at(Kw::External) || self.at_lib();
        ret_if!(external, self.parse_external(start, name));
        let added = self.add_kind_names(&kind_params);
        let parsed = tern!(
            builtin,
            self.parse_builtin_fn(start, name, kind_params),
            self.parse_fn_rest(start, name, false, kind_params)
        );
        self.drop_kind_names(added);
        parsed
    }

    /// Parse the rest of a function declaration after its name, its kind
    /// parameters and `သည်`: the parameters, the opener, the result, then the
    /// block.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `name` - The function name.
    /// * `is_start` - Whether it is the start block.
    /// * `kind_params` - The kind parameters of a generic function.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic.
    fn parse_fn_rest(
        &mut self, start: Span, name: Ident, is_start: bool,
        kind_params: Vec<KindParam>,
    ) -> PResult<Stmt>
    {
        let (params, _, _) = self.parse_params(&[Kw::Accepting], false)?;
        self.parse_fn_block(start, name, is_start, kind_params, params)
    }

    /// Parse the rest of a function declaration after its parameters: the
    /// opener, the result, then the block.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `name` - The function name.
    /// * `is_start` - Whether it is the start block.
    /// * `kind_params` - The kind parameters of a generic function.
    /// * `params` - The parameters.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic.
    pub(crate) fn parse_fn_block(
        &mut self, start: Span, name: Ident, is_start: bool,
        mut kind_params: Vec<KindParam>, params: Vec<Param>,
    ) -> PResult<Stmt>
    {
        let open = self.expect_one_of(&[Kw::BlockOpen, Kw::FnOpenRet])?;
        let RetClause { kind: ret, owned } =
            self.parse_ret_clause(open.kind == TokenKind::Kw(Kw::FnOpenRet))?;
        let fn_span = start.to(&self.prev_span());
        // A function inside a block borrows the locals around it; the start
        // block has its place at the top level alone.
        do_if!(
            self.nested() && is_start,
            self.diags.push(diag::nested_function(fn_span))
        );
        self.expect_stop()?;
        let notes = tern!(self.at(Kw::Here), self.parse_notes()?, Default::default());
        self.give_bounds(&mut kind_params, notes.bounds);
        let stmts = self.parse_stmts_to_close();
        let locals: Vec<String> =
            notes.kinds.iter().map(|(n, _)| n.name.to_string()).collect();
        let closed = self
            .parse_close(open.span, tern!(is_start, Kw::ProgramClose, Kw::BlockClose));
        do_for!(n in &locals, _ = self.structs.remove(n));
        let block_span = closed?;
        let block = BlockStmt::builder().stmts(stmts).span(block_span).build();
        let span = start.to(&block_span);
        #[rustfmt::skip]
        let mut node = FnDecl::builder().name(name).kind_params(kind_params).params(params)
            .maybe_ret(ret).block(block).start(is_start).owned_ret(owned).span(span).build();
        expand_local_kinds(&mut node, &notes.kinds);
        Ok(Stmt::Fn(node))
    }

    /// Parse the rest of the header of an external function, after
    /// `<name> ဟူသော လုပ်ဆောင်ချက် သည်`: `ပြင်ပမှလာသော လုပ်ဆောင်ချက်(တစ်ခု)?ဖြစ်သည်။` with
    /// no parameters and no result, or `... ဖြစ်၍` then the parameters ending in
    /// `လက်ခံမည်` (no result) or `လက်ခံပြီး` and the result clause, or the
    /// result clause alone. The parameters can end in
    /// `အခြားတန်ဖိုးများနှင့်အတူ`, as for a C function that takes more. The host
    /// serves the function, so no block follows. `"<lib>" မှလာသော` in place
    /// of `ပြင်ပမှလာသော` makes it a C function of that shared lib.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `name` - The function name.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic.
    fn parse_external(&mut self, start: Span, name: Ident) -> PResult<Stmt>
    {
        let lib = self.parse_origin()?;
        self.parse_one_function()?;
        let bare =
            self.expect_one_of(&[Kw::And, Kw::Is])?.kind == TokenKind::Kw(Kw::Is);
        let phrases = tern!(bare, Phrases::default(), self.parse_head_phrases()?);
        let ends = [Kw::Accepting, Kw::WillAccept];
        let (params, end, variadic) =
            tern!(bare, (Vec::new(), None, false), self.parse_params(&ends, true)?);
        let RetClause { kind: ret, owned } = tern!(
            bare || end == Some(Kw::WillAccept),
            RetClause::default(),
            self.parse_ret_clause(true)?
        );
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        do_if!(self.nested(), self.diags.push(diag::nested_function(span)));
        let block = BlockStmt::builder().span(stop.span).build();
        #[rustfmt::skip]
        let node = FnDecl::builder().name(name).params(params).maybe_ret(ret).block(block)
            .external(true).maybe_lib(lib).phrases(phrases).variadic(variadic).owned_ret(owned)
            .span(span).build();
        Ok(Stmt::Fn(node))
    }

    /// Whether `"<lib>" မှ` starts here, the origin of a C function.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_lib(&self) -> bool
    {
        matches!(self.peek().kind, TokenKind::Str(_))
            && self.peek_n(1).kind == TokenKind::Kw(Kw::From)
    }

    /// Read where an external function comes from: `ပြင်ပမှလာသော`, the host,
    /// or `"<lib>" မှလာသော`, a shared lib.
    ///
    /// # Returns
    ///
    /// The lib, `None` for the host, or a diagnostic.
    fn parse_origin(&mut self) -> PResult<Option<StrArg>>
    {
        let tok = self.bump();
        let TokenKind::Str(idx) = tok.kind
        else
        {
            return Ok(None);
        };
        self.expect(Kw::From)?;
        self.expect(Kw::Coming)?;
        let lit = self.strings[idx as usize].clone();
        Ok(Some(StrArg::builder().lit(lit).span(tok.span).build()))
    }

    /// Read `လုပ်ဆောင်ချက်တစ်ခု`, or `လုပ်ဆောင်ချက်` with an optional `တစ်ခု` after it.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic.
    fn parse_one_function(&mut self) -> PResult<()>
    {
        ret_if!(self.eat(Kw::OneFunction), Ok(()));
        self.expect(Kw::Function)?;
        let tok = self.peek();
        let one = tok.kind == TokenKind::Ident && self.text_of(tok.span) == ONE_THING;
        do_if!(one, self.advance());
        Ok(())
    }

    /// Parse a kind alias:
    /// `ဒေတာအမျိုးအစား <alias> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။`, or with its
    /// own classifier, `... ဟုလည်း မှတ်ယူနိုင်ပြီး မျိုးပြပစ္စည်းမှာ <word> ဖြစ်သည်။`.
    /// The kind takes no measure word. Kind parameters follow the alias in the
    /// sign form, `<alias><T> ကို`, or in the word form,
    /// `<alias> သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းကို`.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic. A declaration inside a block is
    /// parsed and reported.
    fn parse_alias_decl(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        let name = self.parse_ident()?;
        let mut kind_params = self.parse_notation_params()?;
        if kind_params.is_empty() && self.eat(Kw::Thi)
        {
            kind_params = self.parse_word_params()?;
        }
        self.expect(Kw::Ko)?;
        let added = self.add_kind_names(&kind_params);
        let kind = self.parse_kind();
        self.drop_kind_names(added);
        let kind = kind?;
        let close = self.expect_one_of(&[Kw::AliasClose, Kw::AliasCloseAnd])?;
        let named = close.kind == TokenKind::Kw(Kw::AliasCloseAnd);
        let classifier = tern!(named, Some(self.parse_classifier()?), None);
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        do_if!(self.nested(), self.diags.push(diag::nested_alias(span)));
        #[rustfmt::skip]
        let node = AliasDecl::builder().kind(kind).name(name).kind_params(kind_params)
            .maybe_classifier(classifier).span(span).build();
        Ok(Stmt::Alias(node))
    }

    /// Whether a function declaration starts here:
    /// `<name> ဟူသော [<marks>] လုပ်ဆောင်ချက် သည်`.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    pub(crate) fn at_fn_decl(&self) -> bool { fn_head_at(self.cur.rest()).is_some() }

    /// Whether a test block starts here: `<name> ဟူသော စမ်းသပ်ချက် သည်`.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    fn at_test_decl(&self) -> bool
    {
        let named = matches!(self.peek().kind, TokenKind::Ident | TokenKind::RawIdent);
        let words = [Kw::Called, Kw::Test, Kw::Thi];
        named
            && words
                .iter()
                .enumerate()
                .all(|(i, w)| self.peek_n(i + 1).kind == TokenKind::Kw(*w))
    }

    /// Parse a test block: `<name> ဟူသော စမ်းသပ်ချက် သည် အောက်ပါတို့ကို
    /// လုပ်ဆောင်မည်။ … လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။`. A test inside a block, or
    /// one that gives a result, is parsed and reported.
    ///
    /// # Returns
    ///
    /// The test, a function with its test mark, or a diagnostic.
    fn parse_test_decl(&mut self) -> PResult<Stmt>
    {
        let start = self.peek().span;
        let top = !self.nested();
        let name = self.parse_ident()?;
        self.advance();
        self.advance();
        self.advance();
        let mut stmt =
            self.parse_fn_block(start, name, false, Vec::new(), Vec::new())?;
        let Stmt::Fn(node) = &mut stmt
        else
        {
            return Ok(stmt);
        };
        node.test = true;
        let name = node.name.name.to_string();
        do_if!(
            !self.test_names.insert(name.clone()),
            self.diags.push(diag::test_twice(node.name.span, &name))
        );
        do_if!(
            !top || node.ret.is_some(),
            self.diags.push(diag::test_shape(node.name.span))
        );
        Ok(stmt)
    }

    /// Whether a struct or an enumeration declaration starts here:
    /// `<name> ဟူသော`, then the marks `အများသုံးနိုင်သော` or `မျှဝေနိုင်သော`, then
    /// `ဒေတာအစုအဝေး` or `ဒေတာအမျိုးအစားကွဲ`.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    fn at_kind_decl(&self) -> bool
    {
        let named = matches!(self.peek().kind, TokenKind::Ident | TokenKind::RawIdent);
        let next = 1 + self.angle_len(1).unwrap_or(0);
        ret_if!(!named || kw_of(&self.peek_n(next)) != Some(Kw::Called), false);
        let rest = self.cur.rest();
        let word =
            past_marks(rest, next + 1, &|t: &Token| self.text_of(t.span).to_string());
        matches!(rest.get(word).and_then(kw_of), Some(Kw::StructWord | Kw::EnumWord))
    }

    /// Parse a struct or an enumeration declaration that opens with
    /// `<name> ဟူသော`: its marks, then a struct after `ဒေတာအစုအဝေး`, an
    /// enumeration after `ဒေတာအမျိုးအစားကွဲ`.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic.
    fn parse_kind_decl(&mut self) -> PResult<Stmt>
    {
        let start = self.peek().span;
        let name = self.parse_ident()?;
        let kind_params = self.parse_notation_params()?;
        self.expect(Kw::Called)?;
        let exported = self.eat(Kw::Export);
        do_if!(exported, self.mark_gap(|p, n| p.kind_mark_at(n))?);
        let threads = self.eat_threads_phrase();
        do_if!(threads, self.mark_gap(|p, n| p.kind_mark_at(n))?);
        let shared = self.eat(Kw::Shareable);
        let word = tern!(
            shared,
            self.expect(Kw::StructWord)?,
            self.expect_one_of(&[Kw::StructWord, Kw::EnumWord])?
        );
        let mut stmt = match kw_of(&word)
        {
            Some(Kw::EnumWord) => self.parse_enum_decl(start, name, kind_params)?,
            _ => self.parse_struct_decl(start, name, kind_params)?,
        };
        match &mut stmt
        {
            Stmt::Struct(n) =>
            {
                // Each handle is a shared value, and C passes a struct whole.
                let marked = n.external || n.c_name.is_some();
                let mark = tern!(n.external, OUTSIDE_MARK, USABLE_MARK);
                let clash = diag::mark_clash(n.name.span, SHARED_MARK, mark);
                do_if!(shared && marked, self.diags.push(clash));
                let clash = diag::mark_clash(n.name.span, USABLE_IN_THREADS, mark);
                do_if!(threads && marked, self.diags.push(clash));
                n.exported = exported;
                n.shared |= shared;
                n.threads = threads;
            }
            Stmt::Enum(n) =>
            {
                let word = kw_text(Kw::EnumWord);
                let clash = diag::mark_clash(n.name.span, USABLE_IN_THREADS, &word);
                do_if!(threads, self.diags.push(clash));
                n.exported = exported;
            }
            _ => (),
        }
        Ok(stmt)
    }

    /// Parse a struct declaration after `<name> ဟူသော ... ဒေတာအစုအဝေး`:
    /// `တွင် <fields> တို့ပါဝင်သည်။`, or `သည် [<mark> ဒေတာအစုအဝေး ဖြစ်၍]
    /// [<kind parameters>] ၎င်းတွင် <fields> တို့ပါဝင်သည်။` (with one field
    /// `ပါဝင်သည်`). `…ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <classifier> ဖြစ်သည်။` at the
    /// end gives its classifier. Each field is `<name> ဟူသော <kind>
    /// တစ်<classifier>`. The marks: `"<C name>" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်သော`
    /// gives the struct to C under that name; `ပြင်ပမှ လာသော` makes a kind
    /// from outside, whose fields are C's and which can end at once with
    /// `ဖြစ်သည်။`; `ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော` marks a struct the
    /// compiler gives, in the core.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `name` - The struct name.
    /// * `kind_params` - The kind parameters written in the notation form.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic. A declaration inside a block is
    /// parsed and reported.
    fn parse_struct_decl(
        &mut self, start: Span, name: Ident, mut kind_params: Vec<KindParam>,
    ) -> PResult<Stmt>
    {
        let opened = self.eat(Kw::Thi);
        let (mark, bases) =
            tern!(opened, self.parse_struct_marks()?, (StructMark::None, Vec::new()));
        let external = mark == StructMark::External;
        do_if!(!bases.is_empty() && mark != StructMark::None, {
            let other = struct_mark_text(&mark);
            self.diags.push(diag::mark_clash(name.span, BASED_ON, &other));
        });
        let marked = mark != StructMark::None || !bases.is_empty();
        do_if!(marked, _ = self.expect(Kw::StructWord)?);
        ret_if!(external && self.eat(Kw::Is), self.external_kind(start, name, false));
        do_if!(marked, _ = self.expect(Kw::And)?);
        if opened
        {
            kind_params.extend(self.parse_word_params_open()?);
            self.expect(Kw::Itself)?;
        }
        self.check_mark_params(&name, &mark, &kind_params);
        self.expect(Kw::At)?;
        ret_if!(external, self.external_kind(start, name, true));
        let added = self.add_kind_names(&kind_params);
        let fields = self.parse_listed_fields(true);
        self.drop_kind_names(added);
        // Each struct it is based on is a part of it, held in a field whose
        // name no program can write.
        let parts = bases.iter().map(base_field);
        let fields: Vec<Param> = parts.chain(fields?).collect();
        let hint_of = Some((&name, kind_params.as_slice()));
        let (classifier, hint, span) = self.parse_decl_end(start, hint_of)?;
        do_if!(self.nested(), self.diags.push(diag::nested_struct(span)));
        let builtin = mark == StructMark::Builtin;
        let c_name = tern!(let StructMark::CName(c) = mark, Some(c), None);
        #[rustfmt::skip]
        let node = StructDecl::builder().name(name).kind_params(kind_params).bases(bases).fields(fields)
            .maybe_classifier(classifier).maybe_c_name(c_name).builtin(builtin).hint(hint)
            .span(span).build();
        Ok(Stmt::Struct(node))
    }

    /// Parse the mark of a struct after `ဒေတာအစုအဝေးသည်`, with the
    /// `ဒေတာအစုအဝေး` after it: a C name, `ပြင်ပမှ လာသော`, or the mark of a
    /// struct the compiler gives. The phrase of a C name with no text before
    /// it is an error.
    ///
    /// # Returns
    ///
    /// The mark, `StructMark::None` when none starts here, or a diagnostic.
    fn parse_struct_marks(&mut self) -> PResult<(StructMark, Vec<Kind>)>
    {
        let (mut mark, mut bases) = (StructMark::None, Vec::new());
        loop
        {
            if let Some(more) = self.parse_bases()?
            {
                bases.extend(more);
            }
            else
            {
                let one = self.parse_struct_mark()?;
                ret_if!(one == StructMark::None, Ok((mark, bases)));
                mark = one;
            }
            self.mark_gap(|p, n| p.struct_mark_at(n))?;
            ret_if!(!self.struct_mark_at(0), Ok((mark, bases)));
        }
    }

    /// Whether a mark of a struct after `သည်` starts `n` tokens ahead: a C
    /// name, `ပြင်ပမှ လာသော`, the compiler mark, or the structs it is based on.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn struct_mark_at(&self, n: usize) -> bool
    {
        let named = matches!(self.peek_n(n).kind, TokenKind::Str(_))
            && self.peek_n(n + 1).kind == TokenKind::Kw(Kw::UnderName);
        let word =
            matches!(kw_of(&self.peek_n(n)), Some(Kw::External | Kw::InCompiler));
        named || word || self.bases_len(n).is_some()
    }

    /// The number of tokens of the structs a struct is based on, `n` tokens
    /// ahead: a kind, or kinds joined with `နှင့်`, then `ကို` or `တို့ကို`, then
    /// `အခြေခံသော`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// The token count, or `None` when no such mark starts there.
    fn bases_len(&self, n: usize) -> Option<usize>
    {
        let mut len = self.kind_len_at(n, FnForm::NoParams)?;
        while self.at_kw_n(n + len, Kw::Hnint)
        {
            len += 1 + self.kind_len_at(n + len + 1, FnForm::NoParams)?;
        }
        let object = matches!(kw_of(&self.peek_n(n + len)), Some(Kw::Ko | Kw::ListKo));
        let word = self.peek_n(n + len + 1);
        let based =
            word.kind == TokenKind::Ident && self.text_of(word.span) == BASED_ON;
        (object && based).then_some(len + 2)
    }

    /// Parse the structs a struct is based on, `<struct> ကို အခြေခံသော`, when
    /// the mark stands here.
    ///
    /// # Returns
    ///
    /// The structs, `None` when no such mark stands here, or a diagnostic.
    fn parse_bases(&mut self) -> PResult<Option<Vec<Kind>>>
    {
        ret_if!(self.bases_len(0).is_none(), Ok(None));
        let mut bases = vec![self.parse_kind()?];
        while self.eat(Kw::Hnint)
        {
            bases.push(self.parse_kind()?);
        }
        self.bump();
        self.bump();
        Ok(Some(bases))
    }

    /// Parse one mark of a struct after `ဒေတာအစုအဝေးသည်`.
    ///
    /// # Returns
    ///
    /// The mark, `StructMark::None` when none starts here, or a diagnostic.
    fn parse_struct_mark(&mut self) -> PResult<StructMark>
    {
        ret_if!(self.at(Kw::UsableOutside), Err(self.err_expected(C_NAME_HEAD)));
        let named = matches!(self.peek().kind, TokenKind::Str(_))
            && self.peek_n(1).kind == TokenKind::Kw(Kw::UnderName);
        let mark = match ()
        {
            () if named => StructMark::CName(self.parse_struct_c_name()?),
            () if self.eat(Kw::External) => StructMark::External,
            () if self.eat(Kw::InCompiler) =>
            {
                self.expect(Kw::IncludedWith)?;
                StructMark::Builtin
            }
            () => return Ok(StructMark::None),
        };
        Ok(mark)
    }

    /// Parse `"<C name>" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်သော`, with the text here.
    ///
    /// # Returns
    ///
    /// The C name, or a diagnostic.
    fn parse_struct_c_name(&mut self) -> PResult<StrArg>
    {
        let tok = self.bump();
        let TokenKind::Str(idx) = tok.kind
        else
        {
            return Err(self.err_expected(C_NAME_HEAD));
        };
        self.advance();
        self.expect(Kw::UsableOutside)?;
        let lit = self.strings[idx as usize].clone();
        Ok(StrArg::builder().lit(lit).span(tok.span).build())
    }

    /// Report a mark of a struct that does not go with kind parameters
    /// (ZK0263): C cannot declare a generic struct, and a kind from outside is
    /// one struct of C.
    ///
    /// # Arguments
    ///
    /// * `name` - The struct name.
    /// * `mark` - The mark.
    /// * `kind_params` - The kind parameters.
    fn check_mark_params(
        &mut self, name: &Ident, mark: &StructMark, kind_params: &[KindParam],
    )
    {
        let written = match mark
        {
            StructMark::CName(_) => USABLE_MARK,
            StructMark::External => OUTSIDE_MARK,
            _ => return,
        };
        let clash = diag::mark_clash(name.span, written, GENERIC_MARK);
        do_if!(!kind_params.is_empty(), self.diags.push(clash));
    }

    /// Parse the end of a struct or an enumeration declaration after its
    /// closer: the classifier clause when the closer ends in `ပြီး`, then the
    /// full stop. A struct can end the classifier clause with its hint,
    /// `ဖြစ်ကာ ရေးသားရာတွင် <words> ဟု ရေးသားနိုင်သည်`.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `hint_of` - The name and the kind parameters of a struct, which can
    ///   take a hint; `None` for an enumeration.
    ///
    /// # Returns
    ///
    /// The classifier, when written, the words of the hint, and the span of
    /// the declaration; or a diagnostic.
    fn parse_decl_end(
        &mut self, start: Span, hint_of: Option<(&Ident, &[KindParam])>,
    ) -> PResult<(Option<Ident>, Vec<Ident>, Span)>
    {
        let and_kws =
            [TokenKind::Kw(Kw::ContainsAnd), TokenKind::Kw(Kw::FieldsCloseAnd)];
        let named = self.cur.back(1).is_some_and(|t| and_kws.contains(&t.kind));
        let classifier = tern!(named, Some(self.parse_classifier_word()?), None);
        let hinted = named && hint_of.is_some() && self.eat(Kw::And);
        do_if!(named && !hinted, _ = self.expect(Kw::Is)?);
        let hint = match hint_of.filter(|_| hinted)
        {
            Some((name, params)) => self.parse_hint_decl(name, params, false)?,
            None => Vec::new(),
        };
        do_if!(hinted, _ = self.expect(Kw::CanWrite)?);
        let stop = self.expect_stop()?;
        Ok((classifier, hint, start.to(&stop.span)))
    }

    /// Finish a kind from outside the program after its head: the fields of
    /// the C struct a value points at, from its start, when written, then the
    /// full stop.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `name` - The kind name.
    /// * `with_fields` - Whether `၎င်းတွင်` opened the fields.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic.
    fn external_kind(
        &mut self, start: Span, name: Ident, with_fields: bool,
    ) -> PResult<Stmt>
    {
        let fields = tern!(with_fields, self.parse_listed_fields(false)?, Vec::new());
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        do_if!(self.nested(), self.diags.push(diag::nested_struct(span)));
        #[rustfmt::skip]
        let node = StructDecl::builder().name(name).fields(fields).shared(true).external(true)
            .span(span).build();
        Ok(Stmt::Struct(node))
    }

    /// Parse the fields of a struct and the closer after them, with the list
    /// open.
    ///
    /// # Arguments
    ///
    /// * `classifier_ok` - Whether a classifier clause can follow, so the `ပြီး`
    ///   form of the closer is allowed.
    ///
    /// # Returns
    ///
    /// The fields, or a diagnostic.
    fn parse_listed_fields(&mut self, classifier_ok: bool) -> PResult<Vec<Param>>
    {
        let outer = std::mem::replace(&mut self.in_list, true);
        let fields = self.parse_fields();
        self.in_list = outer;
        let fields = fields?;
        self.expect_one_of(member_closers(fields.len(), classifier_ok))?;
        Ok(fields)
    }

    /// Parse the fields of a struct declaration, with the list open:
    /// `<field> [(၊ <field>)* နှင့် <field>]`.
    ///
    /// # Returns
    ///
    /// The fields, or a diagnostic.
    fn parse_fields(&mut self) -> PResult<Vec<Param>>
    {
        self.parse_word_list(Self::parse_field)
    }

    /// Parse one field: `<name> ဟူသော <kind> တစ်<classifier>`.
    ///
    /// # Returns
    ///
    /// The field, or a diagnostic.
    fn parse_field(&mut self) -> PResult<Param>
    {
        let start = self.peek().span;
        let (kind, name) = self.parse_name_and_kind()?;
        let span = start.to(&self.prev_span());
        Ok(Param::builder().kind(kind).name(name).span(span).build())
    }

    /// Parse an enumeration declaration after
    /// `<name>[<kind parameters>] ဟူသော [အများသုံးနိုင်သော] ဒေတာအမျိုးအစားကွဲ`, with
    /// `သည် <kind parameters> ၎င်း` before `တွင်` in the word form of a generic
    /// one: `တွင် <member>၊ ... နှင့် <member> တို့ပါဝင်သည်။`, or with one member
    /// `တွင် <member> ပါဝင်သည်။`; the `ပြီး` form of the closer, then
    /// `မျိုးပြပစ္စည်းမှာ <classifier> ဖြစ်သည်`, gives its classifier.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `name` - The enumeration name.
    /// * `kind_params` - The kind parameters written in the notation form.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic. A declaration inside a block is
    /// parsed and reported.
    fn parse_enum_decl(
        &mut self, start: Span, name: Ident, mut kind_params: Vec<KindParam>,
    ) -> PResult<Stmt>
    {
        if self.eat(Kw::Thi)
        {
            kind_params.extend(self.parse_word_params_open()?);
            self.expect(Kw::Itself)?;
        }
        self.expect(Kw::At)?;
        let added = self.add_kind_names(&kind_params);
        let members = self.parse_word_list(Self::parse_enum_member);
        self.drop_kind_names(added);
        let members = members?;
        self.expect_one_of(member_closers(members.len(), true))?;
        let hint_of = Some((&name, kind_params.as_slice()));
        let (classifier, hint, span) = self.parse_decl_end(start, hint_of)?;
        do_if!(self.nested(), self.diags.push(diag::nested_enum(span)));
        #[rustfmt::skip]
        let node = EnumDecl::builder().name(name).kind_params(kind_params).members(members)
            .maybe_classifier(classifier).hint(hint).span(span).build();
        Ok(Stmt::Enum(node))
    }

    /// Parse one member of an enumeration:
    /// `[ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်] [တန်ဖိုးအနေဖြင့် <value> ရှိသည့်]
    /// [ဒေတာအနေဖြင့် <kind> တစ်<word> ရှိသည့်] <name>`.
    ///
    /// # Returns
    ///
    /// The member, or a diagnostic.
    fn parse_enum_member(&mut self) -> PResult<EnumMember>
    {
        let initial = self.eat(Kw::InitialMark);
        let start = self.peek().span;
        let value = match self.eat(Kw::ValueAs)
        {
            true =>
            {
                let value = self.parse_expr()?;
                self.expect_reading(Kw::HavingRel)?;
                Some(value)
            }
            false => None,
        };
        let data =
            tern!(self.at_variant_data(), Some(self.parse_variant_data()?), None);
        let name = self.parse_ident()?;
        let span = start.to(&name.span);
        #[rustfmt::skip]
        return Ok(EnumMember::builder().name(name).maybe_value(value).maybe_data(data)
            .initial(initial).span(span).build());
    }

    /// Whether the data of a variant starts here: `ဒေတာအနေဖြင့်`.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_variant_data(&self) -> bool
    {
        let tok = self.peek();
        let data =
            tok.kind == TokenKind::Ident && canonical(self.text_of(tok.span)) == DATA;
        data && self.peek_n(1).kind == TokenKind::Kw(Kw::CountAs)
    }

    /// Parse `ဒေတာအနေဖြင့် <kind> တစ်<word> ရှိသည့်`, the data a variant carries.
    ///
    /// # Returns
    ///
    /// The kind of the data, or a diagnostic.
    fn parse_variant_data(&mut self) -> PResult<Kind>
    {
        (0..2).for_each(|_| _ = self.bump());
        let kind = self.parse_slot_kind()?;
        self.expect_reading(Kw::HavingRel)?;
        Ok(kind)
    }

    /// Parse a switch after its head `<subject> သည် <case>`: the first
    /// `ဖြစ်လျှင်` and its block, each arm `<case> ဖြစ်လျှင် <block>` after it,
    /// and the default arm `တခြား <kind> တစ်<classifier> ဖြစ်လျှင် <block>`. An arm
    /// after the default arm is parsed and reported.
    ///
    /// # Arguments
    ///
    /// * `subject` - The value each case is compared with.
    /// * `case` - The case of the first arm.
    ///
    /// # Returns
    ///
    /// The switch, or a diagnostic.
    fn parse_switch(&mut self, subject: Expr, case: Expr) -> PResult<Stmt>
    {
        let first = self.parse_switch_arm(case)?;
        self.parse_switch_from(subject, first)
    }

    /// Parse a switch whose first arm tests a variant:
    /// `<subject> သည် [<name> ဟူသည့်] <variant> အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင်
    /// <block>`, once the test is read and `လျှင်` is next.
    ///
    /// # Arguments
    ///
    /// * `test` - The test, a verb of [`VerbOp::IsVariant`].
    ///
    /// # Returns
    ///
    /// The switch, or a diagnostic.
    fn parse_variant_switch(&mut self, test: VerbExpr) -> PResult<Stmt>
    {
        let [subject, variant] =
            <[Expr; 2]>::try_from(test.args).expect("a test has two operands");
        let first = self.variant_arm_rest(variant)?;
        self.parse_switch_from(subject, first)
    }

    /// Parse the arms of a switch after its first arm, then the default arm.
    /// An arm after the default arm is parsed and reported.
    ///
    /// # Arguments
    ///
    /// * `subject` - The value each case is compared with.
    /// * `first` - The first arm.
    ///
    /// # Returns
    ///
    /// The switch, or a diagnostic.
    fn parse_switch_from(&mut self, subject: Expr, first: SwitchArm) -> PResult<Stmt>
    {
        let start = subject.span();
        let mut arms = vec![first];
        while let Some(arm) = self.next_switch_arm()?
        {
            arms.push(arm);
        }
        let default =
            tern!(self.at(Kw::Other), Some(self.parse_switch_default()?), None);
        while default.is_some()
        {
            let Some(arm) = self.next_switch_arm()?
            else
            {
                break;
            };
            self.diags.push(diag::arm_after_default(arm.span));
        }
        let span = start.to(&self.prev_span());
        #[rustfmt::skip]
        let node = SwitchStmt::builder().subject(subject).arms(arms).maybe_default(default)
            .span(span).build();
        Ok(Stmt::Switch(node))
    }

    /// Parse the next arm of a switch: `<case> ဖြစ်လျှင် <block>`, or a test of
    /// a variant, `[<name> ဟူသည့်] <variant> အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် <block>`.
    ///
    /// # Returns
    ///
    /// The arm, `None` when no arm starts here, or a diagnostic.
    fn next_switch_arm(&mut self) -> PResult<Option<SwitchArm>>
    {
        if let Some(bound) = self.at_variant_arm()
        {
            let variant = self.parse_variant_operand(bound)?;
            return self.variant_arm_rest(variant).map(Some);
        }
        ret_if!(!self.at_switch_arm(), Ok(None));
        let case = self.inside_word_list(Self::parse_expr)?;
        self.parse_switch_arm(case).map(Some)
    }

    /// Whether an arm that tests a variant starts here: the variant,
    /// `အဖြစ်သို့`, `ပြောင်းလဲနိုင်` and `လျှင်`.
    ///
    /// # Returns
    ///
    /// Whether a name for the data comes first, or `None` at another token.
    fn at_variant_arm(&self) -> Option<bool>
    {
        let bound = self.at_variant_operand()?;
        let verb = tern!(bound, 4, 2);
        let test = kw_of(&self.peek_n(verb)) == Some(Kw::CanBecome);
        let then = kw_of(&self.peek_n(verb + 1)) == Some(Kw::Then);
        (test && then).then_some(bound)
    }

    /// Parse the rest of an arm that tests a variant, after the variant:
    /// `အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် <block>`, or `လျှင် <block>` once the test
    /// is read.
    ///
    /// # Arguments
    ///
    /// * `variant` - The variant the arm names.
    ///
    /// # Returns
    ///
    /// The arm, or a diagnostic.
    fn variant_arm_rest(&mut self, variant: Expr) -> PResult<SwitchArm>
    {
        do_if!(self.at(Kw::AsInto), {
            self.bump();
            self.expect(Kw::CanBecome)?;
        });
        self.expect(Kw::Then)?;
        let block = self.parse_switch_branch()?;
        let span = variant.span().to(&block.span);
        Ok(SwitchArm::builder().cases(vec![variant]).block(block).span(span).build())
    }

    /// Parse the rest of a switch arm after its first case: the cases as a list
    /// in words, then `တို့` when there are several, then `ဖြစ်လျှင် <block>`.
    ///
    /// # Arguments
    ///
    /// * `case` - The first case.
    ///
    /// # Returns
    ///
    /// The arm, or a diagnostic.
    fn parse_switch_arm(&mut self, case: Expr) -> PResult<SwitchArm>
    {
        let start = case.span();
        let cases = self.parse_word_list_from(case, Self::parse_expr)?;
        do_if!(cases.len() > 1, {
            self.expect(Kw::Plural)?;
        });
        self.expect(Kw::ThenIs)?;
        let block = self.parse_switch_branch()?;
        let span = start.to(&block.span);
        Ok(SwitchArm::builder().cases(cases).block(block).span(span).build())
    }

    /// Whether the sentence from here reaches `ဖြစ်လျှင်` before its full
    /// stop or a block.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn reaches_then_is(&self) -> bool
    {
        let ends = |tok: &zakac_lexer::token::Token| {
            matches!(
                tok.kind,
                TokenKind::Kw(
                    Kw::ThenIs | Kw::BlockOpen | Kw::UnsafeBlockOpen | Kw::BlockClose
                ) | TokenKind::Punct(Punct::FullStop)
                    | TokenKind::Eof
            )
        };
        let end =
            (0..).map(|n| self.peek_n(n)).find(ends).expect("the input ends with Eof");
        end.kind == TokenKind::Kw(Kw::ThenIs)
    }

    /// Parse `တခြား <kind> တစ်<classifier> ဖြစ်လျှင် <block>`, the default arm of
    /// a switch.
    ///
    /// # Returns
    ///
    /// The default arm, or a diagnostic.
    fn parse_switch_default(&mut self) -> PResult<SwitchDefault>
    {
        let start = self.bump().span;
        let kind = self.parse_slot_kind()?;
        self.expect(Kw::ThenIs)?;
        let block = self.parse_switch_branch()?;
        let span = start.to(&block.span);
        Ok(SwitchDefault::builder().kind(kind).block(block).span(span).build())
    }

    /// Parse the block of a switch arm. A single statement cannot be a
    /// conditional or another switch, so each arm after it belongs to one
    /// clear switch.
    ///
    /// # Returns
    ///
    /// The block, or a diagnostic.
    fn parse_switch_branch(&mut self) -> PResult<BlockStmt>
    {
        let block = self.parse_branch(false)?;
        let inner =
            block.single && matches!(block.stmts.first(), Some(Stmt::Switch(_)));
        do_if!(inner, self.diags.push(diag::nested_single_line_if(block.span)));
        Ok(block)
    }

    /// Whether a switch arm starts here: a sentence that reaches `ဖြစ်လျှင်`
    /// before its full stop, with no `သည်` before it (that starts a new
    /// switch) and no statement word at its start.
    ///
    /// # Returns
    ///
    /// `true` at an arm.
    fn at_switch_arm(&self) -> bool
    {
        let first = kw_of(&self.peek());
        let opener = first.is_some_and(|kw| {
            matches!(
                kw,
                Kw::If | Kw::ElseIf | Kw::Else | Kw::Other | Kw::Var | Kw::Const
            )
        });
        ret_if!(opener, false);
        let ends = |tok: &zakac_lexer::token::Token| {
            matches!(
                tok.kind,
                TokenKind::Kw(
                    Kw::ThenIs
                        | Kw::Thi
                        | Kw::BlockOpen
                        | Kw::UnsafeBlockOpen
                        | Kw::BlockClose
                        | Kw::ProgramClose
                ) | TokenKind::Punct(Punct::FullStop)
                    | TokenKind::Eof
            )
        };
        let end =
            (0..).map(|n| self.peek_n(n)).find(ends).expect("the input ends with Eof");
        end.kind == TokenKind::Kw(Kw::ThenIs) && self.peek().kind != end.kind
    }

    /// Parse `မျိုးပြပစ္စည်းမှာ <classifier> ဖြစ်သည်`. The classifier is a name,
    /// or the reserved `ကွက်`, which the range form uses as its count word.
    ///
    /// # Returns
    ///
    /// The classifier, or a diagnostic.
    fn parse_classifier(&mut self) -> PResult<Ident>
    {
        let word = self.parse_classifier_word()?;
        self.expect(Kw::Is)?;
        Ok(word)
    }

    /// Parse `မျိုးပြပစ္စည်းမှာ <classifier>`, the start of a classifier clause.
    ///
    /// # Returns
    ///
    /// The classifier, or a diagnostic.
    fn parse_classifier_word(&mut self) -> PResult<Ident>
    {
        self.expect(Kw::ClassifierIs)?;
        let cell = self.peek();
        let word = match kw_of(&cell)
        {
            Some(Kw::CellWord) =>
            {
                self.bump();
                Ident::builder().name(self.text_of(cell.span)).span(cell.span).build()
            }
            _ => self.parse_ident()?,
        };
        Ok(word)
    }

    /// Parse the result clause after `အောက်ပါတို့ကို လုပ်ဆောင်ကာ`: `<kind>
    /// တစ်<classifier> ကို ပြန်ပေးမည်`. `အပိုင်အနေဖြင့်` before `ပြန်ပေးမည်` gives
    /// the result as the program's own; the check allows it on a C function
    /// that gives back text.
    ///
    /// # Arguments
    ///
    /// * `has_ret` - Whether the opener announced a result.
    ///
    /// # Returns
    ///
    /// The clause, blank when not written, or a diagnostic.
    pub(crate) fn parse_ret_clause(&mut self, has_ret: bool) -> PResult<RetClause>
    {
        ret_if!(!has_ret, Ok(RetClause::default()));
        let kind = self.parse_slot_kind()?;
        self.expect(Kw::Ko)?;
        let owned = self.eat(Kw::AsOwned);
        self.expect(Kw::WillReturn)?;
        Ok(RetClause { kind: Some(kind), owned })
    }

    /// Whether a parameter starts here: `<name> ဟူသော`.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    fn at_param(&self) -> bool
    {
        let named = matches!(self.peek().kind, TokenKind::Ident | TokenKind::RawIdent);
        named && kw_of(&self.peek_n(1)) == Some(Kw::Called)
    }

    /// Parse the parameter clause of a header, when one is written: each
    /// parameter, then one of `ends`. With more than one parameter, each
    /// ends in `လည်းကောင်း`; either each has a preposition or none has. With
    /// `rest`, `အခြားတန်ဖိုးများနှင့်အတူ` can stand before the end word, after
    /// parameters that each end in `လည်းကောင်း`, as for a C function.
    ///
    /// # Arguments
    ///
    /// * `ends` - The words that can end it.
    /// * `rest` - Whether `အခြားတန်ဖိုးများနှင့်အတူ` can end the parameters.
    ///
    /// # Returns
    ///
    /// The parameters, in source order, the word that ended them, and
    /// whether the list ends in `အခြားတန်ဖိုးများနှင့်အတူ`; none when no parameter
    /// starts here; or a diagnostic.
    pub(crate) fn parse_params(
        &mut self, ends: &[Kw], rest: bool,
    ) -> PResult<ParamClause>
    {
        ret_if!(!self.at_param(), Ok((Vec::new(), None, false)));
        let outer = std::mem::replace(&mut self.in_list, true);
        let read = self.parse_param_items(ends, rest);
        self.in_list = outer;
        let (params, variadic) = read?;
        let end = self.expect_one_of(ends)?;
        self.check_preposition_places(&params);
        Ok((params, kw_of(&end), variadic))
    }

    /// Check where the prepositions of a header stand: a preposition that
    /// starts with `၏` prepositions the last parameter alone (ZK0249), and
    /// the first preposition is no `ကို သုံး၍`, which starts the
    /// arguments-first call (ZK0241).
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters of the header.
    fn check_preposition_places(&mut self, params: &[Param])
    {
        ret_if!(self.in_builtin);
        let before_last = params.iter().take(params.len().saturating_sub(1));
        let ei_prepositions = before_last
            .filter_map(|p| p.preposition.as_ref())
            .filter(|m| m.name.starts_with(EI));
        ei_prepositions.for_each(|m| {
            self.diags.push(diag::preposition_ei_not_last(m.span, &m.name))
        });
        let prepositions: Vec<&Ident> =
            params.iter().filter_map(|p| p.preposition.as_ref()).collect();
        let opens_using = |m: &&&Ident| {
            let words: Vec<&str> = m.name.split(' ').take(2).collect();
            matches!(words.as_slice(), ["ကို" | "တို့ကို", "သုံး၍"])
        };
        let first = prepositions.first().filter(opens_using);
        do_if!(let Some(m) = first, self.diags.push(diag::bad_preposition(m.span, &m.name)));
    }

    /// Parse the parameters of a parameter clause, with the list open, up
    /// to its end word or `အခြားတန်ဖိုးများနှင့်အတူ`.
    ///
    /// # Arguments
    ///
    /// * `ends` - The words that can end the clause.
    /// * `rest` - Whether `အခြားတန်ဖိုးများနှင့်အတူ` can end the parameters.
    ///
    /// # Returns
    ///
    /// The parameters and whether the list ends in `အခြားတန်ဖိုးများနှင့်အတူ`, or a
    /// diagnostic.
    fn parse_param_items(
        &mut self, ends: &[Kw], rest: bool,
    ) -> PResult<(Vec<Param>, bool)>
    {
        let mut params = Vec::new();
        let mut closed = Vec::new();
        loop
        {
            params.push(self.parse_param()?);
            closed.push(self.eat(Kw::AsWellAs));
            let variadic = rest && self.at(Kw::WithOtherValues);
            do_if!(variadic || ends.iter().any(|kw| self.at(*kw)), break);
            ret_if!(!closed[closed.len() - 1], Err(self.err_as_well_as()));
        }
        let variadic = rest && self.eat(Kw::WithOtherValues);
        let several = params.len() > 1 || variadic;
        ret_if!(several && closed.contains(&false), Err(self.err_as_well_as()));
        Ok((params, variadic))
    }

    /// Parse the markers of a parameter after its `ကို`, each one optional and
    /// in this order: `ဒေတာ ပြောင်းလဲရန်` or `ဒေတာ ထည့်ပေးရန်`, `အပိုင်အနေဖြင့်`,
    /// and `ဝိဘတ် "<preposition>" ကို သုံး၍`.
    ///
    /// # Arguments
    ///
    /// * `fill_ok` - Whether `ဒေတာ ထည့်ပေးရန်` can stand here: in a header, not in
    ///   a function kind.
    ///
    /// # Returns
    ///
    /// Whether the item is owned, whether the function can change it,
    /// whether it writes it alone, and its preposition; or a diagnostic.
    pub(crate) fn parse_item_markers(&mut self, fill_ok: bool) -> PResult<ItemMarkers>
    {
        let fill = fill_ok && self.eat(Kw::DataToFill);
        let edit = fill || self.eat(Kw::DataToChange);
        let owned = self.eat(Kw::AsOwned);
        ret_if!(!self.eat(Kw::PrepositionWord), Ok((owned, edit, fill, None)));
        let (preposition, words) = self.parse_words_string()?;
        do_if!(
            words.is_none(),
            self.diags.push(diag::bad_preposition(preposition.span, &preposition.name))
        );
        self.expect(Kw::Ko)?;
        self.expect(Kw::Using)?;
        Ok((owned, edit, fill, Some(preposition)))
    }

    /// Parse the string of a preposition or a phrase, read as its words.
    ///
    /// # Returns
    ///
    /// The words joined with one space (the text as written when it is no
    /// preposition), and the words, or `None` when the text is no
    /// preposition; or a diagnostic.
    fn parse_words_string(&mut self) -> PResult<(Ident, Option<Vec<String>>)>
    {
        let tok = self.peek();
        let TokenKind::Str(idx) = tok.kind
        else
        {
            return Err(self.err_expected(&t!("parser.preposition_string")));
        };
        self.bump();
        let text = self.strings[idx as usize].value.clone();
        let words = preposition_words(&text);
        let name = words.as_ref().map_or(text, |w| w.join(" "));
        Ok((Ident::builder().name(name).span(tok.span).build(), words))
    }

    /// Parse the phrases of a function head after `သည်`, when written, and
    /// the `သုံး၍` that closes them.
    ///
    /// # Returns
    ///
    /// The phrases, blank when none is written, or a diagnostic.
    pub(crate) fn parse_head_phrases(&mut self) -> PResult<Phrases>
    {
        let phrases = self.parse_phrase_list()?;
        do_if!(!phrases.is_empty(), _ = self.expect(Kw::Using)?);
        Ok(phrases)
    }

    /// Parse a list of phrases, when one is written: a starting phrase, an
    /// ending phrase, or both joined with `နှင့်`, as a list in words,
    /// `အစ စကားစု "<words>" နှင့် အဆုံး စကားစု "<words>" တို့ကို`; the word
    /// after it, `သုံး၍` or `သုံးသော`, is left to the caller.
    ///
    /// # Returns
    ///
    /// The phrases, blank when none is written, or a diagnostic.
    pub(crate) fn parse_phrase_list(&mut self) -> PResult<Phrases>
    {
        let start_first = self.at(Kw::StartPhrase);
        ret_if!(!start_first && !self.at(Kw::EndPhrase), Ok(Phrases::default()));
        let start = tern!(start_first, Some(self.parse_phrase(Kw::StartPhrase)?), None);
        let both = start_first && self.eat(Kw::Hnint);
        let end =
            tern!(!start_first || both, Some(self.parse_phrase(Kw::EndPhrase)?), None);
        self.expect_list_word(usize::from(both) + 1, Kw::Ko, Kw::ListKo)?;
        Ok(Phrases::builder().maybe_start(start).maybe_end(end).build())
    }

    /// Parse one phrase of a function head: its reserved word, then its
    /// string. A string that is no phrase, or that holds `၏`, is reported
    /// (ZK0258) and read all the same.
    ///
    /// # Arguments
    ///
    /// * `kw` - `အစ စကားစု` or `အဆုံး စကားစု`.
    ///
    /// # Returns
    ///
    /// The phrase, its words joined with one space, or a diagnostic.
    fn parse_phrase(&mut self, kw: Kw) -> PResult<Ident>
    {
        self.expect(kw)?;
        let (phrase, words) = self.parse_words_string()?;
        let ei = |w: &Vec<String>| w.iter().any(|w| w == EI);
        do_if!(
            words.is_none_or(|w| ei(&w)),
            self.diags.push(diag::bad_phrase(phrase.span, &phrase.name))
        );
        Ok(phrase)
    }

    /// Check the phrases of a function head against its parameters: a last
    /// preposition that starts with `၏` cannot stand before an ending phrase
    /// (ZK0249).
    ///
    /// # Arguments
    ///
    /// * `node` - The declaration.
    fn check_phrases(&mut self, node: &FnDecl)
    {
        ret_if!(node.phrases.is_empty());
        let last = node.params.last().and_then(|p| p.preposition.as_ref());
        let ei_last =
            last.filter(|m| node.phrases.end.is_some() && m.name.starts_with(EI));
        do_if!(let Some(m) = ei_last, self.diags.push(diag::preposition_ei_not_last(m.span, &m.name)));
    }

    /// Parse one parameter: `<name> ဟူသော <kind> တစ်<classifier> ကို`, then its
    /// markers.
    ///
    /// # Returns
    ///
    /// The parameter, or a diagnostic.
    fn parse_param(&mut self) -> PResult<Param>
    {
        let start = self.peek().span;
        let (kind, name) = self.parse_name_and_kind()?;
        self.expect(Kw::Ko)?;
        let (owned, edit, fill, preposition) = self.parse_item_markers(true)?;
        let span = start.to(&self.prev_span());
        #[rustfmt::skip]
        return Ok(Param::builder().owned(owned).edit(edit).fill(fill).kind(kind).name(name)
            .maybe_preposition(preposition).span(span).build());
    }

    /// Parse the name and the kind of a parameter or field,
    /// `<name> ဟူသော <kind> တစ်<classifier>`. A function kind with parameters
    /// that stands bare is reported and read all the same.
    ///
    /// # Returns
    ///
    /// The kind and the name, or a diagnostic.
    fn parse_name_and_kind(&mut self) -> PResult<(Kind, Ident)>
    {
        let name = self.parse_ident()?;
        self.expect(Kw::Called)?;
        let bare = self.at_bare_listed_kind();
        let kind = tern!(bare, self.parse_bare_listed_kind()?, self.parse_slot_kind()?);
        self.note_preposition_value(&name.name, &kind);
        Ok((kind, name))
    }

    /// Parse `ကွင်းဆက် ကို ရပ်မည်။`.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn parse_break(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        self.expect(Kw::Ko)?;
        self.expect(Kw::Stop)?;
        let stop = self.expect_stop()?;
        Ok(Stmt::Break(BreakStmt::builder().span(start.to(&stop.span)).build()))
    }

    /// Parse `နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။`.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn parse_continue(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        self.expect(Kw::Skip)?;
        let stop = self.expect_stop()?;
        Ok(Stmt::Continue(ContinueStmt::builder().span(start.to(&stop.span)).build()))
    }

    /// Parse `ပြန်ပေးမည်။`, or `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။`, which
    /// ends a function that can fail with a good result that holds no value.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn parse_bare_return(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        let stop = self.expect_stop()?;
        Ok(Stmt::Return(ReturnStmt::builder().span(start.to(&stop.span)).build()))
    }

    /// Parse `လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။`: the function ends with a
    /// failure that holds no error value.
    ///
    /// # Returns
    ///
    /// The return statement, or a diagnostic.
    fn parse_mark_failed(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        let stop = self.expect_stop()?;
        #[rustfmt::skip]
        return Ok(Stmt::Return(ReturnStmt::builder().fault(true).span(start.to(&stop.span)).build()));
    }
}

/// The closers a list of fields or members takes: `ပါဝင်သည်` after one,
/// `တို့ပါဝင်သည်` after several, each with its `ပြီး` form where a classifier
/// clause can follow.
///
/// # Arguments
///
/// * `count` - The number of fields or members in the list.
/// * `classifier_ok` - Whether a classifier clause can follow.
///
/// # Returns
///
/// The closers allowed.
fn member_closers(count: usize, classifier_ok: bool) -> &'static [Kw]
{
    match (count > 1, classifier_ok)
    {
        (true, true) => &[Kw::FieldsClose, Kw::FieldsCloseAnd],
        (true, false) => &[Kw::FieldsClose],
        (false, true) => &[Kw::Contains, Kw::ContainsAnd],
        (false, false) => &[Kw::Contains],
    }
}

/// The part an each loop walks, from the word before `ကို`.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// The part, or `None` for a word that names none.
fn each_part(kw: Kw) -> Option<EachPart>
{
    match kw
    {
        Kw::EachCell => Some(EachPart::Cell),
        Kw::EachChar => Some(EachPart::Char),
        Kw::EachIndex => Some(EachPart::Index),
        Kw::EachRow => Some(EachPart::Row),
        _ => None,
    }
}

/// The reserved words that can stand between `ဟူသော` and `လုပ်ဆောင်ချက်` in the
/// head of a function: `အများသုံးနိုင်သော`, `စိတ်မချရသော` and
/// `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော`.
pub(crate) const FN_MARKS: [Kw; 3] = [Kw::Export, Kw::Unsafe, Kw::Comptime];

/// Set the flag of one mark of a function head on the function. `အများသုံးနိုင်သော`
/// stands on a function at the top level alone (ZK0229), and
/// `ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော` on a function with statements at the
/// top level alone (ZK0256).
///
/// # Arguments
///
/// * `node` - The function.
/// * `mark` - The mark.
/// * `span` - Where the mark stands.
/// * `top` - Whether the function is declared at the top level.
///
/// # Returns
///
/// `Ok`, or the diagnostic of a misplaced mark.
fn mark_fn(node: &mut FnDecl, mark: Option<Kw>, span: Span, top: bool) -> PResult<()>
{
    match mark
    {
        Some(Kw::Export) if top => node.exported = true,
        Some(Kw::Export) => return Err(diag::export_not_on_decl(span)),
        Some(Kw::Unsafe) => node.unchecked = true,
        _ =>
        {
            let runnable = !node.external && !node.builtin && node.lib.is_none();
            ret_if!(!runnable || !top, Err(diag::comptime_mark_misplaced(span)));
            node.comptime = true;
        }
    }
    Ok(())
}
