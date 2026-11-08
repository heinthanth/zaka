use gen_lsp_types::DocumentSymbol;
use gen_lsp_types::SymbolKind;
use zakac_ast::expr::Ident;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::Param;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::check::FileCheck;

/// The outline of the open file: its functions with their parameters, the
/// start block, its structs with their fields, its enumerations with their
/// members, its kind aliases, and its global variables and constants.
///
/// # Arguments
///
/// * `file` - The check of the open file.
///
/// # Returns
///
/// The nodes, in source order; none when the file could not be read.
pub fn outline(file: &FileCheck) -> Vec<DocumentSymbol>
{
    let Some(checked) = &file.checked
    else
    {
        return Vec::new();
    };
    let program = &checked.lowered.checked.program;
    root_stmts(program).filter_map(|s| node_of(file, s)).collect()
}

/// The statements of the open file, the last unit of the program.
///
/// # Arguments
///
/// * `program` - The program.
///
/// # Returns
///
/// The statements.
pub fn root_stmts(program: &Program) -> impl Iterator<Item = &Stmt>
{
    let stmts = program.stmts.iter().enumerate();
    stmts.filter(|(i, _)| program.is_root(program.unit_of(*i))).map(|(_, s)| s)
}

/// The outline node of a top-level statement that declares something.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `stmt` - The statement.
///
/// # Returns
///
/// The node, or `None` for a statement that declares nothing.
fn node_of(file: &FileCheck, stmt: &Stmt) -> Option<DocumentSymbol>
{
    let leaf =
        |name: &Ident, span: Span, kind| node(file, name, span, kind, Vec::new());
    match stmt
    {
        Stmt::Fn(f) => Some(fn_node(file, f)),
        Stmt::Struct(s) =>
        {
            let fields =
                s.fields.iter().map(|p| leaf(&p.name, p.span, SymbolKind::Field));
            Some(node(file, &s.name, s.span, SymbolKind::Struct, fields.collect()))
        }
        Stmt::Enum(e) =>
        {
            let members = e.members.iter();
            let members =
                members.map(|m| leaf(&m.name, m.span, SymbolKind::EnumMember));
            Some(node(file, &e.name, e.span, SymbolKind::Enum, members.collect()))
        }
        Stmt::Alias(a) => Some(leaf(&a.name, a.span, SymbolKind::TypeParameter)),
        Stmt::Decl(d) =>
        {
            let kind = tern!(d.is_const, SymbolKind::Constant, SymbolKind::Variable);
            Some(leaf(&d.name, d.span, kind))
        }
        _ => None,
    }
}

/// The node of a function, a C function or the start block, with its
/// parameters as children.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `f` - The declaration.
///
/// # Returns
///
/// The node.
fn fn_node(file: &FileCheck, f: &FnDecl) -> DocumentSymbol
{
    let param =
        |p: &Param| node(file, &p.name, p.span, SymbolKind::Variable, Vec::new());
    let params = f.params.iter().map(param).collect();
    node(file, &f.name, f.span, SymbolKind::Function, params)
}

/// An outline node with its name, the range of its whole statement and the
/// range of its name.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `name` - The declared name.
/// * `span` - The whole declaration.
/// * `kind` - What it declares.
/// * `children` - The nodes inside it.
///
/// # Returns
///
/// The node.
#[allow(deprecated)]
fn node(
    file: &FileCheck, name: &Ident, span: Span, kind: SymbolKind,
    children: Vec<DocumentSymbol>,
) -> DocumentSymbol
{
    let at = |span: &Span| file.lines.range(file.root_range(span));
    // The whole range must hold the name, as LSP asks.
    let whole = Span::new(span.lo.min(name.span.lo), span.hi.max(name.span.hi));
    DocumentSymbol {
        name: shown_name(name),
        detail: None,
        kind,
        tags: None,
        deprecated: None,
        range: at(&whole),
        selection_range: at(&name.span),
        children: (!children.is_empty()).then_some(children),
    }
}

/// The name as written: the check renames the second and later functions of
/// one name `name#2`, `name#3`, ...; the outline shows the written name.
///
/// # Arguments
///
/// * `name` - The name.
///
/// # Returns
///
/// The written name.
pub fn shown_name(name: &Ident) -> String
{
    let text = name.name.as_str();
    text.split_once('#').map_or(text, |(written, _)| written).to_string()
}
