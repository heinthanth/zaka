//! Call hints: a call writes its arguments with no mark, so the editor marks
//! each argument the function can change (`ဒေတာ ပြောင်းလဲရန်`), each one it
//! fills (`ဒေတာ ထည့်ပေးရန်`) and each one it takes over (`အပိုင်အနေဖြင့်`), where
//! the call itself does not show it.

use std::collections::HashMap;

use gen_lsp_types::InlayHint;
use gen_lsp_types::InlayHintKind;
use zakac_ast::expr::CallExpr;
use zakac_ast::expr::Expr;
use zakac_ast::name::Name;
use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::Param;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_call_expr;
use zakac_ast::visit::walk_call_stmt;
use zakac_ast::visit::walk_fn;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_span::span::Span;

use crate::check::FileCheck;

/// What a parameter asks of its argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark
{
    /// Nothing: the function gets a duplicate.
    Plain,
    /// `ဒေတာ ပြောင်းလဲရန်`: the function can change the caller's place.
    Edit,
    /// `ဒေတာ ထည့်ပေးရန်`: the function writes the caller's place.
    Fill,
    /// `အပိုင်အနေဖြင့်`: the function takes the block over.
    Owned,
}

/// The marks of the parameters of each function, under name and
/// parameter count, so an overload finds its own.
type Sigs = HashMap<(Name, usize), Vec<Mark>>;

/// The hints of the open file.
///
/// # Arguments
///
/// * `file` - The check of the open file.
///
/// # Returns
///
/// One hint before each argument the function can change or takes over;
/// none when the check did not run.
pub fn call_hints(file: &FileCheck) -> Vec<InlayHint>
{
    let Some(checked) = file.checked.as_ref()
    else
    {
        return Vec::new();
    };
    let program = &checked.lowered.checked.program;
    let mut sigs = Marks(Sigs::new());
    sigs.visit_program(program);
    let mut calls = Calls { sigs: sigs.0, found: Vec::new() };
    calls.visit_program(program);
    let shown = calls.found.into_iter().filter(|(span, _)| file.in_root(span));
    shown.map(|(span, mark)| hint(file, span, mark)).collect()
}

/// The hint before one argument.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `arg` - The source region of the argument.
/// * `mark` - What its parameter asks.
///
/// # Returns
///
/// The hint.
fn hint(file: &FileCheck, arg: Span, mark: Mark) -> InlayHint
{
    let range = file.lines.range(file.root_range(&arg));
    let label = match mark
    {
        Mark::Owned => t!("lsp.hint_owned"),
        Mark::Fill => t!("lsp.hint_fill"),
        _ => t!("lsp.hint_edit"),
    };
    InlayHint {
        position: range.start,
        label: label.to_string().into(),
        kind: Some(InlayHintKind::Parameter),
        text_edits: None,
        tooltip: None,
        padding_left: None,
        padding_right: Some(true),
        data: None,
    }
}

/// The mark of one parameter.
///
/// # Arguments
///
/// * `p` - The parameter.
///
/// # Returns
///
/// The mark.
fn mark_of(p: &Param) -> Mark
{
    match (p.owned, p.fill, p.edit)
    {
        (true, ..) => Mark::Owned,
        (_, true, _) => Mark::Fill,
        (.., true) => Mark::Edit,
        _ => Mark::Plain,
    }
}

/// Gathers the marks of each function with a marked parameter.
struct Marks(Sigs);

impl Visitor for Marks
{
    fn visit_fn(&mut self, n: &FnDecl)
    {
        let marks: Vec<Mark> = n.params.iter().map(mark_of).collect();
        let marked = marks.iter().any(|m| *m != Mark::Plain);
        if marked
        {
            self.0.entry((n.name.name, marks.len())).or_insert(marks);
        }
        walk_fn(self, n);
    }
}

/// Finds the arguments that need a hint.
struct Calls
{
    /// The marks of each function.
    sigs: Sigs,
    /// The source region of each argument with its mark, in the order of the
    /// text.
    found: Vec<(Span, Mark)>,
}

impl Calls
{
    /// Note the marked arguments of one call that names its function.
    ///
    /// # Arguments
    ///
    /// * `callee` - The function name.
    /// * `args` - The arguments.
    fn note(&mut self, callee: Name, args: &[Expr])
    {
        let Some(marks) = self.sigs.get(&(callee, args.len()))
        else
        {
            return;
        };
        let marked = args.iter().zip(marks).filter(|(_, m)| **m != Mark::Plain);
        let found: Vec<(Span, Mark)> = marked.map(|(a, m)| (a.span(), *m)).collect();
        self.found.extend(found);
    }
}

impl Visitor for Calls
{
    fn visit_call_expr(&mut self, n: &CallExpr)
    {
        do_if!(n.place.is_none(), self.note(n.callee.name, &n.args));
        walk_call_expr(self, n);
    }

    fn visit_call_stmt(&mut self, n: &CallStmt)
    {
        do_if!(n.place.is_none(), self.note(n.callee.name, &n.args));
        walk_call_stmt(self, n);
    }
}
