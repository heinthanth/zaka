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

/// A borrowed address is freed or resized.
///
/// # Arguments
///
/// * `span` - The borrowed address.
///
/// # Returns
///
/// The `ZK0702` diagnostic.
pub fn freed_borrow(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::FreedBorrow, t!("codes.ZK0702.message"), span)
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

/// The address of a borrowed address is taken.
///
/// # Arguments
///
/// * `span` - The address-of operation.
///
/// # Returns
///
/// The `ZK0705` diagnostic.
pub fn borrow_of_borrow(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::BorrowOfBorrow, t!("codes.ZK0705.message"), span)
}

/// A function returns a borrow of one of its locals.
///
/// # Arguments
///
/// * `span` - The returned borrow.
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

/// A name known to hold ဗလာ is dereferenced.
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

/// A name that can hold ဗလာ is dereferenced without a check.
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

/// A borrow marker sits where none is allowed.
///
/// # Arguments
///
/// * `span` - The marked kind.
///
/// # Returns
///
/// The `ZK0713` diagnostic.
pub fn bad_borrow_marker(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::BadBorrowMarker, t!("codes.ZK0713.message"), span)
}

/// A borrowed address is given where an owner is wanted.
///
/// # Arguments
///
/// * `span` - The borrowed address.
///
/// # Returns
///
/// The `ZK0714` diagnostic.
pub fn borrow_for_owner(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::BorrowForOwner, t!("codes.ZK0714.message"), span)
}

/// An owner is given where a borrow is wanted and would move.
///
/// # Arguments
///
/// * `span` - The owner.
/// * `name` - The variable that holds the owner, or `None` for a value with no
///   name, which has no lend spelling to offer.
///
/// # Returns
///
/// The `ZK0715` diagnostic; for a variable, with a help that shows the lend
/// spelling.
pub fn owner_for_borrow(span: Span, name: Option<&str>) -> Diagnostic
{
    let Some(name) = name
    else
    {
        return Diagnostic::new(Code::OwnerForBorrow, t!("codes.ZK0715.unnamed"), span);
    };
    let msg = t!("codes.ZK0715.message", name = name);
    Diagnostic::new(Code::OwnerForBorrow, msg, span)
        .with_help(t!("codes.ZK0715.help", name = name))
}

/// A raw address is lent.
///
/// # Arguments
///
/// * `span` - The lent raw address.
///
/// # Returns
///
/// The `ZK0716` diagnostic.
pub fn lend_of_raw(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::LendOfRaw, t!("codes.ZK0716.message"), span)
}

/// A raw address is used though raw addresses are turned off.
///
/// # Arguments
///
/// * `span` - The raw address.
///
/// # Returns
///
/// The `ZK0717` diagnostic.
pub fn raw_not_allowed(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::RawNotAllowed, t!("codes.ZK0717.message"), span)
}

/// A struct field holds a borrow.
///
/// # Arguments
///
/// * `span` - The field.
/// * `name` - The field name.
///
/// # Returns
///
/// The `ZK0718` diagnostic.
pub fn borrow_field(span: Span, name: &str) -> Diagnostic
{
    Diagnostic::new(Code::BorrowField, t!("codes.ZK0718.message", n = name), span)
}
