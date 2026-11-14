use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::kind::Kind;
use zakac_ast::stmt::AliasDecl;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::BreakStmt;
use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::ContinueStmt;
use zakac_ast::stmt::CountMode;
use zakac_ast::stmt::CountedStmt;
use zakac_ast::stmt::DeclStmt;
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
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::StrArg;
use zakac_ast::stmt::StructDecl;
use zakac_ast::stmt::SwitchArm;
use zakac_ast::stmt::SwitchDefault;
use zakac_ast::stmt::SwitchStmt;
use zakac_ast::stmt::WhileStmt;
use zakac_ast::visit::Node;
use zakac_diagnostics::parser as diag;
use zakac_i18n::t;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
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
use crate::parser::MAX_NEST;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::kw_of;
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
            self.imports_closed |= !matches!(stmt, Some(Stmt::Import(_)));
            let Some(mut stmt) = stmt
            else
            {
                continue;
            };
            // The functions of a trait block are functions of the file.
            let fns = match &mut stmt
            {
                Stmt::TraitBlock(b) => std::mem::take(&mut b.fns),
                _ => Vec::new(),
            };
            stmts.push(stmt);
            stmts.extend(fns.into_iter().map(Stmt::Fn));
        }
        stmts
    }

    /// Parse one statement, chosen from its first token.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    pub(crate) fn parse_stmt(&mut self) -> PResult<Stmt>
    {
        ret_if!(self.at_kind_decl(), self.parse_kind_decl());
        ret_if!(self.at_builtin_kind(), self.parse_builtin_kind());
        ret_if!(self.at_trait_decl(), self.parse_trait_decl());
        ret_if!(self.at_trait_block(), self.parse_trait_block());
        let tok = self.peek();
        let Some(kw) = kw_of(&tok)
        else
        {
            return self.parse_expr_headed();
        };
        match kw
        {
            Kw::If => self.parse_if(),
            Kw::ElseIf | Kw::Else =>
            {
                Err(diag::else_without_if(tok.span, self.text_of(tok.span)))
            }
            Kw::Var | Kw::Const => self.parse_decl_or_loop(),
            Kw::Function | Kw::Program => self.parse_fn_decl(),
            Kw::DataKindOpen => self.parse_alias_decl(),
            Kw::Export => self.parse_exported(),
            Kw::Unsafe => self.parse_unchecked(),
            Kw::BlockOpen | Kw::UnsafeBlockOpen => self.parse_block().map(Stmt::Block),
            Kw::Loop => self.parse_break(),
            Kw::NextIteration => self.parse_continue(),
            Kw::WillReturn | Kw::MarkSucceeded => self.parse_bare_return(),
            Kw::MarkFailed => self.parse_mark_failed(),
            Kw::AllocHead => self.parse_alloc(),
            Kw::Address => self.parse_heap(),
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
            ClauseOut::Expr(first) => first,
        };
        let cond = self.parse_logical_rest(first, 0)?;
        let is_while = self.at(Kw::While) || self.at(Kw::WhileIs);
        ret_if!(is_while, self.parse_while_tail(cond));
        ret_if!(self.at(Kw::From), self.parse_counted_tail(cond));
        ret_if!(self.at_each_head(), self.parse_each_tail(cond));
        Err(diag::expected_stmt_verb(self.peek().span, &self.found()))
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
        let late = self.depth > 0 || self.imports_closed;
        do_if!(late, self.diags.push(diag::import_not_first(span)));
        let lit = self.strings[idx as usize].clone();
        let path = StrArg::builder().lit(lit).span(tok.span).build();
        Ok(Stmt::Import(
            ImportStmt::builder().path(path).names(names).span(span).build(),
        ))
    }

    /// Parse a declaration after the export mark `အများသုံး` and mark it
    /// exported: a function, a variable, a constant or a kind alias. A struct
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
            Stmt::Fn(n) if !n.start => Some((&mut n.exported, &mut n.span)),
            Stmt::Alias(n) => Some((&mut n.exported, &mut n.span)),
            Stmt::Decl(n) => Some((&mut n.exported, &mut n.span)),
            _ => None,
        };
        let top = self.depth == 0;
        let Some((exported, span)) = flag.filter(|_| top)
        else
        {
            return Err(diag::export_not_on_decl(mark));
        };
        *exported = true;
        *span = mark.to(span);
        Ok(stmt)
    }

    /// Parse an unsafe function, `စိတ်မချရသော`: its statements can work with
    /// pointers, and a call of it needs unsafe code. The start block takes no
    /// such word.
    ///
    /// # Returns
    ///
    /// The function, or a diagnostic when no function follows.
    fn parse_unchecked(&mut self) -> PResult<Stmt>
    {
        let mark = self.bump().span;
        let mut stmt = self.parse_stmt()?;
        let Stmt::Fn(n) = &mut stmt
        else
        {
            return Err(diag::expected_token(mark, "လုပ်ဆောင်ချက်", "စိတ်မချရသော"));
        };
        ret_if!(n.start, Err(diag::expected_token(mark, "လုပ်ဆောင်ချက်", "စိတ်မချရသော")));
        n.unchecked = true;
        n.span = mark.to(&n.span);
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
        let mode = kw_of(&self.peek()).and_then(count_mode);
        let names = [Kw::Counting, Kw::Increasing, Kw::Decreasing];
        let mode = mode.ok_or_else(|| self.err_one_of(&names))?;
        self.bump();
        let block = self.parse_branch(true)?;
        let span = from.span().to(&block.span);
        #[rustfmt::skip]
        let node = CountedStmt::builder().var(var).from(from).to(to).maybe_step(step).mode(mode)
            .block(block).span(span).build();
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
        self.expect(Kw::Taking)?;
        let block = self.parse_branch(true)?;
        let span = source.span().to(&block.span);
        #[rustfmt::skip]
        let node = EachStmt::builder().source(source).part(part).maybe_place(place).var(var).block(block).span(span).build();
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
        let node = IfStmt::builder().arms(arms).maybe_else_block(else_block).span(span).build();
        Ok(Stmt::If(node))
    }

    /// Parse `(အကယ်၍|ထိုသို့မဟုတ်ဘဲ) <cond> (လျှင်|ဖြစ်လျှင်) <block>`.
    ///
    /// # Returns
    ///
    /// The arm, or a diagnostic.
    fn parse_if_arm(&mut self) -> PResult<IfArm>
    {
        let start = self.bump().span;
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
    fn parse_branch(&mut self, allow_if: bool) -> PResult<BlockStmt>
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
        let mut stmts = Vec::new();
        while !self.at_close() && !self.at_eof()
        {
            do_if!(let Some(stmt) = self.parse_stmt_recover(), stmts.push(stmt));
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
        let head = self.parse_decl_head()?;
        // A written kind gives the prepositions of a call through the name; the
        // starting value gives them when no kind is written.
        match (&head.kind, &head.init)
        {
            (Some(kind), _) => self.note_preposition_value(&name.name, kind),
            (None, Some(init)) => self.note_preposition_addr(&name.name, init),
            (None, None) => (),
        }
        if self.at(Kw::Being)
        {
            ret_if!(is_const, Err(diag::const_loop(word.span)));
            return self.parse_generic_tail(word.span, name, head);
        }
        self.expect_one_of(&[Kw::Let, Kw::Being])?;
        let stop = self.expect_stop()?;
        let span = word.span.to(&stop.span);
        #[rustfmt::skip]
        let node = DeclStmt::builder().name(name).maybe_kind(head.kind)
            .is_const(is_const).maybe_init(head.init).span(span).build();
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

    /// Parse `ဖြစ်ပြီး|ဖြစ်ကာ <cond> နေသရွေ့ <block>` after the header of a generic
    /// loop. The block can end with a `ပြီးလျှင်` update.
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
        let CallParts { start, callee, place, args, using, prepositions } = parts;
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
            .maybe_using(using).prepositions(prepositions).span(span).build();
        Ok(Stmt::Call(node))
    }

    /// Parse a function declaration:
    /// `လုပ်ဆောင်ချက် f သည် <params>? အောက်ပါတို့ကို လုပ်ဆောင်မည်။`, or with a
    /// result, `... အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> ရလဒ် ကို ပြန်ပေးမည်။`.
    /// The start block has the same parts after `ပရိုဂရမ်သည်`, with no name.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic. A start block inside a block is
    /// parsed and reported.
    pub(crate) fn parse_fn_decl(&mut self) -> PResult<Stmt>
    {
        let head = self.bump();
        let start = head.span;
        let is_start = head.kind == TokenKind::Kw(Kw::Program);
        let start_name = || Ident::builder().name(START_NAME).span(start).build();
        let name = tern!(is_start, start_name(), self.parse_ident()?);
        let mut kind_params = self.parse_notation_params()?;
        self.expect(Kw::Thi)?;
        if kind_params.is_empty() && self.at(Kw::GenericKind)
        {
            kind_params = self.parse_word_params()?;
            self.expect(Kw::Thi)?;
        }
        let external = self.at(Kw::External) || self.at_lib();
        ret_if!(external && !is_start, self.parse_external(start, name));
        if self.at(Kw::InCompiler) && !is_start
        {
            let added = self.add_kind_names(&kind_params);
            let parsed = self.parse_builtin_fn(start, name, kind_params);
            self.drop_kind_names(added);
            return parsed;
        }
        let added = self.add_kind_names(&kind_params);
        let parsed = self.parse_fn_rest(start, name, is_start, kind_params);
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
        mut kind_params: Vec<KindParam>,
    ) -> PResult<Stmt>
    {
        let (params, _, _) = self.parse_params(&[Kw::Accepting], false)?;
        let open = self.expect_one_of(&[Kw::BlockOpen, Kw::FnOpenRet])?;
        let RetClause { kind: ret, owned } =
            self.parse_ret_clause(open.kind == TokenKind::Kw(Kw::FnOpenRet))?;
        let fn_span = start.to(&self.prev_span());
        // A function inside a block borrows the locals around it; the start
        // block has its place at the top level alone.
        do_if!(
            self.depth > 0 && is_start,
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
    /// `လုပ်ဆောင်ချက် <name> သည်`: `ပြင်ပမှလာသော လုပ်ဆောင်ချက်(တစ်ခု)?ဖြစ်သည်။` with
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
        do_if!(self.depth > 0, self.diags.push(diag::nested_function(span)));
        let block = BlockStmt::builder().span(stop.span).build();
        #[rustfmt::skip]
        let node = FnDecl::builder().name(name).params(params).maybe_ret(ret).block(block)
            .external(true).maybe_lib(lib).variadic(variadic).owned_ret(owned)
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
        do_if!(self.depth > 0, self.diags.push(diag::nested_alias(span)));
        #[rustfmt::skip]
        let node = AliasDecl::builder().kind(kind).name(name).kind_params(kind_params)
            .maybe_classifier(classifier).span(span).build();
        Ok(Stmt::Alias(node))
    }

    /// Whether a struct or an enumeration declaration starts here:
    /// `<name> ဟူသော`, then the marks `အများသုံး`, `မျှဝေနိုင်သော` or
    /// `ပြင်ပမှ လာသော`, then `ဒေတာအစုအဝေး` or `ဒေတာအမျိုးအစားကွဲ`.
    ///
    /// # Returns
    ///
    /// `true` if one does.
    fn at_kind_decl(&self) -> bool
    {
        let named = matches!(self.peek().kind, TokenKind::Ident | TokenKind::RawIdent);
        let next = 1 + self.angle_len(1).unwrap_or(0);
        ret_if!(!named || kw_of(&self.peek_n(next)) != Some(Kw::Called), false);
        let marks = [Kw::Export, Kw::Shareable, Kw::External];
        let word = (next + 1..)
            .map(|n| kw_of(&self.peek_n(n)))
            .find(|kw| !kw.is_some_and(|kw| marks.contains(&kw)));
        matches!(word.flatten(), Some(Kw::StructWord | Kw::EnumWord))
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
        // `အများသုံး` stands after `ပြင်ပမှ လာသော`, right before the noun.
        let external = self.eat(Kw::External);
        let exported = self.eat(Kw::Export);
        let shared = !external && self.eat(Kw::Shareable);
        let word = tern!(
            external || shared,
            self.expect(Kw::StructWord)?,
            self.expect_one_of(&[Kw::StructWord, Kw::EnumWord])?
        );
        let mut stmt = match kw_of(&word)
        {
            Some(Kw::EnumWord) => self.parse_enum_decl(start, name)?,
            _ if external => self.parse_external_kind(start, name)?,
            _ => self.parse_struct_decl(start, name, kind_params)?,
        };
        match &mut stmt
        {
            Stmt::Struct(n) =>
            {
                n.exported = exported;
                n.shared |= shared;
            }
            Stmt::Enum(n) => n.exported = exported,
            _ => (),
        }
        Ok(stmt)
    }

    /// Parse a struct declaration after `<name> ဟူသော ... ဒေတာအစုအဝေး`:
    /// `တွင် <field>၊ ... နှင့် <field> တို့ပါဝင်သည်။`, or with one field
    /// `တွင် <field> ပါဝင်သည်။`; `…ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <classifier> ဖြစ်သည်။`
    /// at the end gives its classifier. Each field is
    /// `<name> ဟူသော <kind> တစ်<classifier>`.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `name` - The struct name.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic. A declaration inside a block is
    /// parsed and reported.
    fn parse_struct_decl(
        &mut self, start: Span, name: Ident, mut kind_params: Vec<KindParam>,
    ) -> PResult<Stmt>
    {
        if kind_params.is_empty() && self.eat(Kw::Thi)
        {
            kind_params = self.parse_word_params()?;
        }
        self.expect(Kw::At)?;
        let added = self.add_kind_names(&kind_params);
        let fields = self.parse_listed_fields(true);
        self.drop_kind_names(added);
        let fields = fields?;
        let (classifier, span) = self.parse_decl_end(start)?;
        do_if!(self.depth > 0, self.diags.push(diag::nested_struct(span)));
        #[rustfmt::skip]
        let node = StructDecl::builder().name(name).kind_params(kind_params).fields(fields)
            .maybe_classifier(classifier).span(span).build();
        Ok(Stmt::Struct(node))
    }

    /// Parse the end of a struct or an enumeration declaration after its
    /// closer: the classifier clause when the closer ends in `ပြီး`, then the
    /// full stop.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    ///
    /// # Returns
    ///
    /// The classifier, when written, and the span of the declaration; or a
    /// diagnostic.
    fn parse_decl_end(&mut self, start: Span) -> PResult<(Option<Ident>, Span)>
    {
        let and_kws =
            [TokenKind::Kw(Kw::ContainsAnd), TokenKind::Kw(Kw::FieldsCloseAnd)];
        let named = self.cur.back(1).is_some_and(|t| and_kws.contains(&t.kind));
        let classifier = tern!(named, Some(self.parse_classifier()?), None);
        let stop = self.expect_stop()?;
        Ok((classifier, start.to(&stop.span)))
    }

    /// Parse a kind that comes from outside the program, after
    /// `<name> ဟူသော ပြင်ပမှ လာသော [အများသုံး] ဒေတာအစုအဝေး`: `ဖြစ်သည်။`, or
    /// `တွင် <field>၊ ... နှင့် <field> တို့ပါဝင်သည်။` (one field: `ပါဝင်သည်`) with the
    /// fields of the C struct a value points at, from its start.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `name` - The kind name.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic.
    fn parse_external_kind(&mut self, start: Span, name: Ident) -> PResult<Stmt>
    {
        let with_fields =
            self.expect_one_of(&[Kw::Is, Kw::At])?.kind == TokenKind::Kw(Kw::At);
        let fields = tern!(with_fields, self.parse_listed_fields(false)?, Vec::new());
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        do_if!(self.depth > 0, self.diags.push(diag::nested_struct(span)));
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
    /// `<name> ဟူသော [အများသုံး] ဒေတာအမျိုးအစားကွဲ`:
    /// `တွင် <member>၊ ... နှင့် <member> တို့ပါဝင်သည်။`, or with one member
    /// `တွင် <member> ပါဝင်သည်။`; the `ပြီး` form of the closer, then
    /// `မျိုးပြပစ္စည်းမှာ <classifier> ဖြစ်သည်`, gives its classifier.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the declaration starts.
    /// * `name` - The enumeration name.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic. A declaration inside a block is
    /// parsed and reported.
    fn parse_enum_decl(&mut self, start: Span, name: Ident) -> PResult<Stmt>
    {
        self.expect(Kw::At)?;
        let members = self.parse_word_list(Self::parse_enum_member)?;
        self.expect_one_of(member_closers(members.len(), true))?;
        let (classifier, span) = self.parse_decl_end(start)?;
        do_if!(self.depth > 0, self.diags.push(diag::nested_enum(span)));
        #[rustfmt::skip]
        let node = EnumDecl::builder().name(name).members(members).maybe_classifier(classifier)
            .span(span).build();
        Ok(Stmt::Enum(node))
    }

    /// Parse one member of an enumeration:
    /// `[ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်] [တန်ဖိုးအနေဖြင့် <value> ရှိသည့်] <name>`.
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
                self.expect(Kw::HavingRel)?;
                Some(value)
            }
            false => None,
        };
        let name = self.parse_ident()?;
        let span = start.to(&name.span);
        #[rustfmt::skip]
        return Ok(EnumMember::builder().name(name).maybe_value(value).initial(initial)
            .span(span).build());
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
        let start = subject.span();
        let mut arms = vec![self.parse_switch_arm(case)?];
        while self.at_switch_arm()
        {
            let case = self.inside_word_list(Self::parse_expr)?;
            arms.push(self.parse_switch_arm(case)?);
        }
        let default =
            tern!(self.at(Kw::Other), Some(self.parse_switch_default()?), None);
        while default.is_some() && self.at_switch_arm()
        {
            let case = self.inside_word_list(Self::parse_expr)?;
            self.diags.push(diag::arm_after_default(case.span()));
            self.parse_switch_arm(case)?;
        }
        let span = start.to(&self.prev_span());
        #[rustfmt::skip]
        let node = SwitchStmt::builder().subject(subject).arms(arms).maybe_default(default)
            .span(span).build();
        Ok(Stmt::Switch(node))
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
        self.expect(Kw::Is)?;
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
        let given = params.iter().filter(|p| p.preposition.is_some()).count();
        let span = params[0].span.to(&params[params.len() - 1].span);
        do_if!(
            given > 0 && given < params.len(),
            self.diags.push(diag::mixed_prepositions(span))
        );
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
        let prepositions: Vec<&Ident> =
            params.iter().filter_map(|p| p.preposition.as_ref()).collect();
        let before_last =
            prepositions.iter().take(prepositions.len().saturating_sub(1));
        let ei_prepositions = before_last.filter(|m| m.name.starts_with(EI));
        ei_prepositions.for_each(|m| {
            self.diags.push(diag::preposition_ei_not_last(m.span, &m.name))
        });
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
        let tok = self.peek();
        let TokenKind::Str(idx) = tok.kind
        else
        {
            return Err(self.err_expected(&t!("parser.preposition_string")));
        };
        self.bump();
        let text = self.strings[idx as usize].value.clone();
        let words = preposition_words(&text);
        do_if!(
            words.is_none(),
            self.diags.push(diag::bad_preposition(tok.span, &text))
        );
        let text = words.map_or(text, |w| w.join(" "));
        self.expect(Kw::Ko)?;
        self.expect(Kw::Using)?;
        let preposition = Ident::builder().name(text).span(tok.span).build();
        Ok((owned, edit, fill, Some(preposition)))
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
