use unicode_width::UnicodeWidthChar;
use zakac_rust_sugars::ret_if;

/// Number of terminal cells a tab takes in a source excerpt.
pub const TAB_CELLS: usize = 4;

/// Per-character cell counts that replace the value from `unicode-width`.
/// Some terminals give Burmese signs a cell count that differs from what
/// `unicode-width` predicts.
const CELL_OVERRIDES: &[(char, usize)] = &[];

/// Number of terminal cells `ch` takes.
///
/// # Arguments
///
/// * `ch` - The character to measure.
///
/// # Returns
///
/// The cell count: [`TAB_CELLS`] for a tab, the override from
/// [`CELL_OVERRIDES`] when one exists, else the value from `unicode-width`
/// (zero for control characters).
pub fn char_cells(ch: char) -> usize
{
    ret_if!(ch == '\t', TAB_CELLS);
    let manual = CELL_OVERRIDES.iter().find(|(c, _)| *c == ch).map(|(_, n)| *n);
    manual.unwrap_or_else(|| ch.width().unwrap_or(0))
}

/// Number of terminal cells `text` takes on one line.
///
/// # Arguments
///
/// * `text` - The text to measure; it should hold no line break.
///
/// # Returns
///
/// The sum of [`char_cells`] over the characters of `text`.
pub fn str_cells(text: &str) -> usize { text.chars().map(char_cells).sum() }
