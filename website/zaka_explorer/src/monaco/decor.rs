use bon::Builder;
use zakac_diagnostics::Level;

use crate::compile::Compiled;
use crate::compile::DiagRow;
use crate::compile::spans::Range;

/// CSS class of comments in the editor.
const COMMENT_CLASS: &str = "tok-comment";

/// Monaco marker level of an error.
const MARK_ERROR: u32 = 8;
/// Monaco marker level of a warning.
const MARK_WARNING: u32 = 4;
/// Monaco marker level of a note.
const MARK_NOTE: u32 = 2;

/// The color of one token or comment in the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct TokenMark
{
    /// UTF-16 start.
    pub lo: u32,
    /// UTF-16 end.
    pub hi: u32,
    /// The CSS class that colors it.
    pub class: &'static str,
}

/// The squiggle of one diagnostic in the editor.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct DiagMark
{
    /// UTF-16 start.
    pub lo: u32,
    /// UTF-16 end, past `lo`.
    pub hi: u32,
    /// The Monaco marker level.
    pub level: u32,
    /// The diagnostic code.
    pub code: String,
    /// The message, with the hint on a new line.
    pub message: String,
}

/// Editor colors of tokens and comments, sorted on `lo`.
///
/// # Arguments
///
/// * `compiled` - The compile output.
///
/// # Returns
///
/// The marks.
pub fn decorations(compiled: &Compiled) -> Vec<TokenMark>
{
    let tokens = compiled.tokens.iter().map(|t| (t.range, t.class));
    let comments = compiled.comments.iter().map(|c| (c.range, COMMENT_CLASS));
    let mut marks: Vec<TokenMark> = tokens
        .chain(comments)
        .filter(|(range, _)| !range.is_blank())
        .map(|(range, class)| token_mark(compiled, &range, class))
        .collect();
    marks.sort_by_key(|mark| mark.lo);
    marks
}

/// One token color in UTF-16 offsets.
///
/// # Arguments
///
/// * `compiled` - The compile output.
/// * `range` - The range in UTF-8 offsets.
/// * `class` - The CSS class.
///
/// # Returns
///
/// The mark.
fn token_mark(compiled: &Compiled, range: &Range, class: &'static str) -> TokenMark
{
    let (lo, hi) = compiled.table.range_to_utf16(range);
    TokenMark::builder().lo(lo).hi(hi).class(class).build()
}

/// Editor squiggles of the diagnostics that point at source text, each with
/// a note after it for each of its labels.
///
/// # Arguments
///
/// * `compiled` - The compile output.
///
/// # Returns
///
/// The marks.
pub fn markers(compiled: &Compiled) -> Vec<DiagMark>
{
    compiled.diagnostics.iter().flat_map(|d| diag_marks(compiled, d)).collect()
}

/// The squiggle of one diagnostic, when it points at source text, and a
/// note for each of its labels.
///
/// # Arguments
///
/// * `compiled` - The compile output.
/// * `diag` - The diagnostic.
///
/// # Returns
///
/// The marks.
fn diag_marks(compiled: &Compiled, diag: &DiagRow) -> Vec<DiagMark>
{
    let help = diag.help.as_ref().map(|h| format!("\nhelp: {h}")).unwrap_or_default();
    let message = format!("{}{help}", diag.message);
    let level = marker_level(diag.level);
    let own = diag.range.map(|r| mark(compiled, &r, level, &diag.code, message));
    let note = |(r, text): &(Range, String)| {
        mark(compiled, r, MARK_NOTE, &diag.code, text.clone())
    };
    own.into_iter().chain(diag.labels.iter().map(note)).collect()
}

/// One squiggle in UTF-16 offsets.
///
/// # Arguments
///
/// * `compiled` - The compile output.
/// * `range` - Its range in UTF-8 offsets.
/// * `level` - The Monaco marker level.
/// * `code` - The diagnostic code.
/// * `message` - The text.
///
/// # Returns
///
/// The mark, at least one unit wide.
fn mark(
    compiled: &Compiled, range: &Range, level: u32, code: &str, message: String,
) -> DiagMark
{
    let (lo, hi) = compiled.table.range_to_utf16(range);
    DiagMark::builder()
        .lo(lo)
        .hi(hi.max(lo + 1))
        .level(level)
        .code(code.to_string())
        .message(message)
        .build()
}

/// Monaco marker level of a diagnostic level.
///
/// # Arguments
///
/// * `level` - The level.
///
/// # Returns
///
/// The Monaco number.
fn marker_level(level: Level) -> u32
{
    match level
    {
        Level::Error => MARK_ERROR,
        Level::Warning => MARK_WARNING,
        Level::Note => MARK_NOTE,
    }
}
