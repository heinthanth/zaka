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
use crate::structs::StructTable;

/// Record the top-level functions of each file, so calls can come before
/// declarations. Two files can each declare a function of one name.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `structs` - The kind of each declared struct.
/// * `diags` - Where to report a name declared twice in one file.
///
/// # Returns
///
/// The functions of each unit, indexed on name; the first declaration of a
/// name in a file wins.
pub(crate) fn collect_fns(
    program: &Program, structs: &StructTable, diags: &mut Vec<Diagnostic>,
) -> Vec<FnSigs>
{
    let mut fns = vec![FnSigs::new(); program.units.len().max(1)];
    let decls = program.stmts.iter().enumerate().filter_map(|(i, s)| {
        tern!(let Stmt::Fn(decl) = s, Some((program.unit_of(i) as usize, decl)), None)
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
/// * `diags` - Where to report a name declared twice.
fn add_fn(
    fns: &mut FnSigs, decl: &FnDecl, structs: &StructTable, diags: &mut Vec<Diagnostic>,
)
{
    let n = &decl.name.name;
    ret_if!(fns.contains_key(n), diags.push(duplicate_function(decl.name.span, n)));
    fns.insert(n.clone(), sig_of(decl, structs));
}

/// The parameter and result kinds of a declaration.
///
/// # Arguments
///
/// * `decl` - The declaration.
/// * `structs` - The kind of each declared struct.
///
/// # Returns
///
/// The signature.
fn sig_of(decl: &FnDecl, structs: &StructTable) -> FnSig
{
    let kind = |k| DataKind::from_ast(k, structs);
    let params = decl.params.iter().map(|p| kind(&p.kind)).collect();
    let ret = decl.ret.as_ref().map(kind);
    FnSig::builder().params(params).maybe_ret(ret).exported(decl.exported).build()
}
