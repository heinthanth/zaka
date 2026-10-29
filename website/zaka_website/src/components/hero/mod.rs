use leptos::html::a;
use leptos::html::div;
use leptos::html::h1;
use leptos::html::img;
use leptos::html::p;
use leptos::html::section;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::tern;
use zakac_span::continues_cluster;

use crate::assets::BLOSSOMS_PHOTO;
use crate::components::header::EXPLORER_URL;
use crate::learn::LEARN_URL;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/hero/mod.sass");

/// Rules of the hero.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-hero", STYLE_SHEET);

/// The line of Zaka the hero writes out: a whole program that prints a
/// greeting.
pub const LINE: &str = "\"မင်္ဂလာပါ\" ဟု ပြသပါ။";

/// What [`LINE`] prints.
pub const PRINTED: &str = "မင်္ဂလာပါ";

/// The page of the backdrop photo on Unsplash. Its license asks for no
/// credit; the hero gives it all the same.
const PHOTO_PAGE: &str = "https://unsplash.com/photos/FIN5CJoBJmI";

/// Who took the backdrop photo.
const PHOTO_AUTHOR: &str = "Antonio Gabriel";

/// The hero: plumeria blossoms against clear blue, mirrored so the flowers
/// stand on the right, and over it the line of Zaka writing itself out, a
/// sentence and the link to the explorer.
/// The writing is a CSS animation, so it needs no script, and with less
/// motion asked for, the whole line stands from the start.
///
/// # Returns
///
/// The view.
pub fn hero() -> impl IntoView
{
    let backdrop =
        div().class(ClassName::HERO_BACKDROP).attr("aria-hidden", "true").child(
            img()
                .src(BLOSSOMS_PHOTO)
                .alt("")
                .attr("width", 1920)
                .attr("height", 1079)
                .attr("fetchpriority", "high"),
        );
    let main = div().class(ClassName::HERO_MAIN).child((
        code_line(),
        p().class(ClassName::HERO_LEDE).child(t!("website.hero.lede")),
        div().class(ClassName::HERO_ACTIONS).child((
            a().class(format!("button {}", ClassName::HERO_ACTION))
                .href(EXPLORER_URL)
                .child(t!("website.header.explorer")),
            a().class(format!(
                "button {} {}",
                ClassName::HERO_ACTION,
                ClassName::HERO_QUIET
            ))
            .href(LEARN_URL)
            .child(t!("website.hero.learn")),
        )),
    ));
    let credit = p().class(ClassName::HERO_CREDIT).child(
        a().href(PHOTO_PAGE).child(t!("website.hero.credit", author = PHOTO_AUTHOR)),
    );
    section().class(ClassName::HERO).attr("data-field", "hero").child((
        sheet(&SHEET),
        backdrop,
        main,
        credit,
    ))
}

/// The line of Zaka, one span per cluster, each set to appear in turn, and
/// the caret after them.
///
/// # Returns
///
/// The view.
fn code_line() -> impl IntoView
{
    let mut quoted = false;
    let keys: Vec<_> = syllables(LINE)
        .into_iter()
        .enumerate()
        .map(|(idx, piece)| {
            let mark = piece == "\"";
            let string = quoted || mark;
            quoted ^= mark;
            span()
                .class(tern!(string, ClassName::HERO_STRING, ClassName::HERO_KEY))
                .attr("style", format!("--i: {idx}"))
                .child(piece)
        })
        .collect();
    let caret = span().class(ClassName::HERO_CARET).attr("aria-hidden", "true");
    h1().class(ClassName::HERO_LINE).lang("my").child((keys, caret))
}

/// Split a line into the pieces it is written in: a Burmese cluster stands
/// whole, with its medials, vowels, asat and stacked consonants, so no half
/// drawn letter is ever shown.
///
/// # Arguments
///
/// * `line` - The text.
///
/// # Returns
///
/// The pieces, in order.
pub fn syllables(line: &str) -> Vec<String>
{
    line.chars().fold(Vec::<String>::new(), |mut pieces, ch| {
        let prev = pieces.last().and_then(|p| p.chars().last());
        let joins = prev.is_some_and(|prev| continues_cluster(prev, ch));
        match pieces.last_mut()
        {
            Some(last) if joins => last.push(ch),
            _ => pieces.push(ch.to_string()),
        }
        pieces
    })
}
