use leptos::html::a;
use leptos::html::div;
use leptos::html::figcaption;
use leptos::html::figure;
use leptos::html::h2;
use leptos::html::img;
use leptos::html::p;
use leptos::html::section;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;

use crate::assets::FRANGIPANI_PHOTO;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/name/mod.sass");

/// Rules of the name section.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-name", STYLE_SHEET);

/// Title of the photograph at its source.
const PHOTO_TITLE: &str = "“Frangipani flowers”";

/// Who took the photograph.
const PHOTO_AUTHOR: &str = "Renesis";

/// License of the photograph.
const PHOTO_LICENSE: &str = "CC BY-SA 3.0";

/// Page of the photograph at its source.
const PHOTO_PAGE: &str =
    "https://commons.wikimedia.org/wiki/File:Frangipani_flowers.jpg";

/// The name: the heading, the name taken apart into the word for speech and
/// the story of the name, beside one photograph of the flower.
///
/// # Returns
///
/// The view.
pub fn name() -> impl IntoView
{
    let caption = t!("website.name.photo").to_string();
    let photo = figure().class(ClassName::NAME_PHOTO).child((
        div().class(ClassName::NAME_FRAME).child(
            img()
                .src(FRANGIPANI_PHOTO)
                .alt(caption.clone())
                .attr("width", 1200)
                .attr("height", 804)
                .attr("loading", "lazy")
                .attr("decoding", "async"),
        ),
        figcaption().child((
            span().child(caption),
            " ",
            span().class(ClassName::NAME_CREDIT).child((
                a().href(PHOTO_PAGE).child(PHOTO_TITLE),
                format!(", {PHOTO_AUTHOR}, {PHOTO_LICENSE}."),
            )),
        )),
    ));
    section().id("name").class(ClassName::NAME).attr("data-field", "paper").child((
        sheet(&SHEET),
        h2().class(ClassName::NAME_HEAD).child(t!("website.name.head")),
        photo,
        equation(),
        p().class(ClassName::NAME_TEXT).child(t!("website.name.text")),
    ))
}

/// The name taken apart: the plumeria, less the mark, gives speech.
///
/// # Returns
///
/// The view.
fn equation() -> impl IntoView
{
    let term = |word: AnyView, gloss: &'static str| {
        span().class(ClassName::NAME_TERM).child((
            span().class(ClassName::NAME_WORD).lang("my").child(word),
            span().class(ClassName::NAME_GLOSS).child(t!(gloss.to_string())),
        ))
    };
    let flower = ("စ", span().class(ClassName::NAME_MARK).child("ံ"), "ကား").into_any();
    // Each sign keeps to the term after it, so a line never ends on a sign.
    let then = |sign: &'static str, word: AnyView, gloss: &'static str| {
        span().class(ClassName::NAME_PAIR).child((
            span().class(ClassName::NAME_OP).attr("aria-hidden", "true").child(sign),
            term(word, gloss),
        ))
    };
    let mark = span().class(ClassName::NAME_MARK).child("◌ံ").into_any();
    div().class(ClassName::NAME_EQUATION).child((
        term(flower, "website.name.plumeria"),
        then("−", mark, "website.name.mark"),
        then("=", "စကား".into_any(), "website.name.speech"),
    ))
}
