include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// The Burmese ranges: the Myanmar block and its two extensions, the joiners
/// and the dotted circle.
const BURMESE_RANGE: &str =
    "U+1000-109F, U+A9E0-A9FF, U+AA60-AA7F, U+200C-200D, U+25CC";

/// The faces of the website, fixed: Myanmar Sabae for Burmese text, Sauce
/// Code Pro (HTMM's Custom) for code, and Myanmar Sabae limited to the
/// Burmese ranges, which comes first in the code stack, draws the Burmese and
/// leaves the rest to Sauce Code Pro. Rules in Rust, not in a sheet, as the
/// hashed addresses are known to the Rust code alone.
///
/// # Returns
///
/// The `@font-face` rules.
pub fn font_faces() -> String
{
    [
        face("Myanmar Sabae", SABAE_FONT, None),
        face("Sauce Code Pro (HTMM's Custom)", SAUCE_FONT, None),
        face("Myanmar Sabae (Burmese)", SABAE_FONT, Some(BURMESE_RANGE)),
    ]
    .concat()
}

/// One `@font-face` rule of a TrueType font at weight 400.
///
/// # Arguments
///
/// * `name` - The font name.
/// * `src` - The address of the file.
/// * `range` - The characters it draws, or all of them.
///
/// # Returns
///
/// The rule.
fn face(name: &str, src: &str, range: Option<&str>) -> String
{
    let range = range.map(|r| format!("unicode-range:{r};")).unwrap_or_default();
    format!(
        "@font-face{{font-family:\"{name}\";src:url(\"{src}\") \
         format(\"truetype\");font-weight:400;font-display:swap;{range}}}"
    )
}
