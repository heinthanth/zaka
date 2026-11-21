//! REPL mode: the top-level statements of the file that is run form its
//! program start block, so a short program can leave the block out.

use zakac_ast::expr::Ident;
use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::START_NAME;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_rust_sugars::ret_if;

use crate::units::runs_in_place;

/// Move the statements that run at the top level of the file that is run
/// into a program start block at its end. A variable or constant that comes
/// after the first of them moves too, so the values are set in the order of
/// the source; one with the export mark remains. A file with nothing that
/// runs gets a blank start block, and a file with a start block remains as
/// it is.
///
/// # Arguments
///
/// * `program` - The tree.
pub(crate) fn implicit_start(program: &mut Program)
{
    let root = program.units.last().map_or(0, |u| u.first_stmt as usize);
    let own = &program.stmts[root.min(program.stmts.len())..];
    ret_if!(own.iter().any(|s| matches!(s, Stmt::Fn(f) if f.start)));
    let first = own.iter().position(runs_in_place);
    let moves = |i: usize, s: &Stmt| match s
    {
        Stmt::Decl(d) => first.is_some_and(|f| i > f) && !d.exported,
        _ => runs_in_place(s),
    };
    let tail = program.stmts.split_off(root.min(program.stmts.len()));
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
