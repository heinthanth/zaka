use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::FieldExpr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::IndexExpr;
use zakac_ast::expr::ListLit;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::RangeExpr;
use zakac_ast::expr::TableRow;
use zakac_ast::expr::VerbExpr;
use zakac_ast::kind::Kind;
use zakac_ast::stmt::AddStmt;
use zakac_ast::stmt::AssignStmt;
use zakac_ast::stmt::LeaveStmt;
use zakac_ast::stmt::PrintStmt;
use zakac_ast::stmt::RemoveStmt;
use zakac_ast::stmt::ReplaceStmt;
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
use zakac_lexer::token::Particle;
use zakac_lexer::token::Punct;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::do_while;
use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::parser::PResult;
use crate::parser::Parser;
use crate::parser::kw_of;
use crate::struct_lit::ONE;
use crate::using::CallParts;
use crate::using::Head;

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
    /// The head of a switch, `<subject> သည် <case>`, before its first
    /// `ဖြစ်လျှင်`.
    Switch(Expr, Expr),
}

/// A stack operand: a value, or the target kind of a conversion.
#[derive(Debug)]
enum Operand
{
    /// A value.
    Expr(Expr),
    /// `<kind>` in `<kind> အဖြစ်`.
    Kind(Kind),
    /// The range of text in `<range> ထိကို`, which a replace statement
    /// changes.
    Range(Box<TextRange>),
}

/// `<text> ၏ အကွက် <start> မှ <end>` or
/// `<text> ၏ <start> လုံးမြောက်စာလုံးမှ <end> လုံးမြောက်စာလုံး`, before `ထိကို`.
#[derive(Debug)]
struct TextRange
{
    /// The text.
    text: Expr,
    /// The first cell or character.
    start: Expr,
    /// The last cell or character.
    end: Expr,
    /// Whether the ends count characters from 1, not cells from 0.
    chars: bool,
    /// Source region, from the text to `ထိ`.
    span: Span,
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
            Operand::Range(r) => r.span,
        }
    }
}

/// The dup word: `<value> ၏ မိတ္တူ` copies the value.
const DUP_WORD: &str = "မိတ္တူ";

/// The end of `တစ်<classifier>လုံး`, "the whole of", which makes a duplicate deep.
const WHOLE: &str = "လုံး";

/// The possessive particle, as it ends a word it is joined to.
const EI: &str = "၏";

/// One item of the operand stack.
#[derive(Debug, Builder)]
struct Item
{
    /// The operand.
    operand: Operand,
    /// The particle after it and the particle span, once one is read.
    slot: Option<(Slot, Span)>,
    /// The classifier of the `တစ်<classifier>လုံး` after it: the whole of
    /// the value, so a duplicate of it is deep.
    whole: Option<Ident>,
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
        TokenKind::Kw(Kw::Into) => Some(Slot::Into),
        TokenKind::Kw(Kw::Within) => Some(Slot::Within),
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
pub(crate) fn as_verb(tok: &Token) -> Option<&'static Verb>
{
    kw_of(tok).and_then(verb_for)
}

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
        let first = match self.parse_head(mode)?
        {
            Head::Stmt(stmt) => return Ok(ClauseOut::Stmt(Box::new(stmt))),
            Head::Expr(first) => first,
        };
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
            // `၏ အကွက်တိုင်း` opens an each-cell loop; the operand before it is
            // what the loop walks.
            ret_if!(self.at_each_head(), self.finish_clause(stack));
            // `၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်` removes the last cell of the operand.
            let last = matches!(tok.kind, TokenKind::Kw(Kw::LastCell | Kw::LastChar));
            ret_if!(last && holds_ei(&stack), self.finish_remove(stack, mode));
            // `ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်` gives outside code leave to call the
            // operand.
            ret_if!(self.at_leave(), self.finish_leave(stack, mode));
            // `<cell> ၏ အရှေ့တွင်` names where an add puts its value.
            if tok.kind == TokenKind::Kw(Kw::Before) && holds_ei(&stack)
            {
                self.attach_before(&mut stack)?;
                continue;
            }
            let fill_end = self.in_fill && tok.kind == TokenKind::Kw(Kw::Hpjint);
            ret_if!(
                fill_end && stack.len() == 1 && stack[0].slot.is_none(),
                self.finish_clause(stack)
            );
            // `၏ မိတ္တူ` copies the operand; `တစ်<classifier>လုံး` before the `၏`
            // makes the duplicate deep.
            if holds_ei(&stack) && self.at_dup_word()
            {
                self.push_dup(&mut stack);
                continue;
            }
            // `ထဲမှ ဒေတာ` reads an optional or a result, and `ဟူသော အမှားအယွင်း`
            // makes an error value, of the operand before it.
            let bare = stack.last().is_some_and(|i| i.slot.is_none());
            if bare && self.at_row_read(0)
            {
                let table = into_expr(stack.pop().expect("an operand comes first"));
                stack.push(Item::expr(self.parse_row(table)?));
                continue;
            }
            if let Some(op) = self.data_read_at(0).filter(|_| bare)
            {
                self.push_postfix(&mut stack, op, 2);
                do_if!(
                    op == VerbOp::Unwrap && self.at_fallback(),
                    self.push_fallback(&mut stack)?
                );
                continue;
            }
            if bare && tok.kind == TokenKind::Kw(Kw::CalledFault)
            {
                self.push_postfix(&mut stack, VerbOp::MakeFault, 1);
                continue;
            }
            if let Some(glued) = self.whole_at()
            {
                self.mark_whole(&mut stack, glued)?;
                continue;
            }
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
            if tok.kind == TokenKind::Kw(Kw::CharKind) && holds_ei(&stack)
            {
                self.push_char_at(&mut stack)?;
                continue;
            }
            if tok.kind == TokenKind::Kw(Kw::CharKind) && holds_ordinal(&stack)
            {
                self.push_char_nth(&mut stack)?;
                continue;
            }
            let Some(verb) = as_verb(&tok)
            else
            {
                let at_arm = tok.kind == TokenKind::Kw(Kw::ThenIs)
                    || (tok.kind == TokenKind::Punct(Punct::ListSep)
                        && self.reaches_then_is());
                ret_if!(let Some(head) = switch_head(&mut stack, at_arm, mode), Ok(head));
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

    /// Whether the current token is the dup word `မိတ္တူ`, which after `၏`
    /// copies the operand; elsewhere it is a name.
    ///
    /// # Returns
    ///
    /// `true` for the dup word.
    fn at_dup_word(&self) -> bool
    {
        let tok = self.peek();
        tok.kind == TokenKind::Ident && self.text_of(tok.span) == DUP_WORD
    }

    /// Whether the current token is `တစ်<classifier>လုံး`, "the whole of", with
    /// the `၏` after it joined or not.
    ///
    /// # Returns
    ///
    /// Whether the `၏` is joined, or `None` for another token.
    fn whole_at(&self) -> Option<bool>
    {
        let tok = self.peek();
        ret_if!(tok.kind != TokenKind::Ident, None);
        whole_word(self.text_of(tok.span))
    }

    /// Mark the top operand as the whole of a value, for a deep duplicate; a
    /// joined `၏` is its particle too.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack.
    /// * `glued` - Whether the `၏` is joined to the word.
    ///
    /// # Returns
    ///
    /// `Ok`, or a `double-particle` diagnostic for an operand that holds a
    /// particle at this point.
    fn mark_whole(&mut self, stack: &mut Vec<Item>, glued: bool) -> PResult<()>
    {
        let tok = self.bump();
        let first = stack.last().and_then(|top| top.slot).map(|(_, at)| at);
        ret_if!(let Some(first) = first, Err(diag::double_particle(tok.span, self.text_of(first))));
        let top = stack.last_mut().expect("the stack holds the first operand");
        let word = self.text_of(tok.span);
        let classifier = whole_classifier(word).unwrap_or_default().to_string();
        top.whole = Some(Ident::builder().name(classifier).span(tok.span).build());
        do_if!(glued, top.slot = Some((Slot::Ei, tok.span)));
        Ok(())
    }

    /// Turn the top operand, which holds `၏`, into a duplicate of it at the dup
    /// word: a deep duplicate when it is marked as the whole of a value.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack.
    fn push_dup(&mut self, stack: &mut Vec<Item>)
    {
        let tok = self.bump();
        let item = stack.pop().expect("the top operand holds ၏");
        let op = tern!(item.whole.is_some(), VerbOp::DeepDupOf, VerbOp::DupOf);
        let classifier = item.whole.clone();
        let value = into_expr(item);
        let span = value.span().to(&tok.span);
        let copied = self.dup_expr(op, value, span, tok.particle);
        let copied = with_classifier(copied, classifier);
        stack.push(Item::expr(copied));
    }

    /// Read the words of a postfix operation and turn the top operand into it.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack; its top has no particle.
    /// * `op` - The operation.
    /// * `words` - The count of tokens that spell it.
    fn push_postfix(&mut self, stack: &mut Vec<Item>, op: VerbOp, words: usize)
    {
        let last =
            (0..words).map(|_| self.bump()).last().expect("an operation has words");
        let value = into_expr(stack.pop().expect("an operand comes first"));
        let span = value.span().to(&last.span);
        let made = self.dup_expr(op, value, span, last.particle);
        stack.push(Item::expr(made));
    }

    /// Turn the read of an optional on top of the stack into the read with a
    /// fallback, `၊ မရှိလျှင် <value>`, which comes next.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack; its top is the read.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic from the fallback.
    fn push_fallback(&mut self, stack: &mut Vec<Item>) -> PResult<()>
    {
        let read = into_expr(stack.pop().expect("the read comes first"));
        let optional = match read
        {
            Expr::Verb(v) =>
            {
                v.args.into_iter().next().expect("a read has its optional")
            }
            other => other,
        };
        stack.push(Item::expr(self.parse_fallback(optional)?));
        Ok(())
    }

    /// A duplicate of a value.
    ///
    /// # Arguments
    ///
    /// * `op` - [`VerbOp::DupOf`] or [`VerbOp::DeepDupOf`].
    /// * `value` - The value.
    /// * `span` - The duplicate.
    /// * `particle` - The particle after the dup word.
    ///
    /// # Returns
    ///
    /// The expression.
    pub(crate) fn dup_expr(
        &mut self, op: VerbOp, value: Expr, span: Span, particle: Particle,
    ) -> Expr
    {
        let id = self.new_id();
        #[rustfmt::skip]
        let node = VerbExpr::builder().id(id).op(op).args(vec![value]).particle(particle)
            .is_cmp(false).span(span).build();
        Expr::Verb(node)
    }

    /// Push the operand after a particle, unless a verb or a shift direction
    /// comes first. After `ကို`, `<kind> တစ်<classifier> အဖြစ်` is the target
    /// of a conversion.
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
        if slot == Slot::Ko && self.starts_kind() && self.slot_then(Kw::As)
        {
            let kind = self.parse_slot_kind()?;
            let as_tok = self.bump();
            let item = Item::builder().operand(Operand::Kind(kind));
            stack.push(item.slot((Slot::KindAhpjit, as_tok.span)).build());
            return Ok(());
        }
        let tok = self.peek();
        ret_if!(as_verb(&tok).is_some() || as_shift_dir(&tok).is_some(), Ok(()));
        let char_word = tok.kind == TokenKind::Kw(Kw::CharKind);
        ret_if!(char_word && matches!(slot, Slot::Ei | Slot::Ordinal), Ok(()));
        let after_ei =
            matches!(tok.kind, TokenKind::Kw(Kw::LastCell | Kw::LastChar | Kw::Before));
        ret_if!(after_ei && slot == Slot::Ei, Ok(()));
        ret_if!(slot == Slot::Ei && self.at_dup_word(), Ok(()));
        let first = self.text_of(self.prev_span());
        ret_if!(as_slot(&tok).is_some(), Err(diag::double_particle(tok.span, first)));
        let expr = self.parse_operand()?;
        let is_field = slot == Slot::Ei && !self.at(Kw::Ordinal);
        let item = match expr
        {
            Expr::Name(n) if is_field => self.field_of(stack, n),
            expr if is_field && reads_a_name(&expr) => self.read_of_field(stack, expr),
            expr => expr,
        };
        stack.push(Item::expr(item));
        Ok(())
    }

    /// Turn the `၏` of the top operand into `၏ အရှေ့တွင်`, then push the
    /// operand that follows it.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack; its top holds `၏`.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic from the operand.
    fn attach_before(&mut self, stack: &mut Vec<Item>) -> PResult<()>
    {
        let tok = self.bump();
        let top = stack.last_mut().expect("the top operand holds ၏");
        let (_, ei) = top.slot.expect("the top operand holds ၏");
        top.slot = Some((Slot::Before, ei.to(&tok.span)));
        self.push_next_operand(stack, Slot::Before)
    }

    /// Rebuild a read of the data of a field, `x ၏ f ထဲမှ ဒေတာ`, whose operand
    /// came back as the read of the bare name `f`: the field comes first,
    /// and each read wraps it.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack; its top holds `၏`.
    /// * `expr` - The reads around the field name.
    ///
    /// # Returns
    ///
    /// The reads of the field.
    fn read_of_field(&mut self, stack: &mut Vec<Item>, expr: Expr) -> Expr
    {
        let Expr::Verb(mut read) = expr
        else
        {
            let Expr::Name(n) = expr
            else
            {
                unreachable!("the reads end in a name");
            };
            return self.field_of(stack, n);
        };
        let inner = read.args.pop().expect("a read takes one operand");
        let field = self.read_of_field(stack, inner);
        read.span = field.span().to(&read.span);
        read.args.push(field);
        Expr::Verb(read)
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
    /// expression, a list literal that starts with one, or a call written
    /// with its arguments first or with no arguments.
    ///
    /// # Returns
    ///
    /// The operand, or a diagnostic.
    fn parse_operand(&mut self) -> PResult<Expr>
    {
        extract!(Head::Expr(expr) = self.parse_head(Mode::Expr)?);
        Ok(expr)
    }

    /// Parse the first operand of a clause, or a call written with its
    /// arguments first (`<args> ကို သုံး၍ ...`), with no arguments
    /// (`<fn> မည်`, `<fn> သည့် ရလဒ်`), or in the long form as a statement
    /// (`<fn> ဟူသော လုပ်ဆောင်ချက်ကို ... လုပ်ဆောင်မည်။`).
    ///
    /// # Arguments
    ///
    /// * `mode` - Whether a call statement can stand here.
    ///
    /// # Returns
    ///
    /// The operand or the call statement, or a diagnostic.
    fn parse_head(&mut self, mode: Mode) -> PResult<Head>
    {
        let first = match self.parse_first(mode)?
        {
            Head::Expr(first) => first,
            stmt => return Ok(stmt),
        };
        let first = match self.parse_short_after(first, mode)?
        {
            Head::Expr(first) => first,
            stmt => return Ok(stmt),
        };
        ret_if!(self.at_using_rest(), self.parse_using(first, mode));
        let end = self.marked_end(0);
        let Some(end) = end
        else
        {
            return Ok(Head::Expr(first));
        };
        self.attempt_marked(first, end, mode)
    }

    /// Parse the first operand of a clause. At the start of a statement a
    /// long call can end in `လုပ်ဆောင်မည်` and be the whole statement; each
    /// other operand is a plain one.
    ///
    /// # Arguments
    ///
    /// * `mode` - Whether a call statement can stand here.
    ///
    /// # Returns
    ///
    /// The operand or the call statement, or a diagnostic.
    fn parse_first(&mut self, mode: Mode) -> PResult<Head>
    {
        let stmt = mode == Mode::Stmt && self.at_long_call();
        ret_if!(!stmt, self.parse_plain_operand().map(Head::Expr));
        let (start, callee, place, args) = self.parse_call_head()?;
        #[rustfmt::skip]
        let parts = CallParts::builder().start(start).callee(callee).maybe_place(place)
            .args(args).build();
        self.finish_using(parts, true, mode)
    }

    /// Parse an operand of a clause that is no call: a struct literal, a
    /// notation expression, or a list literal or a join that starts with one.
    ///
    /// # Returns
    ///
    /// The operand, or a diagnostic.
    fn parse_plain_operand(&mut self) -> PResult<Expr>
    {
        ret_if!(self.at_struct_lit(), self.parse_struct_lit());
        let first = self.parse_notation()?;
        ret_if!(self.at_table_rest(), self.parse_table(first));
        ret_if!(self.at_join_rest(), self.parse_join(first));
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

    /// Whether the rest of a ဇယား literal comes next, after its first index:
    /// `အနေဖြင့်`, with `တို့ပါသော ဇယား` ahead of the next `တို့ဖြင့်`,
    /// `တို့ကို`, `)` or `။` outside parentheses.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_table_rest(&self) -> bool
    {
        let close = |p: &Self, n: usize| {
            p.at_list_close(n) && p.peek_n(n + 1).kind == TokenKind::Kw(Kw::Table)
        };
        self.peek().kind == TokenKind::Kw(Kw::CountAs) && self.close_ahead(1, close)
    }

    /// Parse the rest of a ဇယား literal after its first index:
    /// `အနေဖြင့် <value> (၊ <index> အနေဖြင့် <value>)* တို့ပါသော ဇယား`, and its kind
    /// when `ဟူသော <kind> တစ်<classifier>` follows.
    ///
    /// # Arguments
    ///
    /// * `first` - The first index.
    ///
    /// # Returns
    ///
    /// The literal, or a diagnostic.
    fn parse_table(&mut self, first: Expr) -> PResult<Expr>
    {
        let start = first.span();
        let mut rows = Vec::new();
        let mut index = first;
        loop
        {
            self.expect(Kw::CountAs)?;
            let value = self.parse_notation()?;
            rows.push(TableRow::builder().index(index).value(value).build());
            if !self.eat_punct(Punct::ListSep)
            {
                break;
            }
            index = self.parse_notation()?;
        }
        self.expect(Kw::ListOf)?;
        self.expect(Kw::Table)?;
        self.table_rest(start, rows)
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
            // An unclosed `(` leaves the depth above zero to the end of input.
            ret_if!(kind == TokenKind::Eof, false);
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

    /// Whether the rest of a verbal join comes next: `၊` with
    /// `တို့ကို ဆက်ထားသည့်` ahead of the next `တို့ဖြင့်`, `)` or `။` outside
    /// parentheses.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_join_rest(&self) -> bool
    {
        self.at_punct(Punct::ListSep) && self.close_ahead(1, Self::at_join_close)
    }

    /// Whether `တို့ကို ဆက်ထားသည့်` starts `n` tokens ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far ahead.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_join_close(&self, n: usize) -> bool
    {
        let joined = self.peek_n(n + 1).kind == TokenKind::Kw(Kw::Joined);
        self.peek_n(n).kind == TokenKind::Kw(Kw::ListKo) && joined
    }

    /// Parse the rest of a verbal join after its first piece:
    /// `(၊ <text>)+ တို့ကို ဆက်ထားသည့် စာသား`.
    ///
    /// # Arguments
    ///
    /// * `first` - The first piece.
    ///
    /// # Returns
    ///
    /// The join, or a diagnostic.
    fn parse_join(&mut self, first: Expr) -> PResult<Expr>
    {
        let mut pieces = vec![first];
        do_while!(self.eat_punct(Punct::ListSep), pieces.push(self.parse_notation()?));
        self.expect(Kw::ListKo)?;
        self.expect(Kw::Joined)?;
        let end = self.expect(Kw::TextKind)?;
        let span = pieces[0].span().to(&end.span);
        let id = self.new_id();
        #[rustfmt::skip]
        let node = VerbExpr::builder().id(id).op(VerbOp::Join).args(pieces)
            .particle(Particle::None).is_cmp(false).span(span).build();
        Ok(Expr::Verb(node))
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
    /// `(၊ cell)* တို့ပါသော စာရင်း`, and its kind when `ဟူသော <kind>
    /// တစ်<classifier>` follows.
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
        self.expect(Kw::List)?;
        let kind = tern!(self.at_literal_kind(), Some(self.parse_list_kind()?), None);
        let span = cells[0].span().to(&self.prev_span());
        let id = self.new_id();
        #[rustfmt::skip]
        return Ok(Expr::List(ListLit::builder().id(id).cells(cells).maybe_kind(kind).span(span).build()));
    }

    /// Replace the top operand, which holds `၏`, with its cell
    /// `အကွက် <index>`, counted from 0, or with its range
    /// `အကွက် <start> မှစ၍ <length> ကွက်`. The cell word is current.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic from the index or the range.
    fn push_index(&mut self, stack: &mut Vec<Item>) -> PResult<()>
    {
        self.bump();
        let index = self.parse_notation()?;
        let list = into_expr(stack.pop().expect("the top operand holds ၏"));
        if self.eat(Kw::StartingAt)
        {
            let len = self.parse_notation()?;
            self.expect(Kw::CellWord)?;
            let span = list.span().to(&self.prev_span());
            #[rustfmt::skip]
            let node = RangeExpr::builder().id(self.new_id()).source(list).start(index).len(len).span(span).build();
            stack.push(Item::expr(Expr::Range(node)));
            return Ok(());
        }
        ret_if!(self.at_range_end(), self.push_range(stack, list, index, false));
        let span = list.span().to(&index.span());
        stack.push(Item::expr(self.index_expr(list, index, 0, span)));
        Ok(())
    }

    /// Push `စာလုံး <number>` after an operand that holds `၏`: the character
    /// of text at that number, counted from 0.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack; its top holds `၏`.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic from the number.
    fn push_char_at(&mut self, stack: &mut Vec<Item>) -> PResult<()>
    {
        self.bump();
        let index = self.parse_notation()?;
        let text = into_expr(stack.pop().expect("the top operand holds ၏"));
        let span = text.span().to(&index.span());
        let node = self.char_verb(VerbOp::CharAt, text, index, span);
        stack.push(Item::expr(node));
        Ok(())
    }

    /// Push `စာလုံး` after `<text> ၏ <number> ခုမြောက်`: the character of
    /// the text at that number, counted from 1; or, with `မှ <end> ...
    /// ထိကို` next, the range of characters from that number.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack; its top holds `ခုမြောက်`, the operand under
    ///   it `၏`.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic from the range.
    fn push_char_nth(&mut self, stack: &mut Vec<Item>) -> PResult<()>
    {
        let word = self.bump();
        let index = into_expr(stack.pop().expect("the top operand holds ခုမြောက်"));
        let text = into_expr(stack.pop().expect("the operand under it holds ၏"));
        ret_if!(self.at_range_end(), self.push_range(stack, text, index, true));
        let span = text.span().to(&word.span);
        let node = self.char_verb(VerbOp::CharNth, text, index, span);
        stack.push(Item::expr(node));
        Ok(())
    }

    /// Whether `မှ` comes next and opens the range of a replace statement:
    /// a `ထိ` follows before the full stop. A counted loop `<cell> မှ <end>
    /// အထိ` has no `ထိ` of its own.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_range_end(&self) -> bool
    {
        ret_false!(!self.at(Kw::From));
        let ends = |k: &TokenKind| {
            matches!(
                k,
                TokenKind::Eof
                    | TokenKind::Punct(Punct::FullStop)
                    | TokenKind::Kw(Kw::To)
            )
        };
        let kinds = (1..).map(|n| self.peek_n(n).kind);
        kinds.take_while(|k| !ends(k)).any(|k| k == TokenKind::Kw(Kw::Upto))
    }

    /// Push the range of a replace statement, from its `မှ` to its `ထိကို`,
    /// then the value after it.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack.
    /// * `text` - The text the range is of.
    /// * `start` - The first cell or character.
    /// * `chars` - Whether the ends count characters from 1: the end is
    ///   `<number> လုံးမြောက်စာလုံး`.
    ///
    /// # Returns
    ///
    /// `Ok`, or a diagnostic.
    fn push_range(
        &mut self, stack: &mut Vec<Item>, text: Expr, start: Expr, chars: bool,
    ) -> PResult<()>
    {
        self.bump();
        let end = self.parse_notation()?;
        do_if!(chars, _ = self.expect(Kw::Ordinal)?);
        do_if!(chars, _ = self.expect(Kw::CharKind)?);
        let upto = self.expect(Kw::Upto)?;
        let ko = self.expect(Kw::Ko)?;
        let span = text.span().to(&upto.span);
        let range = TextRange { text, start, end, chars, span };
        let item = Item::builder().operand(Operand::Range(Box::new(range)));
        stack.push(item.slot((Slot::Upto, upto.span.to(&ko.span))).build());
        self.push_next_operand(stack, Slot::Upto)
    }

    /// Build a character read of text.
    ///
    /// # Arguments
    ///
    /// * `op` - `CharAt` or `CharNth`.
    /// * `text` - The text.
    /// * `index` - The character number.
    /// * `span` - The whole read.
    ///
    /// # Returns
    ///
    /// The expression.
    fn char_verb(&mut self, op: VerbOp, text: Expr, index: Expr, span: Span) -> Expr
    {
        let id = self.new_id();
        #[rustfmt::skip]
        let node = VerbExpr::builder().id(id).op(op).args(vec![text, index])
            .particle(Particle::None).is_cmp(false).span(span).build();
        Expr::Verb(node)
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
                Operand::Range(_) =>
                {
                    unreachable!("a range fills the slot of a statement verb alone")
                }
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
        ret_if!(verb.op == VerbOp::Replace, Ok(replace_stmt(args, span)));
        let slot = args[0].slot.map_or(Slot::Ko, |(slot, _)| slot);
        let mut args = args.into_iter().map(into_expr);
        let first = args.next().expect("statement verbs take operands");
        match verb.op
        {
            VerbOp::Assign => Ok(assign_stmt(first, args.next(), span)),
            VerbOp::DupInto | VerbOp::DeepDupInto =>
            {
                let value = args.next().expect("a duplicate takes two operands");
                let deep = verb.op == VerbOp::DeepDupInto;
                let op = tern!(deep, VerbOp::DeepDupOf, VerbOp::DupOf);
                let at = value.span();
                let copied = self.dup_expr(op, value, at, Particle::None);
                Ok(assign_stmt(first, Some(copied), span))
            }
            VerbOp::Print => Ok(print_stmt(first, slot, span)),
            VerbOp::RemoveRow =>
            {
                let node =
                    RemoveStmt::builder().list(first).row(true).span(span).build();
                Ok(Stmt::Remove(node))
            }
            VerbOp::AddCell =>
            {
                let value = args.next().expect("an add statement takes two operands");
                let before = slot == Slot::Before;
                #[rustfmt::skip]
                let node = AddStmt::builder().list(first).value(value).before(before).span(span).build();
                Ok(Stmt::Add(node))
            }
            _ =>
            {
                Ok(Stmt::Return(ReturnStmt::builder().value(first).span(span).build()))
            }
        }
    }

    /// Finish `<list> ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။` at its `နောက်ဆုံးအကွက်`, or
    /// `<text> ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။` at its `နောက်ဆုံးစာလုံး`, and read its
    /// full stop.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack; its top holds `၏`.
    /// * `mode` - Whether a statement can close the clause.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn finish_remove(&mut self, mut stack: Vec<Item>, mode: Mode)
    -> PResult<ClauseOut>
    {
        let tok = self.bump();
        let name = self.text_of(tok.span);
        ret_if!(mode == Mode::Expr, Err(diag::stmt_verb_in_expr(tok.span, name)));
        let chars = tok.kind == TokenKind::Kw(Kw::LastChar);
        let list = into_expr(stack.pop().expect("the top operand holds ၏"));
        self.expect(Kw::Ko)?;
        let verb = self.expect(Kw::Remove)?;
        ret_if!(!stack.is_empty(), Err(self.dangling(&stack, verb)));
        let stop = self.expect_stop()?;
        let span = list.span().to(&stop.span);
        let node = RemoveStmt::builder().list(list).chars(chars).span(span).build();
        Ok(ClauseOut::Stmt(Box::new(Stmt::Remove(node))))
    }

    /// Whether `ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်` or
    /// `ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်` starts here, after the value
    /// of a leave for code outside the program.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn at_leave(&self) -> bool
    {
        let kws = [1, 2, 3].map(|n| kw_of(&self.peek_n(n)));
        let verb = matches!(kws[2], Some(Kw::AllowCall | Kw::RevokeCall));
        self.at(Kw::Ko)
            && kws[0] == Some(Kw::Outside)
            && kws[1] == Some(Kw::From)
            && verb
    }

    /// Finish `<value> ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။` or
    /// `<value> ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်။` at its `ကို`, and
    /// read its full stop.
    ///
    /// # Arguments
    ///
    /// * `stack` - The operand stack: the value alone.
    /// * `mode` - Whether a statement can close the clause.
    ///
    /// # Returns
    ///
    /// The statement, or a diagnostic.
    fn finish_leave(&mut self, mut stack: Vec<Item>, mode: Mode) -> PResult<ClauseOut>
    {
        let ko = self.bump();
        let verb = [self.bump(), self.bump(), self.bump()][2];
        let name = self.text_of(verb.span);
        ret_if!(mode == Mode::Expr, Err(diag::stmt_verb_in_expr(verb.span, name)));
        let value = stack.pop().expect("an operand comes before ကို");
        ret_if!(
            !stack.is_empty() || value.slot.is_some(),
            Err(self.dangling(&stack, ko))
        );
        let value = into_expr(value);
        let stop = self.expect_stop()?;
        let span = value.span().to(&stop.span);
        let grant = verb.kind == TokenKind::Kw(Kw::AllowCall);
        let node = LeaveStmt::builder().value(value).grant(grant).span(span).build();
        Ok(ClauseOut::Stmt(Box::new(Stmt::Leave(node))))
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

/// The head of a switch, when a statement clause holds `<subject> သည်
/// <case>` and `ဖြစ်လျှင်` follows, or a `၊` before more cases.
///
/// # Arguments
///
/// * `stack` - The operand stack.
/// * `at_arm` - Whether the token after the clause goes on with an arm.
/// * `mode` - Whether the clause starts a statement.
///
/// # Returns
///
/// The head, or `None` with the stack unchanged.
fn switch_head(stack: &mut Vec<Item>, at_arm: bool, mode: Mode) -> Option<ClauseOut>
{
    let at_arm = mode == Mode::Stmt && at_arm;
    let shape = stack.len() == 2
        && matches!(stack[0].slot, Some((Slot::Thi, _)))
        && stack[1].slot.is_none()
        && stack.iter().all(|item| matches!(item.operand, Operand::Expr(_)));
    ret_if!(!at_arm || !shape, None);
    let case = into_expr(stack.pop()?);
    let subject = into_expr(stack.pop()?);
    Some(ClauseOut::Switch(subject, case))
}

/// Build a replace statement from its range and its value.
///
/// # Arguments
///
/// * `args` - The range, with `ထိကို`, and the value, with `ဖြင့်`.
/// * `span` - The whole statement.
///
/// # Returns
///
/// The statement.
fn replace_stmt(args: Vec<Item>, span: Span) -> Stmt
{
    let [range, value] = <[Item; 2]>::try_from(args).expect("slots are checked");
    extract!(Operand::Range(r) = range.operand, "the slot ထိကို holds a range");
    #[rustfmt::skip]
    let node = ReplaceStmt::builder().text(r.text).start(r.start).end(r.end).chars(r.chars)
        .value(into_expr(value)).span(span).build();
    Stmt::Replace(node)
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

/// Whether the top operand holds `ခုမြောက်` and the operand under it `၏`,
/// so a character word reads the character at that number.
///
/// # Arguments
///
/// * `stack` - The operand stack.
///
/// # Returns
///
/// `true` when the stack does.
fn holds_ordinal(stack: &[Item]) -> bool
{
    let [.., under, top] = stack
    else
    {
        return false;
    };
    matches!((under.slot, top.slot), (Some((Slot::Ei, _)), Some((Slot::Ordinal, _))))
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

/// A duplicate with the classifier of the `တစ်<classifier>လုံး` before it.
///
/// # Arguments
///
/// * `e` - The duplicate.
/// * `classifier` - The classifier, when written.
///
/// # Returns
///
/// The duplicate.
fn with_classifier(e: Expr, classifier: Option<Ident>) -> Expr
{
    let Expr::Verb(mut node) = e
    else
    {
        return e;
    };
    node.classifier = classifier;
    Expr::Verb(node)
}

/// The classifier of `တစ်<classifier>လုံး`, with the `၏` after it joined or
/// not.
///
/// # Arguments
///
/// * `text` - The word.
///
/// # Returns
///
/// The classifier, or `None` for another word.
fn whole_classifier(text: &str) -> Option<&str>
{
    let text = text.strip_suffix(EI).unwrap_or(text);
    let middle = text.strip_prefix(ONE)?.strip_suffix(WHOLE)?;
    (!middle.is_empty()).then_some(middle)
}

/// Whether a word is `တစ်<classifier>လုံး`, "the whole of", with the `၏` after
/// it joined or not.
///
/// # Arguments
///
/// * `text` - The word.
///
/// # Returns
///
/// Whether the `၏` is joined, or `None` for another word.
pub(crate) fn whole_word(text: &str) -> Option<bool>
{
    let (text, glued) = text.strip_suffix(EI).map_or((text, false), |t| (t, true));
    let middle = text.strip_prefix(ONE)?.strip_suffix(WHOLE)?;
    (!middle.is_empty()).then_some(glued)
}

/// Whether an operand is one or more reads of data, `ထဲမှ ဒေတာ`, around a
/// bare name.
///
/// # Arguments
///
/// * `expr` - The operand.
///
/// # Returns
///
/// `true` for such reads.
fn reads_a_name(expr: &Expr) -> bool
{
    let Expr::Verb(v) = expr
    else
    {
        return false;
    };
    let read =
        matches!(v.op, VerbOp::Unwrap | VerbOp::UnwrapFault) && v.args.len() == 1;
    read && matches!(&v.args[0], Expr::Name(_)) || read && reads_a_name(&v.args[0])
}
