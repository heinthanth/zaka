use std::ops::Add;
use std::ops::Sub;

/// An absolute offset in the [`SourceMap`](crate::SourceMap).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Pos(u32);

/// A offset relative to the start of a [`SourceFile`](crate::SourceFile).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct RelativePos(u32);

/// A u8 offset within a line (Unicode scalar values).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct CharPos(u32);

impl Pos
{
    /// The zero position
    pub const ZERO: Pos = Pos(0);

    /// The first valid position for a real source file.
    pub const ONE: Pos = Pos(1);

    /// Create a new absolute position.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position to create.
    ///
    /// # Returns
    ///
    /// The new position.
    pub const fn new(pos: u32) -> Self { Pos(pos) }

    /// Return the underlying `u32` value.
    ///
    /// # Returns
    ///
    /// The underlying `u32` value.
    pub const fn as_u32(&self) -> u32 { self.0 }

    /// Advance `count` items.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of items to advance.
    ///
    /// # Returns
    ///
    /// The advanced position.
    pub fn advanced(&self, count: usize) -> Self { Pos(self.0 + count as u32) }
}

impl RelativePos
{
    /// The start of a source file.
    pub const ZERO: RelativePos = RelativePos(0);

    /// Create a new relative position.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position to create.
    ///
    /// # Returns
    ///
    /// The new relative position.
    pub const fn new(pos: u32) -> Self { RelativePos(pos) }

    /// Return the internal `u32` value.
    ///
    /// # Returns
    ///
    /// The internal `u32` value.
    pub const fn as_u32(&self) -> u32 { self.0 }
}

impl CharPos
{
    /// The start of a line.
    pub const ZERO: CharPos = CharPos(0);

    /// Create a new position of the u8 character.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position to create.
    ///
    /// # Returns
    ///
    /// The new position of the u8 character.
    pub const fn new(pos: u32) -> Self { CharPos(pos) }

    /// Return the internal `u32` value.
    ///
    /// # Returns
    ///
    /// The internal `u32` value.
    pub const fn as_u32(&self) -> u32 { self.0 }

    /// Create from a `usize` column offset.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position to create.
    ///
    /// # Returns
    ///
    /// The new position of the u8 character.
    pub fn from_usize(pos: usize) -> Self { CharPos(pos as u32) }
}

impl Add<usize> for &Pos
{
    type Output = Pos;

    fn add(self, rhs: usize) -> Self::Output { self.advanced(rhs) }
}

impl Add<usize> for Pos
{
    type Output = Pos;

    /// Add `rhs` items to `self`.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The number of items to add.
    ///
    /// # Returns
    ///
    /// The added position.
    fn add(self, rhs: usize) -> Self::Output { (&self).add(rhs) }
}

impl Sub<&Pos> for &Pos
{
    type Output = u32;

    fn sub(self, rhs: &Pos) -> Self::Output { self.0 - rhs.0 }
}

impl Sub for Pos
{
    type Output = u32;

    /// Subtract `rhs` from `self`.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The position to subtract.
    ///
    /// # Returns
    ///
    /// The subtracted position.
    fn sub(self, rhs: Pos) -> Self::Output { (&self).sub(&rhs) }
}

impl Add<&Pos> for &Pos
{
    type Output = Pos;

    fn add(self, rhs: &Pos) -> Self::Output { Pos(self.0 + rhs.0) }
}

impl Add<Pos> for Pos
{
    type Output = Pos;

    /// Add `rhs` to `self`.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The position to add.
    ///
    /// # Returns
    ///
    /// The added position.
    fn add(self, rhs: Pos) -> Self::Output { (&self).add(&rhs) }
}

impl Add<u32> for &RelativePos
{
    type Output = RelativePos;

    fn add(self, rhs: u32) -> Self::Output { RelativePos(self.0 + rhs) }
}

impl Add<u32> for RelativePos
{
    type Output = RelativePos;

    /// Add `rhs` items to `self`.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The number of items to add.
    ///
    /// # Returns
    ///
    /// The added position.
    fn add(self, rhs: u32) -> Self::Output { (&self).add(rhs) }
}
