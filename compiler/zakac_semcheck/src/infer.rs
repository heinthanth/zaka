use zakac_ast::expr::CallExpr;
use zakac_ast::expr::Expr;
use zakac_ast::expr::FloatLit;
use zakac_ast::expr::InfixExpr;
use zakac_ast::expr::IntLit;
use zakac_ast::expr::LogicExpr;
use zakac_ast::expr::LogicOp;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::PrefixExpr;
use zakac_ast::expr::PrefixOp;
use zakac_ast::expr::VerbExpr;
use zakac_ast::visit::Node;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::resolve as rd;
use zakac_i18n::t;
use zakac_lexer::token::Kw;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::kind::DataKind;
use crate::list::KindUse;
use crate::ops::OpClass;
use crate::ops::Want;
use crate::ops::infix_class;
use crate::ops::int_expect;
use crate::ops::kw_label;
use crate::ops::verb_class;
use crate::ops::verb_label;
use crate::place::place_of;
use crate::scope::Access;
use crate::semcheck::Checker;

impl Checker
{
    /// Infer the kind of an expression and record it.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `expect` - The kind its place expects, when known; unprefixed literals
    ///   take it.
    ///
    /// # Returns
    ///
    /// The kind, or `None` after an error inside the expression.
    pub fn infer(&mut self, e: &Expr, expect: Option<DataKind>) -> Option<DataKind>
    {
        let kind = self.infer_node(e, expect);
        do_if!(let Some(k) = kind, let _ = self.kinds.insert(e.id(), k));
        kind
    }

    /// Infer the kind of one expression form.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `expect` - The kind its place expects, when known.
    ///
    /// # Returns
    ///
    /// The kind, or `None` after an error.
    fn infer_node(&mut self, e: &Expr, expect: Option<DataKind>) -> Option<DataKind>
    {
        match e
        {
            Expr::Int(n) => self.infer_int(n, expect, false),
            Expr::Float(n) => Some(float_kind(n, expect)),
            Expr::Char(_) => Some(DataKind::Char),
            Expr::Bool(_) => Some(DataKind::Bool),
            Expr::Name(n) => self.infer_name(n),
            Expr::Group(n) => self.infer(&n.inner, expect),
            Expr::Prefix(n) => self.infer_prefix(n, expect),
            Expr::Infix(n) => self.infer_infix(n, expect),
            Expr::Not(n) => self.infer_bool(&n.inner, &kw_label(Kw::Not)),
            Expr::Logic(n) => self.infer_logic(n),
            Expr::Verb(n) => self.infer_verb(n, expect),
            Expr::Call(n) => self.infer_call(n),
            Expr::Null(_) =>
            {
                Some(expect.filter(|k| k.is_ptr()).unwrap_or(DataKind::Null))
            }
            Expr::SizeOf(n) =>
            {
                self.written_kind(&n.kind, KindUse::Other);
                Some(DataKind::USIZE)
            }
            Expr::Index(n) => self.infer_index(n),
            Expr::List(n) => self.infer_list(n, expect),
            Expr::BlankList(n) => self.infer_blank(n, expect),
        }
    }

    /// Infer an integer literal.
    ///
    /// # Arguments
    ///
    /// * `n` - The literal.
    /// * `expect` - The kind its place expects, when known.
    /// * `negated` - Whether a negate operator stands before it.
    ///
    /// # Returns
    ///
    /// The prefix kind; else the expected number kind or `isize`, reporting
    /// a value outside its range.
    fn infer_int(
        &mut self, n: &IntLit, expect: Option<DataKind>, negated: bool,
    ) -> Option<DataKind>
    {
        ret_if!(let Some(kind) = &n.kind, Some(DataKind::from_ast(kind)));
        let want = expect.filter(|k| k.is_numeric()).unwrap_or(DataKind::ISIZE);
        let fits = want.is_float() || want.fits_int(n.value, negated, self.platform);
        ret_if!(fits, Some(want));
        let value = tern!(negated, format!("-{}", n.value), n.value.to_string());
        self.report(kd::literal_does_not_fit(n.span, &value, &want.name()));
        Some(want)
    }

    /// Infer a variable.
    ///
    /// # Arguments
    ///
    /// * `n` - The variable.
    ///
    /// # Returns
    ///
    /// Its declared kind, or `None` when it is not declared.
    fn infer_name(&mut self, n: &NameExpr) -> Option<DataKind>
    {
        ret_if!(let Some(binding) = self.scopes.lookup(&n.name), binding.kind);
        self.report(rd::undeclared_name(n.span, &n.name));
        None
    }

    /// Infer a prefix operator.
    ///
    /// # Arguments
    ///
    /// * `n` - The operation.
    /// * `expect` - The kind its place expects, when known.
    ///
    /// # Returns
    ///
    /// The operand kind, or `None` after an error.
    fn infer_prefix(
        &mut self, n: &PrefixExpr, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        match n.op
        {
            PrefixOp::Neg => self.infer_neg(n, expect),
            PrefixOp::BitNot =>
            {
                let kind = self.infer(&n.rhs, int_expect(expect))?;
                self.need(kind, Want::Int, n.op.as_ref(), &n.rhs)
            }
            PrefixOp::Not => self.infer_bool(&n.rhs, n.op.as_ref()),
            PrefixOp::AddrOf => self.infer_addr_of(&n.rhs),
            PrefixOp::Deref => self.infer_deref(&n.rhs),
        }
    }

    /// Infer the address of a place.
    ///
    /// # Arguments
    ///
    /// * `operand` - The place.
    ///
    /// # Returns
    ///
    /// A pointer to the kind of the place, or `None` after an error.
    fn infer_addr_of(&mut self, operand: &Expr) -> Option<DataKind>
    {
        let kind = self.infer(operand, None);
        let place = place_of(operand).filter(|p| !self.in_temp_list(p));
        let Some(place) = place
        else
        {
            self.report(kd::no_address(operand.span(), &what_of(operand)));
            return None;
        };
        let fixed = self.fixed_name(place);
        let diag = fixed.map(|name| kd::fixed_address(operand.span(), name));
        do_if!(let Some(d) = diag, self.report(d));
        kind.map(DataKind::pointer_to)
    }

    /// The name of a place that cannot be changed.
    ///
    /// # Arguments
    ///
    /// * `place` - The place.
    ///
    /// # Returns
    ///
    /// The name of a constant or a counted-loop variable, else `None`.
    fn fixed_name<'a>(&self, place: &'a Expr) -> Option<&'a str>
    {
        let n = self.owner_name(place)?;
        let access = self.scopes.lookup(&n.name)?.access;
        tern!(access == Access::Var, None, Some(&n.name))
    }

    /// Infer a dereference.
    ///
    /// # Arguments
    ///
    /// * `operand` - The pointer.
    ///
    /// # Returns
    ///
    /// The kind the pointer points at, or `None` after an error.
    fn infer_deref(&mut self, operand: &Expr) -> Option<DataKind>
    {
        let kind = self.infer(operand, None)?;
        let pointee = kind.pointee();
        ret_if!(pointee.is_some(), pointee);
        let span = operand.span();
        let opaque = kd::opaque_pointer(span);
        let plain = || kd::not_a_pointer(span, &kind.name());
        self.report(tern!(kind.is_address(), opaque, plain()));
        None
    }

    /// Infer a negation; a literal operand is range-checked as negative.
    ///
    /// # Arguments
    ///
    /// * `n` - The operation.
    /// * `expect` - The kind its place expects, when known.
    ///
    /// # Returns
    ///
    /// The operand kind, or `None` after an error.
    fn infer_neg(
        &mut self, n: &PrefixExpr, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let lit = match n.rhs.as_ref()
        {
            Expr::Int(lit) if lit.kind.is_none() => Some(lit),
            _ => None,
        };
        ret_if!(let Some(lit) = lit, self.infer_neg_lit(lit, expect));
        let kind = self.infer(&n.rhs, expect)?;
        self.need(kind, Want::Signed, "-", &n.rhs)
    }

    /// Infer an unprefixed integer literal under a negate operator.
    ///
    /// # Arguments
    ///
    /// * `lit` - The literal.
    /// * `expect` - The kind its place expects, when known.
    ///
    /// # Returns
    ///
    /// The literal kind.
    fn infer_neg_lit(
        &mut self, lit: &IntLit, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let kind = self.infer_int(lit, expect, true);
        do_if!(let Some(k) = kind, let _ = self.kinds.insert(lit.id, k));
        kind
    }

    /// Infer an operand that must be a boolean.
    ///
    /// # Arguments
    ///
    /// * `operand` - The operand.
    /// * `op` - The operator spelling.
    ///
    /// # Returns
    ///
    /// `bool`, or `None` after an error.
    fn infer_bool(&mut self, operand: &Expr, op: &str) -> Option<DataKind>
    {
        let kind = self.infer(operand, Some(DataKind::Bool))?;
        self.need(kind, Want::Bool, op, operand)
    }

    /// Infer a logical operation.
    ///
    /// # Arguments
    ///
    /// * `n` - The operation.
    ///
    /// # Returns
    ///
    /// `bool`, or `None` after an error.
    fn infer_logic(&mut self, n: &LogicExpr) -> Option<DataKind>
    {
        let kw = tern!(n.op == LogicOp::And, Kw::And, Kw::Or);
        let op = kw_label(kw);
        let lhs = self.infer_bool(&n.lhs, &op);
        let rhs = self.infer_bool(&n.rhs, &op);
        lhs.and(rhs)
    }

    /// Infer a notation operation.
    ///
    /// # Arguments
    ///
    /// * `n` - The operation.
    /// * `expect` - The kind its place expects, when known.
    ///
    /// # Returns
    ///
    /// The result kind, or `None` after an error.
    fn infer_infix(
        &mut self, n: &InfixExpr, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let op = n.op.as_ref();
        let class = infix_class(n.op);
        ret_if!(class == OpClass::Shift, self.infer_shift(op, &n.lhs, &n.rhs, expect));
        self.infer_pair(class, op, &n.lhs, &n.rhs, expect, n.span)
    }

    /// Infer a verb clause.
    ///
    /// # Arguments
    ///
    /// * `n` - The clause.
    /// * `expect` - The kind its place expects, when known.
    ///
    /// # Returns
    ///
    /// The result kind, or `None` after an error.
    fn infer_verb(&mut self, n: &VerbExpr, expect: Option<DataKind>)
    -> Option<DataKind>
    {
        let op = verb_label(n.op);
        let class = verb_class(n.op);
        match (class, n.args.as_slice())
        {
            (OpClass::Cast, [value]) => self.infer_cast(n, value),
            (OpClass::AddrOf, [value]) => self.infer_addr_of(value),
            (OpClass::Deref, [value]) => self.infer_deref(value),
            (OpClass::CellCount, [value]) => self.infer_cell_count(value),
            (OpClass::BitNot, [value]) =>
            {
                let kind = self.infer(value, int_expect(expect))?;
                self.need(kind, Want::Int, &op, value)
            }
            (OpClass::Shift, [value, distance]) =>
            {
                self.infer_shift(&op, value, distance, expect)
            }
            (
                OpClass::Arith | OpClass::Bit | OpClass::Equal | OpClass::Order,
                [l, r],
            ) => self.infer_pair(class, &op, l, r, expect, n.span),
            _ => self.infer_all(&n.args),
        }
    }

    /// Infer a conversion.
    ///
    /// # Arguments
    ///
    /// * `n` - The clause.
    /// * `value` - The converted value.
    ///
    /// # Returns
    ///
    /// The target kind.
    fn infer_cast(&mut self, n: &VerbExpr, value: &Expr) -> Option<DataKind>
    {
        let from = self.infer(value, None);
        let target = self.written_kind(n.kind_arg.as_ref()?, KindUse::Other);
        let bad = from.filter(|f| !f.cast_allowed(target));
        let report = |f: DataKind| kd::invalid_cast(n.span, &f.name(), &target.name());
        do_if!(let Some(f) = bad, self.report(report(f)));
        Some(target)
    }

    /// Infer a call expression.
    ///
    /// # Arguments
    ///
    /// * `n` - The call.
    ///
    /// # Returns
    ///
    /// The result kind of the function, or `None` after an error.
    fn infer_call(&mut self, n: &CallExpr) -> Option<DataKind>
    {
        let ret = self.check_call(&n.callee, &n.args, n.span)?;
        let void = rd::value_from_void_function(n.span, &n.callee.name);
        ret_if!(ret.is_none(), self.report(void), None);
        ret
    }
}

/// What an operand without an address is, for a message.
///
/// # Arguments
///
/// * `e` - The operand.
///
/// # Returns
///
/// The localized words for a literal, a call result or a computed value.
fn what_of(e: &Expr) -> String
{
    let words = match e
    {
        Expr::Group(n) => return what_of(&n.inner),
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Char(_)
        | Expr::Bool(_)
        | Expr::Null(_) =>
        {
            t!("semcheck.what.literal")
        }
        Expr::Call(_) => t!("semcheck.what.call"),
        _ => t!("semcheck.what.value"),
    };
    words.to_string()
}

/// The kind of a floating-point literal.
///
/// # Arguments
///
/// * `n` - The literal.
/// * `expect` - The kind its place expects, when known.
///
/// # Returns
///
/// The prefix kind, else the expected float kind, else `f64`.
fn float_kind(n: &FloatLit, expect: Option<DataKind>) -> DataKind
{
    let prefix = n.kind.as_ref().map(DataKind::from_ast);
    prefix.or(expect.filter(|k| k.is_float())).unwrap_or(DataKind::F64)
}
