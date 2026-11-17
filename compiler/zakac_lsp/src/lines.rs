use gen_lsp_types::Position;
use gen_lsp_types::TextDocumentContentChangeEvent;
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

/// Make one change of a file to its text: a whole new text, or a range of
/// it replaced, in UTF-16 positions of the text before the change.
///
/// # Arguments
///
/// * `doc` - The text, changed in place.
/// * `change` - The change.
pub fn change_text(doc: &mut String, change: TextDocumentContentChangeEvent)
{
    let part = match change
    {
        TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(c) =>
        {
            *doc = c.text;
            return;
        }
        TextDocumentContentChangeEvent::TextDocumentContentChangePartial(c) => c,
    };
    let lines = LineIndex::new(doc);
    let lo = lines.offset(part.range.start) as usize;
    let hi = (lines.offset(part.range.end) as usize).max(lo);
    doc.replace_range(lo..hi, &part.text);
}

#[cfg(test)]
mod tests
{
    use gen_lsp_types::Range as LspRange;
    use gen_lsp_types::TextDocumentContentChangePartial;

    use super::*;

    /// A change of the range from `(l0, c0)` to `(l1, c1)` to `text`.
    fn part(
        l0: u32, c0: u32, l1: u32, c1: u32, text: &str,
    ) -> TextDocumentContentChangeEvent
    {
        let at = |line, character| Position { line, character };
        let range = LspRange { start: at(l0, c0), end: at(l1, c1) };
        TextDocumentContentChangePartial::new(range, None, text.into()).into()
    }

    #[test]
    fn changes_land_in_order_in_utf16_columns()
    {
        // `က` is one UTF-16 unit and three UTF-8 octets.
        let mut doc = "ကခ\nဂ\n".to_string();
        change_text(&mut doc, part(0, 1, 0, 2, "x"));
        assert_eq!(doc, "ကx\nဂ\n");
        // The second change reads the text the first one left.
        change_text(&mut doc, part(1, 1, 1, 1, "z"));
        change_text(&mut doc, part(0, 0, 1, 0, ""));
        assert_eq!(doc, "ဂz\n");
    }

    #[test]
    fn an_insert_at_the_end_of_a_line_keeps_the_break()
    {
        let mut doc = "ab\r\ncd".to_string();
        change_text(&mut doc, part(0, 2, 0, 2, "!"));
        assert_eq!(doc, "ab!\r\ncd");
        change_text(&mut doc, part(1, 2, 1, 2, "?"));
        assert_eq!(doc, "ab!\r\ncd?");
    }
}
