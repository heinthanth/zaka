use zakac_ast::s_expr::ToSexpr;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;

use crate::ir::Access;
use crate::ir::Dest;
use crate::ir::Expr;
use crate::ir::ExprNode;
use crate::ir::Func;
use crate::ir::Module;
use crate::ir::Place;
use crate::ir::PrintValue;
use crate::ir::SliceFrom;
use crate::ir::Slot;
use crate::ir::Stmt;

impl ToSexpr for Module
{
    /// `(module (globals …) (fn …)… (block …))`
    fn to_sexpr(&self) -> String
    {
        let globals = slot_list("globals", &self.globals, Place::Global, 0);
        let funcs = self.funcs.iter().map(ToSexpr::to_sexpr);
        let main = stmt_list("block", &self.main);
        list("module", std::iter::once(globals).chain(funcs).chain([main]))
    }
}

impl ToSexpr for Func
{
    /// `(fn name (params …) (result l1|-) (locals …) (block …))`
    fn to_sexpr(&self) -> String
    {
        let params = self.params as usize;
        let rest = self.result.map_or(params, |r| r as usize + 1);
        let result =
            self.result.map_or("-".to_string(), |r| Place::Local(r).to_string());
        let parts = [
            self.name.clone(),
            slot_list("params", &self.locals[..params], Place::Local, 0),
            list("result", [result]),
            slot_list("locals", &self.locals[rest..], Place::Local, rest),
            stmt_list("block", &self.block),
        ];
        list("fn", parts)
    }
}

impl ToSexpr for Stmt
{
    fn to_sexpr(&self) -> String
    {
        match self
        {
            Stmt::Set { place, value, .. } =>
            {
                list("set", [place.to_string(), value.to_sexpr()])
            }
            Stmt::Print { value, .. } => list("print", [print_value(value)]),
            Stmt::If { cond, then, els, .. } => if_sexpr(cond, then, els),
            Stmt::Loop { init, cond, update, block, .. } =>
            {
                loop_sexpr(init, cond, update, block)
            }
            Stmt::Counted { var, from, to, step, mode, block, .. } =>
            {
                let bounds = [var.to_string(), from.to_sexpr(), to.to_sexpr()];
                let step = step.iter().map(ToSexpr::to_sexpr);
                let parts = bounds.into_iter().chain(step);
                list(mode.as_ref(), parts.chain([stmt_list("block", block)]))
            }
            Stmt::Break { .. } => "(break)".into(),
            Stmt::Continue { .. } => "(continue)".into(),
            Stmt::Return { value, .. } =>
            {
                list("return", value.iter().map(ToSexpr::to_sexpr))
            }
            Stmt::Call { name, args, .. } => call_sexpr(name, args),
            Stmt::Store { addr, value, .. } =>
            {
                list("store", [addr.to_sexpr(), value.to_sexpr()])
            }
            Stmt::Alloc { dest, count, unit, .. } =>
            {
                heap_sexpr("alloc", dest, count, *unit)
            }
            Stmt::Resize { dest, count, unit, .. } =>
            {
                heap_sexpr("resize", dest, count, *unit)
            }
            Stmt::Free { dest, .. } => list("free", [dest_sexpr(dest)]),
        }
    }
}

impl ToSexpr for Expr
{
    fn to_sexpr(&self) -> String
    {
        match &self.node
        {
            ExprNode::Const(v) => v.to_string(),
            ExprNode::Read(place) => place.to_string(),
            ExprNode::Prefix { op, operand } =>
            {
                let head: &'static str = op.into();
                list(head, [operand.to_sexpr()])
            }
            ExprNode::Infix { op, lhs, rhs } =>
            {
                let head: &'static str = op.into();
                list(head, [lhs.to_sexpr(), rhs.to_sexpr()])
            }
            ExprNode::Compare { op, lhs, rhs } =>
            {
                let head: &'static str = op.into();
                list(head, [lhs.to_sexpr(), rhs.to_sexpr()])
            }
            ExprNode::Logic { op, lhs, rhs } =>
            {
                let head: &'static str = op.into();
                list(head, [lhs.to_sexpr(), rhs.to_sexpr()])
            }
            ExprNode::Convert { operand, implicit } =>
            {
                let head = tern!(*implicit, "widen", "cast");
                list(head, [self.kind.name(), operand.to_sexpr()])
            }
            ExprNode::Call { name, args, .. } => call_sexpr(name, args),
            ExprNode::AddrOf(place) => list("addr", [place.to_string()]),
            ExprNode::Load { addr } => list("load", [addr.to_sexpr()]),
            ExprNode::Lend { operand } => list("lend", [operand.to_sexpr()]),
            ExprNode::Take { addr } => list("take", [addr.to_sexpr()]),
            ExprNode::PtrAdd { base, offset } =>
            {
                list("ptr+", [base.to_sexpr(), offset.to_sexpr()])
            }
            ExprNode::PtrSub { lhs, rhs } =>
            {
                list("ptr-", [lhs.to_sexpr(), rhs.to_sexpr()])
            }
            ExprNode::SizeOf(kind) => list("sizeof", [kind.name()]),
            ExprNode::List(cells) => list("list", cells.iter().map(ToSexpr::to_sexpr)),
            ExprNode::BlankList => "blank_list".to_string(),
            ExprNode::CellAddr { list: base, index, len, first, checked } =>
            {
                let head = tern!(*first == 0, "cell@", "cell1@");
                let len = len.map_or("count".to_string(), |n| n.to_string());
                let parts = [base.to_sexpr(), index.to_sexpr(), len];
                let mark = checked.then(|| "checked".to_string());
                list(head, parts.into_iter().chain(mark))
            }
            ExprNode::MakeSlice { source, start, len, from, checked } =>
            {
                let from = match from
                {
                    SliceFrom::List(n) => n.to_string(),
                    SliceFrom::Slice => "slice".to_string(),
                    SliceFrom::Block => "block".to_string(),
                };
                let parts = [source.to_sexpr(), start.to_sexpr(), len.to_sexpr(), from];
                let mark = checked.then(|| "checked".to_string());
                list("slice", parts.into_iter().chain(mark))
            }
            ExprNode::FieldAddr { base, field, offset } =>
            {
                list("field@", [base.to_sexpr(), field.to_string(), offset.to_string()])
            }
        }
    }
}

/// A list form.
///
/// # Arguments
///
/// * `head` - The first atom.
/// * `parts` - The other items.
///
/// # Returns
///
/// `(head part…)`.
fn list(head: &str, parts: impl IntoIterator<Item = String>) -> String
{
    let parts: Vec<String> = parts.into_iter().collect();
    ret_if!(parts.is_empty(), format!("({head})"));
    format!("({head} {})", parts.join(" "))
}

/// A statement list form.
///
/// # Arguments
///
/// * `head` - The first atom.
/// * `stmts` - The statements.
///
/// # Returns
///
/// `(head stmt…)`.
fn stmt_list(head: &str, stmts: &[Stmt]) -> String
{
    list(head, stmts.iter().map(ToSexpr::to_sexpr))
}

/// A list of slots, `(head (g0 name kind)…)`; a constant or a loop variable
/// carries its access after the kind.
///
/// # Arguments
///
/// * `head` - The first atom.
/// * `slots` - The slots.
/// * `place` - Makes a place from a slot index.
/// * `first` - The index of the first slot.
///
/// # Returns
///
/// The form.
fn slot_list(
    head: &str, slots: &[Slot], place: fn(u32) -> Place, first: usize,
) -> String
{
    let slot = |(i, s): (usize, &Slot)| {
        let access: &'static str = s.access.into();
        let mut parts = vec![s.name.clone(), s.kind.name()];
        do_access(&mut parts, s.access, access);
        list(&place((first + i) as u32).to_string(), parts)
    };
    list(head, slots.iter().enumerate().map(slot))
}

/// Add the access word of a slot that is not a plain variable.
///
/// # Arguments
///
/// * `parts` - The slot items.
/// * `access` - The access.
/// * `word` - Its name.
fn do_access(parts: &mut Vec<String>, access: Access, word: &str)
{
    ret_if!(access == Access::Var);
    parts.push(word.to_string());
}

/// The form of a print value: an expression or a quoted string.
///
/// # Arguments
///
/// * `value` - The value.
///
/// # Returns
///
/// The form.
fn print_value(value: &PrintValue) -> String
{
    match value
    {
        PrintValue::Expr(e) => e.to_sexpr(),
        PrintValue::Text(text) => format!("\"{text}\""),
    }
}

/// `(if cond (block …) (block …))`, without the second block when it is
/// blank.
///
/// # Arguments
///
/// * `cond` - The condition.
/// * `then` - The statements when it holds.
/// * `els` - The statements when it does not.
///
/// # Returns
///
/// The form.
fn if_sexpr(cond: &Expr, then: &[Stmt], els: &[Stmt]) -> String
{
    let mut parts = vec![cond.to_sexpr(), stmt_list("block", then)];
    do_if_els(&mut parts, els);
    list("if", parts)
}

/// Add the `else` block of an `if` when it holds statements.
///
/// # Arguments
///
/// * `parts` - The `if` items.
/// * `els` - The statements.
fn do_if_els(parts: &mut Vec<String>, els: &[Stmt])
{
    ret_if!(els.is_empty());
    parts.push(stmt_list("block", els));
}

/// `(loop cond (block …))`, with `(init …)` before the condition and
/// `(update …)` after it when these hold statements.
///
/// # Arguments
///
/// * `init` - The start statements.
/// * `cond` - The condition.
/// * `update` - The update statements.
/// * `block` - The loop statements.
///
/// # Returns
///
/// The form.
fn loop_sexpr(init: &[Stmt], cond: &Expr, update: &[Stmt], block: &[Stmt]) -> String
{
    let init = (!init.is_empty()).then(|| stmt_list("init", init));
    let update = (!update.is_empty()).then(|| stmt_list("update", update));
    let parts = init.into_iter().chain([cond.to_sexpr()]).chain(update);
    list("loop", parts.chain([stmt_list("block", block)]))
}

/// `(head dest count unit)`, without the unit when none is written.
///
/// # Arguments
///
/// * `head` - `alloc` or `resize`.
/// * `dest` - Where the address lives.
/// * `count` - The element count.
/// * `unit` - The element kind, when written.
///
/// # Returns
///
/// The form.
fn heap_sexpr(head: &str, dest: &Dest, count: &Expr, unit: Option<DataKind>) -> String
{
    let parts = [dest_sexpr(dest), count.to_sexpr()];
    list(head, parts.into_iter().chain(unit.map(DataKind::name)))
}

/// A slot as its place, or `(mem addr)` for the cell at an address.
///
/// # Arguments
///
/// * `dest` - The destination.
///
/// # Returns
///
/// The form.
fn dest_sexpr(dest: &Dest) -> String
{
    match dest
    {
        Dest::Slot(place) => place.to_string(),
        Dest::Mem(addr) => list("mem", [addr.to_sexpr()]),
    }
}

/// `(call name arg…)`
///
/// # Arguments
///
/// * `name` - The function name.
/// * `args` - The arguments.
///
/// # Returns
///
/// The form.
fn call_sexpr(name: &str, args: &[Expr]) -> String
{
    list(
        "call",
        std::iter::once(name.to_string()).chain(args.iter().map(ToSexpr::to_sexpr)),
    )
}
