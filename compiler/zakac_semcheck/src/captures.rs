//! The captures of nested functions. A nested function names the locals and
//! the closures around it with no list; the check finds each one as it resolves
//! the names, then writes them into the declaration, so the IR builder knows
//! what the closure holds. Each capture is shared: the function and the block
//! around it read and write one variable.

use std::collections::HashMap;
use std::collections::HashSet;

use zakac_ast::expr::Ident;
use zakac_ast::name::Name;
use zakac_ast::stmt::CaptureItem;
use zakac_ast::stmt::CaptureMode;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::VisitorMut;
use zakac_ast::visit::walk_stmt_mut;
use zakac_rust_sugars::do_if;
use zakac_span::span::Span;

use crate::scope::Capture;

/// The locals and functions each nested function names from around it, under
/// the span of its name, in the order of their first use.
pub(crate) type Captured = HashMap<Span, Vec<Capture>>;

/// Write the captures of each nested function into its declaration.
///
/// # Arguments
///
/// * `program` - The program, checked.
/// * `captured` - The captures of each nested function.
pub(crate) fn write_captures(program: &mut Program, captured: &Captured)
{
    let settled = settle(captured);
    Writer(&settled).visit_program_mut(program);
}

/// Settle which nested functions are closures: one that names a local around
/// it, or a closure, is one. A function that turns out to be no closure is no
/// capture, since a call of it needs nothing it holds.
///
/// # Arguments
///
/// * `captured` - What each nested function names from around it.
///
/// # Returns
///
/// The captures of each closure, under the span of its name.
fn settle(captured: &Captured) -> HashMap<Span, Vec<Name>>
{
    let names_local = |caps: &Vec<Capture>| caps.iter().any(|c| !c.func);
    let mut closures: HashSet<Span> = captured
        .iter()
        .filter(|(_, caps)| names_local(caps))
        .map(|(at, _)| *at)
        .collect();
    loop
    {
        let more: Vec<Span> = captured
            .iter()
            .filter(|(at, _)| !closures.contains(*at))
            .filter(|(_, caps)| {
                caps.iter().any(|c| c.func && closures.contains(&c.decl))
            })
            .map(|(at, _)| *at)
            .collect();
        do_if!(more.is_empty(), break);
        closures.extend(more);
    }
    let kept = |c: &&Capture| !c.func || closures.contains(&c.decl);
    let names =
        |caps: &Vec<Capture>| caps.iter().filter(kept).map(|c| c.name).collect();
    let closed = captured.iter().filter(|(at, _)| closures.contains(*at));
    closed.map(|(at, caps)| (*at, names(caps))).collect()
}

/// The walker that fills the captures of each function from its name.
struct Writer<'a>(&'a HashMap<Span, Vec<Name>>);

impl VisitorMut for Writer<'_>
{
    fn visit_stmt_mut(&mut self, n: &mut Stmt)
    {
        if let Stmt::Fn(f) = n
        {
            let at = f.name.span;
            let names = self.0.get(&at).map(Vec::as_slice).unwrap_or_default();
            f.captures = names.iter().map(|name| item(*name, at)).collect();
        }
        walk_stmt_mut(self, n);
    }
}

/// One capture, shared with the block around the function.
///
/// # Arguments
///
/// * `name` - The name of the local or the closure.
/// * `at` - The name of the function, which stands for the capture.
///
/// # Returns
///
/// The capture.
fn item(name: Name, at: Span) -> CaptureItem
{
    let ident = Ident::builder().name(name).span(at).build();
    CaptureItem::builder().name(ident).mode(CaptureMode::Lend).span(at).build()
}
