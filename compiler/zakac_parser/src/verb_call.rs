//! The verbs of the core the compiler gives, `ပြသ`, `ထည့်`, `ကူးထည့်`, `ဖျက်`
//! and `အစားထိုး`, are calls written as a sentence: the core declares the
//! words of each sentence. The parser turns a call statement of one, with the
//! words of one of its heads, into the statement node of that sentence, so the
//! later stages see the statements they take.

use zakac_ast::expr::CallExpr;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::Phrases;
use zakac_ast::expr::VerbExpr;
use zakac_ast::stmt::AddStmt;
use zakac_ast::stmt::AssignStmt;
use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::PrintStmt;
use zakac_ast::stmt::RemoveStmt;
use zakac_ast::stmt::ReplaceStmt;
use zakac_ast::stmt::Stmt;
use zakac_ast::verb::Slot;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_lexer::token::Particle;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::clause::Mode;
use crate::parser::PResult;
use crate::parser::Parser;
use crate::using::Head;
use crate::using::preposition_words;

/// `ပြသ`: print a value.
const PRINT: &str = "ပြသ";
/// `ထည့်`: add a value to a list or text.
const ADD: &str = "ထည့်";
/// `ကူးထည့်`: put a duplicate of a value into a place.
const DUP: &str = "ကူးထည့်";
/// `ဖျက်`: remove the last cell, the last character or a row.
const REMOVE: &str = "ဖျက်";
/// `အစားထိုး`: replace a part of text.
const REPLACE: &str = "အစားထိုး";

/// The verbs of the core whose call statements the parser turns into their
/// statements.
pub(crate) const VERB_FNS: [&str; 5] = [PRINT, ADD, DUP, REMOVE, REPLACE];

/// The sentence a call of a verb of the core writes.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Form
{
    /// `<value> ကို|ဟု ပြသမည်`, with its particle.
    Print(Slot),
    /// `<list> ထဲသို့ <value> ကို ထည့်မည်`, or with `၏ အရှေ့တွင်` when `true`.
    Add(bool),
    /// `<place> ထဲသို့ <value> ကို ကူးထည့်မည်`, deep when `true`.
    Dup(bool),
    /// `<list> ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်`, or the last character when `true`.
    RemoveLast(bool),
    /// `<table> ထဲမှ အညွှန်း <index> ကို ဖျက်မည်`.
    RemoveRow,
    /// `<text> ၏ အကွက် <start> မှ <end> ထိကို <value> ဖြင့် အစားထိုးမည်`, or
    /// counted in characters when `true`.
    Replace(bool),
}

/// The words of a preposition as a call holds them.
///
/// # Arguments
///
/// * `text` - The preposition as a head writes it.
///
/// # Returns
///
/// The words, one space apart.
fn words(text: &str) -> String
{
    preposition_words(text).map_or_else(|| text.to_string(), |w| w.join(" "))
}

/// The sentence a call of a verb writes, by the name of the verb, the
/// prepositions of its arguments and whether its ending phrase is `အကုန်`.
///
/// # Arguments
///
/// * `call` - The call.
///
/// # Returns
///
/// The sentence, or `None` for words no head of the verb has.
fn form_of(call: &CallStmt) -> Option<Form>
{
    form_of_words(call.callee.name.as_str(), &call.prepositions, &call.phrases)
}

/// The sentence a call of a verb writes, by its name and its words; see
/// [`form_of`].
///
/// # Arguments
///
/// * `name` - The function name.
/// * `prepositions` - The preposition after each argument.
/// * `phrases` - The phrases of the call.
///
/// # Returns
///
/// The sentence, or `None` for words no head of the verb has.
fn form_of_words(
    name: &str, prepositions: &[Option<Ident>], phrases: &Phrases,
) -> Option<Form>
{
    let written: Vec<Option<String>> =
        prepositions.iter().map(|p| p.as_ref().map(|i| i.name.to_string())).collect();
    let shape = |want: &[&str]| {
        let want = want.iter().map(|w| Some(words(w)));
        written.len() == want.len() && written.iter().cloned().eq(want)
    };
    let deep = phrases.end.as_ref().is_some_and(|e| e.name.as_str() == words("အကုန်"));
    let form = match name
    {
        PRINT if shape(&["ကို"]) => Form::Print(Slot::Ko),
        PRINT if shape(&["ဟု"]) => Form::Print(Slot::Hu),
        ADD if shape(&["ထဲသို့", "ကို"]) => Form::Add(false),
        ADD if shape(&["၏ အရှေ့တွင်", "ကို"]) =>
        {
            Form::Add(true)
        }
        DUP if shape(&["ထဲသို့", "ကို"]) => Form::Dup(deep),
        REMOVE if shape(&["၏ နောက်ဆုံးအကွက်ကို"]) =>
        {
            Form::RemoveLast(false)
        }
        REMOVE if shape(&["၏ နောက်ဆုံးစာလုံးကို"]) =>
        {
            Form::RemoveLast(true)
        }
        REMOVE if shape(&["ထဲမှ အညွှန်း", "ကို"]) =>
        {
            Form::RemoveRow
        }
        REPLACE if shape(&["၏ အကွက်", "မှ", "ထိကို", "ဖြင့်"]) =>
        {
            Form::Replace(false)
        }
        REPLACE if shape(&["၏", "လုံးမြောက်စာလုံးမှ", "လုံးမြောက်စာလုံးထိကို", "ဖြင့်"]) =>
        {
            Form::Replace(true)
        }
        REPLACE if shape(&["၏", "ခုမြောက်စာလုံးမှ", "ခုမြောက်စာလုံးထိကို", "ဖြင့်"]) =>
        {
            Form::Replace(true)
        }
        _ => return None,
    };
    Some(form)
}

impl Parser<'_>
{
    /// Turn a call statement of a verb of the core into the statement of
    /// its sentence: a print, an add, a duplicate, a remove or a replace. A
    /// call with other words, through a place, or that passes a failure on
    /// remains a call.
    ///
    /// # Arguments
    ///
    /// * `stmt` - The statement.
    ///
    /// # Returns
    ///
    /// The statement of the sentence, or the statement as it was.
    pub(crate) fn verb_node(&mut self, stmt: Stmt) -> Stmt
    {
        let Stmt::Call(call) = stmt
        else
        {
            return stmt;
        };
        let plain = call.place.is_none() && !call.pass && call.kind_args.is_empty();
        let form = form_of(&call).filter(|_| plain);
        let Some(form) = form
        else
        {
            return Stmt::Call(call);
        };
        let span = call.span;
        let mut args = call.args.into_iter();
        let mut next = || args.next().expect("the words of a head fix its arguments");
        match form
        {
            Form::Print(slot) => print(next(), slot, span),
            Form::Add(before) =>
            {
                let (list, value) = (next(), next());
                #[rustfmt::skip]
                let node = AddStmt::builder().list(list).value(value).before(before).span(span).build();
                Stmt::Add(node)
            }
            Form::Dup(deep) =>
            {
                let (target, value) = (next(), next());
                let op = tern!(deep, VerbOp::DeepDupOf, VerbOp::DupOf);
                let at = value.span();
                let value = self.dup_expr(op, value, at, Particle::None);
                Stmt::Assign(
                    AssignStmt::builder()
                        .target(target)
                        .value(value)
                        .span(span)
                        .build(),
                )
            }
            Form::RemoveLast(chars) =>
            {
                let node =
                    RemoveStmt::builder().list(next()).chars(chars).span(span).build();
                Stmt::Remove(node)
            }
            Form::RemoveRow =>
            {
                let (table, index) = (next(), next());
                let row = self.row_read(table, index);
                Stmt::Remove(
                    RemoveStmt::builder().list(row).row(true).span(span).build(),
                )
            }
            Form::Replace(chars) =>
            {
                let (text, start, end, value) = (next(), next(), next(), next());
                #[rustfmt::skip]
                let node = ReplaceStmt::builder().text(text).start(start).end(end).chars(chars)
                    .value(value).span(span).build();
                Stmt::Replace(node)
            }
        }
    }

    /// Whether a call of a function names a verb of the core, which gives no
    /// value.
    ///
    /// # Arguments
    ///
    /// * `name` - The function name.
    ///
    /// # Returns
    ///
    /// `true` for such a verb.
    pub(crate) fn names_core_verb(&self, call: &CallExpr) -> bool
    {
        form_of_words(call.callee.name.as_str(), &call.prepositions, &call.phrases)
            .is_some()
    }

    /// Read a statement that ends in a call of a verb of the core as that
    /// call, its arguments as whole clauses, when one cut of its words reads.
    ///
    /// # Returns
    ///
    /// The statement, ZK0260 when the words cut into two readings, or `None`
    /// when the statement ends in no such call or no cut reads.
    pub(crate) fn verb_sentence(&mut self) -> Option<PResult<Head>>
    {
        let last = *self.callees_ahead().last()?;
        let name = self.callee_text(&self.peek_n(last));
        let ends = self.call_end_at(last) == Some((2, false))
            && self.peek_n(last + 2).kind == TokenKind::Punct(Punct::FullStop);
        ret_if!(!VERB_FNS.contains(&name.as_str()) || !ends, None);
        self.attempt_clause_call(Mode::Stmt)
    }

    /// The read of the row of a ဇယား that holds an index, `<table> ထဲမှ အညွှန်း
    /// <index>`, as notation reads it.
    ///
    /// # Arguments
    ///
    /// * `table` - The ဇယား.
    /// * `index` - The index.
    ///
    /// # Returns
    ///
    /// The read.
    fn row_read(&mut self, table: Expr, index: Expr) -> Expr
    {
        let span = table.span().to(&index.span());
        let id = self.new_id();
        #[rustfmt::skip]
        let node = VerbExpr::builder().id(id).op(VerbOp::Row).args(vec![table, index])
            .particle(Particle::None).is_cmp(false).span(span).build();
        Expr::Verb(node)
    }
}

/// A print statement.
///
/// # Arguments
///
/// * `value` - The value.
/// * `slot` - `ကို` or `ဟု`.
/// * `span` - The statement.
///
/// # Returns
///
/// The statement.
fn print(value: Expr, slot: Slot, span: zakac_span::span::Span) -> Stmt
{
    Stmt::Print(PrintStmt::builder().value(value).slot(slot).span(span).build())
}
