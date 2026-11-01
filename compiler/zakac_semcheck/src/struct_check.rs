use std::collections::BTreeMap;
use std::collections::BTreeSet;

use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::StructDecl;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::ownership as od;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::kind::DataKind;
use crate::structs::StructDefs;
use crate::structs::StructTable;
use crate::structs::make_all;
use crate::structs::split_base;

/// Make the kinds of the top-level structs, reporting a name declared twice,
/// a field name used twice or taken from a struct, a struct name no
/// declaration gives, a struct that holds itself and a field that holds a
/// borrow. A function with the name of a struct is reported too.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `diags` - Where to report.
///
/// # Returns
///
/// The kind of each struct; the first declaration of a name wins.
pub(crate) fn collect_structs(
    program: &Program, diags: &mut Vec<Diagnostic>,
) -> StructTable
{
    let mut decls: BTreeMap<&str, &StructDecl> = BTreeMap::new();
    for decl in program.stmts.iter().filter_map(as_struct)
    {
        let name = decl.name.name.as_str();
        let twice = decls.contains_key(name);
        do_if!(twice, diags.push(rd::duplicate_struct(decl.name.span, name)));
        do_if!(!twice, _ = decls.insert(name, decl));
    }
    let names: BTreeSet<&str> = decls.keys().copied().collect();
    do_for!(decl in decls.values(), check_fields(decl, &names, diags));
    do_for!(f in program.stmts.iter().filter_map(as_fn_name), report_clash(f, &names, diags));
    let mut defs: StructDefs = decls
        .iter()
        .map(|(name, decl)| (name.to_string(), field_texts(decl, &names)))
        .collect();
    break_loops(&decls, &mut defs, diags);
    let table = make_all(&defs);
    do_for!(decl in decls.values(), check_borrow_fields(decl, &table, diags));
    table
}

/// The declaration of a top-level struct statement.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// The declaration, or `None` for another statement.
fn as_struct(stmt: &Stmt) -> Option<&StructDecl>
{
    tern!(let Stmt::Struct(decl) = stmt, Some(decl), None)
}

/// The name of a top-level function statement.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// The name, or `None` for another statement.
fn as_fn_name(stmt: &Stmt) -> Option<&zakac_ast::expr::Ident>
{
    tern!(let Stmt::Fn(decl) = stmt, Some(&decl.name), None)
}

/// Report a name that a struct has.
///
/// # Arguments
///
/// * `name` - The name.
/// * `structs` - The struct names.
/// * `diags` - Where to report.
fn report_clash(
    name: &zakac_ast::expr::Ident, structs: &BTreeSet<&str>,
    diags: &mut Vec<Diagnostic>,
)
{
    ret_if!(!structs.contains(name.name.as_str()));
    diags.push(rd::struct_name_clash(name.span, &name.name));
}

/// Report a field name used twice or taken from a struct, and a struct name
/// in a field kind that no declaration gives.
///
/// # Arguments
///
/// * `decl` - The declaration.
/// * `names` - The struct names.
/// * `diags` - Where to report.
fn check_fields(decl: &StructDecl, names: &BTreeSet<&str>, diags: &mut Vec<Diagnostic>)
{
    let mut seen = BTreeSet::new();
    for field in &decl.fields
    {
        let name = field.name.name.as_str();
        let twice = !seen.insert(name);
        let owner = &decl.name.name;
        do_if!(twice, diags.push(rd::duplicate_field(field.name.span, owner, name)));
        report_clash(&field.name, names, diags);
        do_if!(let Some(d) = unknown_name(&field.kind, names), diags.push(d));
    }
}

/// The report of a struct name in a kind that no declaration gives.
///
/// # Arguments
///
/// * `kind` - The kind phrase.
/// * `names` - The struct names.
///
/// # Returns
///
/// The diagnostic, or `None` for a kind with no unknown name.
pub(crate) fn unknown_name(kind: &Kind, names: &BTreeSet<&str>) -> Option<Diagnostic>
{
    let name = kind.name.as_ref()?;
    ret_if!(names.contains(name.name.as_str()), None);
    Some(rd::unknown_kind_name(name.span, &name.name))
}

/// The field kinds of a struct in text form, each struct base as its bare
/// name; an unknown name stands for `isize`.
///
/// # Arguments
///
/// * `decl` - The declaration.
/// * `names` - The struct names.
///
/// # Returns
///
/// Each field name with its kind text.
fn field_texts(decl: &StructDecl, names: &BTreeSet<&str>) -> Vec<(String, String)>
{
    let text = |kind: &Kind| {
        let plain = DataKind::from_ast(kind, &StructTable::new()).sig_name();
        let name = kind.name.as_ref().map(|n| n.name.as_str());
        let Some(name) = name.filter(|n| names.contains(n))
        else
        {
            return plain;
        };
        format!("{}{name}", split_base(&plain).0)
    };
    decl.fields.iter().map(|f| (f.name.name.to_string(), text(&f.kind))).collect()
}

/// The struct a field holds inline, not through an address.
///
/// # Arguments
///
/// * `kind` - The field kind phrase.
///
/// # Returns
///
/// The struct name, or `None` for a field that holds no struct inline.
fn held_inline(kind: &Kind) -> Option<&str>
{
    let through_address =
        kind.wraps.iter().any(|w| matches!(w, Wrap::Ptr | Wrap::RawPtr));
    ret_if!(kind.base != BaseKind::Named || through_address, None);
    kind.name.as_ref().map(|n| n.name.as_str())
}

/// Report each struct that holds itself inline, at once or through other
/// structs, and write the fields that close such a loop as `isize`, so the
/// kinds can still be made.
///
/// # Arguments
///
/// * `decls` - The declarations.
/// * `defs` - The field kinds in text form.
/// * `diags` - Where to report.
fn break_loops(
    decls: &BTreeMap<&str, &StructDecl>, defs: &mut StructDefs,
    diags: &mut Vec<Diagnostic>,
)
{
    let reaches = |from: &str| reach_inline(from, decls);
    let looped: Vec<&str> =
        decls.keys().copied().filter(|n| reaches(n).contains(n)).collect();
    for name in looped
    {
        let decl = decls[name];
        diags.push(kd::struct_holds_itself(decl.name.span, name));
        let fields = defs.get_mut(name).expect("each declared struct has its fields");
        for (at, field) in decl.fields.iter().enumerate()
        {
            let closes =
                held_inline(&field.kind).is_some_and(|n| reaches(n).contains(name));
            do_if!(closes, fields[at].1 = "isize".to_string());
        }
    }
}

/// The structs one struct holds inline, at once or through others.
///
/// # Arguments
///
/// * `from` - The struct name.
/// * `decls` - The declarations.
///
/// # Returns
///
/// The names reached, `from` itself among them when it holds itself.
fn reach_inline<'a>(
    from: &str, decls: &BTreeMap<&'a str, &'a StructDecl>,
) -> BTreeSet<&'a str>
{
    let mut seen = BTreeSet::new();
    let mut todo = vec![from.to_string()];
    while let Some(name) = todo.pop()
    {
        let Some(decl) = decls.get(name.as_str())
        else
        {
            continue;
        };
        for held in decl.fields.iter().filter_map(|f| held_inline(&f.kind))
        {
            let Some((found, _)) = decls.get_key_value(held)
            else
            {
                continue;
            };
            do_if!(seen.insert(*found), todo.push(found.to_string()));
        }
    }
    seen
}

/// Report each field of a struct whose kind holds a borrow.
///
/// # Arguments
///
/// * `decl` - The declaration.
/// * `table` - The struct kinds.
/// * `diags` - Where to report.
fn check_borrow_fields(
    decl: &StructDecl, table: &StructTable, diags: &mut Vec<Diagnostic>,
)
{
    for field in &decl.fields
    {
        let holds = DataKind::from_ast(&field.kind, table).holds_borrow();
        do_if!(holds, diags.push(od::borrow_field(field.span, &field.name.name)));
    }
}
