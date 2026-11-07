//! Calls through function values. Such a call names no function, but a
//! function value comes from the address of a function alone, so the
//! functions it can run are those whose address the module takes and whose
//! parameters and result fit its kind. The check treats the call as a call
//! of each of them.

use std::collections::BTreeSet;

use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Module;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::value::Value;

use crate::walk::each_expr;
use crate::walk::each_stmt;
use crate::walk::stmt_exprs;

/// The functions whose address the module takes, in the main program or in
/// a function.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The function numbers.
pub(crate) fn taken_fns(module: &Module) -> BTreeSet<u32>
{
    let mut found = BTreeSet::new();
    let blocks =
        std::iter::once(&module.main).chain(module.funcs.iter().map(|f| &f.block));
    for block in blocks
    {
        each_stmt(block, &mut |s| {
            let mut note = |e: &Expr| {
                found.extend(addressed_fn(e));
            };
            stmt_exprs(s).into_iter().for_each(|e| each_expr(e, &mut note));
        });
    }
    found
}

/// The function whose address an expression is: `FnAddr`, or the constant
/// the folder makes of one, the number of the function plus one.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The function number, or `None` for another expression.
fn addressed_fn(e: &Expr) -> Option<u32>
{
    match e.node
    {
        ExprNode::FnAddr { func, .. } => Some(func),
        ExprNode::Const(Value::Ptr(word)) if e.kind.is_fn() && word > 0 =>
        {
            u32::try_from(word - 1).ok()
        }
        _ => None,
    }
}

/// The functions a call through a value of `kind` can run: each function
/// whose address the module takes and whose shape fits the kind.
///
/// # Arguments
///
/// * `module` - The module.
/// * `kind` - The kind of the function value.
///
/// # Returns
///
/// The function numbers; none for a kind that is no function kind.
pub(crate) fn value_callees(module: &Module, kind: DataKind) -> Vec<u32>
{
    let Some(f) = kind.as_fn()
    else
    {
        return Vec::new();
    };
    let fits =
        |n: &u32| module.funcs.get(*n as usize).is_some_and(|func| func.fits_kind(f));
    taken_fns(module).into_iter().filter(fits).collect()
}
