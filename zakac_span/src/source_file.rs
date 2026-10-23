use bon::Builder;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::filename::FileName;
use crate::hash::SourceFileHash;
use crate::hash::SourceFileHashAlgorithm;
use crate::normalizer::NormalizedPos;
use crate::normalizer::NormalizedSource;
use crate::pos::CharPos;
use crate::pos::Pos;
use crate::pos::RelativePos;

/// A single source file in the [`SourceMap`](crate::SourceMap).
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct SourceFile
{
    /// File name.
    pub name: FileName,
    /// Normalized source text.
    pub src: String,
    /// Hash of the original (pre-normalize) UTF-8 content.
    pub src_hash: SourceFileHash,
    /// Absolute start position in the source map.
    pub start_pos: Pos,
    /// Length of u8s in the original source.
    pub unnormalized_source_len: u32,
    /// u8 offsets of line starts (relative to file start).
    pub lines: Vec<RelativePos>,
    /// Normalization diff map.
    pub normalized_pos: Vec<NormalizedPos>,
}

/// Build a line table from the source text.
///
/// # Arguments
///
/// * `src` - The source text to build the line table from.
///
/// # Returns
///
/// The line table.
fn build_line_table(src: &str) -> Vec<RelativePos>
{
    let mut lines = vec![RelativePos::ZERO];
    for (idx, ch) in src.char_indices()
    {
        do_if!(ch == '\n', lines.push(RelativePos::new((idx + 1) as u32)));
    }
    lines
}

/// Convert a position to a character position.
///
/// # Arguments
///
/// * `slice` - The slice to convert the position to a character position for.
/// * `offset` - The u8 offset to convert.
///
/// # Returns
///
/// The character position.
fn pos_to_char_rpos(slice: &str, offset: usize) -> CharPos
{
    let chars = slice.char_indices().take_while(|(idx, _)| *idx < offset).count();
    CharPos::new(chars as u32)
}

impl SourceFile
{
    /// Create a source file from original text.
    ///
    /// # Arguments
    ///
    /// * `name` - File name.
    /// * `original_src` - Raw source text.
    /// * `start_pos` - Absolute start position in the source map.
    /// * `hash_kind` - Hash algorithm.
    ///
    /// # Returns
    ///
    /// A new source file with normalized text and line table.
    pub fn new(
        name: FileName, original_src: String, start_pos: Pos,
        hash_kind: SourceFileHashAlgorithm,
    ) -> Self
    {
        let src_hash = SourceFileHash::compute(hash_kind, original_src.as_bytes());
        let normalized = NormalizedSource::from(original_src.as_str());
        let lines = build_line_table(&normalized.src);

        SourceFile::builder()
            .name(name)
            .src(normalized.src)
            .src_hash(src_hash)
            .start_pos(start_pos)
            .unnormalized_source_len(normalized.unnormalized_source_len)
            .lines(lines)
            .normalized_pos(normalized.normalized_pos)
            .build()
    }

    /// Exclusive end position in the source map.
    ///
    /// # Returns
    ///
    /// The exclusive end position.
    pub fn end_pos(&self) -> Pos { self.start_pos.advanced(self.src.len()) }

    /// Convert an absolute u8 position to a position relative to this file.
    ///
    /// # Arguments
    ///
    /// * `pos` - The absolute position to convert.
    ///
    /// # Returns
    ///
    /// The relative position.
    pub fn relative_position(&self, pos: &Pos) -> RelativePos
    {
        RelativePos::new(pos.as_u32() - self.start_pos.as_u32())
    }

    /// Map a normalized relative position to the original file offset.
    ///
    /// # Arguments
    ///
    /// * `pos` - The normalized relative position to convert.
    ///
    /// # Returns
    ///
    /// The original relative position.
    pub fn original_relative_pos(&self, pos: &RelativePos) -> RelativePos
    {
        let mut offset = 0i32;
        for ent in &self.normalized_pos
        {
            do_if!(ent.pos > pos.as_u32(), break);
            offset = ent.diff;
        }
        RelativePos::new(pos.as_u32().saturating_add_signed(offset))
    }

    /// Find the line index (0-based) containing `pos`.
    ///
    /// # Arguments
    ///
    /// * `pos` - The relative position to find the line index for.
    ///
    /// # Returns
    ///
    /// The line index (0-based).
    pub fn lookup_line(&self, pos: &RelativePos) -> Option<usize>
    {
        ret_if!(self.lines.is_empty() || pos < &self.lines[0], None);
        match self.lines.binary_search_by(|line| line.cmp(pos))
        {
            Ok(idx) => Some(idx),
            Err(0) => None,
            Err(idx) => Some(idx - 1),
        }
    }

    /// Look up (1-based line, 0-based `CharPos` column) for `pos`.
    ///
    /// # Arguments
    ///
    /// * `pos` - The relative position to find the file position for.
    ///
    /// # Returns
    ///
    /// The file position (1-based line, 0-based character position).
    pub fn lookup_file_pos(&self, pos: &RelativePos) -> (usize, CharPos)
    {
        let line_idx = self.lookup_line(pos).unwrap_or(0);
        let line_start = self.lines[line_idx].as_u32() as usize;
        let line_end = self.line_end(line_idx);

        let src = &self.src[line_start..line_end];
        let col = pos_to_char_rpos(src, pos.as_u32() as usize - line_start);
        (line_idx + 1, col)
    }

    /// Return the end position of the line `line_idx` (0-based).
    ///
    /// # Arguments
    ///
    /// * `line_idx` - The line index to get the end position for (0-based).
    ///
    /// # Returns
    ///
    /// The end position of the line.
    fn line_end(&self, line_idx: usize) -> usize
    {
        ret_if!(line_idx + 1 >= self.lines.len(), self.src.len());
        self.lines[line_idx + 1].as_u32() as usize
    }

    /// Return the text of line `line_number` (0-based).
    ///
    /// # Arguments
    ///
    /// * `line_number` - The line number to get (0-based).
    ///
    /// # Returns
    ///
    /// The line text.
    pub fn get_line(&self, line_number: usize) -> Option<&str>
    {
        ret_if!(line_number >= self.lines.len(), None);
        let start = self.lines[line_number].as_u32() as usize;
        let end = self.line_end(line_number);
        self.src.get(start..end)
    }

    /// Extract `text[start..end]` from normalized source.
    ///
    /// # Arguments
    ///
    /// * `start` - The start position of the snippet.
    /// * `end` - The end position of the snippet.
    ///
    /// # Returns
    ///
    /// The snippet text.
    pub fn snippet(&self, start: &RelativePos, end: &RelativePos) -> Option<&str>
    {
        self.src.get(start.as_u32() as usize..end.as_u32() as usize)
    }
}
