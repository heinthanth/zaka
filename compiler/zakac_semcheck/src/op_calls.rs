//! Operators on structs. A struct takes `+ - * / %`, the bit operators, the
//! shifts, the `-` of one operand, the comparisons, `==` and `!=` through
//! functions of its own, the ones the traits of the core name: `+` on two
//! values of a struct calls its `ပေါင်း`, `<` its `ငယ်မငယ်စစ်`, `>=` the
//! negation of its `ငယ်မငယ်စစ်`, `<<` its `ဘစ်ဘယ်ရွေ့` with the distance, and so
//! on. Printing
//! and a template slot call its `စာသားအဖြစ်ပြောင်း`, and a conversion with no
//! built-in rule calls the `ပြောင်းလဲ` or the `စမ်းသပ်ပြောင်းလဲ` of its two kinds,
//! the functions of the conversion traits. The check records each
//! such operator, and once it settles writes the call into the tree, so the
//! later stages see a plain call.

use std::collections::HashMap;
use std::collections::HashSet;

use zakac_ast::expr::CallExpr;
use zakac_ast::expr::Expr;
use zakac_ast::expr::FnAddrExpr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::InfixOp;
use zakac_ast::expr::NodeId;
use zakac_ast::expr::NullLit;
use zakac_ast::expr::PrefixExpr;
use zakac_ast::expr::PrefixOp;
use zakac_ast::expr::StrExpr;
use zakac_ast::expr::StrPiece;
use zakac_ast::expr::VerbExpr;
use zakac_ast::kind::Kind;
use zakac_ast::name::Name;
use zakac_ast::stmt::CallStmt;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::KindParam;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::VisitorMut;
use zakac_ast::visit::walk_expr_mut;
use zakac_ast::visit::walk_stmt_mut;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::generic::instance_name;
use crate::generic::kind_ast;
use crate::kind::DataKind;
use crate::scope::FnSig;
use crate::scope::Written;
use crate::semcheck::Checker;
use crate::semcheck::KindTable;
use crate::structs::StructTable;
use crate::trait_value::TO_TRAIT_FN;

/// The function `+` calls.
pub const ADD_FN: &str = "ပေါင်း";
/// The function `-` calls.
pub const SUB_FN: &str = "နုတ်";
/// The function `*` calls.
pub const MUL_FN: &str = "မြှောက်";
/// The function `/` calls.
pub const DIV_FN: &str = "စား";
/// The function `%` calls.
pub const REM_FN: &str = "အကြွင်းရှာ";
/// The function `&` calls.
pub const BIT_AND_FN: &str = "ဘစ်ဆုံ";
/// The function `|` calls.
pub const BIT_OR_FN: &str = "ဘစ်ပေါင်းစု";
/// The function `^` calls.
pub const BIT_XOR_FN: &str = "ဘစ်ခြားနား";
/// The function `~` calls.
pub const BIT_NOT_FN: &str = "ဘစ်ပြောင်းပြန်";
/// The function `<<` calls, with the distance.
pub const SHL_FN: &str = "ဘစ်ဘယ်ရွေ့";
/// The function `>>` calls, with the distance.
pub const SHR_FN: &str = "ဘစ်ညာရွေ့";
/// The function the `-` of one operand calls.
pub const NEG_FN: &str = "အနုတ်ပြောင်း";
/// The function `<` calls, and `>=` negates.
pub const LESS_FN: &str = "ငယ်မငယ်စစ်";
/// The function `>` calls, and `<=` negates.
pub const GREATER_FN: &str = "ကြီးမကြီးစစ်";
/// The function `==` calls, and `!=` negates.
pub const EQUAL_FN: &str = "ညီမညီစစ်";
/// The function printing and a template slot call.
pub const TEXT_FN: &str = "စာသားအဖြစ်ပြောင်း";
/// The function of `ပြောင်းလဲနိုင်သော`, which a conversion calls.
pub const CONVERT_FN: &str = "ပြောင်းလဲ";
/// The function of `စမ်းသပ်ပြောင်းလဲနိုင်သော`, which a conversion that can
/// fail calls.
pub const TRY_CONVERT_FN: &str = "စမ်းသပ်ပြောင်းလဲ";

/// The function a conversion goes through.
#[derive(Debug, Clone)]
pub(crate) enum Via
{
    /// A plain function.
    Plain(FnSig),
    /// A generic function, under its name, with the kinds of its kind
    /// parameters.
    Generic(&'static str, Vec<Kind>),
}

/// A call that stands in place of an operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct OpCall
{
    /// The function, as the name the later stages know it.
    callee: Name,
    /// Whether the call stands under `!`, as for `!=`, `<=` and `>=`.
    negate: bool,
}

/// The calls that stand in place of operators, prints and template slots on
/// structs, under the node of the expression.
#[derive(Debug, Clone, Default)]
pub(crate) struct OpCalls
{
    /// The call each operator takes, on its operands.
    pairs: HashMap<NodeId, OpCall>,
    /// The function each printed value, template slot or error value goes
    /// through, with the kind it gives.
    pub(crate) wraps: HashMap<NodeId, (Name, DataKind)>,
    /// The call each field read stands for when no field fits, `x ၏ f` for
    /// `f(x)`, and each conversion that goes through a function, under the node
    /// of the read or the conversion.
    pub(crate) methods: HashMap<NodeId, CallExpr>,
    /// Each conversion to text that takes the text its value prints as, which
    /// becomes the template text `"{x}"`, under the node of the conversion.
    pub(crate) texts: HashSet<NodeId>,
    /// The function each call that passes a failure on turns its error value
    /// through, into the error kind of the calling function, under the span
    /// of the call.
    pub(crate) fault_vias: HashMap<Span, Name>,
    /// The call each statement of a verb the compiler gives stands for when
    /// a function of the file of that name takes its operands, `l ကို ပြသမည်။`
    /// for `ပြသ(l)`, and each call statement that goes to a part, under the
    /// span of the statement.
    pub(crate) verbs: HashMap<Span, CallStmt>,
    /// Each value that goes into a trait value, under its node: the trait
    /// value kind, and for each function of the trait, in its order, the name
    /// the later stages give the function that serves the value's kind.
    pub(crate) to_traits: HashMap<NodeId, (DataKind, Vec<Name>)>,
    /// The read of a part that each read of a field or of a part stands for,
    /// `x ၏ %%B ၏ f` for `x ၏ f`, under the node of the read.
    pub(crate) fields: HashMap<NodeId, Expr>,
}

/// The name of the function of the core a count stands for.
///
/// # Arguments
///
/// * `op` - The verb of the count.
///
/// # Returns
///
/// The name, or `None` for a verb that is no count.
pub(crate) fn count_fn(op: VerbOp) -> Option<&'static str>
{
    match op
    {
        VerbOp::CellCount => Some("အကွက်အရေအတွက်"),
        VerbOp::CharCount => Some("စာလုံးအရေအတွက်"),
        VerbOp::RowCount => Some("အညွှန်းအရေအတွက်"),
        _ => None,
    }
}

impl Checker
{
    /// Check a statement of a verb the compiler gives as the call of a
    /// function of the file of that name, when one takes the operands: a
    /// function of the file joins the overloads of the core's.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the verb's function, `ပြသ`.
    /// * `args` - The operands, in the order of the function's parameters.
    /// * `span` - The statement.
    ///
    /// # Returns
    ///
    /// `true` when a function of the file took the statement.
    pub(crate) fn own_verb(&mut self, name: &str, args: &[Expr], span: Span) -> bool
    {
        ret_if!(!self.own_fits(name, args), false);
        let callee = Ident::builder().name(name).span(span).build();
        let call =
            CallStmt::builder().callee(callee).args(args.to_vec()).span(span).build();
        self.visit_call_stmt(&call);
        self.op_calls.verbs.insert(span, call);
        true
    }

    /// The kind of a count read as the call of a function of the file of its
    /// name, `x ၏ အကွက်အရေအတွက်` for `အကွက်အရေအတွက်(x)`, when one takes `x`.
    ///
    /// # Arguments
    ///
    /// * `n` - The count.
    ///
    /// # Returns
    ///
    /// The kind of the call, or `None` when no function of the file takes it:
    /// the count keeps its built-in rule.
    pub(crate) fn own_count(&mut self, n: &VerbExpr) -> Option<Option<DataKind>>
    {
        let name = count_fn(n.op)?;
        ret_if!(!self.own_fits(name, &n.args), None);
        let callee = Ident::builder().name(name).span(n.span).build();
        #[rustfmt::skip]
        let call = CallExpr::builder().id(n.id).callee(callee).args(n.args.clone())
            .span(n.span).build();
        let kind = self.infer_call(&call);
        self.op_calls.methods.insert(n.id, call);
        Some(kind)
    }

    /// Whether a function the file sees of a name takes these arguments; the
    /// functions the compiler gives are not among them.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `args` - The arguments.
    ///
    /// # Returns
    ///
    /// `true` when one takes them.
    fn own_fits(&mut self, name: &str, args: &[Expr]) -> bool
    {
        let sigs = self.fns.get(&Name::from(name)).cloned().unwrap_or_default();
        sigs.iter().any(|s| self.fits_call(s, args, Written::default()))
    }
}

/// The function a two-operand operator calls, and whether the call is
/// negated.
///
/// # Arguments
///
/// * `op` - The operator.
///
/// # Returns
///
/// The function name and the negation.
pub(crate) fn infix_fn(op: InfixOp) -> (&'static str, bool)
{
    match op
    {
        InfixOp::Add => (ADD_FN, false),
        InfixOp::Sub => (SUB_FN, false),
        InfixOp::Mul => (MUL_FN, false),
        InfixOp::Div => (DIV_FN, false),
        InfixOp::Rem => (REM_FN, false),
        InfixOp::BitAnd => (BIT_AND_FN, false),
        InfixOp::BitOr => (BIT_OR_FN, false),
        InfixOp::BitXor => (BIT_XOR_FN, false),
        InfixOp::Shl => (SHL_FN, false),
        InfixOp::Shr => (SHR_FN, false),
        InfixOp::Lt => (LESS_FN, false),
        InfixOp::Ge => (LESS_FN, true),
        InfixOp::Gt => (GREATER_FN, false),
        InfixOp::Le => (GREATER_FN, true),
        InfixOp::Eq => (EQUAL_FN, false),
        InfixOp::Ne => (EQUAL_FN, true),
    }
}

/// [`infix_fn`] for the verb of a clause.
///
/// # Arguments
///
/// * `op` - The verb.
///
/// # Returns
///
/// The function name and the negation, or `None`.
pub(crate) fn verb_fn(op: VerbOp) -> Option<(&'static str, bool)>
{
    match op
    {
        VerbOp::Add => Some((ADD_FN, false)),
        VerbOp::Sub => Some((SUB_FN, false)),
        VerbOp::Mul => Some((MUL_FN, false)),
        VerbOp::Div => Some((DIV_FN, false)),
        VerbOp::Rem => Some((REM_FN, false)),
        VerbOp::BitAnd => Some((BIT_AND_FN, false)),
        VerbOp::BitOr => Some((BIT_OR_FN, false)),
        VerbOp::BitXor => Some((BIT_XOR_FN, false)),
        VerbOp::Lt => Some((LESS_FN, false)),
        VerbOp::Ge => Some((LESS_FN, true)),
        VerbOp::Gt => Some((GREATER_FN, false)),
        VerbOp::Le => Some((GREATER_FN, true)),
        VerbOp::Eq => Some((EQUAL_FN, false)),
        VerbOp::Ne => Some((EQUAL_FN, true)),
        _ => None,
    }
}

impl Checker
{
    /// The kind of an operator on two values of one struct that has the
    /// function the operator stands for, which the operator then calls.
    ///
    /// # Arguments
    ///
    /// * `op` - The node of the operation, the function and the negation.
    /// * `l` - The kind of the left operand.
    /// * `r` - The kind of the right operand.
    ///
    /// # Returns
    ///
    /// The kind, or `None` when the operator keeps its built-in rule.
    pub(crate) fn struct_operator(
        &mut self, op: (NodeId, (&'static str, bool)), l: DataKind, r: DataKind,
    ) -> Option<DataKind>
    {
        let (id, (name, negate)) = op;
        ret_if!(l != r || l.as_struct().is_none() || self.kind_home(l).is_none(), None);
        let sig = self.struct_fn(name, &[l, l])?;
        let test = matches!(name, LESS_FN | GREATER_FN | EQUAL_FN);
        ret_if!(test && sig.ret != Some(DataKind::Bool), None);
        let callee = self.known_name(&sig);
        self.op_calls.pairs.insert(id, OpCall { callee, negate });
        sig.ret
    }

    /// The kind of an operator whose first operand is a struct with the
    /// function the operator stands for, called with these operand kinds: the
    /// `-` of one operand, `~`, or a shift with its distance.
    ///
    /// # Arguments
    ///
    /// * `id` - The node of the operation.
    /// * `name` - The function.
    /// * `operands` - The operand kinds; the first is the struct.
    ///
    /// # Returns
    ///
    /// The kind, or `None` when the operator keeps its built-in rule.
    pub(crate) fn struct_call(
        &mut self, id: NodeId, name: &'static str, operands: &[DataKind],
    ) -> Option<DataKind>
    {
        let first = operands[0];
        ret_if!(first.as_struct().is_none() || self.kind_home(first).is_none(), None);
        let sig = self.struct_fn(name, operands)?;
        let callee = self.known_name(&sig);
        self.op_calls.pairs.insert(id, OpCall { callee, negate: false });
        sig.ret
    }

    /// Make a print or a template slot of a struct call its
    /// `စာသားအဖြစ်ပြောင်း`, when it has one.
    ///
    /// # Arguments
    ///
    /// * `value` - The value printed.
    /// * `kind` - Its kind.
    pub(crate) fn struct_text(&mut self, value: &Expr, kind: Option<DataKind>)
    {
        let Some(callee) = kind.and_then(|k| self.text_fn(k))
        else
        {
            return;
        };
        self.op_calls.wraps.insert(value.id(), (callee, DataKind::TEXT));
    }

    /// The `စာသားအဖြစ်ပြောင်း` of a struct, which a print of it calls.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The name the later stages give the function, or `None` for a kind
    /// with none.
    pub(crate) fn text_fn(&mut self, kind: DataKind) -> Option<Name>
    {
        ret_if!(kind.as_struct().is_none() || self.kind_home(kind).is_none(), None);
        let sig = self.struct_fn(TEXT_FN, &[kind])?;
        sig.ret.is_some_and(DataKind::is_text).then(|| self.known_name(&sig))
    }

    /// The function a conversion goes through: a `ပြောင်းလဲ` that takes the
    /// source kind and gives the target kind, or a `စမ်းသပ်ပြောင်းလဲ` that gives a
    /// result whose good side is the target kind, from the file of the use or
    /// the file of either kind; else a generic function of one of these names
    /// ([`Checker::generic_conversion`]). A conversion that passes a failure on
    /// takes `စမ်းသပ်ပြောင်းလဲ` first, a plain one `ပြောင်းလဲ`.
    ///
    /// # Arguments
    ///
    /// * `from` - The source kind.
    /// * `to` - The target kind.
    /// * `pass` - Whether the conversion passes a failure on.
    /// * `span` - The conversion, for the kinds a generic function takes.
    ///
    /// # Returns
    ///
    /// The function, or `None` when none fits.
    pub(crate) fn conversion_fn(
        &mut self, from: DataKind, to: DataKind, pass: bool, span: Span,
    ) -> Option<Via>
    {
        let sure =
            self.taking(CONVERT_FN, from, to).into_iter().find(|s| s.ret == Some(to));
        let good = |s: &FnSig| {
            let fails = s.ret.filter(|k| !k.is_optional());
            fails.and_then(DataKind::fallible_parts).and_then(|(ok, _)| ok) == Some(to)
        };
        let tried = self.taking(TRY_CONVERT_FN, from, to).into_iter().find(good);
        let order =
            tern!(pass, [TRY_CONVERT_FN, CONVERT_FN], [CONVERT_FN, TRY_CONVERT_FN]);
        let plain = tern!(pass, tried.clone().or(sure.clone()), sure.or(tried));
        ret_if!(let Some(sig) = plain, Some(Via::Plain(sig)));
        order.into_iter().find_map(|name| self.generic_conversion(name, from, to, span))
    }

    /// A generic function of a conversion trait that converts one kind to
    /// another: it takes one value whose kind is a kind parameter or the source
    /// kind, the kind parameter of the value stands for the source kind and
    /// each other one for the target kind, and the kinds meet the constraints.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function.
    /// * `from` - The source kind.
    /// * `to` - The target kind.
    /// * `span` - The conversion.
    ///
    /// # Returns
    ///
    /// The call through the generic function, or `None` when none fits.
    fn generic_conversion(
        &mut self, name: &'static str, from: DataKind, to: DataKind, span: Span,
    ) -> Option<Via>
    {
        let templates = self.templates.fns_of(&Name::from(name));
        templates.iter().find_map(|t| {
            let kinds = conversion_kinds(t, from, to, &self.structs)?;
            self.generic_result(t, &kinds, span)?;
            let written = kinds.iter().map(|k| kind_ast(*k, span));
            Some(Via::Generic(name, written.collect::<Option<Vec<Kind>>>()?))
        })
    }

    /// The functions of a name that take one value of a kind, in the file of
    /// the use and the files of the two kinds of a conversion.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `from` - The kind of the parameter.
    /// * `to` - The other kind, whose file is looked at too.
    ///
    /// # Returns
    ///
    /// The functions.
    fn taking(&self, name: &str, from: DataKind, to: DataKind) -> Vec<FnSig>
    {
        let here = self.fns.get(name).into_iter().flatten();
        let homes = [self.kind_home(from), self.kind_home(to)].into_iter().flatten();
        let fns = homes.filter_map(|u| self.unit_fns.get(u as usize));
        let there = fns.filter_map(|f| f.get(name)).flatten();
        here.chain(there).filter(|s| s.params == [from]).cloned().collect()
    }

    /// Check a conversion that goes through a function as the call of it,
    /// which takes the node of the conversion.
    ///
    /// # Arguments
    ///
    /// * `n` - The conversion.
    /// * `value` - The value it converts, checked before.
    /// * `via` - The function.
    ///
    /// # Returns
    ///
    /// The result kind of the call: the good result when it passes a failure
    /// on.
    pub(crate) fn infer_conversion(
        &mut self, n: &VerbExpr, value: &Expr, via: Via,
    ) -> Option<DataKind>
    {
        let Via::Plain(sig) = via
        else
        {
            return self.generic_conversion_call(n, value, via);
        };
        let fails = sig.ret.and_then(DataKind::fallible_parts);
        let word = tern!(fails.is_some(), TRY_CONVERT_FN, CONVERT_FN);
        let written = Ident::builder().name(word).span(n.span).build();
        self.check_pass(&written, sig.ret, n.pass, false, n.span);
        let unsafe_call = format!("a call of \"{word}\", an unsafe function,");
        do_if!(sig.unchecked, self.need_unchecked(n.span, &unsafe_call));
        let callee = Ident::builder().name(self.known_name(&sig)).span(n.span).build();
        #[rustfmt::skip]
        let call = CallExpr::builder().id(n.id).callee(callee).args(vec![value.clone()])
            .pass(n.pass).span(n.span).build();
        self.op_calls.methods.insert(n.id, call);
        tern!(let Some((ok, _)) = fails.filter(|_| n.pass), ok, sig.ret)
    }

    /// Check a conversion through a generic function as the call of it with
    /// its kinds written, which takes the node of the conversion.
    ///
    /// # Arguments
    ///
    /// * `n` - The conversion.
    /// * `value` - The value it converts.
    /// * `via` - The generic function and its kinds.
    ///
    /// # Returns
    ///
    /// The result kind of the call.
    fn generic_conversion_call(
        &mut self, n: &VerbExpr, value: &Expr, via: Via,
    ) -> Option<DataKind>
    {
        let Via::Generic(name, kind_args) = via
        else
        {
            return None;
        };
        let callee = Ident::builder().name(name).span(n.span).build();
        #[rustfmt::skip]
        let call = CallExpr::builder().id(n.id).callee(callee).kind_args(kind_args)
            .args(vec![value.clone()]).pass(n.pass).span(n.span).build();
        let kind = self.infer_call(&call);
        self.op_calls.methods.insert(n.id, call);
        kind
    }

    /// The `ပြောင်းလဲ` that turns one error kind into another: a plain function
    /// of `ပြောင်းလဲနိုင်သော` for the pair, else the instance of a generic one.
    ///
    /// # Arguments
    ///
    /// * `from` - The error kind of the value.
    /// * `to` - The error kind the place wants.
    /// * `span` - The value, for the lookup.
    ///
    /// # Returns
    ///
    /// The function, under the name the later stages know it, or `None`.
    pub(crate) fn error_conversion(
        &mut self, from: DataKind, to: DataKind, span: Span,
    ) -> Option<Name>
    {
        let sure =
            self.taking(CONVERT_FN, from, to).into_iter().find(|s| s.ret == Some(to));
        ret_if!(let Some(sig) = sure, Some(self.known_name(&sig)));
        let templates = self.templates.fns_of(&Name::from(CONVERT_FN));
        templates.iter().find_map(|t| self.error_instance(t, from, to, span))
    }

    /// The instance of a generic `ပြောင်းလဲ` that turns one error kind into
    /// another, asked for when it fits.
    ///
    /// # Arguments
    ///
    /// * `t` - The generic function.
    /// * `from` - The error kind of the value.
    /// * `to` - The error kind the place wants.
    /// * `span` - The value.
    ///
    /// # Returns
    ///
    /// The name of the instance, or `None` when the function does not fit or
    /// its instance is not made yet.
    fn error_instance(
        &mut self, t: &FnDecl, from: DataKind, to: DataKind, span: Span,
    ) -> Option<Name>
    {
        let kinds = conversion_kinds(t, from, to, &self.structs)?;
        let parts = self.generic_result(t, &kinds, span)?;
        ret_if!(parts != (Some(to), None), None);
        // An empty span of its own keeps the instance apart from a call that
        // ends at the same place.
        let at = Span::new(span.hi, span.hi);
        let ident = Ident::builder().name(CONVERT_FN).span(at).build();
        let sig = self.instance_sig(t, &ident, Some(kinds.clone()))?;
        (sig.ret == Some(to)).then(|| instance_name(t.name.name, &kinds))
    }

    /// What a generic conversion function gives for some kinds when they meet
    /// its constraints: its good side and its error side, or its result as the
    /// good side for a function that cannot fail.
    ///
    /// # Arguments
    ///
    /// * `t` - The generic function.
    /// * `kinds` - The kind of each kind parameter.
    /// * `span` - The use, for the kind phrases.
    ///
    /// # Returns
    ///
    /// The two sides, or `None` when the kinds do not meet the constraints.
    pub(crate) fn generic_result(
        &mut self, t: &FnDecl, kinds: &[DataKind], span: Span,
    ) -> Option<(Option<DataKind>, Option<DataKind>)>
    {
        let before = self.diags.len();
        let met = self.meets_bounds(&t.kind_params, kinds, span);
        self.diags.truncate(before);
        ret_if!(!met, None);
        let phrases = kinds.iter().map(|k| kind_ast(*k, span));
        let phrases = phrases.collect::<Option<Vec<Kind>>>()?;
        let names = t.kind_params.iter().map(|p| p.name.name);
        let bound: HashMap<Name, Kind> = names.zip(phrases).collect();
        let mut ret = t.ret.clone()?;
        ret.substitute(&bound);
        let ret = DataKind::from_ast(&ret, &self.structs);
        Some(ret.fallible_parts().unwrap_or((Some(ret), None)))
    }

    /// Whether a built-in kind has a conversion trait through a generic
    /// function of it: the function converts the kind to `to` and gives
    /// `want`, the two sides of its result.
    ///
    /// # Arguments
    ///
    /// * `name` - The function of the trait.
    /// * `kind` - The kind that converts.
    /// * `to` - The target kind.
    /// * `want` - The good side and the error side the trait names.
    /// * `span` - The use.
    ///
    /// # Returns
    ///
    /// `true` when a generic function of the name does.
    pub(crate) fn generic_converts(
        &mut self, name: &str, kind: DataKind, to: DataKind,
        want: (Option<DataKind>, Option<DataKind>), span: Span,
    ) -> bool
    {
        let templates = self.templates.fns_of(&Name::from(name));
        templates.iter().any(|t| {
            let kinds = conversion_kinds(t, kind, to, &self.structs);
            kinds.and_then(|k| self.generic_result(t, &k, span)) == Some(want)
        })
    }

    /// Check an error value against the error kind of its place: a value that
    /// does not widen to it goes through the `ပြောင်းလဲ` of the two kinds when
    /// there is one.
    ///
    /// # Arguments
    ///
    /// * `value` - The error value.
    /// * `err` - The error kind of the place.
    pub(crate) fn expect_error(&mut self, value: &Expr, err: DataKind)
    {
        let Some(found) = self.infer(value, Some(err))
        else
        {
            return;
        };
        let fits = found.widens_to(err, self.platform);
        let via = tern!(fits, None, self.error_conversion(found, err, value.span()));
        ret_if!(let Some(callee) = via, _ = self.op_calls.wraps.insert(value.id(), (callee, err)));
        self.expect_fit(value, found, err);
    }

    /// Check a conversion to text with no function of its own as the text the
    /// value prints as, through the `စာသားအဖြစ်ပြောင်း` of a struct when it has
    /// one. Such a conversion cannot fail.
    ///
    /// # Arguments
    ///
    /// * `n` - The conversion.
    /// * `value` - The value it converts, checked before.
    /// * `from` - The kind of the value.
    ///
    /// # Returns
    ///
    /// `စာသား`.
    pub(crate) fn infer_to_text(
        &mut self, n: &VerbExpr, value: &Expr, from: DataKind,
    ) -> Option<DataKind>
    {
        let word = Ident::builder().name(CONVERT_FN).span(n.span).build();
        self.check_pass(&word, Some(DataKind::TEXT), n.pass, false, n.span);
        self.struct_text(value, Some(from));
        self.op_calls.texts.insert(n.id);
        Some(DataKind::TEXT)
    }

    /// The function of a name with these parameter kinds and no other, in the
    /// file of the use or the file of the kind.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `params` - The parameter kinds.
    ///
    /// # Returns
    ///
    /// The function, or `None`.
    fn struct_fn(&self, name: &str, params: &[DataKind]) -> Option<FnSig>
    {
        let home = self.kind_home(params[0]);
        let here = self.fns.get(name).into_iter().flatten();
        let there =
            home.and_then(|u| self.unit_fns.get(u as usize)).and_then(|f| f.get(name));
        let mut all = here.chain(there.into_iter().flatten());
        all.find(|s| s.params == params).cloned()
    }

    /// The name the later stages know a function as, from the file being
    /// checked: its inner name, with the suffix of its file when another
    /// file declares it.
    ///
    /// # Arguments
    ///
    /// * `sig` - The function.
    ///
    /// # Returns
    ///
    /// The name.
    pub(crate) fn known_name(&self, sig: &FnSig) -> Name
    {
        let Some(unit) = sig.unit.filter(|u| *u != self.unit)
        else
        {
            return sig.inner;
        };
        let file = self.unit_names.get(unit as usize).cloned().unwrap_or_default();
        let stem = std::path::Path::new(&file).file_stem().and_then(|s| s.to_str());
        Name::from(format!("{}@{}", sig.inner, stem.unwrap_or(&file)))
    }
}

/// Write the call of each recorded operator, print and template slot into
/// the tree.
///
/// # Arguments
///
/// * `program` - The program, checked.
/// * `calls` - The calls.
/// * `kinds` - The kind of each expression; each new node takes its kind.
/// * `next` - The first free node number.
pub(crate) fn write_op_calls(
    program: &mut Program, calls: &OpCalls, kinds: &mut KindTable, next: u32,
)
{
    ret_if!(
        calls.pairs.is_empty() && calls.wraps.is_empty() && calls.to_traits.is_empty()
    );
    let mut writer = Writer { calls, kinds, next };
    writer.visit_program_mut(program);
}

/// The walker that writes the calls.
struct Writer<'a>
{
    /// The calls.
    calls: &'a OpCalls,
    /// The kind of each expression.
    kinds: &'a mut KindTable,
    /// The next free node number.
    next: u32,
}

impl Writer<'_>
{
    /// A fresh node number with a kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the node.
    ///
    /// # Returns
    ///
    /// The number.
    fn fresh(&mut self, kind: DataKind) -> NodeId
    {
        let id = NodeId(self.next);
        self.next += 1;
        self.kinds.insert(id, kind);
        id
    }
}

impl Writer<'_>
{
    /// Put the call of an operator in place of it.
    ///
    /// # Arguments
    ///
    /// * `n` - The operation.
    /// * `call` - Its call.
    fn pair(&mut self, n: &mut Expr, call: OpCall)
    {
        let (id, span) = (n.id(), n.span());
        let callee = Ident::builder().name(call.callee).span(span).build();
        let placeholder = Expr::Null(NullLit::builder().id(id).span(span).build());
        let args = match std::mem::replace(n, placeholder)
        {
            Expr::Infix(e) => vec![*e.lhs, *e.rhs],
            Expr::Prefix(e) => vec![*e.rhs],
            Expr::Verb(v) => v.args,
            other => return *n = other,
        };
        let at = tern!(call.negate, self.fresh(DataKind::Bool), id);
        let made = Expr::Call(
            CallExpr::builder().id(at).callee(callee).args(args).span(span).build(),
        );
        let not = |rhs| {
            PrefixExpr::builder()
                .id(id)
                .op(PrefixOp::Not)
                .rhs(Box::new(rhs))
                .span(span)
                .build()
        };
        *n = tern!(call.negate, Expr::Prefix(not(made.clone())), made);
    }

    /// Put the call that makes a trait value around a value: its arguments are
    /// the value, then the address of each function that gives the value's
    /// kind a function of the trait.
    ///
    /// # Arguments
    ///
    /// * `n` - The value.
    /// * `kind` - The trait value kind.
    /// * `fns` - The functions, in the trait's order.
    fn to_trait(&mut self, n: &mut Expr, kind: DataKind, fns: &[Name])
    {
        self.wrap(n, Name::new(TO_TRAIT_FN), kind);
        let Expr::Call(call) = n
        else
        {
            return;
        };
        let span = call.span;
        for f in fns
        {
            let name = Ident::builder().name(*f).span(span).build();
            let id = NodeId(self.next);
            self.next += 1;
            let addr = FnAddrExpr::builder().id(id).name(name).span(span).build();
            call.args.push(Expr::FnAddr(addr));
        }
    }

    /// Put a call of a function around a value.
    ///
    /// # Arguments
    ///
    /// * `n` - The value.
    /// * `callee` - The function.
    /// * `kind` - The kind the function gives.
    fn wrap(&mut self, n: &mut Expr, callee: Name, kind: DataKind)
    {
        let span = n.span();
        let at = self.fresh(kind);
        let callee = Ident::builder().name(callee).span(span).build();
        let placeholder = Expr::Null(NullLit::builder().id(at).span(span).build());
        let value = std::mem::replace(n, placeholder);
        *n = Expr::Call(
            CallExpr::builder()
                .id(at)
                .callee(callee)
                .args(vec![value])
                .span(span)
                .build(),
        );
    }
}

impl VisitorMut for Writer<'_>
{
    fn visit_expr_mut(&mut self, n: &mut Expr)
    {
        walk_expr_mut(self, n);
        let id = n.id();
        do_if!(let Some(call) = self.calls.pairs.get(&id).copied(), self.pair(n, call));
        let wrap = self.calls.wraps.get(&id).copied();
        do_if!(let Some((callee, kind)) = wrap, self.wrap(n, callee, kind));
        let to_trait = self.calls.to_traits.get(&id).cloned();
        do_if!(let Some((kind, fns)) = to_trait, self.to_trait(n, kind, &fns));
    }
}

/// Put the call of each field read that stands for one, `x ၏ f` for `f(x)`,
/// in place of the read, before the overloads take their inner names.
///
/// # Arguments
///
/// * `program` - The program, checked with no error.
/// * `calls` - The calls the check recorded.
pub(crate) fn write_methods(program: &mut Program, calls: &OpCalls)
{
    let blank =
        calls.methods.is_empty() && calls.verbs.is_empty() && calls.fields.is_empty();
    ret_if!(blank && calls.texts.is_empty() && calls.fault_vias.is_empty());
    #[rustfmt::skip]
    MethodWriter { methods: &calls.methods, verbs: &calls.verbs, texts: &calls.texts,
        fault_vias: &calls.fault_vias, fields: &calls.fields }.visit_program_mut(program);
}

/// The walker that puts a call in place of a field read.
struct MethodWriter<'a>
{
    /// The call of each read, under its node.
    methods: &'a HashMap<NodeId, CallExpr>,
    /// The call of each verb statement, under its span.
    verbs: &'a HashMap<Span, CallStmt>,
    /// The conversions to text that become template texts.
    texts: &'a HashSet<NodeId>,
    /// The function each call that passes a failure on turns its error value
    /// through, under the span of the call.
    fault_vias: &'a HashMap<Span, Name>,
    /// The read of a part each read stands for.
    fields: &'a HashMap<NodeId, Expr>,
}

impl MethodWriter<'_>
{
    /// The function a call that passes a failure on turns its error value
    /// through.
    ///
    /// # Arguments
    ///
    /// * `span` - The call.
    ///
    /// # Returns
    ///
    /// The name, at the call, or `None`.
    fn via(&self, span: Span) -> Option<Ident>
    {
        let name = self.fault_vias.get(&span)?;
        Some(Ident::builder().name(*name).span(span).build())
    }
}

impl VisitorMut for MethodWriter<'_>
{
    fn visit_stmt_mut(&mut self, n: &mut Stmt)
    {
        let verb = matches!(
            n,
            Stmt::Print(_)
                | Stmt::Add(_)
                | Stmt::Remove(_)
                | Stmt::Replace(_)
                | Stmt::Call(_)
        );
        let call = tern!(verb, self.verbs.get(&n.span()), None);
        do_if!(let Some(call) = call, *n = Stmt::Call(call.clone()));
        do_if!(let Stmt::Call(c) = n, c.fault_via = self.via(c.span));
        walk_stmt_mut(self, n);
    }

    fn visit_expr_mut(&mut self, n: &mut Expr)
    {
        let read = self.fields.get(&n.id()).cloned();
        do_if!(let Some(read) = read, *n = read);
        let replaced = matches!(n, Expr::Field(_) | Expr::Verb(_) | Expr::Call(_));
        let call = tern!(replaced, self.methods.get(&n.id()), None);
        do_if!(let Some(call) = call, *n = Expr::Call(call.clone()));
        do_if!(self.texts.contains(&n.id()), to_template(n));
        do_if!(let Expr::Call(c) = n, c.fault_via = self.via(c.span));
        walk_expr_mut(self, n);
    }
}

/// Put the template text `"{x}"` in place of a conversion of `x` to text.
///
/// # Arguments
///
/// * `n` - The conversion.
fn to_template(n: &mut Expr)
{
    let Expr::Verb(v) = n
    else
    {
        return;
    };
    let Some(value) = v.args.pop()
    else
    {
        return;
    };
    let pieces = vec![StrPiece::TemplateSlot(Box::new(value))];
    *n = Expr::Str(StrExpr::builder().id(v.id).pieces(pieces).span(v.span).build());
}

/// The kinds of the kind parameters of a generic function of a conversion
/// trait for one conversion: the kind parameter its one parameter is written
/// with stands for the source kind, and each other one for the target kind.
/// A parameter of a written kind must be the source kind.
///
/// # Arguments
///
/// * `t` - The generic function.
/// * `from` - The source kind.
/// * `to` - The target kind.
/// * `structs` - The kind of each declared struct.
///
/// # Returns
///
/// The kinds, or `None` when the function takes no such value.
fn conversion_kinds(
    t: &FnDecl, from: DataKind, to: DataKind, structs: &StructTable,
) -> Option<Vec<DataKind>>
{
    let [param] = t.params.as_slice()
    else
    {
        return None;
    };
    let bare = param.kind.args.is_empty() && param.kind.wraps.is_empty();
    let named = param.kind.name.as_ref().map(|i| i.name).filter(|_| bare);
    let source = |p: &KindParam| named == Some(p.name.name);
    let generic = t.kind_params.iter().any(source);
    ret_if!(!generic && DataKind::from_ast(&param.kind, structs) != from, None);
    Some(t.kind_params.iter().map(|p| tern!(source(p), from, to)).collect())
}
