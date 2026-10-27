use zakac_i18n::t;
use zakac_rust_sugars::tern;
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
/// The `ZK0402` diagnostic, with a hint to compare a pointer with null.
pub fn condition_not_bool(span: Span, got: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0402.message", got = got);
    let diag = Diagnostic::new(Code::ConditionNotBool, msg, span);
    let pointer = got.starts_with('*') || got == "ptr";
    tern!(pointer, diag.with_help(t!("codes.ZK0402.help")), diag)
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

/// An assignment target is neither a variable nor a dereference.
///
/// # Arguments
///
/// * `span` - The target operand.
///
/// # Returns
///
/// The `ZK0410` diagnostic.
pub fn not_a_place(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::NotAPlace, t!("codes.ZK0410.message"), span)
}

/// The address of a value that has no address is taken.
///
/// # Arguments
///
/// * `span` - The operand of the address operation.
/// * `what` - What the operand is, in words.
///
/// # Returns
///
/// The `ZK0411` diagnostic.
pub fn no_address(span: Span, what: &str) -> Diagnostic
{
    Diagnostic::new(Code::NoAddress, t!("codes.ZK0411.message", what = what), span)
}

/// The address of a constant or a counted-loop variable is taken.
///
/// # Arguments
///
/// * `span` - The operand of the address operation.
/// * `name` - The fixed name.
///
/// # Returns
///
/// The `ZK0412` diagnostic.
pub fn fixed_address(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::FixedAddress, t!("codes.ZK0412.message", name = name), span)
}

/// A value that is not a pointer is dereferenced.
///
/// # Arguments
///
/// * `span` - The dereferenced operand.
/// * `kind` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0413` diagnostic.
pub fn not_a_pointer(span: Span, kind: &str) -> Diagnostic
{
    Diagnostic::new(Code::NotAPointer, t!("codes.ZK0413.message", kind = kind), span)
}

/// An opaque pointer is dereferenced or offset.
///
/// # Arguments
///
/// * `span` - The opaque pointer operand.
///
/// # Returns
///
/// The `ZK0414` diagnostic, with a hint about the conversion verb.
pub fn opaque_pointer(span: Span) -> Diagnostic
{
    let diag = Diagnostic::new(Code::OpaquePointer, t!("codes.ZK0414.message"), span);
    diag.with_help(t!("codes.ZK0414.help"))
}

/// An operator that pointers do not take is used on a pointer.
///
/// # Arguments
///
/// * `span` - The operation.
/// * `op` - The operator spelling.
///
/// # Returns
///
/// The `ZK0415` diagnostic.
pub fn pointer_operator(span: Span, op: &str) -> Diagnostic
{
    Diagnostic::new(Code::PointerOperator, t!("codes.ZK0415.message", op = op), span)
}

/// Pointers to different kinds are compared or subtracted.
///
/// # Arguments
///
/// * `span` - The operation.
/// * `a` - The kind of the left pointer, as text.
/// * `b` - The kind of the right pointer, as text.
///
/// # Returns
///
/// The `ZK0416` diagnostic.
pub fn pointer_kinds_differ(span: Span, a: &str, b: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0416.message", a = a, b = b);
    Diagnostic::new(Code::PointerKindsDiffer, msg, span)
}

/// A heap statement is given a value that is not a pointer.
///
/// # Arguments
///
/// * `span` - The operand of the heap statement.
/// * `kind` - The found kind, as text.
///
/// # Returns
///
/// The `ZK0417` diagnostic.
pub fn heap_needs_pointer(span: Span, kind: &str) -> Diagnostic
{
    Diagnostic::new(
        Code::HeapNeedsPointer,
        t!("codes.ZK0417.message", kind = kind),
        span,
    )
}

/// A function returns the address of one of its locals.
///
/// # Arguments
///
/// * `span` - The returned value.
/// * `name` - The local whose address is returned.
///
/// # Returns
///
/// The `ZK0418` warning.
pub fn local_address_returned(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0418.message", name = name);
    Diagnostic::new(Code::LocalAddressReturned, msg, span)
}
