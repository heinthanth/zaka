use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::kind::Kind;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::BreakStmt;
use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::ContinueStmt;
use zakac_ast::stmt::CountMode;
use zakac_ast::stmt::CountedStmt;
use zakac_ast::stmt::DeclStmt;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::GenericStmt;
use zakac_ast::stmt::IfArm;
use zakac_ast::stmt::IfStmt;
use zakac_ast::stmt::Param;
use zakac_ast::stmt::PrintArg;
use zakac_ast::stmt::PrintStmt;
use zakac_ast::stmt::ReturnStmt;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::StrArg;
use zakac_ast::stmt::WhileStmt;
use zakac_ast::verb::Slot;
use zakac_ast::visit::Node;
use zakac_diagnostics::parser as diag;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::clause::ClauseOut;
use crate::clause::Mode;
use crate::describe::kw_text;
use crate::describe::quote;
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
            do_if!(let Some(stmt) = self.parse_stmt_recover(), stmts.push(stmt));
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
            Kw::BlockOpen => self.parse_block().map(Stmt::Block),
            Kw::Loop => self.parse_break(),
            Kw::NextIteration => self.parse_continue(),
            Kw::Return => self.parse_bare_return(),
            Kw::Mem => self.parse_alloc(),
            Kw::WhenDone => Err(diag::update_outside_loop(tok.span)),
            Kw::BlockClose => Err(diag::unmatched_block_close(tok.span)),
            _ => self.parse_expr_headed(),
        }
    }

    /// Parse a statement that starts with an expression: a clause closed
    /// with a statement verb, a heap resize or release, a while loop or a
    /// counted loop.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn parse_expr_headed(&mut self) -> PResult<Stmt>
    {
        ret_if!(matches!(self.peek().kind, TokenKind::Str(_)), self.parse_print_str());
        let clause = self.parse_clause(Mode::Stmt)?;
        self.finish_expr_headed(clause)
    }

    /// Finish a statement that starts with a parsed clause: a resize or
    /// release of the address it names, a while loop or a counted loop.
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
            ClauseOut::Expr(first) => first,
        };
        ret_if!(self.at(Kw::ResizeHead), self.parse_resize_tail(first));
        ret_if!(self.at(Kw::FreeHead), self.parse_free_tail(first));
        let cond = self.parse_logical_rest(first, 0)?;
        let is_while = self.at(Kw::While) || self.at(Kw::WhileIs);
        ret_if!(is_while, self.parse_while_tail(cond));
        ret_if!(self.at(Kw::From), self.parse_counted_tail(cond));
        Err(diag::expected_stmt_verb(self.peek().span, &self.found()))
    }

    /// Parse `"..." (ကို|ဟု) ပြသပါ။`.
    ///
    /// # Returns
    ///
    /// The print statement, or a `str-outside-print` diagnostic.
    fn parse_print_str(&mut self) -> PResult<Stmt>
    {
        let tok = self.peek();
        let slot = match self.peek_n(1).kind
        {
            TokenKind::Kw(Kw::Ko) => Slot::Ko,
            TokenKind::Kw(Kw::Hu) => Slot::Hu,
            _ => return Err(diag::str_outside_print(tok.span)),
        };
        let printed = self.peek_n(2).kind == TokenKind::Kw(Kw::Print);
        ret_if!(!printed, Err(diag::str_outside_print(tok.span)));
        let TokenKind::Str(idx) = self.bump().kind
        else
        {
            unreachable!("the current token is a string");
        };
        self.bump();
        self.bump();
        let stop = self.expect_stop()?;
        let lit = self.strings[idx as usize].clone();
        let arg = PrintArg::Str(StrArg::builder().lit(lit).span(tok.span).build());
        let span = tok.span.to(&stop.span);
        Ok(Stmt::Print(PrintStmt::builder().value(arg).slot(slot).span(span).build()))
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
        while !self.at(Kw::BlockClose) && !self.at_eof()
        {
            do_if!(let Some(stmt) = self.parse_stmt_recover(), stmts.push(stmt));
        }
        self.depth -= 1;
        stmts
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
        self.expect(Kw::BlockClose)?;
        let stop = self.expect_stop()?;
        Ok(start.to(&stop.span))
    }

    /// Parse a declaration or a generic loop; both start with
    /// `(ကိန်းရှင်|ကိန်းသေ) <name> သည် <kind> <init>?`, where the value is
    /// written `ဖြစ်၍ တန်ဖိုးသည် <expr>` or right after the kind.
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
        let kind = self.parse_kind()?;
        let init = self.parse_init()?;
        if self.at(Kw::Being)
        {
            ret_if!(is_const, Err(diag::const_loop(word.span)));
            return self.parse_generic_tail(word.span, name, kind, init);
        }
        self.expect_one_of(&[Kw::Let, Kw::Being])?;
        let stop = self.expect_stop()?;
        let span = word.span.to(&stop.span);
        #[rustfmt::skip]
        let node = DeclStmt::builder().name(name).kind(kind).is_const(is_const).maybe_init(init)
            .span(span).build();
        Ok(Stmt::Decl(node))
    }

    /// Parse the initial value of a declaration, when one is written.
    ///
    /// # Returns
    ///
    /// The value, or `None` before `ဖြစ်ပါစေ` and `ဖြစ်ပြီး`.
    fn parse_init(&mut self) -> PResult<Option<Expr>>
    {
        ret_if!(self.at(Kw::Let) || self.at(Kw::Being), Ok(None));
        ret_if!(self.at(Kw::And), self.parse_value_clause().map(Some));
        self.parse_clause_expr().map(Some)
    }

    /// Parse `ဖြစ်၍ တန်ဖိုးသည် <expr>`; `ဖြစ်၍` is the current token.
    ///
    /// # Returns
    ///
    /// The value, or a diagnostic.
    fn parse_value_clause(&mut self) -> PResult<Expr>
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
    /// * `kind` - Its kind.
    /// * `init` - Its initial value.
    ///
    /// # Returns
    ///
    /// The loop, or a diagnostic.
    fn parse_generic_tail(
        &mut self, start: Span, var: Ident, kind: Kind, init: Option<Expr>,
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
        let node = GenericStmt::builder().var(var).kind(kind).maybe_init(init).cond(cond)
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
        while !self.at(Kw::BlockClose) && !self.at_eof()
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
        ret_if!(self.at(Kw::BlockClose) || self.at_eof());
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
        let end = self.expect_one_of(&[Kw::Call, Kw::CallResult])?;
        if end.kind == TokenKind::Kw(Kw::CallResult)
        {
            let call = self.call_expr(start.to(&end.span), callee, args);
            let clause = self.parse_clause_from(call, Mode::Stmt)?;
            return self.finish_expr_headed(clause);
        }
        let stop = self.expect_stop()?;
        let span = start.to(&stop.span);
        Ok(Stmt::Call(CallStmt::builder().callee(callee).args(args).span(span).build()))
    }

    /// Parse a function declaration:
    /// `လုပ်ဆောင်ချက် f သည် <params>? အောက်ပါတို့ကို လုပ်ဆောင်မည်။`, or with a
    /// result, `... အောက်ပါတို့ကို လုပ်ဆောင်ကာ <kind> ရလဒ် ကို ပြန်ပေးမည်။`.
    ///
    /// # Returns
    ///
    /// The declaration, or a diagnostic. A declaration inside a block is
    /// parsed and reported.
    fn parse_fn_decl(&mut self) -> PResult<Stmt>
    {
        let start = self.bump().span;
        let name = self.parse_ident()?;
        self.expect(Kw::Thi)?;
        let params = self.parse_params()?;
        let open = self.expect_one_of(&[Kw::FnOpen, Kw::FnOpenRet])?;
        let ret = self.parse_ret_clause(open.kind == TokenKind::Kw(Kw::FnOpenRet))?;
        let fn_span = start.to(&self.prev_span());
        do_if!(self.depth > 0, self.diags.push(diag::nested_function(fn_span)));
        self.expect_stop()?;
        let stmts = self.parse_stmts_to_close();
        let block_span = self.parse_block_close(open.span)?;
        let block = BlockStmt::builder().stmts(stmts).span(block_span).build();
        let span = start.to(&block_span);
        #[rustfmt::skip]
        let node = FnDecl::builder().name(name).params(params).maybe_ret(ret).block(block)
            .span(span).build();
        Ok(Stmt::Fn(node))
    }

    /// Parse `<kind> ရလဒ် ကို ပြန်ပေးမည်` after `အောက်ပါတို့ကို လုပ်ဆောင်ကာ`.
    ///
    /// # Arguments
    ///
    /// * `has_ret` - Whether the opener announced a result.
    ///
    /// # Returns
    ///
    /// The result kind, `None` without a result clause, or a diagnostic.
    fn parse_ret_clause(&mut self, has_ret: bool) -> PResult<Option<Kind>>
    {
        ret_if!(!has_ret, Ok(None));
        let kind = self.parse_kind()?;
        self.expect(Kw::Result)?;
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
        ret_if!(!self.starts_kind(), Ok(Vec::new()));
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

    /// Parse one parameter: `kind name`.
    ///
    /// # Returns
    ///
    /// The parameter, or a diagnostic.
    fn parse_param(&mut self) -> PResult<Param>
    {
        let kind = self.parse_kind()?;
        let name = self.parse_ident()?;
        let span = kind.span.to(&name.span);
        Ok(Param::builder().kind(kind).name(name).span(span).build())
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

    /// Parse `ပြန်ပေးပါ။`.
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
}
