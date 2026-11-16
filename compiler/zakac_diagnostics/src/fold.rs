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

/// A range of known text starts or ends inside a character.
///
/// # Arguments
///
/// * `span` - The range.
/// * `start` - The first u8, as written.
/// * `len` - The u8 count, as written.
///
/// # Returns
///
/// The `ZK0607` diagnostic.
pub fn range_cuts_char(span: Span, start: &str, len: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0607.message", s = start, len = len);
    Diagnostic::new(Code::RangeCutsChar, msg, span)
}

/// A text constant holds a value that needs the program to run.
///
/// # Arguments
///
/// * `span` - The first part of the value that does not fold.
///
/// # Returns
///
/// The `ZK0606` diagnostic, with help on what to declare instead.
pub fn const_not_known(span: Span) -> Diagnostic
{
    let diag = Diagnostic::new(Code::ConstNotKnown, t!("codes.ZK0606.message"), span);
    diag.with_help(t!("codes.ZK0606.help"))
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

/// An argument of a comptime call in a compile-time spot is not known at
/// compile time.
///
/// # Arguments
///
/// * `span` - The argument.
/// * `name` - The function.
///
/// # Returns
///
/// The `ZK0608` diagnostic.
pub fn comptime_arg_not_known(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0608.message", n = name);
    Diagnostic::new(Code::ComptimeArgNotKnown, msg, span)
        .with_help(t!("codes.ZK0608.help"))
}

/// A comptime call needs a value that waits on the call itself.
///
/// # Arguments
///
/// * `span` - The call.
/// * `name` - The function.
///
/// # Returns
///
/// The `ZK0609` diagnostic.
pub fn comptime_waits_on_itself(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0609.message", n = name);
    Diagnostic::new(Code::ComptimeWaitsOnItself, msg, span)
        .with_help(t!("codes.ZK0609.help"))
}

/// No runner is installed to run a comptime call.
///
/// # Arguments
///
/// * `span` - The call.
/// * `name` - The function.
///
/// # Returns
///
/// The `ZK0610` diagnostic.
pub fn comptime_no_runner(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0610.message", n = name);
    Diagnostic::new(Code::ComptimeNoRunner, msg, span)
        .with_help(t!("codes.ZK0610.help"))
}

/// A comptime call stopped while the compiler ran it.
///
/// # Arguments
///
/// * `span` - The call.
/// * `name` - The function.
/// * `reason` - What stopped it.
/// * `chain` - The calls that led there, innermost last.
///
/// # Returns
///
/// The `ZK0611` diagnostic.
pub fn comptime_run_stopped(
    span: Span, name: &str, reason: &str, chain: &[String],
) -> Diagnostic
{
    let msg = t!("codes.ZK0611.message", n = name, r = reason);
    let diag = Diagnostic::new(Code::ComptimeRunStopped, msg, span);
    if chain.is_empty()
    {
        return diag;
    }
    diag.with_help(t!("codes.ZK0611.help", c = chain.join(" → ")))
}

/// The reason of a comptime call whose code does not compile.
///
/// # Arguments
///
/// * `message` - The first error of the compile.
///
/// # Returns
///
/// The reason, for [`comptime_run_stopped`].
pub fn comptime_compile_reason(message: &str) -> String
{
    t!("codes.ZK0611.compile", m = message).to_string()
}

/// The reason of a comptime call that reached a C function.
///
/// # Arguments
///
/// * `name` - The C name.
///
/// # Returns
///
/// The reason, for [`comptime_run_stopped`].
pub fn comptime_foreign_reason(name: &str) -> String
{
    t!("codes.ZK0611.foreign", n = name).to_string()
}

/// What a comptime call printed while the compiler ran it.
///
/// # Arguments
///
/// * `span` - The call.
/// * `name` - The function.
/// * `text` - What it printed.
///
/// # Returns
///
/// The `ZK0612` note.
pub fn comptime_printed(span: Span, name: &str, text: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0612.message", n = name, t = text.trim_end());
    Diagnostic::new(Code::ComptimePrinted, msg, span)
}
