use zakac_ast::expr::Ident;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::resolve::duplicate_function;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::comptime_calls::holds_as_constant;
use crate::kind::DataKind;
use crate::scope::FnSig;
use crate::scope::FnSigs;
use crate::scope::overload_name;
use crate::structs::StructTable;

/// Record the top-level functions of each file, so calls can come before
/// declarations. Two files can each declare a function of one name, and one
/// file can declare several functions of one name with other parameters.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `structs` - The kind of each declared struct.
/// * `diags` - Where to report a name declared twice in one file.
///
/// # Returns
///
/// The functions of each unit, indexed on name, each name with its overloads.
pub(crate) fn collect_fns(
    program: &Program, structs: &StructTable, diags: &mut Vec<Diagnostic>,
) -> Vec<FnSigs>
{
    let mut fns = vec![FnSigs::new(); program.units.len().max(1)];
    // An instance of a generic function is a function of each file, as its
    // generic is wherever its file is imported.
    let instances = program.stmts.iter().enumerate().filter_map(|(i, s)| match s
    {
        Stmt::Fn(d) if d.name.name.contains('<') => Some((program.unit_of(i), d)),
        _ => None,
    });
    for (home, decl) in instances
    {
        let others = (0..fns.len() as u32).filter(|u| *u != home);
        do_for!(unit in others, add_fn(&mut fns[unit as usize], decl, structs, diags, Some(home)));
    }
    let decls = program.stmts.iter().enumerate().filter_map(|(i, s)| {
        let decl = tern!(let Stmt::Fn(decl) = s, Some(decl), None)?;
        let plain = !decl.start && decl.kind_params.is_empty() && !decl.builtin;
        tern!(plain, Some((program.unit_of(i) as usize, decl)), None)
    });
    do_for!((unit, decl) in decls, add_fn(&mut fns[unit], decl, structs, diags, Some(unit as u32)));
    let comptime = program.stmts.iter().filter_map(|s| match s
    {
        Stmt::Fn(d) if d.comptime && d.kind_params.is_empty() => Some(d),
        _ => None,
    });
    diags.extend(comptime.filter_map(|d| result_kind_fault(d, structs)));
    fns
}

/// The fault of a comptime function whose result is no value a constant
/// can hold, or that gives none.
///
/// # Arguments
///
/// * `decl` - The function.
/// * `structs` - The kind of each declared struct.
///
/// # Returns
///
/// `ZK0489`, or `None` for a result a constant can hold.
fn result_kind_fault(decl: &FnDecl, structs: &StructTable) -> Option<Diagnostic>
{
    let name = decl.name.name.as_str();
    let Some(written) = &decl.ret
    else
    {
        return Some(kd::comptime_result_kind(decl.name.span, name, "nothing"));
    };
    let kind = DataKind::from_ast(written, structs);
    ret_if!(holds_as_constant(kind), None);
    Some(kd::comptime_result_kind(written.span, name, &kind.name()))
}

/// Record one function.
///
/// # Arguments
///
/// * `fns` - The functions so far.
/// * `decl` - The declaration.
/// * `structs` - The kind of each declared struct.
/// * `diags` - Where to report a function whose parameters and prepositions
///   another function of its name has.
/// * `unit` - The file that declares it.
fn add_fn(
    fns: &mut FnSigs, decl: &FnDecl, structs: &StructTable,
    diags: &mut Vec<Diagnostic>, unit: Option<u32>,
)
{
    let n = &decl.name.name;
    let list = fns.entry(*n).or_default();
    let sig = sig_of(decl, structs, list.len(), unit);
    let same = list.iter().any(|s| s.same_params(&sig));
    ret_if!(same, diags.push(duplicate_function(decl.name.span, n)));
    list.push(sig);
}

/// The parameter and result kinds of a declaration.
///
/// # Arguments
///
/// * `decl` - The declaration.
/// * `structs` - The kind of each declared struct.
/// * `index` - Its place among the functions of its name in its file.
/// * `unit` - The file that declares it; `None` for a function inside a
///   function.
///
/// # Returns
///
/// The signature.
pub(crate) fn sig_of(
    decl: &FnDecl, structs: &StructTable, index: usize, unit: Option<u32>,
) -> FnSig
{
    let kind = |k| DataKind::from_ast(k, structs);
    let params = decl.params.iter().map(|p| kind(&p.kind)).collect();
    let ret = decl.ret.as_ref().map(kind);
    let edits = decl.params.iter().map(|p| p.edit).collect();
    let fills = decl.params.iter().map(|p| p.fill).collect();
    let closes = decl.params.iter().map(|p| p.owned && decl.lib.is_some()).collect();
    let prepositions =
        decl.params.iter().map(|p| p.preposition.as_ref().map(|m| m.name)).collect();
    let phrase = |i: &Option<Ident>| i.as_ref().map(|i| i.name);
    #[rustfmt::skip]
    return FnSig::builder().params(params).edits(edits).fills(fills).maybe_ret(ret).exported(decl.exported)
        .maybe_start(phrase(&decl.phrases.start)).maybe_end(phrase(&decl.phrases.end))
        .external(decl.external).unchecked(decl.unchecked).comptime(decl.comptime).variadic(decl.variadic).prepositions(prepositions).closes(closes).inner(overload_name(decl.name.name, index)).decl(decl.name.span).maybe_unit(unit).build();
}
