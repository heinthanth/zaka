use strum::EnumString;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

/// Number of terminal cells a tab takes in a source excerpt.
pub const TAB_CELLS: usize = 4;

/// Whether the rows under a span are drawn, and how their cells are counted.
///
/// Terminals do not agree on the cells a Burmese sign takes, so no count fits
/// all of them. One choice holds for each row of a rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Carets
{
    /// No `^^^` or `---` rows; the span is marked inside the source line.
    #[default]
    Off,
    /// Rows with one cell per code point, as `unicode-width` counts them.
    Codepoint,
    /// Rows with cells per grapheme cluster, as terminals with mode 2027 count
    /// them: a cluster whose later signs take cells is two cells wide.
    Cluster,
    /// Rows under the text where a terminal that flows Burmese text draws it,
    /// such as saturngod's Ghostty fork: it draws the text at the natural
    /// width of its font and pushes the rest of the row right. Where the
    /// terminal does not flow Burmese text, or the row wraps, the rows count
    /// one cell per code point.
    Flow,
    /// Rows for a front end that lays text out itself, such as a browser:
    /// each row repeats the text before the span as [`Paint::Hidden`], then
    /// the span text as [`Paint::Under`], which the front end draws as a row
    /// of marks over the hidden text. The rows then take the widths the text
    /// above takes, in whatever font. `ZAKAC_CARETS` can't pick it.
    ///
    /// [`Paint::Hidden`]: crate::render::Paint::Hidden
    /// [`Paint::Under`]: crate::render::Paint::Under
    #[strum(disabled)]
    Mirror,
}

/// Read the caret setting from its text form.
///
/// # Arguments
///
/// * `value` - The text, such as the value of `ZAKAC_CARETS`.
///
/// # Returns
///
/// The setting it names, or [`Carets::Off`] when it is missing or unknown.
pub fn carets_setting(value: Option<&str>) -> Carets
{
    value.and_then(|text| text.parse().ok()).unwrap_or_default()
}

/// Number of terminal cells `ch` takes.
///
/// # Arguments
///
/// * `ch` - The character to measure.
///
/// # Returns
///
/// The cell count: [`TAB_CELLS`] for a tab, else the value from
/// `unicode-width` (zero for control characters).
pub fn char_cells(ch: char) -> usize
{
    ret_if!(ch == '\t', TAB_CELLS);
    ch.width().unwrap_or(0)
}

/// Number of terminal cells one grapheme cluster takes under mode 2027: the
/// cells of its first character, raised to two when a later character of the
/// cluster takes cells of its own.
///
/// # Arguments
///
/// * `cluster` - The grapheme cluster.
///
/// # Returns
///
/// The cell count.
fn cluster_cells(cluster: &str) -> usize
{
    let mut chars = cluster.chars();
    let first = chars.next().map_or(0, char_cells);
    let widens = chars.any(|ch| char_cells(ch) > 0);
    tern!(widens, first.max(2), first)
}

/// Number of terminal cells `text` takes on one line.
///
/// # Arguments
///
/// * `text` - The text to measure; it should hold no line break.
/// * `carets` - How to count: per grapheme cluster for [`Carets::Cluster`],
///   else per code point.
///
/// # Returns
///
/// The cell count.
pub fn str_cells(text: &str, carets: Carets) -> usize
{
    ret_if!(carets == Carets::Cluster, text.graphemes(true).map(cluster_cells).sum());
    text.chars().map(char_cells).sum()
}
