//! Parts of the lowering of functions that can fail: the struct such a
//! function gives back, the reads of its fields, and the pass-on calls.

use bon::Builder;
use zakac_ast::expr as ast;
use zakac_ast::stmt as st;
use zakac_ast::verb::VerbOp;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_call_expr;
use zakac_ast::visit::walk_verb;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

use crate::ir::Expr;
use crate::ir::ExprNode;
use crate::ir::Place;
use crate::ir::Stmt;
use crate::value::Value;

/// What a function that can fail gives back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub(crate) struct Fault
{
    /// The struct the function gives back: a tag, the good result and the
    /// error value.
    pub wrapper: DataKind,
    /// The result kind, when the function has a result.
    pub ok: Option<DataKind>,
    /// The error kind, when the error side holds a value.
    pub err: Option<DataKind>,
}

impl Fault
{
    /// The shape of the struct a function that can fail gives back.
    ///
    /// # Arguments
    ///
    /// * `wrapper` - The struct kind.
    ///
    /// # Returns
    ///
    /// The shape, or `None` for another kind.
    pub fn of_kind(wrapper: DataKind) -> Option<Fault>
    {
        let (ok, err) = wrapper.fallible_parts()?;
        Some(Fault::builder().wrapper(wrapper).maybe_ok(ok).maybe_err(err).build())
    }

    /// The struct value a function of this shape gives back.
    ///
    /// # Arguments
    ///
    /// * `failed` - Whether it is a failure: tag 0, else tag 1.
    /// * `ok` - The good result; `None` for a failure, or for a good side with
    ///   no value.
    /// * `err` - The error value; `None` for a good result, or for an error
    ///   side with no value.
    /// * `platform` - Bit count of the platform-sized kinds.
    /// * `span` - The source region.
    ///
    /// # Returns
    ///
    /// The value, with the default of each part not given.
    pub fn value(
        self, failed: bool, ok: Option<Expr>, err: Option<Expr>, platform: u16,
        span: Span,
    ) -> Expr
    {
        let tag_kind = self.field_kind("tag");
        let DataKind::Int(int) = tag_kind
        else
        {
            unreachable!("the tag of a fallible result is an integer");
        };
        let tag = Value::int(int, u128::from(!failed), platform);
        let tag = node_expr(tag_kind, ExprNode::Const(tag), span);
        let ok = self.ok.map(|k| ok.unwrap_or_else(|| blank(k, span)));
        let err = self.err.map(|k| err.unwrap_or_else(|| blank(k, span)));
        let cells = [tag].into_iter().chain(ok).chain(err).collect();
        node_expr(self.wrapper, ExprNode::List(cells), span)
    }

    /// The kind of a field of the struct.
    ///
    /// # Arguments
    ///
    /// * `name` - `tag`, `ok` or `err`.
    ///
    /// # Returns
    ///
    /// The kind.
    fn field_kind(self, name: &str) -> DataKind { self.slot(name).1.repr() }

    /// The index and the kind of a field of the struct. An optional names its
    /// good side `some`, which `ok` reaches too.
    ///
    /// # Arguments
    ///
    /// * `name` - `tag`, `ok` or `err`.
    ///
    /// # Returns
    ///
    /// The index and the kind.
    fn slot(self, name: &str) -> (usize, DataKind)
    {
        let shape = self.wrapper.as_struct().expect("a fallible result is a struct");
        let some = tern!(name == "ok", shape.field("some"), None);
        shape.field(name).or(some).expect("a fallible result has the field")
    }

    /// The read of a field of the struct in a place.
    ///
    /// # Arguments
    ///
    /// * `place` - The place that holds the struct.
    /// * `name` - `tag`, `ok` or `err`.
    /// * `platform` - Bit count of the platform-sized kinds.
    /// * `span` - The source region.
    ///
    /// # Returns
    ///
    /// The read.
    pub fn field(self, place: Place, name: &str, platform: u16, span: Span) -> Expr
    {
        let base = node_expr(self.wrapper, ExprNode::Read(place), span);
        self.field_of(base, name, platform, span)
    }

    /// The read of a field of a struct value.
    ///
    /// # Arguments
    ///
    /// * `base` - The struct.
    /// * `name` - `tag`, `ok` or `err`.
    /// * `platform` - Bit count of the platform-sized kinds.
    /// * `span` - The source region.
    ///
    /// # Returns
    ///
    /// The read.
    pub fn field_of(self, base: Expr, name: &str, platform: u16, span: Span) -> Expr
    {
        let shape = self.wrapper.as_struct().expect("a fallible result is a struct");
        let (at, kind) = self.slot(name);
        let kind = kind.repr();
        let offset = shape.offset(at, platform);
        let field = u32::try_from(at).expect("a struct has fewer than 2^32 fields");
        let base = Box::new(base);
        let addr = ExprNode::FieldAddr { base, field, offset };
        let addr = Box::new(node_expr(kind.pointer_to(), addr, span));
        node_expr(kind, ExprNode::Load { addr }, span)
    }

    /// Whether the struct in a place holds a failure: its tag is 0.
    ///
    /// # Arguments
    ///
    /// * `place` - The place that holds the struct.
    /// * `platform` - Bit count of the platform-sized kinds.
    /// * `span` - The source region.
    ///
    /// # Returns
    ///
    /// The boolean test.
    pub fn failed(self, place: Place, platform: u16, span: Span) -> Expr
    {
        let base = node_expr(self.wrapper, ExprNode::Read(place), span);
        self.failed_of(base, platform, span)
    }

    /// Whether a struct value holds a failure: its tag is 0, as the blank
    /// struct holds, so a result with no value set is a failure.
    ///
    /// # Arguments
    ///
    /// * `base` - The struct.
    /// * `platform` - Bit count of the platform-sized kinds.
    /// * `span` - The source region.
    ///
    /// # Returns
    ///
    /// The boolean test.
    pub fn failed_of(self, base: Expr, platform: u16, span: Span) -> Expr
    {
        let tag = self.field_of(base, "tag", platform, span);
        let zero = blank(tag.kind, span);
        let node = ExprNode::Compare {
            op: crate::ir::CmpOp::Eq,
            lhs: Box::new(tag),
            rhs: Box::new(zero),
        };
        node_expr(DataKind::Bool, node, span)
    }
}

/// Wrap each return of a function that can fail that gives back a plain
/// result into the struct the function gives back, with tag 1. A return
/// whose value has the struct kind gives back a failure and is left as it
/// is.
///
/// # Arguments
///
/// * `stmts` - The statements of the function.
/// * `fault` - Its shape.
/// * `platform` - Bit count of the platform-sized kinds.
pub(crate) fn wrap_returns(stmts: &mut [Stmt], fault: Fault, platform: u16)
{
    for s in stmts
    {
        match s
        {
            Stmt::Return { value, span } =>
            {
                let wrapped = value.as_ref().is_some_and(|v| v.kind == fault.wrapper);
                do_if!(
                    !wrapped,
                    *value =
                        Some(fault.value(false, value.take(), None, platform, *span))
                );
            }
            Stmt::If { then, els, .. } =>
            {
                wrap_returns(then, fault, platform);
                wrap_returns(els, fault, platform);
            }
            Stmt::Loop { init, update, block, .. } =>
            {
                do_for!(list in [init, update, block], wrap_returns(list, fault, platform));
            }
            Stmt::Counted { block, .. } => wrap_returns(block, fault, platform),
            _ => (),
        }
    }
}

/// Counts the checked sites in a statement, leaving out the statements of
/// the blocks inside it: the calls that pass a failure on, and the
/// conversions of a result, which can stop the run.
#[derive(Default)]
struct PassCount(usize);

impl Visitor for PassCount
{
    fn visit_verb(&mut self, n: &ast::VerbExpr)
    {
        do_if!(matches!(n.op, VerbOp::Unwrap | VerbOp::UnwrapFault), self.0 += 1);
        walk_verb(self, n);
    }

    fn visit_block(&mut self, _: &st::BlockStmt) {}

    fn visit_fn(&mut self, _: &st::FnDecl) {}

    fn visit_call_expr(&mut self, n: &ast::CallExpr)
    {
        do_if!(n.pass, self.0 += 1);
        walk_call_expr(self, n);
    }
}

/// The number of calls that pass a failure on in the expressions a
/// statement runs itself.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// The count.
pub(crate) fn stmt_passes(s: &st::Stmt) -> usize
{
    let mut count = PassCount::default();
    count.visit_stmt(s);
    count.0
}

/// Whether an expression holds a call that passes a failure on.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// `true` if it does.
pub(crate) fn has_pass(e: &ast::Expr) -> bool
{
    let mut count = PassCount::default();
    count.visit_expr(e);
    count.0 > 0
}

/// Finds a call in the expressions a statement runs itself, leaving out
/// the blocks inside it.
#[derive(Default)]
struct CallFind(bool);

impl Visitor for CallFind
{
    fn visit_block(&mut self, _: &st::BlockStmt) {}

    fn visit_fn(&mut self, _: &st::FnDecl) {}

    fn visit_call_expr(&mut self, n: &ast::CallExpr)
    {
        self.0 = true;
        walk_call_expr(self, n);
    }
}

/// Whether an expression holds a call, which can change a list or a
/// struct that another part of the statement points into.
///
/// # Arguments
///
/// * `e` - The expression.
///
/// # Returns
///
/// `true` if it does.
pub(crate) fn has_call(e: &ast::Expr) -> bool
{
    let mut find = CallFind::default();
    find.visit_expr(e);
    find.0
}

/// The default value of a kind, as the IR spells it.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `span` - The source region.
///
/// # Returns
///
/// The value.
pub(crate) fn blank(kind: DataKind, span: Span) -> Expr
{
    let node = match kind.is_aggregate()
    {
        true => ExprNode::BlankList,
        false => ExprNode::Const(Value::default_of(kind)),
    };
    node_expr(kind, node, span)
}

/// An expression of a kind.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `node` - The operation.
/// * `span` - The source region.
///
/// # Returns
///
/// The expression.
fn node_expr(kind: DataKind, node: ExprNode, span: Span) -> Expr
{
    Expr::builder().kind(kind).node(node).span(span).build()
}
