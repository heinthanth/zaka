use leptos::html::a;
use leptos::html::details;
use leptos::html::div;
use leptos::html::li;
use leptos::html::nav;
use leptos::html::ol;
use leptos::html::span;
use leptos::html::summary;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;

use crate::learn::Chapter;
use crate::learn::LEARN_URL;
use crate::learn::Page;
use crate::learn::chapters;
use crate::learn::page_at;
use crate::learn::render::CONTENTS_ID;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/contents/mod.sass");

/// Rules of the contents.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-contents", STYLE_SHEET);

/// The sheet of the contents.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn contents_sheet() -> impl IntoView { sheet(&SHEET) }

/// The number of a chapter, two digits wide.
///
/// # Arguments
///
/// * `n` - The number.
///
/// # Returns
///
/// The text.
pub fn number(n: usize) -> String { format!("{n:02}") }

/// The contents of the Learn pages: a column on wide screens that stands still
/// as the page scrolls, and a fold at the top of the page on narrow ones. The
/// chapter of the page shows its pages; the others show their titles alone.
///
/// # Arguments
///
/// * `page` - The page the reader is on.
///
/// # Returns
///
/// The view.
pub fn contents(page: &Page) -> impl IntoView + use<>
{
    let label = t!("website.learn.contents").to_string();
    let column = nav()
        .class(ClassName::CONTENTS)
        .attr("aria-label", label.clone())
        .child(contents_list(page));
    let fold = details().class(ClassName::CONTENTS_FOLD).child((
        summary().class(ClassName::CONTENTS_FOLD_HEAD).child(label),
        nav().child(contents_list(page)),
    ));
    div().id(CONTENTS_ID).class(ClassName::CONTENTS_BOX).child((column, fold))
}

/// The list of the contents: the title page, then each chapter.
///
/// # Arguments
///
/// * `page` - The page the reader is on.
///
/// # Returns
///
/// The view.
fn contents_list(page: &Page) -> impl IntoView + use<>
{
    let page = *page;
    let title = a()
        .class(ClassName::CONTENTS_TITLE)
        .href(LEARN_URL)
        .attr("aria-current", page.is_title().then_some("page"))
        .child(page_at("").map(|p| p.title).unwrap_or_default());
    let items: Vec<_> =
        chapters().into_iter().map(|c| chapter_item(&page, c)).collect();
    (title, ol().class(ClassName::CONTENTS_CHAPTERS).child(items))
}

/// One chapter of the contents, with its pages when the reader is in it.
///
/// # Arguments
///
/// * `page` - The page the reader is on.
/// * `chapter` - The chapter.
///
/// # Returns
///
/// The view.
fn chapter_item(page: &Page, chapter: Chapter) -> impl IntoView + use<>
{
    let here = chapter.head == *page || chapter.pages.contains(page);
    let mark = chapter.number.map(number).unwrap_or_else(|| "·".to_string());
    let link = a()
        .class(ClassName::CONTENTS_CHAPTER)
        .href(chapter.head.url())
        .attr("aria-current", (chapter.head == *page).then_some("page"))
        .child((
            span().class(ClassName::CONTENTS_NUMBER).child(mark),
            span().child(chapter.head.title),
        ));
    let page = *page;
    let pages = here.then(|| {
        let item = move |p: &Page| {
            let current = (*p == page).then_some("page");
            li().child(a().href(p.url()).attr("aria-current", current).child(p.title))
        };
        ol().class(ClassName::CONTENTS_PAGES)
            .child(chapter.pages.iter().map(item).collect::<Vec<_>>())
    });
    li().class((ClassName::CONTENTS_HERE, here)).child((link, pages))
}
