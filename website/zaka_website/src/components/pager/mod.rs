use leptos::html::a;
use leptos::html::nav;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;

use crate::manual::Page;
use crate::manual::neighbours;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/pager/mod.sass");

/// Rules of the pager.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-pager", STYLE_SHEET);

/// The links to the pages before and after a page, as two wide blocks at its
/// foot; the next page takes the gold.
///
/// # Arguments
///
/// * `page` - The page.
///
/// # Returns
///
/// The view.
pub fn pager(page: &Page) -> impl IntoView + use<>
{
    let (before, after) = neighbours(page);
    let link = |p: Page, label: String, class: &'static str| {
        a().class(format!("{} {class}", ClassName::PAGER_LINK)).href(p.url()).child((
            span().class(ClassName::PAGER_LABEL).child(label),
            span().class(ClassName::PAGER_TITLE).child(p.title),
        ))
    };
    let before = before.map(|p| {
        link(p, t!("website.learn.before").to_string(), ClassName::PAGER_BEFORE)
    });
    let after = after.map(|p| {
        link(p, t!("website.learn.after").to_string(), ClassName::PAGER_AFTER)
    });
    nav()
        .class(format!("{} rise", ClassName::PAGER))
        .attr("aria-label", t!("website.learn.pager"))
        .child((sheet(&SHEET), before, after))
}
