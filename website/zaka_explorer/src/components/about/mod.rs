use leptos::ev;
use leptos::html::button;
use leptos::html::div;
use leptos::html::img;
use leptos::html::p;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::controls::ClassName as Controls;
use zaka_web_ui_common::controls::modal::Modal;
use zaka_web_ui_common::controls::modal::modal;
use zaka_web_ui_common::credits::credits_roll;
use zaka_web_ui_common::credits::credits_sheet;
use zaka_web_ui_common::icon::chevron;
use zaka_web_ui_common::icon::info;
use zaka_web_ui_common::sheet;
use zakac_consts::ZAKA_AUTHOR;
use zakac_consts::ZAKA_BUILD_NUMBER;
use zakac_consts::ZAKA_FIRST_YEAR;
use zakac_consts::ZAKA_LICENSE;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

use crate::assets::LOGO_DARK;
use crate::assets::LOGO_LIGHT;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/about/mod.sass");

/// Sheet of the about button and modal.
static SHEET: ComponentStyle = ComponentStyle::global_css("zaka-about", STYLE_SHEET);

/// Element id of the modal title, which names the dialog.
const TITLE_ID: &str = "about-modal-title";

/// The about button, which opens the modal.
///
/// # Arguments
///
/// * `open` - Whether the modal shows.
///
/// # Returns
///
/// The view.
pub fn about_button(open: RwSignal<bool>) -> impl IntoView
{
    button()
        .class(Controls::ICON_BUTTON)
        .class((ClassName::ABOUT_BUTTON_ON, move || open.get()))
        .attr("aria-haspopup", "dialog")
        .attr("aria-expanded", move || open.get().to_string())
        .attr("aria-label", t!("explorer.about.open"))
        .title(t!("explorer.about.open"))
        .on(ev::click, move |_| open.set(true))
        .child(info())
}

/// The about modal: the version, the author and the license, with a slide of
/// credits behind a button and a button that starts the guided tour. The
/// toolbar button and the menu of narrow screens both open it.
///
/// # Arguments
///
/// * `open` - Whether the modal shows.
///
/// # Returns
///
/// The view.
pub fn about_modal(open: RwSignal<bool>) -> impl IntoView
{
    let credits = RwSignal::new(false);
    // The modal opens on the about slide.
    Effect::new(move |_| do_if!(open.get(), credits.set(false)));
    let spec = Modal::builder()
        .open(open)
        .class(ClassName::ABOUT_MODAL)
        .title_id(TITLE_ID)
        .title(t!("explorer.about.title"))
        .close_label(t!("explorer.about.close_label"))
        .build();
    (sheet(&SHEET), credits_sheet(), modal(spec, slides(open, credits)))
}

/// The two slides next to each other: about, then credits. The one out of view
/// is inert, so the focus and screen readers skip it.
///
/// # Arguments
///
/// * `open` - Whether the modal shows.
/// * `credits` - Whether the credits slide shows.
///
/// # Returns
///
/// The view.
fn slides(open: RwSignal<bool>, credits: RwSignal<bool>) -> impl IntoView
{
    let inert = move |hidden: bool| hidden.then_some("");
    let about = div()
        .class(ClassName::ABOUT_SLIDE)
        .attr("inert", move || inert(credits.get()))
        .child((brand(), facts(), links(open, credits), rights()));
    let back = button()
        .class(ClassName::ABOUT_BACK)
        .attr("aria-label", t!("explorer.about.back_label"))
        .on(ev::click, move |_| credits.set(false))
        .child((chevron(), span().child(t!("explorer.about.back"))));
    // The roll starts when the slide shows, so it opens at its top.
    let roll = move || credits.get().then(credits_roll);
    let credits_slide = div()
        .class(ClassName::ABOUT_SLIDE)
        .attr("inert", move || inert(!credits.get()))
        .child((back, roll));
    div()
        .class(ClassName::ABOUT_SLIDES)
        .class((ClassName::ABOUT_SLIDES_CREDITS, move || credits.get()))
        .child((about, credits_slide))
}

/// The buttons under the facts: the credits slide and the tour.
///
/// # Arguments
///
/// * `open` - Whether the modal shows.
/// * `credits` - Whether the credits slide shows.
///
/// # Returns
///
/// The view.
fn links(open: RwSignal<bool>, credits: RwSignal<bool>) -> impl IntoView
{
    div()
        .class(ClassName::ABOUT_LINKS)
        .child((credits_button(credits), tour_button(open)))
}

/// The button that closes the modal and starts the guided tour.
///
/// # Arguments
///
/// * `open` - Whether the modal shows.
///
/// # Returns
///
/// The view.
fn tour_button(open: RwSignal<bool>) -> impl IntoView
{
    let state = use_state();
    let start = move |_| {
        open.set(false);
        state.tour.set(true);
    };
    button()
        .class(ClassName::ABOUT_TOUR)
        .on(ev::click, start)
        .child((span().child(t!("explorer.about.tour")), chevron()))
}

/// The button that opens the credits slide.
///
/// # Arguments
///
/// * `credits` - Whether the credits slide shows.
///
/// # Returns
///
/// The view.
fn credits_button(credits: RwSignal<bool>) -> impl IntoView
{
    button()
        .class(ClassName::ABOUT_CREDITS)
        .on(ev::click, move |_| credits.set(true))
        .child((span().child(t!("explorer.about.credits")), chevron()))
}

/// The version: the explorer version and the build number, as the CLI
/// prints them.
///
/// # Returns
///
/// The text, such as `0.1.0+3f2a9c1`.
pub fn version_text() -> String
{
    format!("{}+{ZAKA_BUILD_NUMBER}", env!("CARGO_PKG_VERSION"))
}

/// The logo, the name and the version.
///
/// # Returns
///
/// The view.
fn brand() -> impl IntoView
{
    let state = use_state();
    let logo = img()
        .class(ClassName::ABOUT_LOGO)
        .attr("src", move || tern!(state.is_dark(), LOGO_DARK, LOGO_LIGHT))
        .attr("alt", "")
        .attr("aria-hidden", "true");
    let version = t!("explorer.about.version", version = version_text());
    div().class(ClassName::ABOUT_BRAND).child((
        logo,
        div().class(ClassName::ABOUT_NAMES).child((
            span().class(ClassName::ABOUT_NAME).child(t!("explorer.toolbar.brand")),
            span().class(ClassName::ABOUT_VERSION).child(version),
        )),
    ))
}

/// The author and the license.
///
/// # Returns
///
/// The view.
fn facts() -> impl IntoView
{
    let fact = |id: &'static str, value: &'static str| {
        (
            span().class(ClassName::ABOUT_TERM).child(t!(id.to_string())),
            span().child(value),
        )
    };
    div().class(ClassName::ABOUT_FACTS).child((
        fact("explorer.about.author", ZAKA_AUTHOR),
        fact("explorer.about.license", ZAKA_LICENSE),
    ))
}

/// The rights notice, up to now, over the license line.
///
/// # Returns
///
/// The view.
fn rights() -> impl IntoView
{
    let last = js_sys::Date::new_0().get_full_year();
    let rights = t!(
        "explorer.about.rights",
        first = ZAKA_FIRST_YEAR,
        last = last,
        author = ZAKA_AUTHOR
    );
    let terms = t!("explorer.about.terms", license = ZAKA_LICENSE);
    div().class(ClassName::ABOUT_RIGHTS).child((p().child(rights), p().child(terms)))
}
