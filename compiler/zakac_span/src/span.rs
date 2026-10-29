use serde::Deserialize;
use serde::Serialize;

use crate::pos::Pos;

/// A region of source code between `lo` and `hi` offsets.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
pub struct Span
{
    /// Inclusive start offset.
    pub lo: Pos,
    /// Exclusive end offset.
    pub hi: Pos,
}

/// A value paired with its source span.
#[derive(Clone, PartialEq, Debug)]
pub struct Spanned<T>
{
    /// The wrapped value.
    pub value: T,
    /// The source span of the value.
    pub span: Span,
}

impl Span
{
    /// A placeholder span at `(0, 0)`.
    pub const PLACEHOLDER: Span = Span { lo: Pos::ZERO, hi: Pos::ZERO };

    /// Create a new span from inclusive `lo` and exclusive `hi`.
    pub const fn new(lo: Pos, hi: Pos) -> Self { Span { lo, hi } }

    /// Return `true` if this is the placeholder span.
    ///
    /// # Returns
    ///
    /// `true` if this is the placeholder span, `false` otherwise.
    pub fn is_placeholder(&self) -> bool
    {
        self.lo == Pos::ZERO && self.hi == Pos::ZERO
    }

    /// Return `true` if `hi == lo`.
    ///
    /// # Returns
    ///
    /// `true` if `hi == lo`, `false` otherwise.
    pub fn is_blank(&self) -> bool { self.lo == self.hi }

    /// Return `true` if `self` contains `other`.
    ///
    /// # Arguments
    ///
    /// * `other` - The span to check if it is contained in `self`.
    ///
    /// # Returns
    ///
    /// `true` if `other` is contained in `self`, `false` otherwise.
    pub fn contains(&self, other: &Span) -> bool
    {
        self.lo <= other.lo && other.hi <= self.hi
    }

    /// Return `true` if `self` overlaps `other`.
    ///
    /// # Arguments
    ///
    /// * `other` - The span to check if it overlaps with `self`.
    ///
    /// # Returns
    ///
    /// `true` if `self` overlaps `other`, `false` otherwise.
    pub fn overlaps(&self, other: &Span) -> bool
    {
        !(self.hi <= other.lo || other.hi <= self.lo)
    }

    /// Return `true` if `self` touches or adjoins `other`.
    ///
    /// # Arguments
    ///
    /// * `other` - The span to check if it touches or adjoins `self`.
    ///
    /// # Returns
    ///
    /// `true` if `self` touches or adjoins `other`, `false` otherwise.
    pub fn overlaps_or_adjacent(&self, other: &Span) -> bool
    {
        !(self.hi < other.lo || other.hi < self.lo)
    }

    /// Return a span from `self.lo` to `other.hi`.
    ///
    /// # Arguments
    ///
    /// * `other` - The span to create a span from `self.lo` to `other.hi`.
    ///
    /// # Returns
    ///
    /// The span from `self.lo` to `other.hi`.
    pub const fn to(&self, other: &Span) -> Span { Span::new(self.lo, other.hi) }

    /// Return a span from `lo` to `hi`.
    ///
    /// # Arguments
    ///
    /// * `lo` - The start span.
    /// * `hi` - The end span.
    ///
    /// # Returns
    ///
    /// The span from `lo` to `hi`.
    pub const fn between(lo: &Span, hi: &Span) -> Span { Span::new(lo.lo, hi.hi) }

    /// Return a span from `self.lo` to `other.lo`.
    ///
    /// # Arguments
    ///
    /// * `other` - The span to create a span from `self.lo` to `other.lo`.
    ///
    /// # Returns
    ///
    /// The span from `self.lo` to `other.lo`.
    pub const fn until(&self, other: &Span) -> Span { Span::new(self.lo, other.lo) }

    /// Return an blank span at the start of `self`.
    ///
    /// # Returns
    ///
    /// The span from `self.lo` to `self.lo`.
    pub const fn shrink_to_lo(&self) -> Span { Span::new(self.lo, self.lo) }

    /// Return an blank span at the end of `self`.
    ///
    /// # Returns
    ///
    /// The span from `self.hi` to `self.hi`.
    pub const fn shrink_to_hi(&self) -> Span { Span::new(self.hi, self.hi) }

    /// Return `self` with a new low bound.
    ///
    /// # Arguments
    ///
    /// * `lo` - The new low bound.
    ///
    /// # Returns
    ///
    /// The span with a new low bound.
    pub const fn with_lo(&self, lo: &Pos) -> Span { Span::new(*lo, self.hi) }

    /// Return `self` with a new high bound.
    ///
    /// # Arguments
    ///
    /// * `hi` - The new high bound.
    ///
    /// # Returns
    ///
    /// The span with a new high bound.
    pub const fn with_hi(&self, hi: &Pos) -> Span { Span::new(self.lo, *hi) }
}

impl<T> Spanned<T>
{
    /// Wrap `value` with `span`.
    pub const fn new(value: T, span: Span) -> Self { Spanned { value, span } }
}
