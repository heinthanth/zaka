use leptos::html::code;
use leptos::html::div;
use leptos::html::h2;
use leptos::html::h3;
use leptos::html::p;
use leptos::html::section;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::paint::painted;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/bands/mod.sass");

/// Rules of the bands.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-bands", STYLE_SHEET);

/// One band: a point about the language, a line of Zaka that shows it, and
/// the same line in Rust.
#[derive(Clone, Copy)]
struct Band
{
    /// Locale id of the point; the gloss of the line is under `<id>_more`.
    id: &'static str,
    /// The token color the band is marked with.
    color: &'static str,
    /// The line of Zaka.
    zaka: &'static str,
    /// The same line in Rust.
    rust: &'static str,
}

/// The bands, in order.
const BANDS: [Band; 5] = [
    Band {
        id: "website.bands.verb",
        color: "--tok-verb",
        zaka: "အသက် ကို ပြသပါ။",
        rust: "println!(\"{age}\");",
    },
    Band {
        id: "website.bands.particle",
        color: "--tok-particle",
        zaka: "အသက် သည် ၁၈ ထက် ကြီးသည်",
        rust: "age > 18",
    },
    Band {
        id: "website.bands.digit",
        color: "--tok-number",
        zaka: "၆ * ၇ ကို ပြသပါ။",
        rust: "println!(\"{}\", 6 * 7);",
    },
    Band {
        id: "website.bands.stop",
        color: "--tok-op",
        zaka: "\"က\" ဟု ပြသပါ။ \"ခ\" ဟု ပြသပါ။",
        rust: "println!(\"က\"); println!(\"ခ\");",
    },
    Band {
        id: "website.bands.space",
        color: "--tok-kind",
        zaka: "ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃၀ ဖြစ်ပါစေ။",
        rust: "let mut age: isize = 30;",
    },
];

/// The bands: how a line of Zaka reads as Burmese, one point to a band, each
/// shown with a line of Zaka, what it means and the same line in Rust.
///
/// # Returns
///
/// The view.
pub fn bands() -> impl IntoView
{
    section().id("language").class(ClassName::BANDS).attr("data-field", "canvas").child(
        (
            sheet(&SHEET),
            h2().class(ClassName::BANDS_HEAD).child(t!("website.bands.head")),
            div()
                .class(format!("{} rise", ClassName::BANDS_LIST))
                .child(BANDS.map(band)),
        ),
    )
}

/// One band.
///
/// # Arguments
///
/// * `band` - The band.
///
/// # Returns
///
/// The view.
fn band(band: Band) -> impl IntoView
{
    let proof = div().class(ClassName::BAND_PROOF).child((
        code()
            .class(format!("{} painted", ClassName::BAND_ZAKA))
            .lang("my")
            .child(painted(band.zaka)),
        p().class(ClassName::BAND_GLOSS).child(t!(format!("{}_more", band.id))),
        p().class(ClassName::BAND_RUST).child((
            span().class(ClassName::BAND_RUST_NAME).child(t!("website.bands.rust")),
            code().child(band.rust),
        )),
    ));
    div()
        .class(ClassName::BAND)
        .attr("style", format!("--band: var({})", band.color))
        .child((
            h3().class(ClassName::BAND_POINT).child(t!(band.id.to_string())),
            proof,
        ))
}
