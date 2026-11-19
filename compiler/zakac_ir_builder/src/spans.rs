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
use crate::ir::fmt_values_mut;

/// What a walk of each span and each expression of a module does.
pub trait Walker
{
    /// Change one span.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    fn span(&mut self, span: &mut Span);

    /// Change one expression, before the walk goes into its parts.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    fn expr(&mut self, e: &mut Expr) { _ = e }
}

/// Moves each span with a fixed function.
struct SpanMap<F>(F);

impl<F: Fn(Span) -> Span> Walker for SpanMap<F>
{
    fn span(&mut self, span: &mut Span) { *span = (self.0)(*span) }
}

/// Spells each repeated cell out as a list of its cells.
struct Spell;

impl Walker for Spell
{
    fn span(&mut self, _: &mut Span) {}

    fn expr(&mut self, e: &mut Expr)
    {
        let ExprNode::Repeat { cell, count } = &e.node
        else
        {
            return;
        };
        e.node = ExprNode::List(vec![(**cell).clone(); *count as usize]);
    }
}

/// Move each span of a module with a fixed function.
///
/// # Arguments
///
/// * `module` - The module.
/// * `f` - Maps one span.
pub fn map_spans(module: &mut Module, f: &impl Fn(Span) -> Span)
{
    walk_module(module, &mut SpanMap(f));
}

/// Spell each repeated cell of a module out as a list of its cells, for a
/// backend that writes each cell: the IR holds a long list of one initial
/// value as one cell, and the backends see the cells.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// A clone of the module with the cells.
pub fn spell_repeats(module: &Module) -> Module
{
    let mut spelled = module.clone();
    walk_module(&mut spelled, &mut Spell);
    spelled
}

/// Walk each span and each expression of a module.
///
/// # Arguments
///
/// * `module` - The module.
/// * `w` - The walker.
fn walk_module(module: &mut Module, w: &mut impl Walker)
{
    w.span(&mut module.span);
    do_for!(s in &mut module.globals, slot(s, w));
    do_for!(s in &mut module.main, stmt(s, w));
    for func in &mut module.funcs
    {
        w.span(&mut func.span);
        do_for!(s in &mut func.locals, slot(s, w));
        do_for!(s in &mut func.block, stmt(s, w));
    }
}

/// Move the span of a slot.
///
/// # Arguments
///
/// * `s` - The slot.
/// * `w` - The walker.
fn slot(s: &mut Slot, w: &mut impl Walker) { w.span(&mut s.span) }

/// Move the spans of a statement list.
///
/// # Arguments
///
/// * `stmts` - The statements.
/// * `w` - The walker.
fn stmts(stmts: &mut [Stmt], w: &mut impl Walker)
{
    do_for!(s in stmts, stmt(s, w));
}

/// Move the spans of a statement.
///
/// # Arguments
///
/// * `s` - The statement.
/// * `w` - The walker.
fn stmt(s: &mut Stmt, w: &mut impl Walker)
{
    match s
    {
        Stmt::Set { value, span, .. } =>
        {
            expr(value, w);
            w.span(span);
        }
        Stmt::Wait { job, span, .. } =>
        {
            expr(job, w);
            w.span(span);
        }
        Stmt::Halt { value, span, .. } | Stmt::Rings { octets: value, span, .. } =>
        {
            do_if!(let Some(v) = value, expr(v, w));
            w.span(span);
        }
        Stmt::Leave { value, span, .. } =>
        {
            expr(value, w);
            w.span(span);
        }
        Stmt::CStore { base, value, span, .. } =>
        {
            expr(base, w);
            expr(value, w);
            w.span(span);
        }
        Stmt::Print { value, span } =>
        {
            do_if!(let PrintValue::Expr(e) = value, expr(e, w));
            w.span(span);
        }
        Stmt::If { cond, then, els, span } =>
        {
            expr(cond, w);
            stmts(then, w);
            stmts(els, w);
            w.span(span);
        }
        Stmt::Loop { init, cond, update, block, span } =>
        {
            stmts(init, w);
            expr(cond, w);
            stmts(update, w);
            stmts(block, w);
            w.span(span);
        }
        Stmt::Counted { from, to, step, block, span, .. } =>
        {
            expr(from, w);
            expr(to, w);
            do_if!(let Some(step) = step, expr(step, w));
            stmts(block, w);
            w.span(span);
        }
        Stmt::Break { span } | Stmt::Continue { span } => w.span(span),
        Stmt::Return { value, span } =>
        {
            do_if!(let Some(v) = value, expr(v, w));
            w.span(span);
        }
        Stmt::Call { args, span, .. } | Stmt::Foreign { args, span, .. } =>
        {
            do_for!(a in args, expr(a, w));
            w.span(span);
        }
        Stmt::CallValue { callee, args, span } =>
        {
            do_for!(a in args, expr(a, w));
            expr(callee, w);
            w.span(span);
        }
        Stmt::Store { addr, value, span } =>
        {
            expr(addr, w);
            expr(value, w);
            w.span(span);
        }
        Stmt::Alloc { dest: d, count, span, .. }
        | Stmt::Resize { dest: d, count, span, .. } =>
        {
            dest(d, w);
            expr(count, w);
            w.span(span);
        }
        Stmt::Blit { src, dst, count, span, .. } =>
        {
            expr(src, w);
            expr(dst, w);
            expr(count, w);
            w.span(span);
        }
        Stmt::Fill { dst, count, value, span } =>
        {
            expr(dst, w);
            expr(count, w);
            expr(value, w);
            w.span(span);
        }
        Stmt::Free { dest: d, span }
        | Stmt::Drop { dest: d, span, .. }
        | Stmt::RemoveCell { dest: d, span, .. } =>
        {
            dest(d, w);
            w.span(span);
        }
        Stmt::RemoveRow { dest: d, index, span, .. } =>
        {
            dest(d, w);
            expr(index, w);
            w.span(span);
        }
        Stmt::Replace { dest: d, start, end, value, span, .. } =>
        {
            dest(d, w);
            expr(start, w);
            expr(end, w);
            expr(value, w);
            w.span(span);
        }
        Stmt::SetChar { dest: d, index, value, span, .. }
        | Stmt::Insert { dest: d, index, value, span, .. }
        | Stmt::SetRow { dest: d, index, value, span, .. } =>
        {
            dest(d, w);
            expr(index, w);
            expr(value, w);
            w.span(span);
        }
        Stmt::AddCell { dest: d, value, span, .. }
        | Stmt::AddText { dest: d, value, span, .. } =>
        {
            dest(d, w);
            expr(value, w);
            w.span(span);
        }
    }
}

/// Move the spans of a heap destination.
///
/// # Arguments
///
/// * `d` - The destination.
/// * `w` - The walker.
fn dest(d: &mut Dest, w: &mut impl Walker)
{
    do_if!(let Dest::Mem(addr) = d, expr(addr, w));
}

/// Move the spans of an expression.
///
/// # Arguments
///
/// * `e` - The expression.
/// * `w` - The walker.
fn expr(e: &mut Expr, w: &mut impl Walker)
{
    w.expr(e);
    w.span(&mut e.span);
    match &mut e.node
    {
        ExprNode::Const(_)
        | ExprNode::Read(_)
        | ExprNode::AddrOf(_)
        | ExprNode::SizeOf(_)
        | ExprNode::Text(_)
        | ExprNode::FnAddr { .. }
        | ExprNode::BlankList => (),
        ExprNode::Format(pieces) => do_for!(e in fmt_values_mut(pieces), expr(e, w)),
        ExprNode::Chars { text, index, .. } =>
        {
            expr(text, w);
            do_if!(let Some(i) = index, expr(i, w));
        }
        ExprNode::RowFind { table, index } =>
        {
            expr(table, w);
            expr(index, w);
        }
        ExprNode::Prefix { operand, .. }
        | ExprNode::Convert { operand, .. }
        | ExprNode::Load { addr: operand }
        | ExprNode::Lend { operand }
        | ExprNode::Take { addr: operand }
        | ExprNode::Dup { value: operand, .. }
        | ExprNode::Closure { record: operand, .. }
        | ExprNode::CField { base: operand, .. }
        | ExprNode::Own { value: operand, .. } => expr(operand, w),
        ExprNode::Infix { lhs, rhs, .. }
        | ExprNode::Compare { lhs, rhs, .. }
        | ExprNode::Logic { lhs, rhs, .. }
        | ExprNode::PtrAdd { base: lhs, offset: rhs }
        | ExprNode::PtrSub { lhs, rhs } =>
        {
            expr(lhs, w);
            expr(rhs, w);
        }
        ExprNode::Call { args, .. }
        | ExprNode::Spawn { args, .. }
        | ExprNode::Host { args, .. }
        | ExprNode::Foreign { args, .. }
        | ExprNode::List(args) =>
        {
            do_for!(a in args, expr(a, w))
        }
        ExprNode::Wait(job) => expr(job, w),
        ExprNode::CallValue { callee, args } =>
        {
            do_for!(a in args, expr(a, w));
            expr(callee, w);
        }
        ExprNode::CellAddr { list, index, .. } =>
        {
            expr(list, w);
            expr(index, w);
        }
        ExprNode::FieldAddr { base, .. } | ExprNode::Repeat { cell: base, .. } =>
        {
            expr(base, w)
        }
        ExprNode::MakeSlice { source, start, len, .. } =>
        {
            do_for!(a in [source, start, len], expr(a, w))
        }
    }
}
