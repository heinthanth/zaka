//! What the compiler reads from the settings of Ghostty.

use bon::Builder;

/// Name of a setting that a Ghostty build lists when it draws Burmese text by
/// orthographic unit at the natural width of its font, and flows the row to
/// make room; saturngod's fork adds it.
pub const FLOW_SETTING: &str = "myanmar-prompt-width";

/// The Burmese fonts such a build looks for, in its order.
pub const BURMESE_FONTS: [&str; 4] =
    ["Noto Sans Myanmar", "Myanmar MN", "Myanmar Text", "Padauk"];

/// Font size, in points, when the settings name none.
pub const DEFAULT_POINTS: f64 = 13.0;

/// Width of a cell of the main font, in ems, when the font can't be read.
pub const DEFAULT_CELL_EM: f64 = 0.6;

/// The settings of the terminal that the placement needs.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct Settings
{
    /// Whether the build flows Burmese text: it lists [`FLOW_SETTING`].
    pub flows: bool,
    /// Whether the build gives a grapheme cluster its cells (mode 2027), as
    /// `grapheme-width-method = unicode`, the default, asks; `legacy` gives
    /// each code point its own.
    pub clusters: bool,
    /// The font size, in points.
    pub points: f64,
    /// The name of the main font, when one is set.
    pub font: Option<String>,
}

/// Read the settings from the output of `ghostty +show-config`, one
/// `name = value` per line.
///
/// # Arguments
///
/// * `text` - The output.
///
/// # Returns
///
/// The settings; a missing font size reads as [`DEFAULT_POINTS`].
pub fn read_settings(text: &str) -> Settings
{
    let pairs: Vec<(&str, &str)> = text
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(name, value)| (name.trim(), value.trim()))
        .collect();
    let value = |wanted: &str| {
        pairs
            .iter()
            .find(|(name, value)| *name == wanted && !value.is_empty())
            .map(|(_, value)| *value)
    };
    Settings::builder()
        .flows(pairs.iter().any(|(name, _)| *name == FLOW_SETTING))
        .clusters(value("grapheme-width-method") != Some("legacy"))
        .points(
            value("font-size").and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_POINTS),
        )
        .maybe_font(value("font-family").map(|v| v.trim_matches('"').to_string()))
        .build()
}

/// The pixel size of the font, from the cell width the terminal reports.
///
/// The terminal makes a cell `round(cell_em * points * dpi / 72)` pixels
/// wide, and draws Burmese text at `points * dpi / 72` pixels. The dpi is
/// 72 or 96 times the scale of the screen; the scale that best fits the cell
/// width wins.
///
/// # Arguments
///
/// * `cell` - The cell width, in pixels.
/// * `points` - The font size, in points.
/// * `cell_em` - The cell width of the main font, in ems.
///
/// # Returns
///
/// The font size, in pixels.
pub fn font_px(cell: f64, points: f64, cell_em: f64) -> f64
{
    let scales = [1.0, 1.25, 1.5, 1.75, 2.0, 2.25, 2.5, 3.0];
    let sizes = [72.0, 96.0]
        .into_iter()
        .flat_map(|dpi| scales.map(|scale| points * dpi * scale / 72.0));
    let miss = |px: f64| {
        ((cell_em * px).round() - cell).abs() * 1000.0 + (cell_em * px - cell).abs()
    };
    sizes.min_by(|a, b| miss(*a).total_cmp(&miss(*b))).unwrap_or(cell / cell_em)
}
