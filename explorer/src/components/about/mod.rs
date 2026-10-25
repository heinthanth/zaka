use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::div;
use leptos::html::img;
use leptos::html::p;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::scss_file;
use zakac_consts::ZAKA_AUTHOR;
use zakac_consts::ZAKA_BUILD_NUMBER;
use zakac_consts::ZAKA_FIRST_YEAR;
use zakac_consts::ZAKA_LICENSE;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

use crate::components::icon::chevron;
use crate::components::icon::info;
use crate::components::modal::Modal;
use crate::components::modal::modal;
use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::components::toolbar::LOGO_DARK;
use crate::components::toolbar::LOGO_LIGHT;
use crate::state::use_state;

mod credits;

use credits::credits_roll;

/// Sheet of the about button and modal.
static SHEET: ScopedSheet =
    ScopedSheet::new("zaka-about", scss_file!("src/components/about/mod.sass"));

/// Element id of the modal title, which names the dialog.
const TITLE_ID: &str = "about-modal-title";

/// The about button and the modal it opens: the version, the author and
/// the license, with a slide of credits behind a button and a button that
/// starts the guided tour.
///
/// # Returns
///
/// The view.
pub fn about_menu() -> impl IntoView
{
    let open = RwSignal::new(false);
    let credits = RwSignal::new(false);
    // The modal opens on the about slide.
    Effect::new(move |_| do_if!(open.get(), credits.set(false)));
    let root = NodeRef::<html::Div>::new();
    let trigger = button()
        .class("icon-button about-button")
        .class(("about-button-on", move || open.get()))
        .attr("aria-haspopup", "dialog")
        .attr("aria-expanded", move || open.get().to_string())
        .attr("aria-label", t!("explorer.about.open"))
        .title(t!("explorer.about.open"))
        .on(ev::click, move |_| open.set(true))
        .child(info());
    let spec = Modal::builder()
        .open(open)
        .class("about-modal")
        .title_id(TITLE_ID)
        .title("explorer.about.title")
        .close_label("explorer.about.close_label")
        .build();
    let view = div()
        .class("about")
        .node_ref(root)
        .child((trigger, modal(spec, slides(open, credits))));
    scoped(&SHEET, root, view)
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
        .class("about-slide")
        .attr("inert", move || inert(credits.get()))
        .child((brand(), facts(), links(open, credits), rights()));
    let back = button()
        .class("about-back")
        .attr("aria-label", t!("explorer.about.back_label"))
        .on(ev::click, move |_| credits.set(false))
        .child((chevron(), span().child(t!("explorer.about.back"))));
    // The roll starts when the slide shows, so it opens at its top.
    let roll = move || credits.get().then(credits_roll);
    let credits_slide = div()
        .class("about-slide credits-slide")
        .attr("inert", move || inert(!credits.get()))
        .child((back, roll));
    div()
        .class("about-slides")
        .class(("about-slides-credits", move || credits.get()))
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
    div().class("about-links").child((credits_button(credits), tour_button(open)))
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
        .class("about-tour")
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
        .class("about-credits")
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
        .class("about-logo")
        .attr("src", move || tern!(state.is_dark(), LOGO_DARK, LOGO_LIGHT))
        .attr("alt", "")
        .attr("aria-hidden", "true");
    let version = t!("explorer.about.version", version = version_text());
    div().class("about-brand").child((
        logo,
        div().class("about-names").child((
            span().class("about-name").child(t!("explorer.toolbar.brand")),
            span().class("about-version").child(version),
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
            span().class("about-term").child(t!(id.to_string())),
            span().class("about-value").child(value),
        )
    };
    div().class("about-facts").child((
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
    div().class("about-rights").child((p().child(rights), p().child(terms)))
}
