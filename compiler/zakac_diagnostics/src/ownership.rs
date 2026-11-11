use strum::IntoStaticStr;
use zakac_i18n::t;
use zakac_span::span::Span;

use crate::code::Code;
use crate::diagnostic::Diagnostic;

/// What an owner does to its block while a borrow of it is still in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Invalidation
{
    /// The block is freed.
    Freed,
    /// The block is resized, so it can move.
    Resized,
    /// The block moves to another owner.
    Moved,
    /// A plain list is written, so its cells can move to a block of its own.
    Edited,
}

impl Invalidation
{
    /// The localized past participle for the `ZK0704` message.
    ///
    /// # Returns
    ///
    /// The text, for example `freed`.
    pub fn text(self) -> String
    {
        let name = format!("codes.ZK0704.{}", <&'static str>::from(self));
        t!(name).to_string()
    }
}

/// A name is used after its block moved to another owner.
///
/// # Arguments
///
/// * `span` - The use of the moved name.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0701` diagnostic.
pub fn moved_out(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::MovedOut, t!("codes.ZK0701.message", name = name), span)
}

/// A pointer to a variable, not to a heap block, is freed or resized.
///
/// # Arguments
///
/// * `span` - The statement.
///
/// # Returns
///
/// The `ZK0702` diagnostic.
pub fn bad_free(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::BadFree, t!("codes.ZK0702.message"), span)
}

/// A borrowed address is stored where it could outlast its owner.
///
/// # Arguments
///
/// * `span` - The store.
///
/// # Returns
///
/// The `ZK0703` diagnostic.
pub fn borrow_stored(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::BorrowStored, t!("codes.ZK0703.message"), span)
}

/// An owner changes its block while a borrow of it is used later.
///
/// # Arguments
///
/// * `span` - The operation that changes the block.
/// * `name` - The owner.
/// * `what` - What happens to the block, from [`Invalidation::text`].
/// * `other` - The borrow that is still in use.
/// * `use_span` - The later use of the borrow, shown as a label.
///
/// # Returns
///
/// The `ZK0704` diagnostic.
pub fn invalidated_loan(
    span: Span, name: &str, what: &str, other: &str, use_span: Span,
) -> Diagnostic
{
    let msg = t!("codes.ZK0704.message", name = name, what = what, other = other);
    let label = t!("codes.ZK0704.label", other = other);
    Diagnostic::new(Code::InvalidatedLoan, msg, span).with_label(use_span, label)
}

/// A function returns the address of one of its locals.
///
/// # Arguments
///
/// * `span` - The returned address.
/// * `name` - The local.
///
/// # Returns
///
/// The `ZK0706` diagnostic.
pub fn local_borrow_returned(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0706.message", name = name);
    Diagnostic::new(Code::LocalBorrowReturned, msg, span)
}

/// A call frees through one argument while another reaches the same block.
///
/// # Arguments
///
/// * `span` - The call.
/// * `func` - The called function.
/// * `n` - The number of the argument it frees through.
/// * `m` - The number of the argument that reaches the same block.
///
/// # Returns
///
/// The `ZK0707` diagnostic.
pub fn aliased_argument(span: Span, func: &str, n: usize, m: usize) -> Diagnostic
{
    let msg = t!("codes.ZK0707.message", func = func, n = n, m = m);
    Diagnostic::new(Code::AliasedArgument, msg, span)
}

/// An owner goes out of scope without its block freed or moved.
///
/// # Arguments
///
/// * `span` - The owner.
/// * `name` - The owner name.
///
/// # Returns
///
/// The `ZK0708` diagnostic.
pub fn block_leaked(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::BlockLeaked, t!("codes.ZK0708.message", name = name), span)
}

/// The result variable of a function with an unmarked result still holds a
/// block the function made when the function returns.
///
/// # Arguments
///
/// * `span` - The exit.
/// * `name` - The result variable.
///
/// # Returns
///
/// The `ZK0708` diagnostic, with help to mark the result.
pub fn result_block_leaked(span: Span, name: &str) -> Diagnostic
{
    block_leaked(span, name).with_help(t!("codes.ZK0708.help_result"))
}

/// A name known to hold the null address is dereferenced.
///
/// # Arguments
///
/// * `span` - The dereference.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0709` diagnostic.
pub fn null_deref(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::NullDeref, t!("codes.ZK0709.message", name = name), span)
}

/// A constant offset is past the end of its block.
///
/// # Arguments
///
/// * `span` - The offset operation.
/// * `off` - The offset.
/// * `len` - The number of elements in the block.
///
/// # Returns
///
/// The `ZK0710` diagnostic.
pub fn offset_past_end(span: Span, off: i64, len: u64) -> Diagnostic
{
    let msg = t!("codes.ZK0710.message", off = off, len = len);
    Diagnostic::new(Code::OffsetPastEnd, msg, span)
}

/// A name that can hold the null address is dereferenced without a check.
///
/// # Arguments
///
/// * `span` - The dereference.
/// * `name` - The name.
///
/// # Returns
///
/// The `ZK0711` warning.
pub fn null_deref_possible(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0711.message", name = name);
    Diagnostic::new(Code::NullDerefPossible, msg, span)
}

/// Pointers into different blocks are subtracted.
///
/// # Arguments
///
/// * `span` - The subtraction.
///
/// # Returns
///
/// The `ZK0712` diagnostic.
pub fn mixed_blocks(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::MixedBlocks, t!("codes.ZK0712.message"), span)
}

/// With `--overflow stop`, an integer operation overflows for each value its
/// operands can hold.
///
/// # Arguments
///
/// * `span` - The operation.
/// * `op` - The operator, as written.
/// * `kind` - The integer kind, as its short name.
///
/// # Returns
///
/// The `ZK0719` diagnostic, with help on what to change.
pub fn overflow_proven(span: Span, op: &str, kind: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0719.message", op = op, kind = kind);
    Diagnostic::new(Code::OverflowProven, msg, span).with_help(t!("codes.ZK0719.help"))
}

/// The two ranges of a `ကူးယူမည်` overlap for certain.
///
/// # Arguments
///
/// * `span` - The statement.
/// * `octets` - The count of octets the two ranges share.
///
/// # Returns
///
/// The `ZK0721` diagnostic, with help that names the statement for ranges
/// that overlap.
pub fn blit_ranges_overlap(span: Span, octets: u64) -> Diagnostic
{
    let msg = t!("codes.ZK0721.message", n = octets);
    Diagnostic::new(Code::BlitRangesOverlap, msg, span)
        .with_help(t!("codes.ZK0721.help"))
}

/// A read of a value that can be unset on a path to it.
///
/// # Arguments
///
/// * `span` - The read.
/// * `name` - The variable.
///
/// # Returns
///
/// The `ZK0722` diagnostic.
pub fn read_unset(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::ReadUnset, t!("codes.ZK0722.message", n = name), span)
        .with_help(t!("codes.ZK0722.help"))
}

/// A read of a cell the check cannot prove set.
///
/// # Arguments
///
/// * `span` - The read.
/// * `name` - The list.
///
/// # Returns
///
/// The `ZK0723` warning.
pub fn read_unset_doubt(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::ReadUnsetDoubt, t!("codes.ZK0723.message", n = name), span)
        .with_help(t!("codes.ZK0723.help"))
}

/// A call of a function that reads a global that can be unset at the call.
///
/// # Arguments
///
/// * `span` - The call.
/// * `func` - The function.
/// * `global` - The global.
///
/// # Returns
///
/// The `ZK0724` diagnostic.
pub fn global_unset_at_call(span: Span, func: &str, global: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0724.message", f = func, g = global);
    Diagnostic::new(Code::GlobalUnsetAtCall, msg, span)
        .with_help(t!("codes.ZK0724.help", g = global))
}

/// A return that can leave a parameter marked `ဒေတာ ထည့်ပေးရန်` unset.
///
/// # Arguments
///
/// * `span` - The return, or the end of the function.
/// * `name` - The parameter.
///
/// # Returns
///
/// The `ZK0725` diagnostic.
pub fn fill_unset_at_return(span: Span, name: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0725.message", n = name);
    Diagnostic::new(Code::FillUnsetAtReturn, msg, span)
        .with_help(t!("codes.ZK0725.help", n = name))
}
