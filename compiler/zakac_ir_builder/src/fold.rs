use std::collections::HashMap;

use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::fold as fd;
use zakac_diagnostics::kinds as kd;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::IntKind;
use zakac_semcheck::kind::Width;
use zakac_span::span::Span;

use crate::ir::Access;
use crate::ir::Dest;
use crate::ir::Expr;
use crate::ir::ExprNode;
use crate::ir::FoldOptions;
use crate::ir::InfixOp;
use crate::ir::LogicOp;
use crate::ir::Module;
use crate::ir::Place;
use crate::ir::PrintValue;
use crate::ir::Slot;
use crate::ir::Stmt;
use crate::value;
use crate::value::EvalError;
use crate::value::Value;

/// The integer kind of `usize`, the kind of a size.
const USIZE: IntKind = IntKind::new(false, Width::Platform);

/// Fold the constants of a module in place. Each operation whose operands
/// are constants is worked out, and its errors are reported, whatever the
/// options hold; the options decide what is rewritten.
///
/// # Arguments
///
/// * `module` - The module.
/// * `options` - Which foldings rewrite the module.
/// * `platform_bits` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The diagnostics, in source order.
pub fn fold(
    module: &mut Module, options: &FoldOptions, platform_bits: u16,
) -> Vec<Diagnostic>
{
    let mut folder = Folder {
        options: *options,
        platform: platform_bits,
        consts: HashMap::new(),
        global_consts: const_flags(&module.globals),
        local_consts: Vec::new(),
        diags: Vec::new(),
    };
    folder.stmts(&mut module.main);
    for func in &mut module.funcs
    {
        folder.local_consts = const_flags(&func.locals);
        folder.consts.retain(|place, _| matches!(place, Place::Global(_)));
        folder.stmts(&mut func.block);
    }
    folder.diags.sort_by_key(|d| d.span.lo);
    folder.diags
}

/// Which slots are constants.
///
/// # Arguments
///
/// * `slots` - The slots.
///
/// # Returns
///
/// One flag per slot.
fn const_flags(slots: &[Slot]) -> Vec<bool>
{
    slots.iter().map(|s| s.access == Access::Const).collect()
}

/// The fold walk. The `Folder` struct is built once, in [`fold`], from
/// fields that need no defaults.
struct Folder
{
    /// Which foldings rewrite the module.
    options: FoldOptions,
    /// Bit count of the platform-sized kinds.
    platform: u16,
    /// Values of constant slots set so far.
    consts: HashMap<Place, Value>,
    /// Which global slots are constants.
    global_consts: Vec<bool>,
    /// Which local slots of the current function are constants.
    local_consts: Vec<bool>,
    /// Diagnostics so far.
    diags: Vec<Diagnostic>,
}

impl Folder
{
    /// Whether a slot is a constant.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// `true` for a `ကိန်းသေ`.
    fn is_const(&self, place: Place) -> bool
    {
        let (flags, n) = match place
        {
            Place::Global(n) => (&self.global_consts, n),
            Place::Local(n) => (&self.local_consts, n),
        };
        flags.get(n as usize).copied().unwrap_or(false)
    }

    /// Fold a statement list in place; a constant branch is spliced into
    /// the list.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    fn stmts(&mut self, stmts: &mut Vec<Stmt>)
    {
        let old = std::mem::take(stmts);
        do_for!(s in old, self.stmt(s, stmts));
    }

    /// Fold a statement and append what is left of it to `out`.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    /// * `out` - The list it goes to.
    fn stmt(&mut self, mut s: Stmt, out: &mut Vec<Stmt>)
    {
        match &mut s
        {
            Stmt::Set { place, value, .. } =>
            {
                let v = self.expr(value);
                let known = v.filter(|_| self.is_const(*place));
                do_if!(let Some(v) = known, let _ = self.consts.insert(*place, v));
            }
            Stmt::Print { value: PrintValue::Expr(e), .. } => self.drop_value(e),
            Stmt::Print { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => (),
            Stmt::If { cond, then, els, .. } =>
            {
                let taken = self.expr(cond).and_then(Value::as_bool);
                self.stmts(then);
                self.stmts(els);
                let taken = taken.filter(|_| self.options.branches);
                ret_if!(let Some(b) = taken, out.append(tern!(b, then, els)));
            }
            Stmt::Loop { init, cond, update, block, .. } =>
            {
                self.stmts(init);
                let held = self.expr(cond).and_then(Value::as_bool);
                self.stmts(update);
                self.stmts(block);
                let dead = held == Some(false) && self.options.branches;
                ret_if!(dead, out.append(init));
            }
            Stmt::Counted { from, to, step, block, .. } =>
            {
                self.drop_value(from);
                self.drop_value(to);
                do_if!(let Some(step) = step, self.drop_value(step));
                self.stmts(block);
            }
            Stmt::Return { value, .. } =>
            {
                do_if!(let Some(v) = value, self.drop_value(v))
            }
            Stmt::Call { args, .. } => do_for!(a in args, self.drop_value(a)),
            Stmt::Store { addr, value, .. } =>
            {
                self.drop_value(addr);
                self.drop_value(value);
            }
            Stmt::Alloc { dest, count, .. } | Stmt::Resize { dest, count, .. } =>
            {
                self.dest(dest);
                self.drop_value(count);
            }
            Stmt::Free { dest, .. } => self.dest(dest),
        }
        out.push(s);
    }

    /// Fold the address of a heap destination.
    ///
    /// # Arguments
    ///
    /// * `dest` - The destination.
    fn dest(&mut self, dest: &mut Dest)
    {
        do_if!(let Dest::Mem(addr) = dest, self.drop_value(addr));
    }

    /// Fold an expression whose value is not needed here.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    fn drop_value(&mut self, e: &mut Expr) { self.expr(e); }

    /// Fold an expression in place.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// Its value, when it is known at compile time.
    fn expr(&mut self, e: &mut Expr) -> Option<Value>
    {
        let span = e.span;
        ret_if!(let ExprNode::Read(place) = e.node, self.read(e, place));
        let (value, rest) = match &mut e.node
        {
            ExprNode::Const(v) => return Some(*v),
            ExprNode::Read(_) => return None,
            ExprNode::Prefix { op, operand } =>
            {
                let (op, v) = (*op, self.expr(operand));
                let v = v.map(|v| value::prefix(op, v, self.platform));
                (self.checked(v, op.into(), e.kind.name(), span), None)
            }
            ExprNode::Infix { op, lhs, rhs } =>
            {
                let (op, l, r) = (*op, self.expr(lhs), self.expr(rhs));
                let shift = matches!(op, InfixOp::Shl | InfixOp::Shr);
                let far = r.and_then(|r| self.too_far(shift, e.kind, r));
                let v = far.map(Err).or_else(|| {
                    l.zip(r).map(|(l, r)| value::infix(op, l, r, self.platform))
                });
                let at = tern!(shift, rhs.span, span);
                (self.checked(v, op.into(), e.kind.name(), at), None)
            }
            ExprNode::Compare { op, lhs, rhs } =>
            {
                let (op, l, r) = (*op, self.expr(lhs), self.expr(rhs));
                let v = l.zip(r).and_then(|(l, r)| value::compare(op, l, r).ok());
                (v.map(Value::Bool), None)
            }
            ExprNode::Logic { op, lhs, rhs } =>
            {
                let (op, l, r) = (*op, self.expr(lhs), self.expr(rhs));
                (logic_value(op, l, r), short_circuit(op, l, rhs))
            }
            ExprNode::Convert { operand, .. } =>
            {
                let v = self.expr(operand);
                (v.and_then(|v| value::convert(v, e.kind, self.platform).ok()), None)
            }
            ExprNode::Call { args, .. } =>
            {
                do_for!(a in args.iter_mut(), self.drop_value(a));
                (None, None)
            }
            ExprNode::AddrOf(_) => (None, None),
            ExprNode::Load { addr }
            | ExprNode::Lend { operand: addr }
            | ExprNode::Take { addr }
            | ExprNode::FieldAddr { base: addr, .. } =>
            {
                self.drop_value(addr);
                (None, None)
            }
            ExprNode::PtrAdd { base: lhs, offset: rhs }
            | ExprNode::PtrSub { lhs, rhs } =>
            {
                self.drop_value(lhs);
                self.drop_value(rhs);
                (None, None)
            }
            ExprNode::SizeOf(kind) =>
            {
                let octets = u128::from(kind.size(self.platform));
                (Some(Value::int(USIZE, octets, self.platform)), None)
            }
            ExprNode::List(cells) =>
            {
                do_for!(c in cells.iter_mut(), self.drop_value(c));
                (None, None)
            }
            ExprNode::BlankList => (None, None),
            ExprNode::CellAddr { list, index, len, first, checked } =>
            {
                self.drop_value(list);
                let at = self.expr(index);
                *checked = self.cell_in_range(at, *len, *first, index.span);
                (None, None)
            }
        };
        ret_if!(!self.options.exprs, value);
        do_if!(let Some(v) = value, e.node = ExprNode::Const(v));
        do_if!(value.is_none() && rest.is_some(), *e = rest.expect("checked above"));
        value
    }

    /// Whether a known cell number names a cell of its list; a known one
    /// outside the list is reported.
    ///
    /// # Arguments
    ///
    /// * `at` - The cell number, when it is known.
    /// * `len` - The number of cells.
    /// * `first` - The number of the first cell: 0 or 1.
    /// * `span` - The cell number.
    ///
    /// # Returns
    ///
    /// `true` when the number is known and in range.
    fn cell_in_range(
        &mut self, at: Option<Value>, len: u32, first: u8, span: Span,
    ) -> bool
    {
        let Some(Value::Int { kind, raw }) = at
        else
        {
            return false;
        };
        let wide = i128::try_from(raw).unwrap_or(i128::MAX);
        let index = tern!(kind.signed, raw as i128, wide);
        let lo = i128::from(first);
        ret_if!(index >= lo && index < lo + i128::from(len), true);
        let last = (u64::from(len) + u64::from(first)).saturating_sub(1);
        self.diags.push(fd::cell_out_of_range(span, &index.to_string(), first, last));
        false
    }

    /// Fold a read of a slot: a constant slot set to a known value reads
    /// as that value.
    ///
    /// # Arguments
    ///
    /// * `e` - The read.
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// The value, when propagation is on and the slot holds a known one.
    fn read(&mut self, e: &mut Expr, place: Place) -> Option<Value>
    {
        ret_if!(!self.options.consts, None);
        let v = self.consts.get(&place).copied()?;
        e.node = ExprNode::Const(v);
        Some(v)
    }

    /// Whether a known shift distance is not below the bit width of the
    /// shifted kind; the shifted value need not be known.
    ///
    /// # Arguments
    ///
    /// * `shift` - Whether the operation is a shift.
    /// * `kind` - The shifted kind.
    /// * `distance` - The distance.
    ///
    /// # Returns
    ///
    /// The error, or `None` when the distance is fine.
    fn too_far(&self, shift: bool, kind: DataKind, distance: Value)
    -> Option<EvalError>
    {
        let Value::Int { raw, .. } = distance
        else
        {
            return None;
        };
        let bits = kind.bits(self.platform);
        (shift && raw >= u128::from(bits))
            .then_some(EvalError::ShiftTooFar { distance: raw, bits })
    }

    /// Report an operation on constants that has no value.
    ///
    /// # Arguments
    ///
    /// * `v` - The result, `None` when an operand is not a constant.
    /// * `op` - The operator spelling.
    /// * `kind` - The result kind, as text.
    /// * `span` - Where to report it; the distance for a shift.
    ///
    /// # Returns
    ///
    /// The value, or `None`.
    fn checked(
        &mut self, v: Option<Result<Value, EvalError>>, op: &str, kind: String,
        span: Span,
    ) -> Option<Value>
    {
        let err = match v?
        {
            Ok(v) => return Some(v),
            Err(err) => err,
        };
        let diag = match err
        {
            EvalError::Overflow => fd::constant_overflow(span, op, &kind),
            EvalError::ZeroDivisor => fd::constant_zero_divisor(span, op),
            EvalError::ShiftTooFar { distance, bits } =>
            {
                kd::shift_too_far(span, &distance.to_string(), bits)
            }
            EvalError::Mismatch => return None,
        };
        self.diags.push(diag);
        None
    }
}

/// The value of a logical operation: known when both sides are constants,
/// or when the left side decides it (`false and x`, `true or x`).
///
/// # Arguments
///
/// * `op` - The operator.
/// * `l` - The left value, when known.
/// * `r` - The right value, when known.
///
/// # Returns
///
/// The value, or `None`.
fn logic_value(op: LogicOp, l: Option<Value>, r: Option<Value>) -> Option<Value>
{
    let decided = l.and_then(Value::as_bool).filter(|b| *b == (op == LogicOp::Or));
    ret_if!(let Some(b) = decided, Some(Value::Bool(b)));
    l.zip(r).and_then(|(l, r)| value::logic(op, l, r).ok()).map(Value::Bool)
}

/// The operand a logical operation reduces to when its left side is a
/// constant that does not decide it: `true and x` is `x`, `false or x` is
/// `x`.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `l` - The left value, when known.
/// * `rhs` - The right operand.
///
/// # Returns
///
/// The right operand, or `None` when the left side does not reduce.
fn short_circuit(op: LogicOp, l: Option<Value>, rhs: &mut Expr) -> Option<Expr>
{
    let l = l.and_then(Value::as_bool)?;
    let keeps_rhs = tern!(op == LogicOp::And, l, !l);
    keeps_rhs.then(|| rhs.clone())
}
