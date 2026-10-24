use zakac_i18n::t;
use zakac_span::span::Span;

use crate::code::Code;
use crate::diagnostic::Diagnostic;

/// A value does not convert to the kind its place expects.
///
/// # Arguments
///
/// * `span` - The value.
/// * `want` - The expected kind, as text.
/// * `got` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0401` diagnostic, with a hint about the conversion verb.
pub fn kind_mismatch(span: Span, want: &str, got: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0401.message", want = want, got = got);
    Diagnostic::new(Code::KindMismatch, msg, span).with_help(t!("codes.ZK0401.help"))
}

/// A condition is not a boolean.
///
/// # Arguments
///
/// * `span` - The condition.
/// * `got` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0402` diagnostic.
pub fn condition_not_bool(span: Span, got: &str) -> Diagnostic
{
    Diagnostic::new(Code::ConditionNotBool, t!("codes.ZK0402.message", got = got), span)
}

/// An operand has a kind its operator does not take.
///
/// # Arguments
///
/// * `span` - The operand.
/// * `op` - The operator spelling.
/// * `want` - What the operator takes, in words.
/// * `got` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0403` diagnostic.
pub fn operand_kind(span: Span, op: &str, want: &str, got: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0403.message", op = op, want = want, got = got);
    Diagnostic::new(Code::OperandKind, msg, span)
}

/// Two operands have no common kind.
///
/// # Arguments
///
/// * `span` - The operation.
/// * `lhs` - The kind of the left operand, as text.
/// * `rhs` - The kind of the right operand, as text.
///
/// # Returns
///
/// The `ZK0404` diagnostic.
pub fn incompatible_operands(span: Span, lhs: &str, rhs: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0404.message", l = lhs, r = rhs);
    Diagnostic::new(Code::IncompatibleOperands, msg, span)
}

/// A counted-loop bound or step is not an integer.
///
/// # Arguments
///
/// * `span` - The bound or step.
/// * `got` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0405` diagnostic.
pub fn loop_bound_not_int(span: Span, got: &str) -> Diagnostic
{
    Diagnostic::new(Code::LoopBoundNotInt, t!("codes.ZK0405.message", got = got), span)
}

/// An integer literal does not fit the kind its place expects.
///
/// # Arguments
///
/// * `span` - The literal.
/// * `value` - The literal value, as text.
/// * `kind` - The expected kind, as text.
///
/// # Returns
///
/// The `ZK0406` diagnostic.
pub fn literal_does_not_fit(span: Span, value: &str, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0406.message", v = value, k = kind);
    Diagnostic::new(Code::LiteralDoesNotFit, msg, span)
}

/// A constant shift distance is not below the bit width.
///
/// # Arguments
///
/// * `span` - The distance.
/// * `distance` - The distance, as text.
/// * `width` - The bit width of the shifted value.
///
/// # Returns
///
/// The `ZK0407` diagnostic.
pub fn shift_too_far(span: Span, distance: &str, width: u16) -> Diagnostic
{
    let width = width.to_string();
    let msg = t!("codes.ZK0407.message", n = distance, w = width);
    Diagnostic::new(Code::ShiftTooFar, msg, span)
}

/// A conversion between two kinds is not allowed.
///
/// # Arguments
///
/// * `span` - The conversion.
/// * `from` - The source kind, as text.
/// * `to` - The target kind, as text.
///
/// # Returns
///
/// The `ZK0408` diagnostic.
pub fn invalid_cast(span: Span, from: &str, to: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0408.message", from = from, to = to);
    Diagnostic::new(Code::InvalidCast, msg, span)
}

/// A counted-loop step is not positive.
///
/// # Arguments
///
/// * `span` - The step.
///
/// # Returns
///
/// The `ZK0409` diagnostic.
pub fn step_not_positive(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::StepNotPositive, t!("codes.ZK0409.message"), span)
}
