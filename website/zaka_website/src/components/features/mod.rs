use leptos::html::a;
use leptos::html::article;
use leptos::html::div;
use leptos::html::h2;
use leptos::html::h3;
use leptos::html::p;
use leptos::html::section;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;

use crate::book::Book;
use crate::book::page_at;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/features/mod.sass");

/// Rules of the features.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-features", STYLE_SHEET);

/// One part of the language, and the page of a book that teaches it.
#[derive(Clone, Copy)]
struct Feature
{
    /// Locale id of the name; the text is under `<id>_text`.
    id: &'static str,
    /// The book of the page.
    book: Book,
    /// Address of the page under the root of the book.
    slug: &'static str,
}

/// The features, in order.
const FEATURES: [Feature; 8] = [
    Feature { id: "website.features.structs", book: Book::Learn, slug: "structs" },
    Feature { id: "website.features.tables", book: Book::Learn, slug: "tables" },
    Feature {
        id: "website.features.results",
        book: Book::Learn,
        slug: "error-handling",
    },
    Feature { id: "website.features.generics", book: Book::Learn, slug: "generics" },
    Feature {
        id: "website.features.closures",
        book: Book::Learn,
        slug: "function-values/closures",
    },
    Feature { id: "website.features.stdlib", book: Book::Learn, slug: "stdlib" },
    Feature { id: "website.features.c", book: Book::Learn, slug: "functions/c" },
    Feature {
        id: "website.features.editors",
        book: Book::Manual,
        slug: "compiler/editors",
    },
];

/// The features: what the language has, a card for each part, its name a
/// link to the page that teaches it.
///
/// # Returns
///
/// The view.
pub fn features() -> impl IntoView
{
    section()
        .id("features")
        .class(ClassName::FEATURES)
        .attr("data-field", "paper")
        .child((
            sheet(&SHEET),
            h2().class(ClassName::FEATURES_HEAD).child(t!("website.features.head")),
            div()
                .class(format!("{} rise", ClassName::FEATURES_LIST))
                .child(FEATURES.map(feature_card)),
        ))
}

/// One feature.
///
/// # Arguments
///
/// * `item` - The feature.
///
/// # Returns
///
/// The view.
fn feature_card(item: Feature) -> impl IntoView
{
    let href = page_at(item.book, item.slug).map(|page| page.url()).unwrap_or_default();
    article().class(ClassName::FEATURE).child((
        h3().class(ClassName::FEATURE_NAME).child(
            a().class(ClassName::FEATURE_LINK)
                .href(href)
                .child((t!(item.id.to_string()), "\u{a0}→")),
        ),
        p().class(ClassName::FEATURE_TEXT).child(t!(format!("{}_text", item.id))),
    ))
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn each_feature_links_to_a_page()
    {
        FEATURES.iter().for_each(|item| {
            assert!(page_at(item.book, item.slug).is_some(), "{}", item.slug);
        });
    }
}
