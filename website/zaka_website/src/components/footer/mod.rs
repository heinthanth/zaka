use leptos::html::a;
use leptos::html::div;
use leptos::html::footer as footer_el;
use leptos::html::p;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_consts::ZAKA_AUTHOR;
use zakac_consts::ZAKA_FIRST_YEAR;
use zakac_consts::ZAKA_LICENSE;
use zakac_i18n::t;

use crate::components::header::EXPLORER_URL;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/footer/mod.sass");

/// Rules of the footer.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-footer", STYLE_SHEET);

/// When the page was built, for the rights line.
const BUILT_IN: &str = "2026";

/// The footer: a gold field with notes at the top and a row of links and
/// rights at the foot.
///
/// # Returns
///
/// The view.
pub fn footer() -> impl IntoView
{
    let notes = div().class(ClassName::FOOTER_NOTES).child((
        p().child(t!("website.footer.what")),
        p().child(t!("website.footer.license", license = ZAKA_LICENSE)),
    ));
    let top = div().class(ClassName::FOOTER_TOP).child((
        notes,
        a().class(ClassName::FOOTER_BACK).href("#top").child(t!("website.footer.top")),
    ));
    let links = div()
        .class(ClassName::FOOTER_LINKS)
        .child(a().href(EXPLORER_URL).child(t!("website.footer.explorer")));
    let rights = p().class(ClassName::FOOTER_RIGHTS).child(t!(
        "website.footer.rights",
        author = ZAKA_AUTHOR,
        first = ZAKA_FIRST_YEAR,
        last = BUILT_IN
    ));
    footer_el().class(ClassName::FOOTER).attr("data-field", "gold").child((
        sheet(&SHEET),
        top,
        div().class(ClassName::FOOTER_META).child((links, rights)),
    ))
}
