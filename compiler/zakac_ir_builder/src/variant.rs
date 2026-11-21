//! Values of an enumeration whose variants hold data: the struct of a
//! variant, the read of its tag, and the read of the data of a variant.

use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::variants::Variants;
use zakac_span::span::Span;

use crate::fault::blank;
use crate::ir::CmpOp;
use crate::ir::Expr;
use crate::ir::ExprNode;
use crate::value::Value;

/// The value of a variant: its tag, its data in its own field, and each
/// other field at zero.
///
/// # Arguments
///
/// * `owner` - The enumeration.
/// * `at` - The position of the variant.
/// * `data` - The data of the variant, for a variant that holds some.
/// * `platform` - Bit count of the platform-sized kinds.
/// * `span` - The source region.
///
/// # Returns
///
/// The value.
pub(crate) fn value(
    owner: Variants, at: usize, data: Option<Expr>, platform: u16, span: Span,
) -> Expr
{
    let own = owner.data(at).map(|(field, _)| field);
    let mut data = data;
    let fields =
        owner.shape().fields().iter().enumerate().skip(1).map(|(field, (_, k))| {
            let given = tern!(own == Some(field), data.take(), None);
            given.unwrap_or_else(|| blank(*k, span))
        });
    let cells =
        std::iter::once(tag_const(owner, at, platform, span)).chain(fields).collect();
    node_expr(owner.kind(), ExprNode::List(cells), span)
}

/// Whether a value is a variant: its tag `==` the position of the variant.
///
/// # Arguments
///
/// * `owner` - The enumeration.
/// * `base` - The value.
/// * `at` - The position of the variant.
/// * `platform` - Bit count of the platform-sized kinds.
/// * `span` - The source region.
///
/// # Returns
///
/// The boolean test.
pub(crate) fn is(
    owner: Variants, base: Expr, at: usize, platform: u16, span: Span,
) -> Expr
{
    let tag = field_of(owner, base, 0, platform, span);
    let lhs = Box::new(tag);
    let rhs = Box::new(tag_const(owner, at, platform, span));
    node_expr(DataKind::Bool, ExprNode::Compare { op: CmpOp::Eq, lhs, rhs }, span)
}

/// The read of the data of a variant out of a value.
///
/// # Arguments
///
/// * `owner` - The enumeration.
/// * `base` - The value.
/// * `at` - The position of the variant; it holds data.
/// * `platform` - Bit count of the platform-sized kinds.
/// * `span` - The source region.
///
/// # Returns
///
/// The read.
pub(crate) fn data_of(
    owner: Variants, base: Expr, at: usize, platform: u16, span: Span,
) -> Expr
{
    let (field, _) = owner.data(at).expect("the check found data in the variant");
    field_of(owner, base, field, platform, span)
}

/// The tag of a variant, a constant of the tag kind.
///
/// # Arguments
///
/// * `owner` - The enumeration.
/// * `at` - The position of the variant.
/// * `platform` - Bit count of the platform-sized kinds.
/// * `span` - The source region.
///
/// # Returns
///
/// The constant.
fn tag_const(owner: Variants, at: usize, platform: u16, span: Span) -> Expr
{
    let kind = owner.tag_kind();
    let int = kind.int_kind().expect("a tag is an integer");
    node_expr(kind, ExprNode::Const(Value::int(int, at as u128, platform)), span)
}

/// The read of a field of the struct of a value.
///
/// # Arguments
///
/// * `owner` - The enumeration.
/// * `base` - The value.
/// * `field` - The position of the field.
/// * `platform` - Bit count of the platform-sized kinds.
/// * `span` - The source region.
///
/// # Returns
///
/// The read.
fn field_of(
    owner: Variants, base: Expr, field: usize, platform: u16, span: Span,
) -> Expr
{
    let kind = owner.shape().fields()[field].1.repr();
    let offset = owner.shape().offset(field, platform);
    let field = u32::try_from(field).expect("a struct has fewer than 2^32 fields");
    let addr = ExprNode::FieldAddr { base: Box::new(base), field, offset };
    let addr = Box::new(node_expr(kind.pointer_to(), addr, span));
    node_expr(kind, ExprNode::Load { addr }, span)
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
