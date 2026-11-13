//! Operators on structs. A struct takes `+ - * / %`, the bit operators, the
//! shifts, the `-` of one operand, the comparisons, `==` and `!=` through
//! functions of its own, the ones the traits of the prelude name: `+` on two
//! values of a struct calls its `ပေါင်း`, `<` its `ငယ်မငယ်စစ်`, `>=` the
//! negation of its `ငယ်မငယ်စစ်`, `<<` its `ဘစ်ဘယ်ရွေ့` with the distance, and so
//! on. Printing
//! and a template slot call its `စာသားအဖြစ်ပြောင်း`. The check records each
//! such operator, and once it settles writes the call into the tree, so the
//! later stages see a plain call.

use std::collections::HashMap;

use zakac_ast::expr::CallExpr;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::InfixOp;
use zakac_ast::expr::NodeId;
use zakac_ast::expr::NullLit;
use zakac_ast::expr::PrefixExpr;
use zakac_ast::expr::PrefixOp;
use zakac_ast::name::Name;
use zakac_ast::stmt::Program;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Node;
use zakac_ast::visit::VisitorMut;
use zakac_ast::visit::walk_expr_mut;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::kind::DataKind;
use crate::scope::FnSig;
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
