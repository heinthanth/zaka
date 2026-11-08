use gen_lsp_types::Position;
use zakac_span::text_range::OffsetTable;
use zakac_span::text_range::Range;

/// Converts between UTF-8 offsets of a text and the line and UTF-16 column
/// positions that LSP uses. Built once per text so that each lookup is cheap.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LineIndex
{
    /// The UTF-8 offset of the start of each line.
    starts: Vec<u32>,
    /// UTF-8 to UTF-16 offsets over the whole text.
    table: OffsetTable,
}

impl LineIndex
{
    /// Build the index of `text`.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    ///
    /// # Returns
    ///
    /// The index.
    pub fn new(text: &str) -> Self
    {
        let breaks = text.match_indices('\n').map(|(at, _)| at as u32 + 1);
        let starts = std::iter::once(0).chain(breaks).collect();
        LineIndex { starts, table: OffsetTable::new(text) }
    }

    /// The position of a UTF-8 offset.
    ///
    /// # Arguments
    ///
    /// * `off` - The UTF-8 offset; past the end means the end.
    ///
    /// # Returns
    ///
    /// The zero-based line and UTF-16 column.
    pub fn position(&self, off: u32) -> Position
    {
        let line = self.starts.partition_point(|&start| start <= off) - 1;
        let column = self.table.to_utf16(off) - self.table.to_utf16(self.starts[line]);
        Position { line: line as u32, character: column }
    }

    /// The UTF-8 offset of a position.
    ///
    /// # Arguments
    ///
    /// * `pos` - The zero-based line and UTF-16 column; past the end of a line
    ///   or of the text means its end.
    ///
    /// # Returns
    ///
    /// The UTF-8 offset of a char start.
    pub fn offset(&self, pos: Position) -> u32
    {
        let last = self.starts.len() - 1;
        let line = (pos.line as usize).min(last);
        let start = self.table.to_utf16(self.starts[line]);
        let line_end = self.starts.get(line + 1).map_or(u32::MAX, |&next| next - 1);
        self.table.to_utf8(start + pos.character).min(line_end)
    }

    /// The LSP range of a range of UTF-8 offsets.
    ///
    /// # Arguments
    ///
    /// * `range` - The range.
    ///
    /// # Returns
    ///
    /// The LSP range.
    pub fn range(&self, range: Range) -> gen_lsp_types::Range
    {
        gen_lsp_types::Range {
            start: self.position(range.lo),
            end: self.position(range.hi),
        }
    }

    /// The UTF-8 offset where each line starts.
    ///
    /// # Returns
    ///
    /// The starts, the first one 0.
    pub fn starts(&self) -> &[u32] { &self.starts }
}
