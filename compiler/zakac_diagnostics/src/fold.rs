use zakac_i18n::t;
use zakac_span::span::Span;

use crate::code::Code;
use crate::diagnostic::Diagnostic;

/// An operation on constants gives a value outside its kind.
///
/// # Arguments
///
/// * `span` - The operation.
/// * `op` - The operator spelling.
/// * `kind` - The result kind, as text.
///
/// # Returns
///
/// The `ZK0601` diagnostic.
pub fn constant_overflow(span: Span, op: &str, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0601.message", op = op, kind = kind);
    Diagnostic::new(Code::ConstantOverflow, msg, span)
}

/// An integer division or remainder has the constant zero as divisor.
///
/// # Arguments
///
/// * `span` - The operation.
/// * `op` - The operator spelling.
///
/// # Returns
///
/// The `ZK0602` diagnostic.
pub fn constant_zero_divisor(span: Span, op: &str) -> Diagnostic
{
    Diagnostic::new(
        Code::ConstantZeroDivisor,
        t!("codes.ZK0602.message", op = op),
        span,
    )
}
