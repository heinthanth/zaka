use std::collections::BTreeMap;
use std::collections::BTreeSet;

use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::name::Name;
use zakac_ast::stmt::Param;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::StructDecl;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::resolve as rd;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::buildable::report_unbuildable;
use crate::fn_kind::FnText;
use crate::fn_kind::join_fn;
use crate::foreign;
use crate::kind::DataKind;
use crate::kind::FALLIBLE;
use crate::kind::split_marks;
use crate::kind::table_text;
use crate::structs::EXTERN_FIELD;
use crate::structs::StructDefs;
use crate::structs::StructTable;
use crate::structs::make_all;

/// Make the kinds of the top-level structs, reporting a name declared twice,
/// a field name used twice or taken from a struct, a struct name no
/// declaration gives, a struct that holds itself and a field that holds a
/// borrow. A function with the name of a struct is reported too.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `enums` - The kind of each enumeration, which a field can name.
/// * `diags` - Where to report.
///
/// # Returns
///
/// The kind of each struct; the first declaration of a name wins.
pub(crate) fn collect_structs(
    program: &Program, enums: &StructTable, diags: &mut Vec<Diagnostic>,
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
    let shared: BTreeSet<&str> =
        decls.iter().filter(|(_, d)| d.shared).map(|(n, _)| *n).collect();
    let known: BTreeSet<&str> =
        names.iter().copied().chain(enums.keys().map(|n| n.as_str())).collect();
    do_for!(decl in decls.values(), check_fields(decl, &known, diags));
    do_for!(f in program.stmts.iter().filter_map(as_fn_name), report_clash(f, &names, diags));
    // A kind from outside holds C's address alone; its written fields are C's,
    // and their names and kinds go into the name of that one field, so two
    // programs that give one name to two C structs make two kinds.
    let held = |decl: &StructDecl| {
        let texts = field_texts(decl, &names, &shared, enums);
        ret_if!(!decl.external, texts);
        let field = extern_field(&texts);
        vec![(field, DataKind::handle_pointer().sig_name())]
    };
    let mut defs: StructDefs =
        decls.iter().map(|(name, decl)| (name.to_string(), held(decl))).collect();
    break_loops(&decls, &mut defs, diags);
    report_unbuildable(&decls, diags);
    let mut table = make_all(&defs);
    // A shared struct names its object: the box of the struct, which copies
    // share.
    for name in &shared
    {
        let boxed = Name::new(name);
        do_if!(let Some(k) = table.get(&boxed).copied(), _ = table.insert(boxed, DataKind::box_of(k)));
    }
    let external = decls.values().filter(|d| d.external);
    do_for!(decl in external, set_c_fields(decl, &table, diags));
    table
}

/// The name of the one field of the box of a kind from outside:
/// [`EXTERN_FIELD`], then a hash of the name and kind of each C field, which
/// is the same in each run.
///
/// # Arguments
///
/// * `fields` - The C fields, each a name and a kind in text form.
///
/// # Returns
///
/// The name.
fn extern_field(fields: &[(String, String)]) -> String
{
    // FNV-1a over the fields, each as `name:kind;`.
    let text: String = fields.iter().map(|(f, k)| format!("{f}:{k};")).collect();
    let fnv = |h: u64, b: &u8| (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3);
    let hash = text.as_bytes().iter().fold(0xcbf2_9ce4_8422_2325, fnv);
    format!("{EXTERN_FIELD}{hash:016x}")
}

/// Give the struct of a kind from outside its C fields: numbers, booleans,
/// counted lists of numbers, which C holds in place, text, a `char *` that a
/// read copies, and a handle of a kind from outside, a pointer to another
/// struct C keeps.
///
/// # Arguments
///
/// * `decl` - The declaration.
/// * `table` - The kind of each struct.
/// * `diags` - Where a field of another kind is reported.
fn set_c_fields(decl: &StructDecl, table: &StructTable, diags: &mut Vec<Diagnostic>)
{
    let Some(shape) = table.get(&decl.name.name).and_then(|k| k.as_external())
    else
    {
        return;
    };
    let kind_of = |f: &Param| DataKind::from_ast(&f.kind, table);
    let readable = |k: DataKind| {
        let number = matches!(k, DataKind::Int(_) | DataKind::Float(_))
            && foreign::crosses(k, 64);
        let list = k.as_list().is_some() && foreign::number_cells(k, 64).is_some();
        number
            || list
            || k == DataKind::Bool
            || k.is_text()
            || k.as_external().is_some()
    };
    let stuck = decl.fields.iter().filter(|f| !readable(kind_of(f)));
    do_for!(f in stuck, diags.push(kd::kind_not_across_c(f.span, &kind_of(f).name())));
    let fields =
        decl.fields.iter().map(|f| (f.name.name.to_string(), kind_of(f))).collect();
    shape.set_c_fields(fields);
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

/// Report a field name used twice or taken from a struct or an enumeration,
/// and a name in a field kind that no declaration gives.
///
/// # Arguments
///
/// * `decl` - The declaration.
/// * `names` - The struct and enumeration names.
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
    if let Some(sig) = kind.sig.as_deref()
    {
        let params = sig.params.iter().map(|p| &p.kind);
        return params.chain(&sig.ret).find_map(|k| unknown_name(k, names));
    }
    if let Some(parts) = kind.parts.as_deref()
    {
        let side = |k: &Option<Kind>| k.as_ref().and_then(|k| unknown_name(k, names));
        return side(&parts.ok).or_else(|| side(&parts.err));
    }
    if let Some(t) = kind.table.as_deref()
    {
        return unknown_name(&t.index, names).or_else(|| unknown_name(&t.value, names));
    }
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
/// * `enums` - The kind of each enumeration, written as its signature.
///
/// # Returns
///
/// Each field name with its kind text.
fn field_texts(
    decl: &StructDecl, names: &BTreeSet<&str>, shared: &BTreeSet<&str>,
    enums: &StructTable,
) -> Vec<(String, String)>
{
    let text = |kind: &Kind| field_text(kind, names, shared, enums);
    decl.fields.iter().map(|f| (f.name.name.to_string(), text(&f.kind))).collect()
}

/// One field kind in text form, with each declared struct as its bare name;
/// a result kind writes its sides so too.
///
/// # Arguments
///
/// * `kind` - The field kind.
/// * `names` - The struct names.
/// * `enums` - The kind of each enumeration, written as its signature.
///
/// # Returns
///
/// The text.
fn field_text(
    kind: &Kind, names: &BTreeSet<&str>, shared: &BTreeSet<&str>, enums: &StructTable,
) -> String
{
    if let Some(sig) = kind.sig.as_deref()
    {
        let text = |k: &Kind| field_text(k, names, shared, enums);
        let params: Vec<(bool, String, Option<&str>)> = sig
            .params
            .iter()
            .map(|p| (p.edit, text(&p.kind), p.mark.as_ref().map(|m| m.name.as_str())))
            .collect();
        let ret = sig.ret.as_ref().map(text);
        let params = params.iter().map(|(e, k, m)| (*e, k.as_str(), *m)).collect();
        let parts = FnText { params, ret: ret.as_deref() };
        let marks = DataKind::from_ast(kind, &StructTable::new()).sig_name();
        return format!("({}){}", join_fn(&parts), split_marks(&marks).1);
    }
    if let Some(parts) = kind.parts.as_deref()
    {
        let side = |field: &str, k: &Option<Kind>| {
            k.as_ref().map_or(String::new(), |k| {
                format!(",{field}:{}", field_text(k, names, shared, enums))
            })
        };
        let ok_name = tern!(parts.optional, "some", "ok");
        let (ok, err) = (side(ok_name, &parts.ok), side("err", &parts.err));
        let marks = DataKind::from_ast(kind, &StructTable::new()).sig_name();
        return format!("{FALLIBLE}{{tag:u8{ok}{err}}}{}", split_marks(&marks).1);
    }
    if let Some(t) = kind.table.as_deref()
    {
        let text = |k: &Kind| field_text(k, names, shared, enums);
        let marks = DataKind::from_ast(kind, &StructTable::new()).sig_name();
        let table = table_text(&text(&t.index), &text(&t.value));
        return format!("{table}{}", split_marks(&marks).1);
    }
    let plain = DataKind::from_ast(kind, enums).sig_name();
    let name = kind.name.as_ref().map(|n| n.name.as_str());
    let Some(name) = name.filter(|n| names.contains(n))
    else
    {
        return plain;
    };
    // A shared struct is held as its object, never inline.
    let object = tern!(shared.contains(name), "@", "");
    format!("{name}{object}{}", split_marks(&plain).1)
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
    let through_address = kind.wraps.iter().any(|w| matches!(w, Wrap::Ptr));
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
            // A shared struct is held as its object, so it closes no loop.
            let Some((found, _)) = decls.get_key_value(held).filter(|(_, d)| !d.shared)
            else
            {
                continue;
            };
            do_if!(seen.insert(*found), todo.push(found.to_string()));
        }
    }
    seen
}
