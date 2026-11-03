include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// The Burmese ranges: the Myanmar block and its two extensions, the joiners
/// and the dotted circle.
const BURMESE_RANGE: &str =
    "U+1000-109F, U+A9E0-A9FF, U+AA60-AA7F, U+200C-200D, U+25CC";

/// The size of the Burmese in the handwritten notes against their Latin, in
/// percent, so the two read as one size.
const HAND_SCALE: u16 = 78;

/// The faces of the website, fixed: Myanmar Sabae for Burmese text, Sauce
/// Code Pro (HTMM's Custom) for code, and Myanmar Sabae limited to the
/// Burmese ranges, which comes first in the code stack, draws the Burmese and
/// leaves the rest to Sauce Code Pro. A smaller cut of that last one draws
/// the Burmese in the handwritten notes, whose hand face is small for its
/// size. Rules in Rust, not in a sheet, as the
/// hashed addresses are known to the Rust code alone.
///
/// # Returns
///
/// The `@font-face` rules.
pub fn font_faces() -> String
{
    [
        face("Myanmar Sabae", SABAE_FONT, None, 100),
        face("Sauce Code Pro (HTMM's Custom)", SAUCE_FONT, None, 100),
        face("Myanmar Sabae (Burmese)", SABAE_FONT, Some(BURMESE_RANGE), 100),
        face("Myanmar Sabae (Hand)", SABAE_FONT, Some(BURMESE_RANGE), HAND_SCALE),
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
/// * `scale` - Its size against the size of the text, in percent.
///
/// # Returns
///
/// The rule.
fn face(name: &str, src: &str, range: Option<&str>, scale: u16) -> String
{
    let range = range.map(|r| format!("unicode-range:{r};")).unwrap_or_default();
    format!(
        "@font-face{{font-family:\"{name}\";src:url(\"{src}\") \
         format(\"truetype\");font-weight:400;font-display:swap;size-adjust:{scale}%;\
         {range}}}"
    )
}
