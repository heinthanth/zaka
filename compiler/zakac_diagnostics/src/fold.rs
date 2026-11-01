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

/// A constant cell number is outside its list.
///
/// # Arguments
///
/// * `span` - The cell number.
/// * `index` - The cell number, as text.
/// * `first` - The number of the first cell: 0 or 1.
/// * `last` - The number of the last cell.
///
/// # Returns
///
/// The `ZK0603` diagnostic.
pub fn cell_out_of_range(span: Span, index: &str, first: u8, last: u64) -> Diagnostic
{
    let msg = t!("codes.ZK0603.message", i = index, lo = first, hi = last);
    Diagnostic::new(Code::CellOutOfRange, msg, span)
}

/// A constant range passes the end of a list of known count.
///
/// # Arguments
///
/// * `span` - The range.
/// * `start` - The first cell of the range.
/// * `len` - The length of the range.
/// * `count` - The cell count of the list.
///
/// # Returns
///
/// The `ZK0605` diagnostic.
pub fn range_out_of_list(span: Span, start: &str, len: &str, count: u32) -> Diagnostic
{
    let msg = t!("codes.ZK0605.message", s = start, len = len, n = count);
    Diagnostic::new(Code::RangeOutOfList, msg, span)
}

/// A variable or constant of an imported file starts with a value that is
/// not known at compile time.
///
/// # Arguments
///
/// * `span` - The declaration.
/// * `name` - The declared name.
///
/// # Returns
///
/// The `ZK0604` diagnostic, with help on what is known.
pub fn global_init_not_constant(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0604.message", n = name);
    let diag = Diagnostic::new(Code::GlobalInitNotConstant, msg, span);
    diag.with_help(t!("codes.ZK0604.help"))
}
