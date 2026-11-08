//! The values of constants and list counts, worked out while the check runs
//! with the folder's own math, so a count takes each expression the folder
//! reduces to an integer.

use std::collections::HashMap;
use std::collections::HashSet;

use zakac_ast::expr::Expr;
use zakac_ast::expr::FloatLit;
use zakac_ast::expr::InfixOp as AstInfix;
use zakac_ast::expr::IntLit;
use zakac_ast::expr::LogicOp as AstLogic;
use zakac_ast::expr::NodeId;
use zakac_ast::expr::PrefixExpr;
use zakac_ast::expr::PrefixOp as AstPrefix;
use zakac_ast::expr::StrExpr;
use zakac_ast::expr::StrPiece;
use zakac_ast::expr::VerbExpr;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_ast::name::Name;
use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::verb::Dir;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_ast::visit::VisitorMut;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::fold as fd;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::parser as pd;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::const_op::CmpOp;
use crate::const_op::InfixOp;
use crate::const_op::LogicOp;
use crate::const_op::PrefixOp;
use crate::enums::MemberValue;
use crate::kind::DataKind;
use crate::scope::Access;
use crate::scope::Binding;
use crate::semcheck::Checker;
use crate::value;
use crate::value::EvalError;
use crate::value::Value;
use crate::value::value_text;

/// The value of a constant, as far as the check knows it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum ConstVal
{
    /// The value.
    Known(Value),
    /// The value of a list or a struct, at this place of
    /// [`Checker::aggs`].
    Agg(u32),
    /// A value that waits for a list count the check has not worked
    /// out so far.
    Pending,
    /// No value at compile time, or no constant.
    #[default]
    Unknown,
}

/// A constant value as the folder holds it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Folded
{
    /// One value.
    Value(Value),
    /// The cells of a list or the fields of a struct, in order.
    Items(Vec<Folded>),
    /// Text whose characters the folder knows.
    Text(String),
    /// A value the folder knows as a whole and reads no part of: text with a
    /// template slot that does not fold, or a list with no cells written.
    Whole,
}

/// A top-level constant one run of the check found.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Seen
{
    /// Its value.
    Folded(Folded),
    /// A value that waits for a list count.
    Pending,
    /// No value at compile time.
    Unknown,
}

/// The reason an expression has no value so far.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Stop
{
    /// It waits for a list count the check has not worked out so far.
    Pending,
    /// It has none; with the diagnostic of the operation that failed, when
    /// one did.
    Not(Option<Diagnostic>),
}

/// The value of an expression, or the reason it has none.
pub(crate) type Worked = Result<Value, Stop>;

/// The count of each list count written as an expression, on the node
/// number of the expression; or the diagnostic to report when it remains
/// open, `None` when the check reported its mistake before.
pub(crate) type CountTable = HashMap<NodeId, Result<u32, Option<Diagnostic>>>;

/// The top-level constants of each file, on the unit and the name: the kind
/// of each and its value, known or waiting for a count.
pub(crate) type KnownConsts = HashMap<(u32, Name), (Option<DataKind>, Seen)>;

/// No value, with no diagnostic.
const NONE: Stop = Stop::Not(None);

impl Checker
{
    /// Work out each list count of a written kind that is an expression,
    /// and record it in [`Checker::counts`]. A count can name a top-level
    /// constant of the file declared below it, with the value an earlier
    /// round found.
    ///
    /// # Arguments
    ///
    /// * `written` - The kind phrase.
    pub(crate) fn work_out_counts(&mut self, written: &Kind)
    {
        for wrap in &written.wraps
        {
            let Wrap::List(Cells::Expr(e)) = wrap
            else
            {
                continue;
            };
            self.scopes.push();
            self.declare_known(e);
            let before = self.diags.len();
            let kind = self.infer(e, None);
            let worked = self.work_out(e);
            self.scopes.pop();
            let reported = self.diags.len() > before;
            let count = count_of(e, kind, worked, reported);
            self.counts.insert(e.id(), count);
        }
    }

    /// Declare, for the count being checked, each top-level constant of the
    /// file that it names and that is not in scope: one declared below,
    /// whose value an earlier round found.
    ///
    /// # Arguments
    ///
    /// * `e` - The count.
    fn declare_known(&mut self, e: &Expr)
    {
        let names = crate::semcheck::names_in(e);
        let unseen = names.into_iter().filter(|n| self.scopes.lookup(n).is_none());
        let known: Vec<_> = unseen
            .filter_map(|n| self.known.get(&(self.unit, n)).map(|v| (n, v.clone())))
            .collect();
        for (name, (kind, seen)) in known
        {
            let value = self.adopt(seen);
            // The constant is declared below: its declared name is the place to
            // go to from this use.
            let decl =
                self.decls.constant(self.unit, &name).unwrap_or(Span::PLACEHOLDER);
            #[rustfmt::skip]
            let binding = Binding::builder().maybe_kind(kind).access(Access::Const).span(e.span())
                .from(decl).value(value).build();
            _ = self.scopes.declare(name, binding);
        }
    }

    /// The value of a constant declaration, converted to the kind of its
    /// name as the store of its value converts it.
    ///
    /// # Arguments
    ///
    /// * `init` - The value as written.
    /// * `kind` - The kind of the name.
    ///
    /// # Returns
    ///
    /// The value.
    pub(crate) fn const_value(
        &mut self, init: &Expr, kind: Option<DataKind>,
    ) -> ConstVal
    {
        let worked = kind.ok_or(NONE).and_then(|k| self.item(init, k));
        match worked
        {
            Ok(folded) => self.adopt(Seen::Folded(folded)),
            Err(Stop::Pending) => ConstVal::Pending,
            Err(Stop::Not(_)) => ConstVal::Unknown,
        }
    }

    /// The binding value of what a run found, keeping the value of a list
    /// or a struct in [`Checker::aggs`].
    ///
    /// # Arguments
    ///
    /// * `seen` - What the run found.
    ///
    /// # Returns
    ///
    /// The value.
    pub(crate) fn adopt(&mut self, seen: Seen) -> ConstVal
    {
        match seen
        {
            Seen::Folded(Folded::Value(v)) => ConstVal::Known(v),
            Seen::Folded(agg) =>
            {
                self.aggs.push(agg);
                ConstVal::Agg(self.aggs.len() as u32 - 1)
            }
            Seen::Pending => ConstVal::Pending,
            Seen::Unknown => ConstVal::Unknown,
        }
    }

    /// What a binding value holds, for the next run.
    ///
    /// # Arguments
    ///
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// What the run found.
    pub(crate) fn seen(&self, value: ConstVal) -> Seen
    {
        match value
        {
            ConstVal::Known(v) => Seen::Folded(Folded::Value(v)),
            ConstVal::Agg(at) => Seen::Folded(self.aggs[at as usize].clone()),
            ConstVal::Pending => Seen::Pending,
            ConstVal::Unknown => Seen::Unknown,
        }
    }

    /// Work out a value headed for a place of a kind: a value of one kind
    /// converted to it, or the items of a list or a struct.
    ///
    /// # Arguments
    ///
    /// * `e` - The value.
    /// * `want` - The kind of the place.
    ///
    /// # Returns
    ///
    /// The value, or the reason it has none.
    fn item(&self, e: &Expr, want: DataKind) -> Result<Folded, Stop>
    {
        ret_if!(want.is_aggregate(), self.folded(e));
        self.coerced(e, want).map(Folded::Value)
    }

    /// Work out an expression of each kind as the folder holds it: a list or
    /// a struct literal as its items, each item known, and a cell or a field
    /// of a known list or struct.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The value, or the reason it has none.
    pub(crate) fn folded(&self, e: &Expr) -> Result<Folded, Stop>
    {
        let kind = self.kinds.get(&e.id()).copied().ok_or(NONE)?;
        match e
        {
            Expr::List(n) =>
            {
                let cell = kind.cell_kind().ok_or(NONE)?;
                let items: Result<Vec<_>, _> =
                    n.cells.iter().map(|c| self.item(c, cell)).collect();
                items.map(Folded::Items)
            }
            Expr::Struct(n) =>
            {
                let s = kind.as_struct().ok_or(NONE)?;
                let field = |(name, k): &(String, DataKind)| {
                    let init = n.fields.iter().find(|f| f.field.name == name.as_str());
                    init.map_or_else(
                        || Ok(default_folded(*k)),
                        |f| self.item(&f.value, *k),
                    )
                };
                let items: Result<Vec<_>, _> = s.fields().iter().map(field).collect();
                items.map(Folded::Items)
            }
            Expr::BlankList(_) => Ok(Folded::Whole),
            Expr::Str(n) => Ok(self.str_folded(n)),
            Expr::Member(n) =>
            {
                let v = kind.as_enum().and_then(|e| e.member(&n.member.name));
                v.map(member_folded).ok_or(NONE)
            }
            Expr::Infix(n) if n.op == AstInfix::Add && kind.is_text() =>
            {
                self.joined([n.lhs.as_ref(), n.rhs.as_ref()].into_iter())
            }
            Expr::Verb(n)
                if matches!(n.op, VerbOp::Add | VerbOp::Join) && kind.is_text() =>
            {
                self.joined(n.args.iter())
            }
            Expr::Group(n) => self.folded(&n.inner),
            Expr::Name(n) => self.named_folded(n.name),
            Expr::Index(n) =>
            {
                let Folded::Items(items) = self.folded(&n.list)?
                else
                {
                    return Err(NONE);
                };
                let at = self.work_out(&n.index)?;
                let place = cell_place(at, items.len(), n.first, n.index.span())?;
                Ok(items[place].clone())
            }
            Expr::Field(n) =>
            {
                let Folded::Items(items) = self.folded(&n.base)?
                else
                {
                    return Err(NONE);
                };
                let base = self.kinds.get(&n.base.id()).and_then(|k| k.as_struct());
                let (at, _) = base.and_then(|s| s.field(&n.field.name)).ok_or(NONE)?;
                items.get(at).cloned().ok_or(NONE)
            }
            _ => self.work_out(e).map(Folded::Value),
        }
    }

    /// The folded text of a string literal: its text, with each template
    /// slot written as a print writes its value. A slot that does not fold
    /// leaves the text known as a whole.
    ///
    /// # Arguments
    ///
    /// * `n` - The literal.
    ///
    /// # Returns
    ///
    /// The text, or [`Folded::Whole`].
    fn str_folded(&self, n: &StrExpr) -> Folded
    {
        let mut out = String::new();
        for piece in &n.pieces
        {
            let text = match piece
            {
                StrPiece::Text(t) => Some(t.clone()),
                StrPiece::TemplateSlot(e) => self.slot_text(e),
            };
            let Some(text) = text
            else
            {
                return Folded::Whole;
            };
            out.push_str(&text);
        }
        Folded::Text(out)
    }

    /// The text a template slot writes, when its value folds: text as it is,
    /// a member of an enumeration as its name, and each other value as a print
    /// writes it.
    ///
    /// # Arguments
    ///
    /// * `e` - The value of the slot.
    ///
    /// # Returns
    ///
    /// The text, or `None` when the value does not fold to text or a scalar.
    fn slot_text(&self, e: &Expr) -> Option<String>
    {
        let kind = self.kinds.get(&e.id()).copied()?;
        let folded = self.folded(e).ok()?;
        if let Some(owner) = kind.as_enum()
        {
            return owner
                .find(|m| member_folded(m) == folded)
                .map(|at| owner.text_at(at));
        }
        match folded
        {
            Folded::Text(t) => Some(t),
            Folded::Value(v) => Some(value_text(v)),
            _ => None,
        }
    }

    /// The folded text of pieces joined in order.
    ///
    /// # Arguments
    ///
    /// * `pieces` - The pieces.
    ///
    /// # Returns
    ///
    /// The text, or the reason it has none: a piece that is no known text.
    fn joined<'e>(&self, pieces: impl Iterator<Item = &'e Expr>)
    -> Result<Folded, Stop>
    {
        let mut out = String::new();
        for piece in pieces
        {
            let Folded::Text(t) = self.folded(piece)?
            else
            {
                return Err(NONE);
            };
            out.push_str(&t);
        }
        Ok(Folded::Text(out))
    }

    /// The value of a constant in scope, of each kind.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The value, or the reason it has none.
    fn named_folded(&self, name: Name) -> Result<Folded, Stop>
    {
        let binding = self.scopes.lookup(&name).ok_or(NONE)?;
        ret_if!(binding.access != Access::Const, Err(NONE));
        match binding.value
        {
            ConstVal::Known(v) => Ok(Folded::Value(v)),
            ConstVal::Agg(at) => Ok(self.aggs[at as usize].clone()),
            ConstVal::Pending => Err(Stop::Pending),
            ConstVal::Unknown => Err(NONE),
        }
    }

    /// Work out an expression as lowering and the folder would: each
    /// operand in the kind the check recorded, converted where lowering
    /// converts it.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    ///
    /// # Returns
    ///
    /// The value, or the reason it has none.
    pub(crate) fn work_out(&self, e: &Expr) -> Worked
    {
        let kind = self.kinds.get(&e.id()).copied().ok_or(NONE)?;
        let kind = tern!(kind == DataKind::Null, DataKind::OPAQUE, kind);
        match e
        {
            Expr::Int(n) => Ok(self.int_lit(n, kind, false)),
            Expr::Float(n) => Ok(float_lit(n, kind)),
            Expr::Char(n) => Ok(Value::Char(n.value)),
            Expr::Bool(n) => Ok(Value::Bool(n.value)),
            Expr::Name(n) => self.named(n.name),
            Expr::Group(n) => self.work_out(&n.inner),
            Expr::Prefix(n) => self.prefix(n, kind),
            Expr::Not(n) => self.one_operand(PrefixOp::Not, &n.inner, kind, n.span),
            Expr::Infix(n) => self.infix(n.op, &n.lhs, &n.rhs, kind, n.span),
            Expr::Logic(n) =>
            {
                let op = tern!(n.op == AstLogic::And, LogicOp::And, LogicOp::Or);
                let (l, r) = (self.work_out(&n.lhs)?, self.work_out(&n.rhs)?);
                value::logic(op, l, r).map(Value::Bool).map_err(|_| NONE)
            }
            Expr::Verb(n) => self.verb(n, kind),
            Expr::SizeOf(n) => self.size_of(&n.kind),
            Expr::Member(n) =>
            {
                let v = kind.as_enum().and_then(|en| en.member(&n.member.name));
                v.and_then(MemberValue::scalar).ok_or(NONE)
            }
            Expr::Index(_) | Expr::Field(_) => match self.folded(e)?
            {
                Folded::Value(v) => Ok(v),
                _ => Err(NONE),
            },
            _ => Err(NONE),
        }
    }

    /// Work out an expression and convert it to a kind, as lowering
    /// converts an operand whose kind differs.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `want` - The kind.
    ///
    /// # Returns
    ///
    /// The value of kind `want`, or the reason it has none.
    pub(crate) fn coerced(&self, e: &Expr, want: DataKind) -> Worked
    {
        let v = self.work_out(e)?;
        ret_if!(v.kind() == want, Ok(v));
        value::convert(v, want, self.platform).map_err(|_| NONE)
    }

    /// The value of a constant in scope.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The value, or the reason it has none.
    fn named(&self, name: Name) -> Worked
    {
        let Some(binding) = self.scopes.lookup(&name)
        else
        {
            return Err(NONE);
        };
        match binding.value
        {
            ConstVal::Known(v) if binding.access == Access::Const => Ok(v),
            ConstVal::Pending if binding.access == Access::Const => Err(Stop::Pending),
            _ => Err(NONE),
        }
    }

    /// The value of an integer literal of the recorded kind.
    ///
    /// # Arguments
    ///
    /// * `n` - The literal.
    /// * `kind` - The recorded kind, an integer or a float.
    /// * `negated` - Whether a negate operator stands before it.
    ///
    /// # Returns
    ///
    /// The value.
    fn int_lit(&self, n: &IntLit, kind: DataKind, negated: bool) -> Value
    {
        let DataKind::Int(k) = kind
        else
        {
            let text = tern!(negated, format!("-{}", n.value), n.value.to_string());
            let bits = kind.bits(self.platform);
            return Value::float(&text, bits).unwrap_or(Value::default_of(kind));
        };
        let pattern = tern!(negated, n.value.wrapping_neg(), n.value);
        Value::int(k, pattern, self.platform)
    }

    /// Work out a prefix operator; `-` before an integer literal with no
    /// kind prefix is a negative literal.
    ///
    /// # Arguments
    ///
    /// * `n` - The expression.
    /// * `kind` - Its recorded kind.
    ///
    /// # Returns
    ///
    /// The value, or the reason it has none.
    fn prefix(&self, n: &PrefixExpr, kind: DataKind) -> Worked
    {
        if let Expr::Int(lit) = n.rhs.as_ref()
        {
            let negative = lit.kind.is_none() && n.op == AstPrefix::Neg;
            ret_if!(negative, Ok(self.int_lit(lit, kind, true)));
        }
        let op = match n.op
        {
            AstPrefix::Neg => PrefixOp::Neg,
            AstPrefix::BitNot => PrefixOp::BitNot,
            AstPrefix::Not => PrefixOp::Not,
            AstPrefix::AddrOf | AstPrefix::Deref => return Err(NONE),
        };
        self.one_operand(op, &n.rhs, kind, n.span)
    }

    /// Work out a one-operand operator.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `operand` - The operand.
    /// * `kind` - The recorded kind of the operation.
    /// * `span` - The operation.
    ///
    /// # Returns
    ///
    /// The value, or the reason it has none.
    fn one_operand(
        &self, op: PrefixOp, operand: &Expr, kind: DataKind, span: Span,
    ) -> Worked
    {
        let v = self.work_out(operand)?;
        value::prefix(op, v, self.platform)
            .map_err(|e| failed(e, op.into(), kind, span))
    }

    /// Work out a notation operator.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    /// * `kind` - The recorded kind of the operation.
    /// * `span` - The operation.
    ///
    /// # Returns
    ///
    /// The value, or the reason it has none.
    fn infix(
        &self, op: AstInfix, lhs: &Expr, rhs: &Expr, kind: DataKind, span: Span,
    ) -> Worked
    {
        let cmp = match op
        {
            AstInfix::Eq => Some(CmpOp::Eq),
            AstInfix::Ne => Some(CmpOp::Ne),
            AstInfix::Lt => Some(CmpOp::Lt),
            AstInfix::Gt => Some(CmpOp::Gt),
            AstInfix::Le => Some(CmpOp::Le),
            AstInfix::Ge => Some(CmpOp::Ge),
            _ => None,
        };
        ret_if!(let Some(cmp) = cmp, self.compare(cmp, lhs, rhs));
        let op = match op
        {
            AstInfix::Add => InfixOp::Add,
            AstInfix::Sub => InfixOp::Sub,
            AstInfix::Mul => InfixOp::Mul,
            AstInfix::Div => InfixOp::Div,
            AstInfix::Rem => InfixOp::Rem,
            AstInfix::BitAnd => InfixOp::BitAnd,
            AstInfix::BitOr => InfixOp::BitOr,
            AstInfix::BitXor => InfixOp::BitXor,
            AstInfix::Shl => InfixOp::Shl,
            _ => InfixOp::Shr,
        };
        self.arith(op, lhs, rhs, kind, span)
    }

    /// Work out an arithmetic, bitwise or shift operation: both operands in
    /// the result kind, except a shift distance, which keeps its own.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    /// * `kind` - The result kind.
    /// * `span` - The operation.
    ///
    /// # Returns
    ///
    /// The value, `ZK0601`, `ZK0602` or `ZK0407` for an operation with no
    /// value, or the reason it has none.
    fn arith(
        &self, op: InfixOp, lhs: &Expr, rhs: &Expr, kind: DataKind, span: Span,
    ) -> Worked
    {
        let kind_of = |e: &Expr| self.kinds.get(&e.id()).copied();
        let ptr = kind_of(lhs).is_some_and(DataKind::is_ptr)
            || kind_of(rhs).is_some_and(DataKind::is_ptr);
        ret_if!(ptr || kind.is_text(), Err(NONE));
        let shift = matches!(op, InfixOp::Shl | InfixOp::Shr);
        let l = self.coerced(lhs, kind)?;
        let r = tern!(shift, self.work_out(rhs)?, self.coerced(rhs, kind)?);
        let bits = kind.bits(self.platform);
        if let Value::Int { raw, .. } = r
        {
            let far = shift && raw >= u128::from(bits);
            ret_if!(
                far,
                Err(Stop::Not(Some(kd::shift_too_far(
                    rhs.span(),
                    &raw.to_string(),
                    bits
                ))))
            );
        }
        value::infix(op, l, r, self.platform)
            .map_err(|e| failed(e, op.into(), kind, span))
    }

    /// Work out a comparison: both operands in their common kind.
    ///
    /// # Arguments
    ///
    /// * `op` - The comparison.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    ///
    /// # Returns
    ///
    /// The truth value, or the reason it has none.
    fn compare(&self, op: CmpOp, lhs: &Expr, rhs: &Expr) -> Worked
    {
        let kind_of = |e: &Expr| self.kinds.get(&e.id()).copied().ok_or(NONE);
        let (a, b) = (kind_of(lhs)?, kind_of(rhs)?);
        ret_if!(a.is_textual() || b.is_textual(), Err(NONE));
        let common = tern!(a.same_shape(b) || !a.widens_to(b, self.platform), a, b);
        let common = tern!(common == DataKind::Null, DataKind::OPAQUE, common);
        let (l, r) = (self.coerced(lhs, common)?, self.coerced(rhs, common)?);
        value::compare(op, l, r).map(Value::Bool).map_err(|_| NONE)
    }

    /// Work out a verb clause: an arithmetic, bitwise or shift verb, a
    /// comparison, `~` or a conversion.
    ///
    /// # Arguments
    ///
    /// * `n` - The clause.
    /// * `kind` - Its recorded kind.
    ///
    /// # Returns
    ///
    /// The value, or the reason it has none.
    fn verb(&self, n: &VerbExpr, kind: DataKind) -> Worked
    {
        let op = match n.op
        {
            VerbOp::Add => Some(InfixOp::Add),
            VerbOp::Sub => Some(InfixOp::Sub),
            VerbOp::Mul => Some(InfixOp::Mul),
            VerbOp::Div => Some(InfixOp::Div),
            VerbOp::Rem => Some(InfixOp::Rem),
            VerbOp::BitAnd => Some(InfixOp::BitAnd),
            VerbOp::BitOr => Some(InfixOp::BitOr),
            VerbOp::BitXor => Some(InfixOp::BitXor),
            VerbOp::Shift =>
            {
                Some(tern!(n.dir == Some(Dir::Right), InfixOp::Shr, InfixOp::Shl))
            }
            _ => None,
        };
        let args = n.args.as_slice();
        ret_if!(let (Some(op), [l, r]) = (op, args), self.arith(op, l, r, kind, n.span));
        let cmp = match n.op
        {
            VerbOp::Eq => Some(CmpOp::Eq),
            VerbOp::Ne => Some(CmpOp::Ne),
            VerbOp::Gt => Some(CmpOp::Gt),
            VerbOp::Le => Some(CmpOp::Le),
            VerbOp::Lt => Some(CmpOp::Lt),
            VerbOp::Ge => Some(CmpOp::Ge),
            _ => None,
        };
        ret_if!(let (Some(cmp), [l, r]) = (cmp, args), self.compare(cmp, l, r));
        match (n.op, args)
        {
            (VerbOp::BitNot, [v]) =>
            {
                self.one_operand(PrefixOp::BitNot, v, kind, n.span)
            }
            (VerbOp::Cast, [v]) =>
            {
                value::convert(self.work_out(v)?, kind, self.platform).map_err(|_| NONE)
            }
            (VerbOp::CellCount, [v]) => self.cell_count(v),
            _ => Err(NONE),
        }
    }

    /// The cell count of a list: the count of a counted list, or of a list
    /// pointed at, from its kind, as lowering takes it; the count of a plain
    /// list from its known cells, as the folder takes it.
    ///
    /// # Arguments
    ///
    /// * `list` - The list.
    ///
    /// # Returns
    ///
    /// The count, a `usize`, or the reason it has none.
    fn cell_count(&self, list: &Expr) -> Worked
    {
        let kind = self.kinds.get(&list.id()).copied().ok_or(NONE)?;
        let counted =
            kind.as_list().or_else(|| kind.pointee().and_then(DataKind::as_list));
        let usize_kind = DataKind::USIZE.int_kind().expect("usize is an integer kind");
        let count = |n: u128| Value::int(usize_kind, n, self.platform);
        // A count of 0 stands for one still open.
        ret_if!(counted.is_some_and(|l| l.len == 0), Err(Stop::Pending));
        ret_if!(let Some(l) = counted, Ok(count(u128::from(l.len))));
        ret_if!(!kind.is_seq() || kind.is_textual(), Err(NONE));
        let Folded::Items(items) = self.folded(list)?
        else
        {
            return Err(NONE);
        };
        Ok(count(items.len() as u128))
    }

    /// The size of a kind, in octets, as a `usize`; it waits while the
    /// size hangs on a count still open.
    ///
    /// # Arguments
    ///
    /// * `written` - The kind phrase.
    ///
    /// # Returns
    ///
    /// The size, or the reason it has none.
    fn size_of(&self, written: &Kind) -> Worked
    {
        ret_if!(size_waits(written, &self.open_structs), Err(Stop::Pending));
        let kind = DataKind::from_ast(written, &self.structs);
        let usize_kind = DataKind::USIZE.int_kind().expect("usize is an integer kind");
        Ok(Value::int(usize_kind, u128::from(kind.size(self.platform)), self.platform))
    }
}

/// The place of a cell from its number, as the folder takes it: a number
/// outside the list is `ZK0603`.
///
/// # Arguments
///
/// * `at` - The cell number.
/// * `len` - The number of cells.
/// * `first` - The number of the first cell: 0 or 1.
/// * `span` - The cell number.
///
/// # Returns
///
/// The place, from 0, or the reason it has none.
fn cell_place(at: Value, len: usize, first: u8, span: Span) -> Result<usize, Stop>
{
    let Value::Int { kind, raw } = at
    else
    {
        return Err(NONE);
    };
    let wide = i128::try_from(raw).unwrap_or(i128::MAX);
    let index = tern!(kind.signed, raw as i128, wide);
    let lo = i128::from(first);
    ret_if!(index >= lo && index < lo + len as i128, Ok((index - lo) as usize));
    let last = (len as u64 + u64::from(first)).saturating_sub(1);
    Err(Stop::Not(Some(fd::cell_out_of_range(span, &index.to_string(), first, last))))
}

/// The count of a list count, from its kind and its value.
///
/// # Arguments
///
/// * `e` - The count as written.
/// * `kind` - The kind the check found for it.
/// * `worked` - Its value, or the reason it has none.
/// * `reported` - Whether the check of it reported a mistake.
///
/// # Returns
///
/// The count, or the diagnostic to report while it remains open.
fn count_of(
    e: &Expr, kind: Option<DataKind>, worked: Worked, reported: bool,
) -> Result<u32, Option<Diagnostic>>
{
    let bad = |text: &str| Some(pd::bad_list_count(e.span(), text));
    ret_if!(reported, Err(None));
    ret_if!(!kind.is_some_and(DataKind::is_int), Err(bad(&e.to_sexpr())));
    let v = match worked
    {
        Ok(v) => v,
        Err(Stop::Not(Some(d))) => return Err(Some(d)),
        Err(_) => return Err(bad(&e.to_sexpr())),
    };
    let Value::Int { kind, raw } = v
    else
    {
        return Err(bad(&value_text(v)));
    };
    let fits = !(kind.signed && (raw as i128) < 0);
    let count = u32::try_from(raw).ok().filter(|c| fits && *c > 0);
    count.ok_or_else(|| bad(&value_text(v)))
}

/// The diagnostic of an operation on constants with no value, as the
/// folder reports it.
///
/// # Arguments
///
/// * `err` - The reason it has none.
/// * `op` - The operator spelling.
/// * `kind` - The result kind.
/// * `span` - The operation.
///
/// # Returns
///
/// The reason.
fn failed(err: EvalError, op: &str, kind: DataKind, span: Span) -> Stop
{
    let diag = match err
    {
        EvalError::Overflow => fd::constant_overflow(span, op, &kind.name()),
        EvalError::ZeroDivisor => fd::constant_zero_divisor(span, op),
        EvalError::ShiftTooFar { distance, bits } =>
        {
            kd::shift_too_far(span, &distance.to_string(), bits)
        }
        EvalError::Mismatch => return NONE,
    };
    Stop::Not(Some(diag))
}

/// The value of a float literal of the recorded kind.
///
/// # Arguments
///
/// * `n` - The literal.
/// * `kind` - The recorded kind.
///
/// # Returns
///
/// The value.
fn float_lit(n: &FloatLit, kind: DataKind) -> Value
{
    Value::float(&n.text, kind.bits(64)).unwrap_or(Value::default_of(kind))
}

/// Whether the size of a kind waits for a count still open: a count
/// of the counted lists that hold the rest inline, or a struct held inline
/// that has one. A level of address, a slice or a plain list holds the rest
/// elsewhere, so its size does not wait.
///
/// # Arguments
///
/// * `written` - The kind phrase.
/// * `open` - The structs whose size waits.
///
/// # Returns
///
/// `true` when it waits.
fn size_waits(written: &Kind, open: &HashSet<Name>) -> bool
{
    for wrap in written.wraps.iter().rev()
    {
        match wrap
        {
            Wrap::List(Cells::Expr(_)) => return true,
            Wrap::List(Cells::Fixed(_)) => continue,
            _ => return false,
        }
    }
    written.name.as_ref().is_some_and(|n| open.contains(&n.name))
}

/// The structs whose size waits for a count still open: one with
/// such a count in a field held inline, or one that holds such a struct
/// inline.
///
/// # Arguments
///
/// * `program` - The program.
///
/// # Returns
///
/// The struct names.
pub(crate) fn open_structs(program: &Program) -> HashSet<Name>
{
    let decls: Vec<_> = program
        .stmts
        .iter()
        .filter_map(|s| tern!(let Stmt::Struct(d) = s, Some(d), None))
        .collect();
    let mut open = HashSet::new();
    loop
    {
        let grown = decls
            .iter()
            .filter(|d| !open.contains(&d.name.name))
            .find(|d| d.fields.iter().any(|f| size_waits(&f.kind, &open)));
        let Some(d) = grown
        else
        {
            return open;
        };
        open.insert(d.name.name);
    }
}

/// Write each list count that the check worked out back into the program
/// as digits.
///
/// # Arguments
///
/// * `program` - The program.
/// * `counts` - The counts.
///
/// # Returns
///
/// The number of counts written.
pub(crate) fn write_counts(program: &mut Program, counts: &CountTable) -> usize
{
    /// The walker that writes the counts.
    struct Writer<'a>(&'a CountTable, usize);
    impl VisitorMut for Writer<'_>
    {
        fn visit_kind_mut(&mut self, n: &mut Kind)
        {
            for wrap in &mut n.wraps
            {
                let Wrap::List(cells) = wrap
                else
                {
                    continue;
                };
                let Cells::Expr(e) = cells
                else
                {
                    continue;
                };
                // A count can hold kinds with counts of their own, as in a
                // size.
                self.visit_expr_mut(e);
                let Some(Ok(count)) = self.0.get(&e.id())
                else
                {
                    continue;
                };
                *cells = Cells::Fixed(*count);
                self.1 += 1;
            }
        }
    }
    let mut writer = Writer(counts, 0);
    writer.visit_program_mut(program);
    writer.1
}

/// The folded value of a field that a struct literal leaves out: the default
/// of its kind, the first member for an enumeration.
///
/// # Arguments
///
/// * `kind` - The kind of the field.
///
/// # Returns
///
/// The value.
fn default_folded(kind: DataKind) -> Folded
{
    ret_if!(let Some(e) = kind.as_enum(), member_folded(e.first()));
    ret_if!(kind.is_text(), Folded::Text(String::new()));
    tern!(kind.is_aggregate(), Folded::Whole, Folded::Value(Value::default_of(kind)))
}

/// A member value as the folder holds it.
///
/// # Arguments
///
/// * `v` - The member value.
///
/// # Returns
///
/// The value.
pub(crate) fn member_folded(v: &MemberValue) -> Folded
{
    match v
    {
        MemberValue::Scalar(x) => Folded::Value(*x),
        MemberValue::Text(t) => Folded::Text(t.clone()),
        MemberValue::Items(items) =>
        {
            Folded::Items(items.iter().map(member_folded).collect())
        }
    }
}
