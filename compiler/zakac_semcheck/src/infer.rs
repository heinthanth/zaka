use zakac_ast::expr::CallExpr;
use zakac_ast::expr::Expr;
use zakac_ast::expr::FloatLit;
use zakac_ast::expr::Ident;
use zakac_ast::expr::InfixExpr;
use zakac_ast::expr::IntLit;
use zakac_ast::expr::LogicExpr;
use zakac_ast::expr::LogicOp;
use zakac_ast::expr::MemberExpr;
use zakac_ast::expr::NameExpr;
use zakac_ast::expr::PrefixExpr;
use zakac_ast::expr::PrefixOp;
use zakac_ast::expr::StrExpr;
use zakac_ast::expr::VerbExpr;
use zakac_ast::name::Name;
use zakac_ast::verb::Dir;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::kinds as kd;
use zakac_diagnostics::kinds::NoInitial;
use zakac_diagnostics::naturalness as nd;
use zakac_diagnostics::resolve as rd;
use zakac_i18n::t;
use zakac_lexer::reserved::CHAR_NAME;
use zakac_lexer::reserved::PLURAL;
use zakac_lexer::token::Kw;
use zakac_naturalness::classifiers::GENERAL_CLASSIFIER;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::kind::DataKind;
use crate::kind::InitialGap;
use crate::list::KindUse;
use crate::list::data_of;
use crate::op_calls::BIT_NOT_FN;
use crate::op_calls::NEG_FN;
use crate::op_calls::SHL_FN;
use crate::op_calls::SHR_FN;
use crate::op_calls::infix_fn;
use crate::op_calls::verb_fn;
use crate::ops::OpClass;
use crate::ops::Want;
use crate::ops::infix_class;
use crate::ops::int_expect;
use crate::ops::kw_label;
use crate::ops::verb_class;
use crate::ops::verb_label;
use crate::place::place_of;
use crate::proof::result_var;
use crate::scope::Access;
use crate::scope::FnSig;
use crate::semcheck::Checker;
use crate::structs::StructTable;

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
            Expr::Call(n) =>
            {
                self.call_expect = expect;
                self.infer_call(n)
            }
            Expr::Null(n) if n.blank => self.infer_no_data(n.span, expect),
            Expr::Null(n) if n.initial => self.infer_initial(n.span, expect),
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
            Expr::Field(n) => self.infer_field(n),
            Expr::Range(n) => self.infer_range(n),
            Expr::List(n) => self.infer_list(n, expect),
            Expr::Struct(n) => self.infer_struct_lit(n, expect),
            Expr::BlankList(n) => self.infer_blank(n, expect),
            Expr::Table(n) => self.infer_table(n, expect),
            Expr::Str(n) => self.infer_str(n),
            Expr::Member(n) => self.infer_member(n),
            Expr::FnAddr(n) => self.infer_fn_addr(n, expect),
        }
    }

    /// Infer `<enum>များထဲမှ <member>`: the name before `ထဲမှ` is the stem of
    /// an enumeration name with `များ` after it.
    ///
    /// # Arguments
    ///
    /// * `n` - The member.
    ///
    /// # Returns
    ///
    /// The enumeration, or `None` after an error.
    fn infer_member(&mut self, n: &MemberExpr) -> Option<DataKind>
    {
        let written = n.qualifier.name.as_str();
        let found =
            |name: &str| self.structs.get(&Name::new(name)).and_then(|k| k.as_enum());
        let plain = written.strip_suffix(PLURAL).and_then(found);
        let exact = plain.filter(|e| format!("{}{PLURAL}", e.stem()) == written);
        let Some(owner) = exact
        else
        {
            let near = plain.or_else(|| found(written));
            let fix = near.map(|e| format!("{}{PLURAL}", e.stem()));
            self.report(rd::unknown_enum(n.qualifier.span, written, fix.as_deref()));
            return None;
        };
        let (q, m) = (&n.qualifier, &n.member);
        do_if!(let Some(d) = self.decls.name(&owner.name), self.refer(&q.name, q.span, d));
        let member = self.decls.member(&owner.name, &m.name);
        do_if!(let Some(d) = member, self.refer(&m.name, m.span, d));
        ret_if!(owner.member(&n.member.name).is_some(), Some(DataKind::Enum(owner)));
        self.report(rd::unknown_member(n.member.span, &owner.name, &n.member.name));
        None
    }

    /// Infer a string literal: owned text, whatever the kind of each template
    /// slot.
    ///
    /// # Arguments
    ///
    /// * `n` - The literal.
    ///
    /// # Returns
    ///
    /// `စာသား`, or `None` after an error in a template slot.
    fn infer_str(&mut self, n: &StrExpr) -> Option<DataKind>
    {
        let slots: Vec<_> = n
            .template_slots()
            .map(|h| {
                let kind = self.infer(h, None);
                self.struct_text(h, kind);
                kind
            })
            .collect();
        tern!(slots.iter().all(Option::is_some), Some(DataKind::TEXT), None)
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
        ret_if!(let Some(kind) = &n.kind, Some(DataKind::from_ast(kind, &self.structs)));
        let want = expect.map(data_of).filter(|k| k.is_numeric());
        let want = want.unwrap_or(DataKind::ISIZE);
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
        if let Some(binding) = self.scopes.lookup(&n.name)
        {
            self.refer(&n.name, n.span, binding.home());
            return binding.kind;
        }
        self.report(self.with_hidden(rd::undeclared_name(n.span, &n.name), &n.name));
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
                let called = self.struct_call(n.id, BIT_NOT_FN, &[kind]);
                ret_if!(called.is_some(), called);
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
        self.need_unchecked(operand.span(), "\"၏ လိပ်စာ\"");
        let kind = self.infer(operand, None);
        ret_if!(!self.addressable(operand), None);
        Some(kind?.pointer_to())
    }

    /// Whether an operand is a place whose address can be taken, reporting
    /// one that is no place, or a cell of a list that is no place, and one
    /// that cannot be changed.
    ///
    /// # Arguments
    ///
    /// * `operand` - The operand.
    ///
    /// # Returns
    ///
    /// `true` for a place; a constant counts, after its report.
    pub(crate) fn addressable(&mut self, operand: &Expr) -> bool
    {
        let place = place_of(operand).filter(|p| !self.in_temp_list(p));
        let Some(place) = place
        else
        {
            self.report(kd::no_address(operand.span(), &what_of(operand)));
            return false;
        };
        let fixed = self.fixed_name(place);
        let diag = fixed.map(|name| kd::fixed_address(operand.span(), name));
        do_if!(let Some(d) = diag, self.report(d));
        true
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
        self.need_unchecked(operand.span(), "\"၏ တန်ဖိုး\"");
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
        let called = self.struct_call(n.id, NEG_FN, &[kind]);
        ret_if!(called.is_some(), called);
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
        let (name, negate) = infix_fn(n.op);
        ret_if!(
            class == OpClass::Shift,
            self.infer_shift(op, (n.id, name), &n.lhs, &n.rhs, expect)
        );
        self.pending_op = Some((n.id, (name, negate)));
        self.infer_pair(class, op, &n.lhs, &n.rhs, expect, n.span)
    }

    /// Report the classifier of `တစ်<classifier>လုံး` before a deep duplicate
    /// when it is not the one of the kind duplicated: the classifier a struct,
    /// an object or an enumeration declares, `ခု` for each other kind (ZK0509).
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind duplicated.
    /// * `found` - The classifier written.
    fn check_whole(&mut self, kind: DataKind, found: &Ident)
    {
        let shape = kind.as_object().or_else(|| kind.as_struct());
        let name = shape.map(|s| &s.name).or_else(|| kind.as_enum().map(|e| &e.name));
        let own = name.and_then(|n| self.classifiers.get(n)).map(String::as_str);
        let want = own.unwrap_or(GENERAL_CLASSIFIER).to_string();
        ret_if!(found.name.as_str() == want);
        let shown = shape.map_or_else(|| kind.name(), |s| s.name.clone());
        self.report(nd::whole_classifier_mismatch(
            found.span,
            &shown,
            &want,
            &found.name,
        ));
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
        // A function of the file of a count's name, taking its operand, is
        // called in its place.
        ret_if!(let Some(kind) = self.own_count(n), kind);
        let op = verb_label(n.op);
        let class = verb_class(n.op);
        match (n.op, n.args.as_slice())
        {
            (VerbOp::CharCount, [text]) => return self.infer_chars(text, None),
            (VerbOp::Row | VerbOp::RowValue, [table, index]) =>
            {
                return self.infer_row(n, table, index);
            }
            (VerbOp::OrElse, [optional, fallback]) =>
            {
                return self.infer_or_else(optional, fallback);
            }
            (VerbOp::RowCount, [table]) => return self.infer_row_count(table),
            (VerbOp::Join, pieces) => return self.infer_join(pieces),
            (VerbOp::CharAt | VerbOp::CharNth, [text, index]) =>
            {
                return self.infer_chars(text, Some(index));
            }
            _ => (),
        }
        match (class, n.args.as_slice())
        {
            (OpClass::Cast, [value]) => self.infer_cast(n, value),
            (OpClass::Result, [value]) => self.infer_result_part(n.op, value),
            (OpClass::MakeFault, [value]) => self.infer_make_fault(value, expect),
            (OpClass::AddrOf, [value]) => self.infer_addr_of(value),
            (OpClass::Deref, [value]) => self.infer_deref(value),
            (OpClass::CellCount, [value]) => self.infer_cell_count(value),
            (OpClass::BitNot, [value]) =>
            {
                let kind = self.infer(value, int_expect(expect))?;
                let called = self.struct_call(n.id, BIT_NOT_FN, &[kind]);
                ret_if!(called.is_some(), called);
                self.need(kind, Want::Int, &op, value)
            }
            // A duplicate has the kind of what it duplicates.
            (OpClass::Dup, [value]) =>
            {
                let kind = self.infer(value, expect)?;
                let handle = kind.as_external();
                do_if!(let Some(s) = handle, self.report(kd::handle_in_zaka(value.span(), &s.name, kd::HandleUse::Duplicate)));
                do_if!(let Some(found) = &n.classifier, self.check_whole(kind, found));
                Some(kind)
            }
            (OpClass::Shift, [value, distance]) =>
            {
                let name = tern!(n.dir == Some(Dir::Right), SHR_FN, SHL_FN);
                self.infer_shift(&op, (n.id, name), value, distance, expect)
            }
            (
                OpClass::Arith
                | OpClass::Bit
                | OpClass::Equal
                | OpClass::Order
                | OpClass::Same,
                [l, r],
            ) =>
            {
                self.pending_op = verb_fn(n.op).map(|f| (n.id, f));
                self.infer_pair(class, &op, l, r, expect, n.span)
            }
            _ => self.infer_all(&n.args),
        }
    }

    /// Infer a test or a conversion of a result of a function that can
    /// fail: the test is a boolean, and the conversion the good result or
    /// the error value.
    ///
    /// # Arguments
    ///
    /// * `op` - The verb.
    /// * `value` - The result.
    ///
    /// # Returns
    ///
    /// The kind, or `None` after an error.
    pub(crate) fn infer_result_part(
        &mut self, op: VerbOp, value: &Expr,
    ) -> Option<DataKind>
    {
        let kind = self.infer(value, None)?;
        let Some((ok, err)) = kind.fallible_parts()
        else
        {
            self.report(kd::not_a_result(value.span(), &kind.name()));
            return None;
        };
        // An optional is tested for data, and a result for its good side.
        let tests_data = matches!(op, VerbOp::HasData | VerbOp::LacksData);
        let tests_result = matches!(op, VerbOp::IsGood | VerbOp::IsFault);
        let fault_read = op == VerbOp::UnwrapFault;
        let mismatch = (tests_data && !kind.is_optional())
            || ((tests_result || fault_read) && kind.is_optional());
        ret_if!(
            mismatch,
            self.report(kd::test_word_mismatch(
                value.span(),
                &kind.name(),
                kind.is_optional()
            )),
            None
        );
        ret_if!(tests_data || tests_result, Some(DataKind::Bool));
        let proven =
            result_var(value).and_then(|n| self.proven.get(&n).map(|g| (n, *g)));
        let good = op == VerbOp::Unwrap;
        if let Some((name, holds)) = proven.filter(|(_, holds)| *holds != good)
        {
            let optional = kind.is_optional();
            self.report(kd::wrong_part_proven(value.span(), &name, holds, optional));
        }
        let no_err = op == VerbOp::UnwrapFault && err.is_none();
        ret_if!(no_err, self.report(kd::no_fault_value(value.span())), None);
        ret_if!(op == VerbOp::UnwrapFault, err);
        ret_if!(ok.is_none(), self.report(kd::no_good_value(value.span())), None);
        ok
    }

    /// Infer an error value, `<value> ဟူသော အမှားအယွင်း`: the place it goes
    /// into names the result kind, and the value is its error side.
    ///
    /// # Arguments
    ///
    /// * `value` - The error value.
    /// * `expect` - The kind the place wants.
    ///
    /// # Returns
    ///
    /// The result kind, or `None` after an error.
    /// Infer `အလွတ်`: the optional the place expects.
    ///
    /// # Arguments
    ///
    /// * `span` - The value.
    /// * `expect` - The kind the place expects.
    ///
    /// # Returns
    ///
    /// The optional, or `None` after an error.
    fn infer_no_data(
        &mut self, span: Span, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let optional = expect.filter(|k| k.is_optional());
        do_if!(optional.is_none(), self.report(kd::no_data_needs_optional(span)));
        optional
    }

    /// Infer `ကနဦးတန်ဖိုး`, which takes the kind its place wants.
    ///
    /// # Arguments
    ///
    /// * `span` - The value.
    /// * `expect` - The kind the place wants.
    ///
    /// # Returns
    ///
    /// The kind, or `None` when no kind is known or the kind has no initial
    /// value.
    fn infer_initial(
        &mut self, span: Span, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let Some(kind) = expect
        else
        {
            self.report(kd::initial_needs_kind(span));
            return None;
        };
        let Some(gap) = kind.initial_gap()
        else
        {
            return Some(kind);
        };
        let why = match &gap
        {
            InitialGap::Function => NoInitial::Function,
            InitialGap::Handle => NoInitial::Handle,
            InitialGap::Member(e) => NoInitial::Member(&e.name),
            InitialGap::Result => NoInitial::Result,
            InitialGap::Part(p) => NoInitial::Part(p),
        };
        self.report(kd::no_initial_value(span, &kind.name(), why));
        None
    }

    fn infer_make_fault(
        &mut self, value: &Expr, expect: Option<DataKind>,
    ) -> Option<DataKind>
    {
        let result = expect.filter(|k| !k.is_optional());
        let err = result.and_then(DataKind::fallible_parts).and_then(|(_, err)| err);
        let Some(err) = err
        else
        {
            _ = self.infer(value, None);
            self.report(kd::fault_needs_result(value.span()));
            return None;
        };
        self.expect_kind(value, Some(err));
        result
    }

    /// Infer a verbal join: each piece must be text, owned or a view.
    ///
    /// # Arguments
    ///
    /// * `pieces` - The pieces.
    ///
    /// # Returns
    ///
    /// `စာသား`, or `None` after an error.
    fn infer_join(&mut self, pieces: &[Expr]) -> Option<DataKind>
    {
        let label = kw_label(Kw::Joined);
        let mut fine = true;
        for piece in pieces
        {
            let kind = self.infer(piece, None);
            fine &=
                kind.is_some_and(|k| self.need(k, Want::Text, &label, piece).is_some());
        }
        tern!(fine, Some(DataKind::TEXT), None)
    }

    /// Infer the character count of text, or the character at a number.
    ///
    /// # Arguments
    ///
    /// * `text` - The text, owned or a view.
    /// * `index` - The number, counted from 0; `None` for the count.
    ///
    /// # Returns
    ///
    /// `usize` for the count, `စာလုံး` for a character, or `None` after an
    /// error.
    fn infer_chars(&mut self, text: &Expr, index: Option<&Expr>) -> Option<DataKind>
    {
        let kind = self.infer(text, None);
        let at = index.map(|i| (i, self.infer(i, Some(DataKind::USIZE))));
        let bad = at.and_then(|(i, k)| Some((i, k?))).filter(|(_, k)| !k.is_int());
        do_if!(let Some((i, k)) = bad, self.report(kd::cell_number_not_int(i.span(), &k.name())));
        let label = tern!(
            index.is_some(),
            CHAR_NAME.to_string(),
            verb_label(VerbOp::CharCount)
        );
        self.need(kind?, Want::Text, &label, text)?;
        ret_if!(bad.is_some() || at.is_some_and(|(_, k)| k.is_none()), None);
        Some(tern!(index.is_some(), DataKind::Char, DataKind::USIZE))
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
        let kind_arg = n.kind_arg.as_ref()?;
        let target = self.written_kind(kind_arg, KindUse::Other);
        let built_in = from.is_none_or(|f| f.cast_allowed(target));
        let via =
            from.filter(|_| !built_in).and_then(|f| self.conversion_fn(f, target));
        ret_if!(let Some(func) = via, self.infer_conversion(n, value, func));
        let word =
            Ident::builder().name(kw_label(Kw::ConvertGood)).span(n.span).build();
        self.check_pass(&word, Some(target), n.pass, false, n.span);
        let bad = from.filter(|f| !f.cast_allowed(target));
        do_if!(let Some(f) = bad, self.report(cast_report(n.span, f, target)));
        let pointer = target.is_address() || from.is_some_and(DataKind::is_address);
        do_if!(pointer, self.need_unchecked(n.span, "a pointer conversion"));
        Some(target)
    }

    /// The function a conversion goes through when the kinds have no built-in
    /// conversion: a function the file sees whose name ends in `အဖြစ်ပြောင်း`,
    /// that takes one value of the source kind, and gives the target kind or a
    /// result whose good side is the target kind. The stdlib names such a
    /// function `<K>အဖြစ်ပြောင်း`, `K` the word of the target kind.
    ///
    /// # Arguments
    ///
    /// * `from` - The source kind.
    /// * `to` - The target kind.
    ///
    /// # Returns
    ///
    /// The function name, or `None` when no function fits.
    fn conversion_fn(&self, from: DataKind, to: DataKind) -> Option<Name>
    {
        let gives = |ret: Option<DataKind>| {
            let ok = ret.and_then(DataKind::fallible_parts).and_then(|(ok, _)| ok);
            ret == Some(to) || ok == Some(to)
        };
        let fits = |sig: &FnSig| sig.params == [from] && gives(sig.ret);
        let named = self.fns.iter().filter(|(name, _)| name.ends_with(CONVERT_SUFFIX));
        let found = named.filter(|(_, sigs)| sigs.iter().any(fits));
        let mut names: Vec<Name> = found.map(|(name, _)| *name).collect();
        names.sort();
        names.into_iter().next()
    }

    /// Check a conversion that goes through a function, `<K>အဖြစ်ပြောင်း`, as
    /// the call of it, which takes the node of the conversion.
    ///
    /// # Arguments
    ///
    /// * `n` - The conversion.
    /// * `value` - The value it converts.
    /// * `func` - The function.
    ///
    /// # Returns
    ///
    /// The result kind of the call, or `None` after an error.
    fn infer_conversion(
        &mut self, n: &VerbExpr, value: &Expr, func: Name,
    ) -> Option<DataKind>
    {
        let callee = Ident::builder().name(func).span(n.span).build();
        #[rustfmt::skip]
        let call = CallExpr::builder().id(n.id).callee(callee).args(vec![value.clone()])
            .pass(n.pass).span(n.span).build();
        let kind = self.infer_call(&call);
        self.op_calls.methods.insert(n.id, call);
        kind
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
    pub(crate) fn infer_call(&mut self, n: &CallExpr) -> Option<DataKind>
    {
        let place = n.place.as_deref();
        let ret = self.check_call(
            &n.callee,
            &n.kind_args,
            place,
            &n.args,
            &n.prepositions,
            n.span,
        )?;
        self.check_pass(&n.callee, ret, n.pass, false, n.span);
        // Without a pass-on the value is the whole result; with one, the good
        // result.
        let parts = ret.and_then(DataKind::fallible_parts);
        let ret = tern!(let Some((ok, _)) = parts.filter(|_| n.pass), ok, ret);
        let void = rd::value_from_void_function(n.span, &n.callee.name);
        ret_if!(ret.is_none(), self.report(void), None);
        ret
    }
}

/// The end of the name of each function a conversion can go through.
const CONVERT_SUFFIX: &str = "အဖြစ်ပြောင်း";

/// The report of a conversion the verb cannot make. An integer headed for
/// an owner or a borrow, and a cast that reads stored addresses as another
/// kind, get help naming the raw address to go through.
///
/// # Arguments
///
/// * `span` - The conversion.
/// * `from` - The source kind.
/// * `to` - The target kind.
///
/// # Returns
///
/// The `ZK0408` diagnostic.
fn cast_report(span: Span, from: DataKind, to: DataKind) -> Diagnostic
{
    kd::invalid_cast(span, &from.name(), &to.name())
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
pub(crate) fn what_of(e: &Expr) -> String
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
/// The prefix kind, else the expected float kind, else `f64`. A prefix is
/// a number kind, so it names no struct.
fn float_kind(n: &FloatLit, expect: Option<DataKind>) -> DataKind
{
    let prefix = n.kind.as_ref().map(|k| DataKind::from_ast(k, &StructTable::new()));
    prefix.or(expect.map(data_of).filter(|k| k.is_float())).unwrap_or(DataKind::F64)
}
