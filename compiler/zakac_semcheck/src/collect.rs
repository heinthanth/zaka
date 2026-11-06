use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::resolve::duplicate_function;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

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
    let decls = program.stmts.iter().enumerate().filter_map(|(i, s)| {
        let decl = tern!(let Stmt::Fn(decl) = s, Some(decl), None)?;
        tern!(decl.start, None, Some((program.unit_of(i) as usize, decl)))
    });
    do_for!((unit, decl) in decls, add_fn(&mut fns[unit], decl, structs, diags));
    fns
}

/// Record one function.
///
/// # Arguments
///
/// * `fns` - The functions so far.
/// * `decl` - The declaration.
/// * `structs` - The kind of each declared struct.
/// * `diags` - Where to report a function whose parameters and marks another
///   function of its name has.
fn add_fn(
    fns: &mut FnSigs, decl: &FnDecl, structs: &StructTable, diags: &mut Vec<Diagnostic>,
)
{
    let n = &decl.name.name;
    let list = fns.entry(*n).or_default();
    let sig = sig_of(decl, structs, list.len());
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
///
/// # Returns
///
/// The signature.
fn sig_of(decl: &FnDecl, structs: &StructTable, index: usize) -> FnSig
{
    let kind = |k| DataKind::from_ast(k, structs);
    let params = decl.params.iter().map(|p| kind(&p.kind)).collect();
    let ret = decl.ret.as_ref().map(kind);
    let edits = decl.params.iter().map(|p| p.edit).collect();
    let marks: Option<Vec<_>> =
        decl.params.iter().map(|p| p.mark.as_ref().map(|m| m.name)).collect();
    #[rustfmt::skip]
    return FnSig::builder().params(params).edits(edits).maybe_ret(ret).exported(decl.exported)
        .maybe_marks(marks).inner(overload_name(decl.name.name, index)).build();
}
