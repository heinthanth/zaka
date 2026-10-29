use leptos::html::a;
use leptos::html::h1;
use leptos::html::main;
use leptos::html::p;
use leptos::html::section;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zaka_web_ui_common::site::HOME_URL;
use zakac_i18n::t;

use crate::components::footer::footer;
use crate::components::header::SiteHeader;
use crate::components::header::SiteHeaderProps;
use crate::components::header::header_sheet;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/missing/mod.sass");

/// Rules of the page for a missing address.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-missing", STYLE_SHEET);

/// The page for an address with no page: the header, one gold field with the
/// code of the error, the headline, a sentence and the link home, then the
/// footer. The rest of the site is left to the header.
///
/// # Returns
///
/// The view.
pub fn missing() -> impl IntoView
{
    let home = a()
        .class(format!("button {}", ClassName::MISSING_ACTION))
        .href(HOME_URL)
        .child(t!("website.missing.home"));
    let field = section().class(ClassName::MISSING).attr("data-field", "gold").child((
        p().class(ClassName::MISSING_MARK).child(t!("website.missing.mark")),
        h1().class(ClassName::MISSING_HEAD).child(t!("website.missing.head")),
        p().class(ClassName::MISSING_TEXT).child(t!("website.missing.text")),
        home,
    ));
    (
        (header_sheet(), sheet(&SHEET)),
        SiteHeader(SiteHeaderProps::builder().here(String::new()).build()).into_any(),
        main().id("top").child(field),
        footer().into_any(),
    )
}
