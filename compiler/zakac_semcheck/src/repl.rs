//! REPL mode: the top-level statements of the file that is run form its
//! program start block, so a short program can leave the block out.

use std::collections::HashSet;

use zakac_ast::expr::Ident;
use zakac_ast::expr::NameExpr;
use zakac_ast::name::Name;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::START_NAME;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_name;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::units::runs_in_place;

/// The names that a group of statements reads or writes.
#[derive(Default)]
struct Used(HashSet<Name>);

impl Visitor for Used
{
    fn visit_name(&mut self, n: &NameExpr)
    {
        self.0.insert(n.name);
        walk_name(self, n);
    }
}

/// Which variables and constants of a file remain at the top level: those
/// with the export mark, those a function or another declaration that
/// remains uses, and so on down the chain.
///
/// # Arguments
///
/// * `own` - The top-level statements of the file.
///
/// # Returns
///
/// For each statement, whether it is a declaration that remains.
fn kept_decls(own: &[Stmt]) -> Vec<bool>
{
    let mut used = Used::default();
    let decls = |s: &&Stmt| runs_in_place(s) || matches!(s, Stmt::Decl(_));
    own.iter().filter(|s| !decls(s)).for_each(|s| used.visit_stmt(s));
    let mut keep = vec![false; own.len()];
    // A value can use the declarations above it alone, so one pass upward
    // reaches each declaration that a kept one uses.
    for (i, s) in own.iter().enumerate().rev()
    {
        let Stmt::Decl(d) = s
        else
        {
            continue;
        };
        keep[i] = d.exported || used.0.contains(&d.name.name);
        do_if!(keep[i], used.visit_stmt(s));
    }
    keep
}

/// Move the statements that run at the top level of the file that is run
/// into a program start block at its end, and each variable and constant
/// with them unless it remains ([`kept_decls`]), so the values are set in
/// the order of the source, as locals of the block. A file with nothing that
/// runs gets a blank start block, and a file with a start block remains as
/// it is.
///
/// # Arguments
///
/// * `program` - The tree.
pub(crate) fn implicit_start(program: &mut Program)
{
    let root = program.units.last().map_or(0, |u| u.first_stmt as usize);
    let root = root.min(program.stmts.len());
    let own = &program.stmts[root..];
    ret_if!(own.iter().any(|s| matches!(s, Stmt::Fn(f) if f.start)));
    let kept = kept_decls(own);
    let moves = |i: usize, s: &Stmt| match s
    {
        Stmt::Decl(_) => !kept[i],
        _ => runs_in_place(s),
    };
    let tail = program.stmts.split_off(root);
    let (moved, kept): (Vec<_>, Vec<_>) =
        tail.into_iter().enumerate().partition(|(i, s)| moves(*i, s));
    program.stmts.extend(kept.into_iter().map(|(_, s)| s));
    let moved: Vec<Stmt> = moved.into_iter().map(|(_, s)| s).collect();
    let end = program.span.shrink_to_hi();
    let lo = moved.first().map_or(end, Node::span);
    let span = lo.to(&moved.last().map_or(end, Node::span));
    let name = Ident::builder().name(START_NAME).span(span.shrink_to_lo()).build();
    let block = BlockStmt::builder().stmts(moved).span(span).build();
    #[rustfmt::skip]
    let start = FnDecl::builder().name(name).block(block).start(true)
        .span(span).build();
    program.stmts.push(Stmt::Fn(start));
}
