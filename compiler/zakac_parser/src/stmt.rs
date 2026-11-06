use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::UsingMark;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::BreakStmt;
use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::ContinueStmt;
use zakac_ast::stmt::CountMode;
use zakac_ast::stmt::CountedStmt;
use zakac_ast::stmt::DeclStmt;
use zakac_ast::stmt::EachStmt;
use zakac_ast::stmt::EnumDecl;
use zakac_ast::stmt::EnumMember;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::GenericStmt;
use zakac_ast::stmt::IfArm;
use zakac_ast::stmt::IfStmt;
use zakac_ast::stmt::ImportStmt;
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
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::clause::ClauseOut;
use crate::clause::Mode;
use crate::decl_head::DeclHead;
use crate::describe::kw_text;
use crate::describe::quote;
use crate::notation::Args;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::kw_of;

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
            do_if!(let Some(stmt) = stmt, stmts.push(stmt));
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
            Kw::Function => self.parse_fn_or_call(),
            Kw::Program => self.parse_fn_decl(),
            Kw::StructOpen => self.parse_struct_decl(),
            Kw::EnumOpen => self.parse_enum_decl(),
            Kw::Export => self.parse_exported(),
            Kw::BlockOpen => self.parse_block().map(Stmt::Block),
            Kw::Loop => self.parse_break(),
            Kw::NextIteration => self.parse_continue(),
            Kw::Return | Kw::MarkSucceeded => self.parse_bare_return(),
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
        ret_if!(
            is_str && self.peek_n(1).kind == TokenKind::Kw(Kw::From),
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
            ClauseOut::Switch(subject, case) =>
            {
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

    /// Parse `"<file>" မှ <name> (၊ <name>)* (တို့ကို|ကို) ထည့်သွင်းပါ။`. An
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
        let mut names = vec![self.parse_ident()?];
        while self.eat_punct(Punct::ListSep)
        {
            names.push(self.parse_ident()?);
        }
        self.expect_one_of(&[Kw::ListKo, Kw::Ko])?;
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
    /// exported. The mark before another statement, or before a declaration
    /// inside a block, is reported.
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
            Stmt::Struct(n) => Some((&mut n.exported, &mut n.span)),
            Stmt::Enum(n) => Some((&mut n.exported, &mut n.span)),
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

    /// Whether `၏ အကွက်တိုင်း` or `၏ စာလုံးတိုင်း` comes next: the head of an
    /// each-cell or each-character loop.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    pub(crate) fn at_each_head(&self) -> bool
    {
        let word = kw_of(&self.peek_n(1));
        self.at_punct(Punct::Ei) && matches!(word, Some(Kw::EachCell | Kw::EachChar))
    }

    /// Parse `၏ အကွက်တိုင်း|စာလုံးတိုင်း ကို <name> အနေဖြင့် ယူ၍ <block>` after
    /// the list or the text an each-cell loop walks.
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
        let chars = self.bump().kind == TokenKind::Kw(Kw::EachChar);
        self.expect(Kw::Ko)?;
        let var = self.parse_ident()?;
        self.expect(Kw::CountAs)?;
        self.expect(Kw::Taking)?;
        let block = self.parse_branch(true)?;
        let span = source.span().to(&block.span);
        #[rustfmt::skip]
        let node = EachStmt::builder().source(source).chars(chars).var(var).block(block).span(span).build();
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
        ret_if!(self.at(Kw::BlockOpen), self.parse_block());
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

    /// Parse `အောက်ပါတို့ကို လုပ်ဆောင်ပါ။ <stmt>* လုပ်ဆောင်စရာများ ပြီးပါပြီ။`.
    ///
    /// # Returns
    ///
    /// The block, or a diagnostic.
    pub(crate) fn parse_block(&mut self) -> PResult<BlockStmt>
    {
        let start = self.expect(Kw::BlockOpen)?.span;
        self.expect_stop()?;
        let stmts = self.parse_stmts_to_close();
        let span = self.parse_block_close(start)?;
        Ok(BlockStmt::builder().stmts(stmts).span(span).build())
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
        let name = self.parse_ident()?;
        self.expect(Kw::Thi)?;
        let head = self.parse_decl_head()?;
        if self.at(Kw::Being)
        {
            ret_if!(is_const, Err(diag::const_loop(word.span)));
            return self.parse_generic_tail(word.span, name, head);
        }
        self.expect_one_of(&[Kw::Let, Kw::Being])?;
        let stop = self.expect_stop()?;
        let span = word.span.to(&stop.span);
        #[rustfmt::skip]
        let node = DeclStmt::builder().name(name).maybe_kind(head.kind).maybe_classifier(head.classifier)
            .is_const(is_const).maybe_init(head.init).span(span).build();
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
        let open = self.expect(Kw::BlockOpen)?;
        self.expect_stop()?;
        let (stmts, update) = self.parse_loop_stmts();
        let block_span = self.parse_block_close(open.span)?;
        let block = BlockStmt::builder().stmts(stmts).span(block_span).build();
        let span = start.to(&block_span);
        #[rustfmt::skip]
        let node = GenericStmt::builder().var(var).maybe_kind(head.kind)
            .maybe_classifier(head.classifier).maybe_init(head.init).cond(cond)
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

    /// Parse a function declaration, or a statement that starts with a call:
    /// `လုပ်ဆောင်ချက် f သည်` declares, `လုပ်ဆောင်ချက် f ကို` calls.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn parse_fn_or_call(&mut self) -> PResult<Stmt>
    {
        let is_decl = self.peek_n(2).kind == TokenKind::Kw(Kw::Thi);
        ret_if!(is_decl, self.parse_fn_decl());
        let (start, callee, args) = self.parse_call_head()?;
        let ends = [Kw::Call, Kw::WillDo, Kw::CallResult, Kw::CallGoodResult];
        let end = self.expect_one_of(&ends)?;
        ret_if!(
            matches!(kw_of(&end), Some(Kw::Call | Kw::WillDo)),
            self.finish_call_stmt(start, callee, args, None)
        );
        let pass = end.kind == TokenKind::Kw(Kw::CallGoodResult);
        let call = self.call_expr(start.to(&end.span), callee, args, pass);
        let clause = self.parse_clause_from(call, Mode::Stmt)?;
        self.finish_expr_headed(clause)
    }

    /// Finish a call statement after its end word: the pass-on
    /// `၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေး(ပါ|မည်)` when written, then the
    /// full stop.
    ///
    /// # Arguments
    ///
    /// * `start` - Where the statement starts.
    /// * `callee` - The function name.
    /// * `args` - The arguments, with the mark of each.
    /// * `using` - The particle that closes the arguments of a call written
    ///   arguments first.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    pub(crate) fn finish_call_stmt(
        &mut self, start: Span, callee: Ident, args: Args, using: Option<UsingMark>,
    ) -> PResult<Stmt>
    {
        let (args, edits) = args;
        let pass = self.at_punct(Punct::ListSep)
            && kw_of(&self.peek_n(1)) == Some(Kw::PassFault);
        do_if!(pass, {
            self.bump();
            self.bump();
        });
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        #[rustfmt::skip]
        let node = CallStmt::builder().callee(callee).args(args).edits(edits).pass(pass)
            .maybe_using(using).span(span).build();
        Ok(Stmt::Call(node))
    }

    /// Parse a function declaration:
    /// `လုပ်ဆောင်ချက် f သည် <params>? အောက်ပါတို့ကို လုပ်ဆောင်မည်။`, or with a
    /// result, `... အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> ရလဒ် ကို ပြန်ပေးမည်။`.
    /// The start block has the same parts after `ပရိုဂရမ်သည်`, with no name.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic. A declaration inside a block is
    /// parsed and reported.
    fn parse_fn_decl(&mut self) -> PResult<Stmt>
    {
        let head = self.bump();
        let start = head.span;
        let is_start = head.kind == TokenKind::Kw(Kw::Program);
        let start_name = || Ident::builder().name(START_NAME).span(start).build();
        let name = tern!(is_start, start_name(), self.parse_ident()?);
        self.expect(Kw::Thi)?;
        let params = self.parse_params()?;
        let open = self.expect_one_of(&[Kw::FnOpen, Kw::FnOpenRet])?;
        let ret = self.parse_ret_clause(open.kind == TokenKind::Kw(Kw::FnOpenRet))?;
        let fn_span = start.to(&self.prev_span());
        do_if!(self.depth > 0, self.diags.push(diag::nested_function(fn_span)));
        self.expect_stop()?;
        let stmts = self.parse_stmts_to_close();
        let block_span = self.parse_close(
            open.span,
            tern!(is_start, Kw::ProgramClose, Kw::BlockClose),
        )?;
        let block = BlockStmt::builder().stmts(stmts).span(block_span).build();
        let span = start.to(&block_span);
        #[rustfmt::skip]
        let node = FnDecl::builder().name(name).params(params).maybe_ret(ret).block(block)
            .start(is_start).span(span).build();
        Ok(Stmt::Fn(node))
    }

    /// Parse a struct declaration:
    /// `ဒေတာအမျိုးအစား <name> တွင် <kind> <field> (၊ <kind> <field>)* တို့ ပါဝင်သည်။`,
    /// or `တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ <classifier> ဖြစ်သည်။` at the end.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic. A declaration inside a block is
    /// parsed and reported.
    fn parse_struct_decl(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        let name = self.parse_ident()?;
        self.expect(Kw::At)?;
        let mut fields = vec![self.parse_param()?];
        while self.eat_punct(Punct::ListSep)
        {
            fields.push(self.parse_param()?);
        }
        let close = self.expect_one_of(&[Kw::FieldsClose, Kw::FieldsCloseAnd])?;
        let named = close.kind == TokenKind::Kw(Kw::FieldsCloseAnd);
        let classifier = tern!(named, Some(self.parse_classifier()?), None);
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        do_if!(self.depth > 0, self.diags.push(diag::nested_struct(span)));
        #[rustfmt::skip]
        let node = StructDecl::builder().name(name).fields(fields).maybe_classifier(classifier)
            .span(span).build();
        Ok(Stmt::Struct(node))
    }

    /// Parse an enumeration declaration:
    /// `အမျိုးအစား <name> ဆိုသည်မှာ <member> (၊ <member>)* တို့ ဖြစ်သည်။`, each
    /// member a name with an optional value in parentheses.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic. A declaration inside a block is
    /// parsed and reported.
    fn parse_enum_decl(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        let name = self.parse_ident()?;
        self.expect(Kw::Meaning)?;
        let mut members = vec![self.parse_enum_member()?];
        while self.eat_punct(Punct::ListSep)
        {
            members.push(self.parse_enum_member()?);
        }
        // one member reads `<member> ဖြစ်သည်`, with no `တို့`
        do_if!(members.len() > 1 || self.at(Kw::Plural), {
            self.expect(Kw::Plural)?;
        });
        self.expect(Kw::Is)?;
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        do_if!(self.depth > 0, self.diags.push(diag::nested_enum(span)));
        let node = EnumDecl::builder().name(name).members(members).span(span).build();
        Ok(Stmt::Enum(node))
    }

    /// Parse one member of an enumeration: `<name>`, or `<name> (<value>)`.
    ///
    /// # Returns
    ///
    /// The member, or a diagnostic.
    fn parse_enum_member(&mut self) -> PResult<EnumMember>
    {
        let name = self.parse_ident()?;
        let span = name.span;
        ret_if!(
            !self.at_punct(Punct::LParen),
            Ok(EnumMember::builder().name(name).span(span).build())
        );
        self.bump();
        let value = self.parse_expr()?;
        let close = self.expect_punct(Punct::RParen)?;
        let span = span.to(&close.span);
        Ok(EnumMember::builder().name(name).value(value).span(span).build())
    }

    /// Parse a switch after its head `<subject> သည် <case>`: the first
    /// `ဖြစ်လျှင်` and its block, each arm `<case> ဖြစ်လျှင် <block>` after it,
    /// and the default arm `တခြား <kind> ဖြစ်လျှင် <block>`. An arm after the
    /// default arm is parsed and reported.
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
            let case = self.parse_expr()?;
            arms.push(self.parse_switch_arm(case)?);
        }
        let default =
            tern!(self.at(Kw::Other), Some(self.parse_switch_default()?), None);
        while default.is_some() && self.at_switch_arm()
        {
            let case = self.parse_expr()?;
            self.diags.push(diag::arm_after_default(case.span()));
            self.parse_switch_arm(case)?;
        }
        let span = start.to(&self.prev_span());
        #[rustfmt::skip]
        let node = SwitchStmt::builder().subject(subject).arms(arms).maybe_default(default)
            .span(span).build();
        Ok(Stmt::Switch(node))
    }

    /// Parse the rest of a switch arm after its first case: each case after
    /// a `၊`, then `တို့` when there are several, then `ဖြစ်လျှင် <block>`.
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
        let mut cases = vec![case];
        while self.eat_punct(Punct::ListSep)
        {
            cases.push(self.parse_expr()?);
        }
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
                TokenKind::Kw(Kw::ThenIs | Kw::BlockOpen | Kw::BlockClose)
                    | TokenKind::Punct(Punct::FullStop)
                    | TokenKind::Eof
            )
        };
        let end =
            (0..).map(|n| self.peek_n(n)).find(ends).expect("the input ends with Eof");
        end.kind == TokenKind::Kw(Kw::ThenIs)
    }

    /// Parse `တခြား <kind> ဖြစ်လျှင် <block>`, the default arm of a switch.
    ///
    /// # Returns
    ///
    /// The default arm, or a diagnostic.
    fn parse_switch_default(&mut self) -> PResult<SwitchDefault>
    {
        let start = self.bump().span;
        let kind = self.parse_kind()?;
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

    /// Parse the result clause after `အောက်ပါတို့ကို လုပ်ဆောင်ကာ`: `<kind> ရလဒ်
    /// ကို ပြန်ပေးမည်`, or a result kind and `ကို ပြန်ပေးမည်` for a function that
    /// can fail.
    ///
    /// # Arguments
    ///
    /// * `has_ret` - Whether the opener announced a result.
    ///
    /// # Returns
    ///
    /// The result kind, `None` when not written, or a diagnostic.
    fn parse_ret_clause(&mut self, has_ret: bool) -> PResult<Option<Kind>>
    {
        ret_if!(!has_ret, Ok(None));
        let kind = self.parse_kind()?;
        do_if!(kind.base != BaseKind::Result, _ = self.expect(Kw::Result)?);
        self.expect(Kw::Ko)?;
        self.expect(Kw::WillReturn)?;
        Ok(Some(kind))
    }

    /// Parse the optional parameter clause:
    /// `kind name (၊ kind name)* (နှင့် kind name)? (တို့ကို|ကို) လက်ခံပြီး`.
    ///
    /// # Returns
    ///
    /// The parameters, none when no kind starts the clause, or a diagnostic.
    fn parse_params(&mut self) -> PResult<Vec<Param>>
    {
        ret_if!(!self.starts_kind() && !self.at(Kw::EditParam), Ok(Vec::new()));
        let mut params = vec![self.parse_param()?];
        while self.eat_punct(Punct::ListSep)
        {
            params.push(self.parse_param()?);
        }
        do_if!(self.eat(Kw::Hnint), params.push(self.parse_param()?));
        self.expect_one_of(&[Kw::ListKo, Kw::Ko])?;
        self.expect(Kw::Accepting)?;
        Ok(params)
    }

    /// Parse one parameter: `kind name`, with
    /// `လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော` before it for one the function can
    /// change.
    ///
    /// # Returns
    ///
    /// The parameter, or a diagnostic.
    fn parse_param(&mut self) -> PResult<Param>
    {
        let start = self.peek().span;
        let edit = self.eat(Kw::EditParam);
        let kind = self.parse_kind()?;
        let name = self.parse_ident()?;
        let span = start.to(&name.span);
        Ok(Param::builder().edit(edit).kind(kind).name(name).span(span).build())
    }

    /// Parse `ကွင်းဆက် ကို ရပ်ပါ။`.
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

    /// Parse `နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။`.
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

    /// Parse `ပြန်ပေးပါ။`, or `လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။`, which
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
