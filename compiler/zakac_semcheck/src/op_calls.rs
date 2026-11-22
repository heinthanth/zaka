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
use zakac_ast::expr::Ident;
use zakac_ast::expr::InfixOp;
use zakac_ast::expr::NodeId;
use zakac_ast::expr::NullLit;
use zakac_ast::expr::PrefixExpr;
use zakac_ast::expr::PrefixOp;
use zakac_ast::expr::StrExpr;
use zakac_ast::expr::StrPiece;
use zakac_ast::expr::VerbExpr;
use zakac_ast::name::Name;
use zakac_ast::stmt::CallStmt;
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

use crate::kind::DataKind;
use crate::scope::FnSig;
use crate::scope::Written;
use crate::semcheck::Checker;
use crate::semcheck::KindTable;

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
    /// The function each printed value or template slot goes through.
    wraps: HashMap<NodeId, Name>,
    /// The call each field read stands for when no field fits, `x ၏ f` for
    /// `f(x)`, and each conversion that goes through a function, under the node
    /// of the read or the conversion.
    pub(crate) methods: HashMap<NodeId, CallExpr>,
    /// Each conversion to text that takes the text its value prints as, which
    /// becomes the template text `"{x}"`, under the node of the conversion.
    pub(crate) texts: HashSet<NodeId>,
    /// The call each statement of a verb the compiler gives stands for when
    /// a function of the file of that name takes its operands, `l ကို ပြသမည်။`
    /// for `ပြသ(l)`, under the span of the statement.
    pub(crate) verbs: HashMap<Span, CallStmt>,
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
        let Some(kind) =
            kind.filter(|k| k.as_struct().is_some() && self.kind_home(*k).is_some())
        else
        {
            return;
        };
        let Some(sig) = self
            .struct_fn(TEXT_FN, &[kind])
            .filter(|s| s.ret.is_some_and(DataKind::is_text))
        else
        {
            return;
        };
        let callee = self.known_name(&sig);
        self.op_calls.wraps.insert(value.id(), callee);
    }

    /// The function a conversion goes through: a `ပြောင်းလဲ` that takes the
    /// source kind and gives the target kind, or a `စမ်းသပ်ပြောင်းလဲ` that gives a
    /// result whose good side is the target kind, from the file of the use or
    /// the file of either kind. A conversion that passes a failure on takes the
    /// second first, a plain one the first.
    ///
    /// # Arguments
    ///
    /// * `from` - The source kind.
    /// * `to` - The target kind.
    /// * `pass` - Whether the conversion passes a failure on.
    ///
    /// # Returns
    ///
    /// The function, or `None` when none fits.
    pub(crate) fn conversion_fn(
        &self, from: DataKind, to: DataKind, pass: bool,
    ) -> Option<FnSig>
    {
        let sure =
            self.taking(CONVERT_FN, from, to).into_iter().find(|s| s.ret == Some(to));
        let good = |s: &FnSig| {
            let fails = s.ret.filter(|k| !k.is_optional());
            fails.and_then(DataKind::fallible_parts).and_then(|(ok, _)| ok) == Some(to)
        };
        let tried = self.taking(TRY_CONVERT_FN, from, to).into_iter().find(good);
        tern!(pass, tried.or(sure), sure.or(tried))
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
    /// * `sig` - The function.
    ///
    /// # Returns
    ///
    /// The result kind of the call: the good result when it passes a failure
    /// on.
    pub(crate) fn infer_conversion(
        &mut self, n: &VerbExpr, value: &Expr, sig: &FnSig,
    ) -> Option<DataKind>
    {
        let fails = sig.ret.and_then(DataKind::fallible_parts);
        let word = tern!(fails.is_some(), TRY_CONVERT_FN, CONVERT_FN);
        let written = Ident::builder().name(word).span(n.span).build();
        self.check_pass(&written, sig.ret, n.pass, false, n.span);
        let unsafe_call = format!("a call of \"{word}\", an unsafe function,");
        do_if!(sig.unchecked, self.need_unchecked(n.span, &unsafe_call));
        let callee = Ident::builder().name(self.known_name(sig)).span(n.span).build();
        #[rustfmt::skip]
        let call = CallExpr::builder().id(n.id).callee(callee).args(vec![value.clone()])
            .pass(n.pass).span(n.span).build();
        self.op_calls.methods.insert(n.id, call);
        tern!(let Some((ok, _)) = fails.filter(|_| n.pass), ok, sig.ret)
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
    fn known_name(&self, sig: &FnSig) -> Name
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
    ret_if!(calls.pairs.is_empty() && calls.wraps.is_empty());
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

    /// Put a call of a function around a value.
    ///
    /// # Arguments
    ///
    /// * `n` - The value.
    /// * `callee` - The function.
    fn wrap(&mut self, n: &mut Expr, callee: Name)
    {
        let span = n.span();
        let at = self.fresh(DataKind::TEXT);
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
        do_if!(let Some(callee) = self.calls.wraps.get(&id).copied(), self.wrap(n, callee));
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
    let blank = calls.methods.is_empty() && calls.verbs.is_empty();
    ret_if!(blank && calls.texts.is_empty());
    MethodWriter { methods: &calls.methods, verbs: &calls.verbs, texts: &calls.texts }
        .visit_program_mut(program);
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
}

impl VisitorMut for MethodWriter<'_>
{
    fn visit_stmt_mut(&mut self, n: &mut Stmt)
    {
        let verb = matches!(
            n,
            Stmt::Print(_) | Stmt::Add(_) | Stmt::Remove(_) | Stmt::Replace(_)
        );
        let call = tern!(verb, self.verbs.get(&n.span()), None);
        do_if!(let Some(call) = call, *n = Stmt::Call(call.clone()));
        walk_stmt_mut(self, n);
    }

    fn visit_expr_mut(&mut self, n: &mut Expr)
    {
        let replaced = matches!(n, Expr::Field(_) | Expr::Verb(_));
        let call = tern!(replaced, self.methods.get(&n.id()), None);
        do_if!(let Some(call) = call, *n = Expr::Call(call.clone()));
        do_if!(self.texts.contains(&n.id()), to_template(n));
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
