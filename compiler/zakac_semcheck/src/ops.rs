use zakac_ast::expr::Expr;
use zakac_ast::expr::InfixOp;
use zakac_ast::expr::PrefixOp;
use zakac_ast::verb::VERBS;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::kinds as kd;
use zakac_i18n::t;
use zakac_lexer::token::Kw;
use zakac_parser::describe::kw_text;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::kind::Base;
use crate::kind::DataKind;
use crate::semcheck::Checker;

/// What an operator does with its operands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpClass
{
    /// `+ - * / %` and their verbs.
    Arith,
    /// `& | ^` and their verbs.
    Bit,
    /// `== !=` and their verbs.
    Equal,
    /// `< > <= >=` and their verbs.
    Order,
    /// `<< >>` and the shift verb.
    Shift,
    /// The bitwise-not verb.
    BitNot,
    /// The conversion verb.
    Cast,
    /// The address-of verb.
    AddrOf,
    /// The dereference verb.
    Deref,
    /// The lend verb.
    Lend,
    /// The list cell verbs.
    CellCount,
    /// A statement verb, which never stands inside an expression.
    Stmt,
}

impl OpClass
{
    /// Whether the operator compares, giving a boolean.
    ///
    /// # Returns
    ///
    /// `true` for equalities and orderings.
    pub fn is_cmp(self) -> bool { matches!(self, OpClass::Equal | OpClass::Order) }

    /// What both operands of a two-operand class must be.
    ///
    /// # Returns
    ///
    /// The demand, or `None` when operands of all kinds are accepted.
    fn want(self) -> Option<Want>
    {
        match self
        {
            OpClass::Arith => Some(Want::Number),
            OpClass::Bit | OpClass::BitNot | OpClass::Shift => Some(Want::Int),
            OpClass::Order => Some(Want::Ordered),
            OpClass::Equal
            | OpClass::Cast
            | OpClass::AddrOf
            | OpClass::Deref
            | OpClass::Lend
            | OpClass::CellCount
            | OpClass::Stmt => None,
        }
    }
}

/// What an operator demands of an operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Want
{
    /// An integer or a float.
    Number,
    /// An integer.
    Int,
    /// An unsigned integer.
    Unsigned,
    /// A signed integer or a float.
    Signed,
    /// A boolean.
    Bool,
    /// A number or a character.
    Ordered,
}

impl Want
{
    /// Whether `kind` meets the demand.
    ///
    /// # Arguments
    ///
    /// * `kind` - The operand kind.
    ///
    /// # Returns
    ///
    /// `true` when it does.
    pub fn accepts(self, kind: DataKind) -> bool
    {
        match self
        {
            Want::Number => kind.is_numeric(),
            Want::Int => kind.is_int(),
            Want::Unsigned => kind.is_unsigned(),
            Want::Signed => kind.is_float() || (kind.is_int() && !kind.is_unsigned()),
            Want::Bool => kind == DataKind::Bool,
            Want::Ordered => kind.is_numeric() || kind == DataKind::Char,
        }
    }

    /// The demand in words, for a message.
    ///
    /// # Returns
    ///
    /// The localized words.
    pub fn text(self) -> String
    {
        let text = match self
        {
            Want::Number => t!("semcheck.want.number"),
            Want::Int => t!("semcheck.want.int"),
            Want::Unsigned => t!("semcheck.want.unsigned"),
            Want::Signed => t!("semcheck.want.signed"),
            Want::Bool => t!("semcheck.want.bool"),
            Want::Ordered => t!("semcheck.want.ordered"),
        };
        text.to_string()
    }
}

/// The class of a notation operator.
///
/// # Arguments
///
/// * `op` - The operator.
///
/// # Returns
///
/// The class.
pub(crate) fn infix_class(op: InfixOp) -> OpClass
{
    match op
    {
        InfixOp::Add | InfixOp::Sub | InfixOp::Mul | InfixOp::Div | InfixOp::Rem =>
        {
            OpClass::Arith
        }
        InfixOp::BitAnd | InfixOp::BitOr | InfixOp::BitXor => OpClass::Bit,
        InfixOp::Shl | InfixOp::Shr => OpClass::Shift,
        InfixOp::Eq | InfixOp::Ne => OpClass::Equal,
        InfixOp::Lt | InfixOp::Gt | InfixOp::Le | InfixOp::Ge => OpClass::Order,
    }
}

/// The class of a verb.
///
/// # Arguments
///
/// * `op` - The verb operation.
///
/// # Returns
///
/// The class.
pub(crate) fn verb_class(op: VerbOp) -> OpClass
{
    match op
    {
        VerbOp::Add | VerbOp::Sub | VerbOp::Mul | VerbOp::Div | VerbOp::Rem =>
        {
            OpClass::Arith
        }
        VerbOp::BitAnd | VerbOp::BitOr | VerbOp::BitXor => OpClass::Bit,
        VerbOp::BitNot => OpClass::BitNot,
        VerbOp::Shift => OpClass::Shift,
        VerbOp::Eq | VerbOp::Ne => OpClass::Equal,
        VerbOp::Gt | VerbOp::Le | VerbOp::Lt | VerbOp::Ge => OpClass::Order,
        VerbOp::Cast => OpClass::Cast,
        VerbOp::AddrOf => OpClass::AddrOf,
        VerbOp::Deref => OpClass::Deref,
        VerbOp::Lend => OpClass::Lend,
        VerbOp::Cell | VerbOp::CellCount => OpClass::CellCount,
        VerbOp::Print | VerbOp::Assign | VerbOp::Return => OpClass::Stmt,
    }
}

/// The spelling of a verb, for a message.
///
/// # Arguments
///
/// * `op` - The verb operation.
///
/// # Returns
///
/// The verb as written in the source language.
pub(crate) fn verb_label(op: VerbOp) -> String
{
    VERBS.iter().find(|v| v.op == op).map(|v| kw_text(v.kw)).unwrap_or_default()
}

/// The spelling of a reserved word, for a message.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// The word.
pub(crate) fn kw_label(kw: Kw) -> String { kw_text(kw) }

/// Whether an expression is built from unprefixed literals alone, so its
/// kind can still follow the place it stands in.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// `true` for an unprefixed number literal, or an arithmetic, bitwise,
/// negate or shift tree whose value operands are such literals.
pub(crate) fn is_flexible(e: &Expr) -> bool
{
    match e
    {
        Expr::Int(n) => n.kind.is_none(),
        Expr::Float(n) => n.kind.is_none(),
        Expr::Group(n) => is_flexible(&n.inner),
        Expr::Prefix(n) =>
        {
            matches!(n.op, PrefixOp::Neg | PrefixOp::BitNot) && is_flexible(&n.rhs)
        }
        Expr::Infix(n) => infix_flexible(infix_class(n.op), &n.lhs, &n.rhs),
        Expr::Verb(n) => verb_flexible(verb_class(n.op), &n.args),
        Expr::List(_) | Expr::BlankList(_) => true,
        _ => false,
    }
}

/// Whether a notation operation is built from flexible operands.
///
/// # Arguments
///
/// * `class` - The operator class.
/// * `lhs` - The left operand.
/// * `rhs` - The right operand.
///
/// # Returns
///
/// `true` when the result follows the place it stands in.
fn infix_flexible(class: OpClass, lhs: &Expr, rhs: &Expr) -> bool
{
    match class
    {
        OpClass::Arith | OpClass::Bit => is_flexible(lhs) && is_flexible(rhs),
        OpClass::Shift => is_flexible(lhs),
        _ => false,
    }
}

/// Whether a verb clause is built from flexible operands.
///
/// # Arguments
///
/// * `class` - The verb class.
/// * `args` - The operands.
///
/// # Returns
///
/// `true` when the result follows the place it stands in.
fn verb_flexible(class: OpClass, args: &[Expr]) -> bool
{
    match class
    {
        OpClass::Arith | OpClass::Bit =>
        {
            !args.is_empty() && args.iter().all(is_flexible)
        }
        OpClass::Shift | OpClass::BitNot => args.first().is_some_and(is_flexible),
        _ => false,
    }
}

/// The expected kind for an operand of an operator that takes integers.
///
/// # Arguments
///
/// * `expect` - The kind the place of the operation expects.
///
/// # Returns
///
/// `expect` when it is an integer kind, else `None`, so a float place does
/// not turn an integer literal operand into a float.
pub(crate) fn int_expect(expect: Option<DataKind>) -> Option<DataKind>
{
    expect.filter(|k| k.is_int())
}

/// Whether an operator spelling is the given notation operator or verb.
///
/// # Arguments
///
/// * `op` - The operator spelling.
/// * `infix` - The notation operator.
/// * `verb` - The verb of the same operation.
///
/// # Returns
///
/// `true` when `op` spells one of the two.
fn is_op(op: &str, infix: InfixOp, verb: VerbOp) -> bool
{
    op == infix.as_ref() || op == verb_label(verb)
}

/// The diagnostic for two operands an address operation does not join.
///
/// # Arguments
///
/// * `l` - The left kind.
/// * `r` - The right kind.
/// * `op` - The operator spelling.
/// * `span` - The operation.
///
/// # Returns
///
/// `ZK0416` for two addresses of differing kinds, else `ZK0415`.
fn ptr_misuse(l: DataKind, r: DataKind, op: &str, span: Span) -> Diagnostic
{
    let differ = l.is_address() && r.is_address() && !l.same_shape(r);
    let (a, b) = (l.name(), r.name());
    tern!(
        differ,
        kd::pointer_kinds_differ(span, &a, &b),
        kd::pointer_operator(span, op)
    )
}

/// Whether a flexible expression holds a floating-point literal.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// `true` when one of its literals is a float.
fn has_float(e: &Expr) -> bool
{
    match e
    {
        Expr::Float(_) => true,
        Expr::Group(n) => has_float(&n.inner),
        Expr::Prefix(n) => has_float(&n.rhs),
        Expr::Infix(n) => has_float(&n.lhs) || has_float(&n.rhs),
        Expr::Verb(n) => n.args.iter().any(has_float),
        _ => false,
    }
}

impl Checker
{
    /// The wider of two kinds, when one converts to the other.
    ///
    /// # Arguments
    ///
    /// * `a` - One kind.
    /// * `b` - The other kind.
    ///
    /// # Returns
    ///
    /// The common kind, or `None`.
    fn wider(&self, a: DataKind, b: DataKind) -> Option<DataKind>
    {
        ret_if!(a.widens_to(b, self.platform), Some(b));
        tern!(b.widens_to(a, self.platform), Some(a), None)
    }

    /// The common kind of two operands, reporting when there is none.
    ///
    /// # Arguments
    ///
    /// * `a` - One kind.
    /// * `b` - The other kind.
    /// * `span` - The operation.
    ///
    /// # Returns
    ///
    /// The common kind, or `None` after the report.
    pub fn common_or_report(
        &mut self, a: DataKind, b: DataKind, span: Span,
    ) -> Option<DataKind>
    {
        let common = self.wider(a, b);
        ret_if!(common.is_some(), common);
        self.report(kd::incompatible_operands(span, &a.name(), &b.name()));
        None
    }

    /// Report an operand that does not meet its operator's demand; an
    /// address gets the pointer-operator error.
    ///
    /// # Arguments
    ///
    /// * `kind` - The operand kind.
    /// * `want` - The demand.
    /// * `op` - The operator spelling.
    /// * `operand` - The operand.
    ///
    /// # Returns
    ///
    /// The kind when it meets the demand.
    pub fn need(
        &mut self, kind: DataKind, want: Want, op: &str, operand: &Expr,
    ) -> Option<DataKind>
    {
        ret_if!(want.accepts(kind), Some(kind));
        let span = operand.span();
        let ptr = kind.is_address();
        let other = || kd::operand_kind(span, op, &want.text(), &kind.name());
        self.report(tern!(ptr, kd::pointer_operator(span, op), other()));
        None
    }

    /// Infer two operands so that a literal operand follows the other one.
    ///
    /// # Arguments
    ///
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    /// * `expect` - The kind the place of the operation expects.
    ///
    /// # Returns
    ///
    /// The two kinds, each `None` after an error.
    pub fn infer_sides(
        &mut self, lhs: &Expr, rhs: &Expr, expect: Option<DataKind>,
    ) -> (Option<DataKind>, Option<DataKind>)
    {
        match (is_flexible(lhs), is_flexible(rhs))
        {
            (true, false) =>
            {
                let r = self.infer(rhs, expect);
                (self.infer(lhs, r.or(expect)), r)
            }
            (false, true) =>
            {
                let l = self.infer(lhs, expect);
                (l, self.infer(rhs, l.or(expect)))
            }
            (both, _) =>
            {
                let float =
                    both && expect.is_none() && (has_float(lhs) || has_float(rhs));
                let want = tern!(float, Some(DataKind::F64), expect);
                (self.infer(lhs, want), self.infer(rhs, want))
            }
        }
    }

    /// Infer a two-operand operation that is no shift.
    ///
    /// # Arguments
    ///
    /// * `class` - The operator class.
    /// * `op` - The operator spelling.
    /// * `lhs` - The left operand.
    /// * `rhs` - The right operand.
    /// * `expect` - The kind the place of the operation expects.
    /// * `span` - The operation.
    ///
    /// # Returns
    ///
    /// The result kind, or `None` after an error.
    #[rustfmt::skip]
    pub fn infer_pair(
        &mut self, class: OpClass, op: &str, lhs: &Expr, rhs: &Expr,
        expect: Option<DataKind>, span: Span,
    ) -> Option<DataKind>
    {
        let side = match class
        {
            OpClass::Equal | OpClass::Order => None,
            OpClass::Bit => int_expect(expect),
            _ => expect.filter(|k| !k.is_address()),
        };
        let (l, r) = self.infer_sides(lhs, rhs, side);
        let (l, r) = (l?, r?);
        let block = l.is_aggregate() || r.is_aggregate();
        ret_if!(block, self.infer_list_pair(class, op, (l, lhs), (r, rhs), span));
        let ptr = l.is_address() || r.is_address();
        ret_if!(ptr, self.infer_ptr_pair(class, op, (l, lhs), (r, rhs), span));
        let want = class.want();
        let l = want.map_or(Some(l), |w| self.need(l, w, op, lhs))?;
        let r = want.map_or(Some(r), |w| self.need(r, w, op, rhs))?;
        let common = self.common_or_report(l, r, span)?;
        Some(tern!(class.is_cmp(), DataKind::Bool, common))
    }

    /// Infer a two-operand operation where at least one side is an address.
    ///
    /// # Arguments
    ///
    /// * `class` - The operator class.
    /// * `op` - The operator spelling.
    /// * `lhs` - The kind and the expression of the left operand.
    /// * `rhs` - The kind and the expression of the right operand.
    /// * `span` - The operation.
    ///
    /// # Returns
    ///
    /// The result kind, or `None` after an error.
    pub fn infer_ptr_pair(
        &mut self, class: OpClass, op: &str, lhs: (DataKind, &Expr),
        rhs: (DataKind, &Expr), span: Span,
    ) -> Option<DataKind>
    {
        match class
        {
            OpClass::Equal => self.ptr_compare(op, lhs.0, rhs.0, true, span),
            OpClass::Order => self.ptr_compare(op, lhs.0, rhs.0, false, span),
            OpClass::Arith => self.ptr_arith(op, lhs, rhs, span),
            _ =>
            {
                self.report(kd::pointer_operator(span, op));
                None
            }
        }
    }

    /// Infer a two-operand operation where at least one side is a list or a
    /// struct: `==` and `!=` compare two lists of one kind, one cell at a
    /// time, and other operators take no lists. No operator takes a struct.
    ///
    /// # Arguments
    ///
    /// * `class` - The operator class.
    /// * `op` - The operator spelling.
    /// * `lhs` - The kind and the expression of the left operand.
    /// * `rhs` - The kind and the expression of the right operand.
    /// * `span` - The operation.
    ///
    /// # Returns
    ///
    /// `bool` for a comparison, or `None` after an error.
    fn infer_list_pair(
        &mut self, class: OpClass, op: &str, lhs: (DataKind, &Expr),
        rhs: (DataKind, &Expr), span: Span,
    ) -> Option<DataKind>
    {
        let structs = lhs.0.is_struct() || rhs.0.is_struct();
        let equal = class == OpClass::Equal && !structs;
        ret_if!(equal && lhs.0 == rhs.0, Some(DataKind::Bool));
        let mismatch = kd::incompatible_operands(span, &lhs.0.name(), &rhs.0.name());
        ret_if!(equal, self.report(mismatch), None);
        let (kind, e) = tern!(lhs.0.is_aggregate(), lhs, rhs);
        self.need(kind, class.want().unwrap_or(Want::Number), op, e)
    }

    /// Infer a comparison of addresses.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator spelling.
    /// * `l` - The left kind.
    /// * `r` - The right kind.
    /// * `loose` - `true` for `== !=`, which also join a pointer with `null` or
    ///   `ptr`.
    /// * `span` - The operation.
    ///
    /// # Returns
    ///
    /// `bool`, or `None` after an error.
    fn ptr_compare(
        &mut self, op: &str, l: DataKind, r: DataKind, loose: bool, span: Span,
    ) -> Option<DataKind>
    {
        let same = l.same_shape(r) && (loose || l.is_ptr());
        let (l_owner, r_owner) = (l.as_owner(), r.as_owner());
        let joined = self.wider(l_owner, r_owner).is_some_and(DataKind::is_address);
        let joined = loose && (joined || l_owner.meets_pointer(r_owner));
        ret_if!(same || joined, Some(DataKind::Bool));
        self.report(ptr_misuse(l, r, op, span));
        None
    }

    /// Infer pointer arithmetic: a pointer moved forward or back, or the
    /// distance between two pointers of one kind.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator spelling.
    /// * `lhs` - The kind and the expression of the left operand.
    /// * `rhs` - The kind and the expression of the right operand.
    /// * `span` - The operation.
    ///
    /// # Returns
    ///
    /// The pointer kind as a borrow, since an address inside a block cannot
    /// own it; `isize` for a distance, or `None` after an error.
    fn ptr_arith(
        &mut self, op: &str, lhs: (DataKind, &Expr), rhs: (DataKind, &Expr), span: Span,
    ) -> Option<DataKind>
    {
        let add = is_op(op, InfixOp::Add, VerbOp::Add);
        let sub = is_op(op, InfixOp::Sub, VerbOp::Sub);
        ret_if!(!add && !sub, self.report(kd::pointer_operator(span, op)), None);
        let blank =
            [lhs, rhs].into_iter().find(|(k, _)| k.levels() == (1, Base::Opaque));
        let opaque = blank.map(|(_, e)| kd::opaque_pointer(e.span()));
        ret_if!(let Some(d) = opaque, self.report(d), None);
        let (l, r) = (lhs.0, rhs.0);
        ret_if!(l.is_ptr() && r.is_int(), Some(l.as_borrowed()));
        ret_if!(add && l.is_int() && r.is_ptr(), Some(r.as_borrowed()));
        ret_if!(sub && l.is_ptr() && l.same_shape(r), Some(DataKind::ISIZE));
        self.report(ptr_misuse(l, r, op, span));
        None
    }

    /// Infer a shift: the value keeps its kind, the distance is unsigned.
    ///
    /// # Arguments
    ///
    /// * `op` - The operator spelling.
    /// * `value` - The shifted value.
    /// * `distance` - The distance.
    /// * `expect` - The kind the place of the shift expects.
    ///
    /// # Returns
    ///
    /// The kind of the value, or `None` after an error.
    pub fn infer_shift(
        &mut self, op: &str, value: &Expr, distance: &Expr, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let k = self.infer(value, int_expect(expect));
        let d = self.infer(distance, Some(DataKind::USIZE));
        let k = k.and_then(|k| self.need(k, Want::Int, op, value));
        d.and_then(|d| self.need(d, Want::Unsigned, op, distance))?;
        let k = k?;
        self.check_distance(distance, k);
        Some(k)
    }

    /// Report a literal shift distance that is not below the bit width.
    ///
    /// # Arguments
    ///
    /// * `distance` - The distance.
    /// * `kind` - The kind of the shifted value.
    fn check_distance(&mut self, distance: &Expr, kind: DataKind)
    {
        let Expr::Int(lit) = distance
        else
        {
            return;
        };
        let bits = kind.bits(self.platform);
        ret_if!(lit.value < u128::from(bits));
        self.report(kd::shift_too_far(lit.span, &lit.value.to_string(), bits));
    }
}
