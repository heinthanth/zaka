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

/// Record the top-level functions, so calls can come before declarations.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `diags` - Where to report a name declared twice.
///
/// # Returns
///
/// The functions, indexed on name; the first declaration of a name wins.
pub(crate) fn collect_fns(program: &Program, diags: &mut Vec<Diagnostic>) -> FnSigs
{
    let mut fns = FnSigs::new();
    let decls = program
        .stmts
        .iter()
        .filter_map(|s| tern!(let Stmt::Fn(decl) = s, Some(decl), None));
    do_for!(decl in decls, add_fn(&mut fns, decl, diags));
    fns
}

/// Record one function.
///
/// # Arguments
///
/// * `fns` - The functions so far.
/// * `decl` - The declaration.
/// * `diags` - Where to report a name declared twice.
fn add_fn(fns: &mut FnSigs, decl: &FnDecl, diags: &mut Vec<Diagnostic>)
{
    let n = &decl.name.name;
    ret_if!(fns.contains_key(n), diags.push(duplicate_function(decl.name.span, n)));
    fns.insert(n.clone(), sig_of(decl));
}

/// The parameter and result kinds of a declaration.
///
/// # Arguments
///
/// * `decl` - The declaration.
///
/// # Returns
///
/// The signature.
fn sig_of(decl: &FnDecl) -> FnSig
{
    let params = decl.params.iter().map(|p| DataKind::from_ast(&p.kind)).collect();
    let ret = decl.ret.as_ref().map(DataKind::from_ast);
    FnSig::builder().params(params).maybe_ret(ret).build()
}
