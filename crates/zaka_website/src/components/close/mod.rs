use leptos::html::a;
use leptos::html::div;
use leptos::html::h2;
use leptos::html::img;
use leptos::html::p;
use leptos::html::section;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;

use crate::assets::EXPLORER_PHOTO;
use crate::components::header::EXPLORER_URL;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/close/mod.sass");

/// Rules of the close.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-close", STYLE_SHEET);

/// The close: the page turned over, ink for ground, with the sentence the
/// hero began and the door into the explorer; the explorer itself shows
/// through, faint and blurred under the ink.
///
/// # Returns
///
/// The view.
pub fn close() -> impl IntoView
{
    section()
        .id("explorer")
        .class(format!("{} rise", ClassName::CLOSE))
        .attr("data-field", "night")
        .child((
            sheet(&SHEET),
            div().class(ClassName::CLOSE_SHOT).attr("aria-hidden", "true").child(
                img()
                    .src(EXPLORER_PHOTO)
                    .alt("")
                    .attr("width", 1600)
                    .attr("height", 1000)
                    .attr("loading", "lazy")
                    .attr("decoding", "async"),
            ),
            h2().class(ClassName::CLOSE_HEAD).child(t!("website.close.head")),
            div().class(ClassName::CLOSE_FOOT).child((
                p().class(ClassName::CLOSE_TEXT).child(t!("website.close.text")),
                a().class(format!("button {}", ClassName::CLOSE_ACTION))
                    .href(EXPLORER_URL)
                    .child(t!("website.header.explorer")),
            )),
        ))
}
