use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::FieldExpr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::IndexExpr;
use zakac_ast::expr::ListLit;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::VerbExpr;
use zakac_ast::kind::Kind;
use zakac_ast::stmt::AssignStmt;
use zakac_ast::stmt::PrintArg;
use zakac_ast::stmt::PrintStmt;
use zakac_ast::stmt::ReturnStmt;
use zakac_ast::stmt::Stmt;
use zakac_ast::verb::Dir;
use zakac_ast::verb::Slot;
use zakac_ast::verb::Verb;
use zakac_ast::verb::VerbOp;
use zakac_ast::verb::verb_for;
use zakac_ast::visit::Node;
use zakac_diagnostics::parser as diag;
use zakac_i18n::t;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::kw_of;

/// Whether statement verbs are allowed in a clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode
{
    /// Inside an expression: statement verbs are errors.
    Expr,
    /// At the start of a statement: a statement verb can close the clause.
    Stmt,
}

/// What a clause produced.
#[derive(Debug)]
pub(crate) enum ClauseOut
{
    /// A value.
    Expr(Expr),
    /// A statement, its full stop included.
    Stmt(Box<Stmt>),
}

/// A stack operand: a value, or the target kind of a conversion.
#[derive(Debug)]
enum Operand
{
    /// A value.
    Expr(Expr),
    /// `<kind>` in `<kind> အဖြစ်`.
    Kind(Kind),
}

impl Operand
{
    /// Source region of the operand.
    ///
    /// # Returns
    ///
    /// The span.
    fn span(&self) -> Span
    {
        match self
        {
            Operand::Expr(e) => e.span(),
            Operand::Kind(k) => k.span,
        }
    }
}

/// One item of the operand stack.
#[derive(Debug, Builder)]
struct Item
{
    /// The operand.
    operand: Operand,
    /// The particle after it and the particle span, once one is read.
    slot: Option<(Slot, Span)>,
}

impl Item
{
    /// An item for a value without a particle so far.
    ///
    /// # Arguments
    ///
    /// * `expr` - The value.
    ///
    /// # Returns
    ///
    /// The item.
    fn expr(expr: Expr) -> Self { Item::builder().operand(Operand::Expr(expr)).build() }
}

/// The clause particle a token spells.
///
/// The lexer calls more words particles than this: `မှ`, `ဖြစ်ပြီး` and the
/// list closers end a clause instead of marking an operand.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// The slot, or `None` for another token.
fn as_slot(tok: &Token) -> Option<Slot>
{
    match tok.kind
    {
        TokenKind::Punct(Punct::Ei) => Some(Slot::Ei),
        TokenKind::Kw(Kw::Hnint) => Some(Slot::Hnint),
        TokenKind::Kw(Kw::Ko) => Some(Slot::Ko),
        TokenKind::Kw(Kw::Hpjint) => Some(Slot::Hpjint),
        TokenKind::Kw(Kw::Thi) => Some(Slot::Thi),
        TokenKind::Kw(Kw::Htet) => Some(Slot::Htet),
        TokenKind::Kw(Kw::Auk) => Some(Slot::Auk),
        TokenKind::Kw(Kw::Hu) => Some(Slot::Hu),
        TokenKind::Kw(Kw::Ordinal) => Some(Slot::Ordinal),
        _ => None,
    }
}

/// The shift direction a token spells.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// The direction, or `None` for another token.
fn as_shift_dir(tok: &Token) -> Option<Dir>
{
    match kw_of(tok)?
    {
        Kw::ShiftLeft => Some(Dir::Left),
        Kw::ShiftRight => Some(Dir::Right),
        _ => None,
    }
}

/// The valence table row of a verb token.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// The row, or `None` for another token.
fn as_verb(tok: &Token) -> Option<&'static Verb> { kw_of(tok).and_then(verb_for) }

/// The value of an operand whose slot is not the conversion slot.
///
/// # Arguments
///
/// * `item` - The item; [`Parser::check_slots`] has passed.
///
/// # Returns
///
/// The value.
fn into_expr(item: Item) -> Expr
{
    extract!(Operand::Expr(expr) = item.operand, "slots are checked before");
    expr
}

impl Parser<'_>
{
    /// Parse a clause that starts with a notation expression.
    ///
    /// # Arguments
    ///
    /// * `mode` - Whether a statement verb can close the clause.
    ///
    /// # Returns
    ///
    /// A value or a statement, or a diagnostic.
    pub(crate) fn parse_clause(&mut self, mode: Mode) -> PResult<ClauseOut>
    {
        let first = self.parse_operand()?;
        self.parse_clause_from(first, mode)
    }

    /// Parse a clause from a first operand parsed before.
    ///
    /// # Arguments
    ///
    /// * `first` - The first operand.
    /// * `mode` - Whether a statement verb can close the clause.
    ///
    /// # Returns
    ///
    /// A value or a statement, or a diagnostic.
    pub(crate) fn parse_clause_from(
        &mut self, first: Expr, mode: Mode,
    ) -> PResult<ClauseOut>
    {
        let mut stack = vec![Item::expr(first)];
        loop
        {
            let tok = self.peek();
            if let Some(slot) = as_slot(&tok)
            {
                self.attach_slot(&mut stack, slot)?;
                continue;
            }
            if let Some(dir) = as_shift_dir(&tok)
            {
                self.push_shift(&mut stack, dir)?;
                continue;
            }
            if tok.kind == TokenKind::Kw(Kw::Cell) && holds_ei(&stack)
            {
                self.push_index(&mut stack)?;
                continue;
            }
            let Some(verb) = as_verb(&tok)
            else
            {
                return self.finish_clause(stack);
            };
            ret_if!(let Some(stmt) = self.reduce(&mut stack, verb, mode)?, Ok(ClauseOut::Stmt(Box::new(stmt))));
        }
    }

    /// Attach the current particle to the top operand, then push the operand
    /// that follows it.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack; never blank.
    /// * `slot` - The particle.
    ///
    /// # Returns
    ///
    /// `Ok`, or a `double-particle` diagnostic.
    fn attach_slot(&mut self, stack: &mut Vec<Item>, slot: Slot) -> PResult<()>
    {
        let tok = self.peek();
        let top = stack.last_mut().expect("the stack holds the first operand");
        if let Some((_, first)) = top.slot
        {
            return Err(diag::double_particle(tok.span, self.text_of(first)));
        }
        top.slot = Some((slot, tok.span));
        self.bump();
        self.push_next_operand(stack, slot)
    }

    /// Push the operand after a particle, unless a verb or a shift direction
    /// comes first. After `ကို`, `<kind> အဖြစ်` is the target of a
    /// conversion.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack.
    /// * `slot` - The particle just read.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic from the operand.
    fn push_next_operand(&mut self, stack: &mut Vec<Item>, slot: Slot) -> PResult<()>
    {
        if slot == Slot::Ko && self.starts_kind() && self.kind_then(Kw::As)
        {
            let kind = self.parse_kind()?;
            let as_tok = self.bump();
            let item = Item::builder().operand(Operand::Kind(kind));
            stack.push(item.slot((Slot::KindAhpjit, as_tok.span)).build());
            return Ok(());
        }
        let tok = self.peek();
        ret_if!(as_verb(&tok).is_some() || as_shift_dir(&tok).is_some(), Ok(()));
        let first = self.text_of(self.prev_span());
        ret_if!(as_slot(&tok).is_some(), Err(diag::double_particle(tok.span, first)));
        let expr = self.parse_operand()?;
        let is_field = slot == Slot::Ei && !self.at(Kw::Ordinal);
        let item = match expr
        {
            Expr::Name(n) if is_field => self.field_of(stack, n),
            expr => expr,
        };
        stack.push(Item::expr(item));
        Ok(())
    }

    /// Take the top operand, which holds `၏`, as the struct of a field.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack.
    /// * `name` - The field name, read as a variable.
    ///
    /// # Returns
    ///
    /// The field access.
    fn field_of(&mut self, stack: &mut Vec<Item>, name: NameExpr) -> Expr
    {
        let base = into_expr(stack.pop().expect("the top operand holds ၏"));
        let span = base.span().to(&name.span);
        let field = Ident::builder().name(name.name).span(name.span).build();
        let id = self.new_id();
        let node = FieldExpr::builder().id(id).base(base).field(field).span(span);
        Expr::Field(node.build())
    }

    /// Parse an operand of a clause: a struct literal, a notation
    /// expression, or a list literal that starts with one.
    ///
    /// # Returns
    ///
    /// The operand, or a diagnostic.
    fn parse_operand(&mut self) -> PResult<Expr>
    {
        ret_if!(self.at_struct_lit(), self.parse_struct_lit());
        let first = self.parse_notation()?;
        ret_if!(!self.at_list_rest(), Ok(first));
        self.parse_list(first)
    }

    /// Whether the rest of a list literal comes next: `တို့ပါသော`, or `၊` with
    /// `တို့ပါသော` ahead of the next `တို့ဖြင့်`, `တို့ကို`, `)` or `။` outside
    /// parentheses.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_list_rest(&self) -> bool
    {
        ret_if!(self.at_list_close(0), true);
        ret_if!(!self.at_punct(Punct::ListSep), false);
        self.close_ahead(1, Self::at_list_close)
    }

    /// Whether `close` holds for a token from `from` on, outside parentheses,
    /// before the next `တို့ဖြင့်`, `တို့ကို`, `)` or `။`.
    ///
    /// # Arguments
    ///
    /// * `from` - How far ahead the scan starts.
    /// * `close` - The test, given the parser and how far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it holds.
    pub(crate) fn close_ahead(
        &self, from: usize, close: fn(&Self, usize) -> bool,
    ) -> bool
    {
        let mut parens = 0usize;
        for n in from..
        {
            let kind = self.peek_n(n).kind;
            ret_if!(parens == 0 && close(self, n), true);
            ret_if!(parens == 0 && ends_list_scan(kind), false);
            match kind
            {
                TokenKind::Punct(Punct::LParen) => parens += 1,
                TokenKind::Punct(Punct::RParen) => parens -= 1,
                _ => (),
            }
        }
        false
    }

    /// Whether `တို့ပါသော` is the token `n` ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_list_close(&self, n: usize) -> bool
    {
        self.peek_n(n).kind == TokenKind::Kw(Kw::ListOf)
    }

    /// Parse the rest of a list literal after its first cell:
    /// `(၊ cell)* တို့ပါသော စာရင်း`.
    ///
    /// # Arguments
    ///
    /// * `first` - The first cell.
    ///
    /// # Returns
    ///
    /// The literal, or a diagnostic.
    fn parse_list(&mut self, first: Expr) -> PResult<Expr>
    {
        let mut cells = vec![first];
        while self.eat_punct(Punct::ListSep)
        {
            cells.push(self.parse_notation()?);
        }
        self.expect(Kw::ListOf)?;
        let end = self.expect(Kw::List)?;
        let span = cells[0].span().to(&end.span);
        let id = self.new_id();
        Ok(Expr::List(ListLit::builder().id(id).cells(cells).span(span).build()))
    }

    /// Replace the top operand, which holds `၏`, with its cell
    /// `အကွက် <index>`, counted from 0. The cell word is current.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic from the index.
    fn push_index(&mut self, stack: &mut Vec<Item>) -> PResult<()>
    {
        self.bump();
        let index = self.parse_notation()?;
        let list = into_expr(stack.pop().expect("the top operand holds ၏"));
        let span = list.span().to(&index.span());
        stack.push(Item::expr(self.index_expr(list, index, 0, span)));
        Ok(())
    }

    /// Build a list cell node.
    ///
    /// # Arguments
    ///
    /// * `list` - The list, or a pointer to one.
    /// * `index` - The cell number.
    /// * `first` - The number of the first cell: 0 or 1.
    /// * `span` - The whole cell expression.
    ///
    /// # Returns
    ///
    /// The expression.
    fn index_expr(&mut self, list: Expr, index: Expr, first: u8, span: Span) -> Expr
    {
        let id = self.new_id();
        #[rustfmt::skip]
        let node = IndexExpr::builder().id(id).list(list).index(index).first(first).span(span).build();
        Expr::Index(node)
    }

    /// Push `ဘယ်သို့|ညာသို့ <distance> ဘစ်`; the direction word is current.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack.
    /// * `dir` - The direction.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic.
    fn push_shift(&mut self, stack: &mut Vec<Item>, dir: Dir) -> PResult<()>
    {
        self.bump();
        let dist = self.parse_notation()?;
        let bit = self.expect(Kw::Bit)?;
        let item = Item::builder().operand(Operand::Expr(dist));
        stack.push(item.slot((Slot::Bit(dir), bit.span)).build());
        Ok(())
    }

    /// Reduce the operand stack on a verb, which is the current token.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack.
    /// * `verb` - The verb row.
    /// * `mode` - Whether a statement verb is allowed.
    ///
    /// # Returns
    ///
    /// The statement for a statement verb, `None` after pushing a value, or
    /// a diagnostic.
    fn reduce(
        &mut self, stack: &mut Vec<Item>, verb: &'static Verb, mode: Mode,
    ) -> PResult<Option<Stmt>>
    {
        let tok = self.peek();
        let name = self.text_of(tok.span);
        let in_expr = verb.is_stmt && mode == Mode::Expr;
        ret_if!(in_expr, Err(diag::stmt_verb_in_expr(tok.span, name)));
        let need = verb.slots.len();
        let have = stack.len();
        ret_if!(have < need, Err(diag::missing_operand(tok.span, name, need, have)));
        let args = stack.split_off(have - need);
        self.check_slots(&args, verb, name)?;
        self.check_non_assoc(&args, verb, tok)?;
        self.bump();
        ret_if!(verb.is_stmt, self.build_stmt(stack, args, verb, tok).map(Some));
        let expr = self.build_verb_expr(args, verb, tok);
        stack.push(Item::expr(expr));
        Ok(None)
    }

    /// Check that each operand carries the particle its verb wants.
    ///
    /// # Arguments
    ///
    /// * `args` - The operands, in source order.
    /// * `verb` - The verb row.
    /// * `name` - The verb text.
    ///
    /// # Returns
    ///
    /// `Ok`, or a `slot-mismatch` diagnostic for the first wrong operand.
    fn check_slots(&self, args: &[Item], verb: &Verb, name: &str) -> PResult<()>
    {
        let wrong =
            args.iter().zip(verb.slots).enumerate().find(|(_, (item, want))| {
                !item.slot.is_some_and(|(slot, _)| slot.fills(**want))
            });
        let Some((idx, (item, want))) = wrong
        else
        {
            return Ok(());
        };
        let found = item.slot.map(|(_, span)| self.text_of(span).to_string());
        let found = found.unwrap_or_else(|| t!("parser.no_particle").to_string());
        let span = item.operand.span();
        Err(diag::slot_mismatch(span, name, idx + 1, want.as_ref(), &found))
    }

    /// Reject a comparison whose operand is an unparenthesized comparison.
    ///
    /// # Arguments
    ///
    /// * `args` - The operands.
    /// * `verb` - The verb row.
    /// * `tok` - The verb token.
    ///
    /// # Returns
    ///
    /// `Ok`, or a `chained-comparison` diagnostic.
    fn check_non_assoc(&self, args: &[Item], verb: &Verb, tok: Token) -> PResult<()>
    {
        let is_cmp = |e: &Item| matches!(&e.operand, Operand::Expr(x) if x.is_cmp());
        ret_if!(!verb.is_cmp || !args.iter().any(is_cmp), Ok(()));
        Err(diag::chained_comparison(args[0].operand.span().to(&tok.span)))
    }

    /// Build the value of a verb clause.
    ///
    /// # Arguments
    ///
    /// * `args` - The operands; their slots are checked.
    /// * `verb` - The verb row.
    /// * `tok` - The verb token.
    ///
    /// # Returns
    ///
    /// The expression.
    fn build_verb_expr(&mut self, args: Vec<Item>, verb: &Verb, tok: Token) -> Expr
    {
        let span = args[0].operand.span().to(&tok.span);
        if verb.op == VerbOp::Cell
        {
            let [list, index] = <[Item; 2]>::try_from(args).expect("slots are checked");
            return self.index_expr(into_expr(list), into_expr(index), 1, span);
        }
        let dir = args.iter().find_map(|e| match e.slot
        {
            Some((Slot::Bit(dir), _)) => Some(dir),
            _ => None,
        });
        let (mut exprs, mut kind_arg) = (Vec::new(), None);
        for item in args
        {
            match item.operand
            {
                Operand::Expr(e) => exprs.push(e),
                Operand::Kind(k) => kind_arg = Some(k),
            }
        }
        let id = self.new_id();
        #[rustfmt::skip]
        let node = VerbExpr::builder().id(id).op(verb.op).args(exprs).maybe_kind_arg(kind_arg)
            .maybe_dir(dir).particle(tok.particle).is_cmp(verb.is_cmp).span(span).build();
        Expr::Verb(node)
    }

    /// Build the statement a statement verb closes, and read its full stop.
    ///
    /// # Arguments
    ///
    /// * `rest` - The operands left on the stack; must be blank.
    /// * `args` - The operands of the verb.
    /// * `verb` - The verb row.
    /// * `tok` - The verb token.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn build_stmt(
        &mut self, rest: &[Item], args: Vec<Item>, verb: &Verb, tok: Token,
    ) -> PResult<Stmt>
    {
        ret_if!(!rest.is_empty(), Err(self.dangling(rest, tok)));
        let stop = self.expect_stop()?;
        let span = args[0].operand.span().to(&stop.span);
        let slot = args[0].slot.map_or(Slot::Ko, |(slot, _)| slot);
        let mut args = args.into_iter().map(into_expr);
        let first = args.next().expect("statement verbs take operands");
        match verb.op
        {
            VerbOp::Assign => Ok(assign_stmt(first, args.next(), span)),
            VerbOp::Print => Ok(print_stmt(first, slot, span)),
            _ =>
            {
                Ok(Stmt::Return(ReturnStmt::builder().value(first).span(span).build()))
            }
        }
    }

    /// A `dangling-operands` diagnostic for operands left before a
    /// statement verb.
    ///
    /// # Arguments
    ///
    /// * `rest` - The left-over operands; not blank.
    /// * `tok` - The statement verb.
    ///
    /// # Returns
    ///
    /// The diagnostic.
    fn dangling(&self, rest: &[Item], tok: Token) -> zakac_diagnostics::Diagnostic
    {
        let span = rest[0].operand.span().to(&rest[rest.len() - 1].operand.span());
        diag::dangling_operands(
            span,
            self.text_of(tok.span),
            &self.describe_stack(rest),
        )
    }

    /// End a clause: one operand without a particle must remain.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack.
    ///
    /// # Returns
    ///
    /// The value, or an `unclosed-clause` diagnostic.
    fn finish_clause(&self, mut stack: Vec<Item>) -> PResult<ClauseOut>
    {
        let is_value = matches!(stack[0].operand, Operand::Expr(_));
        let lone = stack.len() == 1 && stack[0].slot.is_none() && is_value;
        ret_if!(lone, Ok(ClauseOut::Expr(into_expr(stack.remove(0)))));
        let top = &stack[stack.len() - 1];
        let last = top.slot.map_or(top.operand.span(), |(_, span)| span);
        let span = stack[0].operand.span().to(&last);
        Err(diag::unclosed_clause(span, &self.describe_stack(&stack)))
    }

    /// List stack operands with their particles, such as `က (နှင့်), ခ (၏)`.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operands.
    ///
    /// # Returns
    ///
    /// The list.
    fn describe_stack(&self, stack: &[Item]) -> String
    {
        let describe = |e: &Item| {
            let text = self.text_of(e.operand.span());
            let particle = e.slot.map_or_else(
                || t!("parser.no_particle").to_string(),
                |(_, span)| self.text_of(span).to_string(),
            );
            format!("{text} ({particle})")
        };
        stack.iter().map(describe).collect::<Vec<_>>().join(", ")
    }
}

/// Whether the top operand holds `၏`, so a cell word after it opens an index
/// counted from 0.
///
/// # Arguments
///
/// * `stack` - The operand stack.
///
/// # Returns
///
/// `true` if it does.
fn holds_ei(stack: &[Item]) -> bool
{
    matches!(stack.last().and_then(|item| item.slot), Some((Slot::Ei, _)))
}

/// Whether a token ends the scan for the rest of a list literal.
///
/// # Arguments
///
/// * `kind` - The token kind.
///
/// # Returns
///
/// `true` for `တို့ဖြင့်`, `တို့ကို`, `)`, `။` and the end of the file.
fn ends_list_scan(kind: TokenKind) -> bool
{
    matches!(
        kind,
        TokenKind::Kw(Kw::ListWith | Kw::ListKo)
            | TokenKind::Punct(Punct::RParen | Punct::FullStop)
            | TokenKind::Eof
    )
}

/// Build a print statement.
///
/// # Arguments
///
/// * `value` - The printed value.
/// * `slot` - The particle written after it.
/// * `span` - The statement.
///
/// # Returns
///
/// The statement.
fn print_stmt(value: Expr, slot: Slot, span: Span) -> Stmt
{
    let value = PrintArg::Expr(value);
    Stmt::Print(PrintStmt::builder().value(value).slot(slot).span(span).build())
}

/// Build an assignment. The parser takes each operand as a target; the
/// checker tells places from other values.
///
/// # Arguments
///
/// * `target` - The operand before `ကို`.
/// * `value` - The operand before `ဟု`.
/// * `span` - The statement.
///
/// # Returns
///
/// The statement.
fn assign_stmt(target: Expr, value: Option<Expr>, span: Span) -> Stmt
{
    let value = value.expect("assignment takes two operands");
    Stmt::Assign(AssignStmt::builder().target(target).value(value).span(span).build())
}
