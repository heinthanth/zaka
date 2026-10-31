use leptos::html::div;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_explorer::app::app_shell;
use zaka_explorer::assets::ship;
use zaka_explorer::state::lang::load_stored_lang;
use zaka_web_ui_common::sheet;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/explorer/mod.sass");

/// Rules of the frame of the explorer.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-explorer", STYLE_SHEET);

/// The sheets of the explorer page: the frame, and the page-wide sheets of
/// the explorer. They mount outside the island, as a global sheet renders on
/// the server alone.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn explorer_sheets() -> impl IntoView
{
    (sheet(&SHEET), zaka_explorer::app::app_sheets())
}

/// The compiler explorer, in a frame under the header, which names the site
/// and links to its pages in place of the toolbar of the explorer. The server
/// writes the empty frame; the explorer starts in the browser alone, with the
/// stored language and its hashed files. The island loads late, on its own, so
/// the explorer reaches this page and no other.
///
/// # Returns
///
/// The view.
#[island(lazy)]
pub fn Explorer() -> impl IntoView
{
    let started = RwSignal::new(false);
    Effect::new(move |_| {
        zakac_i18n::init_locale();
        load_stored_lang();
        ship();
        started.set(true);
    });
    div()
        .class(ClassName::EXPLORER)
        .child(move || started.get().then(|| app_shell(true)))
}
