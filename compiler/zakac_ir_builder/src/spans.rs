use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_span::span::Span;

use crate::ir::Dest;
use crate::ir::Expr;
use crate::ir::ExprNode;
use crate::ir::Module;
use crate::ir::PrintValue;
use crate::ir::Slot;
use crate::ir::Stmt;

/// Move each span of a module with a fixed function.
///
/// # Arguments
///
/// * `module` - The module.
/// * `f` - Maps one span.
pub fn map_spans(module: &mut Module, f: &impl Fn(Span) -> Span)
{
    module.span = f(module.span);
    do_for!(s in &mut module.globals, slot(s, f));
    do_for!(s in &mut module.main, stmt(s, f));
    for func in &mut module.funcs
    {
        func.span = f(func.span);
        do_for!(s in &mut func.locals, slot(s, f));
        do_for!(s in &mut func.block, stmt(s, f));
    }
}

/// Move the span of a slot.
///
/// # Arguments
///
/// * `s` - The slot.
/// * `f` - Maps one span.
fn slot(s: &mut Slot, f: &impl Fn(Span) -> Span) { s.span = f(s.span) }

/// Move the spans of a statement list.
///
/// # Arguments
///
/// * `stmts` - The statements.
/// * `f` - Maps one span.
fn stmts(stmts: &mut [Stmt], f: &impl Fn(Span) -> Span)
{
    do_for!(s in stmts, stmt(s, f));
}

/// Move the spans of a statement.
///
/// # Arguments
///
/// * `s` - The statement.
/// * `f` - Maps one span.
fn stmt(s: &mut Stmt, f: &impl Fn(Span) -> Span)
{
    match s
    {
        Stmt::Set { value, span, .. } =>
        {
            expr(value, f);
            *span = f(*span);
        }
        Stmt::Print { value, span } =>
        {
            do_if!(let PrintValue::Expr(e) = value, expr(e, f));
            *span = f(*span);
        }
        Stmt::If { cond, then, els, span } =>
        {
            expr(cond, f);
            stmts(then, f);
            stmts(els, f);
            *span = f(*span);
        }
        Stmt::Loop { init, cond, update, block, span } =>
        {
            stmts(init, f);
            expr(cond, f);
            stmts(update, f);
            stmts(block, f);
            *span = f(*span);
        }
        Stmt::Counted { from, to, step, block, span, .. } =>
        {
            expr(from, f);
            expr(to, f);
            do_if!(let Some(step) = step, expr(step, f));
            stmts(block, f);
            *span = f(*span);
        }
        Stmt::Break { span } | Stmt::Continue { span } => *span = f(*span),
        Stmt::Return { value, span } =>
        {
            do_if!(let Some(v) = value, expr(v, f));
            *span = f(*span);
        }
        Stmt::Call { args, span, .. } =>
        {
            do_for!(a in args, expr(a, f));
            *span = f(*span);
        }
        Stmt::Store { addr, value, span } =>
        {
            expr(addr, f);
            expr(value, f);
            *span = f(*span);
        }
        Stmt::Alloc { dest: d, count, span, .. }
        | Stmt::Resize { dest: d, count, span, .. } =>
        {
            dest(d, f);
            expr(count, f);
            *span = f(*span);
        }
        Stmt::Free { dest: d, span } =>
        {
            dest(d, f);
            *span = f(*span);
        }
    }
}

/// Move the spans of a heap destination.
///
/// # Arguments
///
/// * `d` - The destination.
/// * `f` - Maps one span.
fn dest(d: &mut Dest, f: &impl Fn(Span) -> Span)
{
    do_if!(let Dest::Mem(addr) = d, expr(addr, f));
}

/// Move the spans of an expression.
///
/// # Arguments
///
/// * `e` - The expression.
/// * `f` - Maps one span.
fn expr(e: &mut Expr, f: &impl Fn(Span) -> Span)
{
    e.span = f(e.span);
    match &mut e.node
    {
        ExprNode::Const(_)
        | ExprNode::Read(_)
        | ExprNode::AddrOf(_)
        | ExprNode::SizeOf(_)
        | ExprNode::BlankList => (),
        ExprNode::Prefix { operand, .. }
        | ExprNode::Convert { operand, .. }
        | ExprNode::Load { addr: operand }
        | ExprNode::Lend { operand }
        | ExprNode::Take { addr: operand } => expr(operand, f),
        ExprNode::Infix { lhs, rhs, .. }
        | ExprNode::Compare { lhs, rhs, .. }
        | ExprNode::Logic { lhs, rhs, .. }
        | ExprNode::PtrAdd { base: lhs, offset: rhs }
        | ExprNode::PtrSub { lhs, rhs } =>
        {
            expr(lhs, f);
            expr(rhs, f);
        }
        ExprNode::Call { args, .. } | ExprNode::List(args) =>
        {
            do_for!(a in args, expr(a, f))
        }
        ExprNode::CellAddr { list, index, .. } =>
        {
            expr(list, f);
            expr(index, f);
        }
        ExprNode::FieldAddr { base, .. } => expr(base, f),
    }
}
