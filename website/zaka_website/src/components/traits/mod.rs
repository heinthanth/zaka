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
use zaka_web_ui_common::site::DOCS_URL;
use zakac_i18n::t;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/traits/mod.sass");

/// Rules of the traits.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-traits", STYLE_SHEET);

/// One trait of the language: a word for it, what backs the word, and the
/// Docs page that states the rules behind it.
#[derive(Clone, Copy)]
struct Trait
{
    /// Locale id of the word; the text is under `<id>_text` and the name of
    /// the link under `<id>_link`.
    id: &'static str,
    /// Address of the Docs page under the root of the manual.
    slug: &'static str,
}

/// The traits, in order.
const TRAITS: [Trait; 3] = [
    Trait { id: "website.traits.natural", slug: "language/naturalness" },
    Trait { id: "website.traits.safe", slug: "language/ownership" },
    Trait { id: "website.traits.portable", slug: "vm" },
];

/// The traits: Zaka in three words, each with what backs it and a link to
/// the Docs page that proves it.
///
/// # Returns
///
/// The view.
pub fn traits() -> impl IntoView
{
    section().id("traits").class(ClassName::TRAITS).attr("data-field", "gold").child((
        sheet(&SHEET),
        h2().class(ClassName::TRAITS_HEAD).child(t!("website.traits.head")),
        div()
            .class(format!("{} rise", ClassName::TRAITS_LIST))
            .child(TRAITS.map(trait_card)),
    ))
}

/// One trait.
///
/// # Arguments
///
/// * `item` - The trait.
///
/// # Returns
///
/// The view.
fn trait_card(item: Trait) -> impl IntoView
{
    article().class(ClassName::TRAIT).child((
        h3().class(ClassName::TRAIT_WORD).child(t!(item.id.to_string())),
        p().class(ClassName::TRAIT_TEXT).child(t!(format!("{}_text", item.id))),
        a().class(ClassName::TRAIT_LINK)
            .href(format!("{DOCS_URL}{}/", item.slug))
            .child((t!(format!("{}_link", item.id)), " →")),
    ))
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::book::Book;
    use crate::book::page_at;

    #[test]
    fn each_trait_links_to_a_docs_page()
    {
        TRAITS.iter().for_each(|item| {
            assert!(page_at(Book::Manual, item.slug).is_some(), "{}", item.slug);
        });
    }
}
