use leptos::ev;
use leptos::html::a;
use leptos::html::button;
use leptos::html::div;
use leptos::html::header;
use leptos::html::img;
use leptos::html::nav;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_use_event_listener::use_event_listener;
use leptos_use_window::use_window;
use wasm_bindgen::JsCast;
use web_sys::Element;
use zaka_web_ui_common::controls::pickers::lang_icon_picker;
use zaka_web_ui_common::controls::pickers::lang_picker;
use zaka_web_ui_common::controls::pickers::theme_icon_picker;
use zaka_web_ui_common::controls::pickers::theme_picker;
use zaka_web_ui_common::icon::cross;
use zaka_web_ui_common::icon::globe;
use zaka_web_ui_common::icon::menu;
use zaka_web_ui_common::icon::swatches;
use zaka_web_ui_common::sheet;
use zaka_web_ui_common::site::HOME_URL;
use zaka_web_ui_common::theme::ThemeMode;
use zaka_web_ui_common::theme::theme_mode;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

use crate::assets::LOGO_DARK;
use crate::assets::LOGO_LIGHT;
use crate::book::DOCS_URL;
use crate::book::LEARN_URL;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/header/mod.sass");

/// Rules of the header.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-header", STYLE_SHEET);

/// Where the explorer lives on the site.
pub const EXPLORER_URL: &str = zaka_web_ui_common::site::EXPLORER_URL;

/// The name of the language in Burmese, written beside the logo.
const NAME: &str = "စံကား";

/// Element id of the menu, which its button controls.
const MENU_ID: &str = "site-menu";

/// Attribute on the root of the page while the menu is open; the sheet holds
/// the page still under it.
const MENU_MARK: &str = "data-menu";

/// Distance in px from the top of the window to the line the header reads
/// the chapter at.
const READ_LINE: f64 = 50.0;

/// Chapter of the page when no field is under the header.
const PAPER: &str = "paper";

/// The sheet of the header. It mounts outside the island: a global sheet
/// renders on the server alone, so inside an island the server and the
/// browser would disagree on its contents.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn header_sheet() -> impl IntoView { sheet(&SHEET) }

/// The header: the wordmark, the links to Learn and the Docs, the theme and
/// language pickers, and the explorer. It sits over the page and takes the
/// ground and ink of the chapter under it. An island: the pickers and the
/// chapter colors run in the browser.
///
/// # Arguments
///
/// * `here` - The address of the part of the site the page is in: `/` on the
///   landing page, where the wordmark leads to the top, or the root of a book,
///   whose link the header marks as current; on other pages the wordmark leads
///   home.
///
/// # Returns
///
/// The view.
#[island]
pub fn SiteHeader(here: String) -> impl IntoView
{
    let home = here == HOME_URL;
    let theme = theme_mode();
    let chapter = RwSignal::new(PAPER.to_string());
    let moved = RwSignal::new(false);
    follow_chapters(chapter, moved);
    let mark = move || tern!(theme.is_dark(), LOGO_DARK, LOGO_LIGHT);
    let brand = a()
        .class(ClassName::HEADER_BRAND)
        .href(tern!(home, "#top", "/"))
        .attr("aria-label", t!("website.header.home"))
        .child((
            img()
                .class(ClassName::HEADER_LOGO)
                .src(mark)
                .alt("")
                .attr("aria-hidden", "true"),
            span().class(ClassName::HEADER_NAME).lang("my").child(NAME),
        ));
    let action = a()
        .class(format!("button {}", ClassName::HEADER_ACTION))
        .href(EXPLORER_URL)
        .child(t!("website.header.explorer"));
    let section = |url: &'static str, label: &str| {
        a().class(ClassName::HEADER_LINK)
            .href(url)
            .attr("aria-current", (here == url).then_some("page"))
            .child(t!(label.to_string()))
    };
    let learn = section(LEARN_URL, "website.header.learn");
    let docs = section(DOCS_URL, "website.header.docs");
    let pickers = div()
        .class(ClassName::HEADER_PICKERS)
        .child((theme_icon_picker(theme), lang_icon_picker(|| ())));
    let open = RwSignal::new(false);
    follow_menu(open);
    // The links come first, then the pickers, then the explorer, the one
    // call to action, at the end.
    let actions = div().class(ClassName::HEADER_ACTIONS).child((
        learn,
        docs,
        pickers,
        action,
        toggle(open),
    ));
    header()
        .class(ClassName::HEADER)
        .class((ClassName::HEADER_OPEN, move || open.get()))
        .attr("data-chapter", move || chapter.get())
        .attr("data-moved", move || moved.get().then_some(""))
        .child((brand, actions, menu_panel(theme, open)))
}

/// The theme and language pickers of the menu, each across it, with an icon
/// in front that tells what it picks.
///
/// # Arguments
///
/// * `theme` - The theme mode.
///
/// # Returns
///
/// The view.
fn menu_pickers(theme: ThemeMode) -> impl IntoView
{
    (
        span()
            .class(ClassName::HEADER_PICKER)
            .child((swatches(), theme_picker(theme, false))),
        span().class(ClassName::HEADER_PICKER).child((globe(), lang_picker(|| ()))),
    )
}

/// The button that opens and shuts the menu on narrow screens.
///
/// # Arguments
///
/// * `open` - Whether the menu is open.
///
/// # Returns
///
/// The view.
fn toggle(open: RwSignal<bool>) -> impl IntoView
{
    let label = move || {
        tern!(open.get(), t!("website.header.menu_shut"), t!("website.header.menu"))
    };
    button()
        .class(ClassName::HEADER_MENU_BUTTON)
        .r#type("button")
        .attr("aria-controls", MENU_ID)
        .attr("aria-expanded", move || open.get().to_string())
        .attr("aria-label", label)
        .on(ev::click, move |_| open.update(|o| *o = !*o))
        .child(move || tern!(open.get(), cross().into_any(), menu().into_any()))
}

/// The menu on narrow screens: the explorer and the links, then the pickers
/// at its foot, over the whole page under the bar. A link shuts it.
///
/// # Arguments
///
/// * `theme` - The theme mode.
/// * `open` - Whether the menu is open.
///
/// # Returns
///
/// The view.
fn menu_panel(theme: ThemeMode, open: RwSignal<bool>) -> impl IntoView
{
    let shut = move |_| open.set(false);
    let link = move |(href, id): (&'static str, &str)| {
        a().href(href).on(ev::click, shut).child(t!(id.to_string()))
    };
    div()
        .id(MENU_ID)
        .class(ClassName::HEADER_MENU)
        .class((ClassName::HEADER_MENU_OPEN, move || open.get()))
        // Lenis leaves the wheel to the menu, which scrolls on its own.
        .attr("data-lenis-prevent", "")
        .attr("inert", move || !open.get())
        .child((
            nav().class(ClassName::HEADER_MENU_LINKS).child((
                link((LEARN_URL, "website.header.learn")),
                link((DOCS_URL, "website.header.docs")),
                link((EXPLORER_URL, "website.header.explorer_link")),
            )),
            div().class(ClassName::HEADER_MENU_PICKERS).child(menu_pickers(theme)),
        ))
}

/// Hold the page still while the menu is open, and shut the menu on Escape.
///
/// # Arguments
///
/// * `open` - Whether the menu is open.
fn follow_menu(open: RwSignal<bool>)
{
    Effect::new(move |_| {
        let Some(root) = document().document_element()
        else
        {
            return;
        };
        let marked = tern!(
            open.get(),
            root.set_attribute(MENU_MARK, "open"),
            root.remove_attribute(MENU_MARK)
        );
        do_if!(let Err(err) = marked, web_sys::console::error_1(&err));
    });
    let _ = use_event_listener(use_window(), ev::keydown, move |e| {
        do_if!(e.key() == "Escape", open.set(false));
    });
}

/// Keep `chapter` on the chapter of the field under the header line, and
/// `moved` on whether the page has left its top, as the page scrolls.
///
/// # Arguments
///
/// * `chapter` - Where the chapter goes.
/// * `moved` - Whether the page has scrolled at all.
fn follow_chapters(chapter: RwSignal<String>, moved: RwSignal<bool>)
{
    let read = move || {
        chapter.set(chapter_at(READ_LINE));
        moved.set(window().scroll_y().is_ok_and(|top| top > 0.0));
    };
    let _ = use_event_listener(use_window(), ev::scroll, move |_| read());
    Effect::new(move |_| read());
}

/// The chapter of the field that spans a line of the window.
///
/// # Arguments
///
/// * `line` - Distance in px from the top of the window.
///
/// # Returns
///
/// The chapter, or [`PAPER`] when no field spans the line.
fn chapter_at(line: f64) -> String
{
    let Ok(fields) = document().query_selector_all("[data-field]")
    else
    {
        return PAPER.into();
    };
    let spans = |el: &Element| {
        let rect = el.get_bounding_client_rect();
        rect.top() <= line && rect.bottom() > line
    };
    (0..fields.length())
        .filter_map(|idx| fields.item(idx))
        .filter_map(|node| node.dyn_into::<Element>().ok())
        .filter(spans)
        .last()
        .and_then(|el| el.get_attribute("data-field"))
        .unwrap_or_else(|| PAPER.into())
}
