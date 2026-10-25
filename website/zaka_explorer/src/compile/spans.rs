use zakac_rust_sugars::ret_if;
use zakac_span::pos::Pos;
use zakac_span::source_file::SourceFile;
use zakac_span::span::Span;

/// A region of the text the user wrote, in UTF-8 offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Range
{
    /// Inclusive start offset.
    pub lo: u32,
    /// Exclusive end offset.
    pub hi: u32,
}

impl Range
{
    /// Create a range.
    ///
    /// # Arguments
    ///
    /// * `lo` - Inclusive start offset.
    /// * `hi` - Exclusive end offset.
    ///
    /// # Returns
    ///
    /// The range.
    pub const fn new(lo: u32, hi: u32) -> Self { Range { lo, hi } }

    /// Whether the range covers no text.
    ///
    /// # Returns
    ///
    /// `true` when `lo == hi`.
    pub fn is_blank(&self) -> bool { self.lo == self.hi }
}

/// Move `off` down to the start of the char of `text` that holds it.
///
/// # Arguments
///
/// * `text` - The text.
/// * `off` - A UTF-8 offset, clamped to the text length.
///
/// # Returns
///
/// The snapped offset.
pub fn snap(text: &str, off: u32) -> u32
{
    let mut off = (off as usize).min(text.len());
    while !text.is_char_boundary(off)
    {
        off -= 1;
    }
    off as u32
}

/// Map a phase span over the normalized text to a range of the original text.
///
/// # Arguments
///
/// * `file` - The source file the span points into.
/// * `original` - The original text of the file.
/// * `span` - An absolute span.
///
/// # Returns
///
/// The range of the original text, on char starts; a placeholder span
/// gives a blank range at the start.
pub fn to_original(file: &SourceFile, original: &str, span: &Span) -> Range
{
    ret_if!(span.is_placeholder(), Range::default());
    let lo = snap(original, original_offset(file, &span.lo));
    let hi = snap(original, original_offset(file, &span.hi)).max(lo);
    Range::new(lo, hi)
}

/// Map an absolute position over the normalized text to an original offset.
///
/// # Arguments
///
/// * `file` - The source file.
/// * `pos` - The absolute position.
///
/// # Returns
///
/// The offset in the original text.
fn original_offset(file: &SourceFile, pos: &Pos) -> u32
{
    let relative = file.relative_position(pos);
    file.original_relative_pos(&relative).as_u32()
}

/// Find the range that holds `off`, in ranges sorted on `lo` that do not
/// overlap. An offset at the end of a range counts as inside it, so a caret
/// right after a word still points at that word.
///
/// # Arguments
///
/// * `ranges` - The sorted ranges.
/// * `off` - The offset.
///
/// # Returns
///
/// The index of the range, or `None` when `off` falls between ranges.
pub fn find_range(ranges: &[Range], off: u32) -> Option<usize>
{
    find_in(ranges, off, |range| *range)
}

/// [`find_range`] over items that hold a range.
///
/// # Arguments
///
/// * `items` - Items sorted on the `lo` of their ranges, not overlapping.
/// * `off` - The offset.
/// * `range_of` - The range of an item.
///
/// # Returns
///
/// The index of the item, or `None` when `off` falls between items.
pub fn find_in<T>(
    items: &[T], off: u32, range_of: impl Fn(&T) -> Range,
) -> Option<usize>
{
    let after = items.partition_point(|item| range_of(item).lo <= off);
    let idx = after.checked_sub(1)?;
    (off <= range_of(&items[idx]).hi).then_some(idx)
}

/// Converts between UTF-8 offsets and the UTF-16 offsets that the editor
/// uses. Built once per text so that each lookup is cheap.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OffsetTable
{
    /// For each UTF-8 offset, the UTF-16 offset of the char that holds it;
    /// one extra item for the end of the text.
    units: Vec<u32>,
}

impl OffsetTable
{
    /// Build the table for `text`.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    ///
    /// # Returns
    ///
    /// The table.
    pub fn new(text: &str) -> Self
    {
        let mut units = Vec::with_capacity(text.len() + 1);
        let mut count = 0u32;
        for ch in text.chars()
        {
            units.extend(std::iter::repeat_n(count, ch.len_utf8()));
            count += ch.len_utf16() as u32;
        }
        units.push(count);
        OffsetTable { units }
    }

    /// The UTF-16 offset of a UTF-8 offset.
    ///
    /// # Arguments
    ///
    /// * `off` - The UTF-8 offset; past the end means the end.
    ///
    /// # Returns
    ///
    /// The UTF-16 offset.
    pub fn to_utf16(&self, off: u32) -> u32
    {
        let last = self.units.len() - 1;
        self.units[(off as usize).min(last)]
    }

    /// The UTF-8 offset of a UTF-16 offset.
    ///
    /// # Arguments
    ///
    /// * `idx` - The UTF-16 offset; past the end means the end.
    ///
    /// # Returns
    ///
    /// The UTF-8 offset of a char start.
    pub fn to_utf8(&self, idx: u32) -> u32
    {
        let end = self.units.len() - 1;
        self.units.partition_point(|&units| units < idx).min(end) as u32
    }

    /// The UTF-16 bounds of a range.
    ///
    /// # Arguments
    ///
    /// * `range` - The range in UTF-8 offsets.
    ///
    /// # Returns
    ///
    /// `(lo, hi)` in UTF-16 offsets.
    pub fn range_to_utf16(&self, range: &Range) -> (u32, u32)
    {
        (self.to_utf16(range.lo), self.to_utf16(range.hi))
    }
}
