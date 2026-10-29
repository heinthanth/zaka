use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::PrintValue;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::do_for;

/// The direct operands of an expression.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The operands, left to right.
pub(crate) fn sub_exprs(e: &Expr) -> Vec<&Expr>
{
    match &e.node
    {
        ExprNode::Const(_)
        | ExprNode::Read(_)
        | ExprNode::AddrOf(_)
        | ExprNode::SizeOf(_)
        | ExprNode::BlankList => Vec::new(),
        ExprNode::Prefix { operand, .. }
        | ExprNode::Convert { operand, .. }
        | ExprNode::Load { addr: operand }
        | ExprNode::Lend { operand }
        | ExprNode::Take { addr: operand } => vec![operand.as_ref()],
        ExprNode::Infix { lhs, rhs, .. }
        | ExprNode::Compare { lhs, rhs, .. }
        | ExprNode::Logic { lhs, rhs, .. }
        | ExprNode::PtrAdd { base: lhs, offset: rhs }
        | ExprNode::PtrSub { lhs, rhs }
        | ExprNode::CellAddr { list: lhs, index: rhs, .. } =>
        {
            vec![lhs.as_ref(), rhs.as_ref()]
        }
        ExprNode::Call { args, .. } | ExprNode::List(args) => args.iter().collect(),
    }
}

/// The expressions a statement runs itself, without those of the
/// statements nested in it.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// The expressions, in the order the statement runs them.
pub(crate) fn stmt_exprs(s: &Stmt) -> Vec<&Expr>
{
    match s
    {
        Stmt::Set { value, .. } => vec![value],
        Stmt::Print { value: PrintValue::Expr(e), .. } => vec![e],
        Stmt::If { cond, .. } | Stmt::Loop { cond, .. } => vec![cond],
        Stmt::Counted { from, to, step, .. } =>
        {
            [from, to].into_iter().chain(step.as_ref()).collect()
        }
        Stmt::Return { value, .. } => value.iter().collect(),
        Stmt::Call { args, .. } => args.iter().collect(),
        Stmt::Store { addr, value, .. } => vec![addr, value],
        Stmt::Alloc { dest, count, .. } | Stmt::Resize { dest, count, .. } =>
        {
            dest_expr(dest).into_iter().chain([count]).collect()
        }
        Stmt::Free { dest, .. } => dest_expr(dest).into_iter().collect(),
        Stmt::Print { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => Vec::new(),
    }
}

/// The address expression of a heap destination.
///
/// # Arguments
///
/// * `dest` - The destination.
///
/// # Returns
///
/// The address for a cell in mem, `None` for a slot.
pub(crate) fn dest_expr(dest: &Dest) -> Option<&Expr>
{
    match dest
    {
        Dest::Mem(addr) => Some(addr),
        Dest::Slot(_) => None,
    }
}

/// The statement lists nested in a statement.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// The lists, in source order.
pub(crate) fn nested_blocks(s: &Stmt) -> Vec<&[Stmt]>
{
    match s
    {
        Stmt::If { then, els, .. } => vec![then, els],
        Stmt::Loop { init, update, block, .. } => vec![init, block, update],
        Stmt::Counted { block, .. } => vec![block],
        _ => Vec::new(),
    }
}

/// Call `f` on an expression and on each expression inside it, outer
/// first.
///
/// # Arguments
///
/// * `e` - The expression.
/// * `f` - The callback.
pub(crate) fn each_expr<'m>(e: &'m Expr, f: &mut impl FnMut(&'m Expr))
{
    f(e);
    do_for!(sub in sub_exprs(e), each_expr(sub, f));
}

/// Call `f` on each statement of a list and on each statement nested in
/// it, outer first.
///
/// # Arguments
///
/// * `stmts` - The statements.
/// * `f` - The callback.
pub(crate) fn each_stmt<'m>(stmts: &'m [Stmt], f: &mut impl FnMut(&'m Stmt))
{
    for s in stmts
    {
        f(s);
        do_for!(block in nested_blocks(s), each_stmt(block, f));
    }
}

/// The direct operands of an expression, to change.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// The operands, left to right.
fn sub_exprs_mut(e: &mut Expr) -> Vec<&mut Expr>
{
    match &mut e.node
    {
        ExprNode::Const(_)
        | ExprNode::Read(_)
        | ExprNode::AddrOf(_)
        | ExprNode::SizeOf(_)
        | ExprNode::BlankList => Vec::new(),
        ExprNode::Prefix { operand, .. }
        | ExprNode::Convert { operand, .. }
        | ExprNode::Load { addr: operand }
        | ExprNode::Lend { operand }
        | ExprNode::Take { addr: operand } => vec![operand.as_mut()],
        ExprNode::Infix { lhs, rhs, .. }
        | ExprNode::Compare { lhs, rhs, .. }
        | ExprNode::Logic { lhs, rhs, .. }
        | ExprNode::PtrAdd { base: lhs, offset: rhs }
        | ExprNode::PtrSub { lhs, rhs }
        | ExprNode::CellAddr { list: lhs, index: rhs, .. } =>
        {
            vec![lhs.as_mut(), rhs.as_mut()]
        }
        ExprNode::Call { args, .. } | ExprNode::List(args) => args.iter_mut().collect(),
    }
}

/// The expressions a statement runs itself, to change.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// The expressions, in the order the statement runs them.
fn stmt_exprs_mut(s: &mut Stmt) -> Vec<&mut Expr>
{
    match s
    {
        Stmt::Set { value, .. } => vec![value],
        Stmt::Print { value: PrintValue::Expr(e), .. } => vec![e],
        Stmt::If { cond, .. } | Stmt::Loop { cond, .. } => vec![cond],
        Stmt::Counted { from, to, step, .. } =>
        {
            [from, to].into_iter().chain(step.as_mut()).collect()
        }
        Stmt::Return { value, .. } => value.iter_mut().collect(),
        Stmt::Call { args, .. } => args.iter_mut().collect(),
        Stmt::Store { addr, value, .. } => vec![addr, value],
        Stmt::Alloc { dest: d, count, .. } | Stmt::Resize { dest: d, count, .. } =>
        {
            dest_expr_mut(d).into_iter().chain([count]).collect()
        }
        Stmt::Free { dest: d, .. } => dest_expr_mut(d).into_iter().collect(),
        Stmt::Print { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => Vec::new(),
    }
}

/// The address expression of a heap destination, to change.
///
/// # Arguments
///
/// * `dest` - The destination.
///
/// # Returns
///
/// The address for a cell in mem, `None` for a slot.
fn dest_expr_mut(dest: &mut Dest) -> Option<&mut Expr>
{
    match dest
    {
        Dest::Mem(addr) => Some(addr),
        Dest::Slot(_) => None,
    }
}

/// Call `f` on each expression a statement runs itself and on each
/// expression inside them, to change.
///
/// # Arguments
///
/// * `s` - The statement.
/// * `f` - The callback.
pub(crate) fn each_expr_mut(s: &mut Stmt, f: &mut impl FnMut(&mut Expr))
{
    do_for!(e in stmt_exprs_mut(s), expr_tree_mut(e, f));
}

/// Call `f` on an expression and on each expression inside it.
///
/// # Arguments
///
/// * `e` - The expression.
/// * `f` - The callback.
fn expr_tree_mut(e: &mut Expr, f: &mut impl FnMut(&mut Expr))
{
    f(e);
    do_for!(sub in sub_exprs_mut(e), expr_tree_mut(sub, f));
}

/// Call `f` on each statement of a list and on each statement nested in
/// it, to change.
///
/// # Arguments
///
/// * `stmts` - The statements.
/// * `f` - The callback.
pub(crate) fn each_stmt_mut(stmts: &mut [Stmt], f: &mut impl FnMut(&mut Stmt))
{
    for s in stmts
    {
        f(s);
        let blocks: Vec<&mut Vec<Stmt>> = match s
        {
            Stmt::If { then, els, .. } => vec![then, els],
            Stmt::Loop { init, update, block, .. } => vec![init, block, update],
            Stmt::Counted { block, .. } => vec![block],
            _ => Vec::new(),
        };
        do_for!(block in blocks, each_stmt_mut(block, f));
    }
}
