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
use zaka_web_ui_common::controls::ClassName as Controls;
use zaka_web_ui_common::controls::dropdown::Choice;
use zaka_web_ui_common::controls::dropdown::Dropdown;
use zaka_web_ui_common::controls::dropdown::dropdown;
use zaka_web_ui_common::controls::settings::settings_button;
use zaka_web_ui_common::icon::bug;
use zaka_web_ui_common::icon::cross;
use zaka_web_ui_common::icon::gear;
use zaka_web_ui_common::icon::info;
use zaka_web_ui_common::icon::left_pane;
use zaka_web_ui_common::icon::menu;
use zaka_web_ui_common::sheet;
use zaka_web_ui_common::site::HOME_URL;
use zaka_web_ui_common::site::MANUAL_URL;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

use crate::assets::LOGO_DARK;
use crate::assets::LOGO_LIGHT;
use crate::compile::samples::GROUPS;
use crate::compile::samples::find_sample;
use crate::compile::samples::in_group;
use crate::components::about::about_button;
use crate::components::about::about_modal;
use crate::components::settings::settings_modal;
use crate::state::tour::TOUR;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/toolbar/mod.sass");

/// Sheet of the toolbar.
static SHEET: ComponentStyle = ComponentStyle::global_css("zaka-toolbar", STYLE_SHEET);

/// Element id of the menu of narrow screens, which its button controls.
const MENU_ID: &str = "toolbar-menu";

/// The bar above the panes: the brand and the links to the website and the
/// manual on the left; samples, the debug mode switch, the editor switch,
/// about and settings on the right. On narrow screens the links, about and
/// settings move into a menu behind a button at the end of the bar.
///
/// # Returns
///
/// The view.
pub fn toolbar() -> impl IntoView
{
    let state = use_state();
    let logo = img()
        .class(ClassName::BRAND_LOGO)
        .attr("src", move || tern!(state.is_dark(), LOGO_DARK, LOGO_LIGHT))
        .attr("alt", "")
        .attr("aria-hidden", "true");
    let brand = div().class(ClassName::BRAND).child((
        logo,
        span().class(ClassName::BRAND_NAME).child(t!("explorer.toolbar.brand")),
    ));
    let about = RwSignal::new(false);
    let settings = RwSignal::new(false);
    let open = RwSignal::new(false);
    let wide = div()
        .class(ClassName::TOOLBAR_WIDE)
        .child((about_button(about), settings_button(settings)));
    let view = header().class(ClassName::TOOLBAR).child((
        brand,
        site_links(),
        span().class(ClassName::TOOLBAR_GAP),
        sample_picker(),
        div().class(ClassName::TOOLBAR_ACTIONS).attr(TOUR, "settings").child((
            debug_switch(),
            editor_switch(),
            wide,
            menu_button(open),
        )),
        menu_panel(open, about, settings),
    ));
    (sheet(&SHEET), view, about_modal(about), settings_modal(settings))
}

/// The links back to the website the explorer sits in: its landing page and
/// the manual.
///
/// # Returns
///
/// The view.
fn site_links() -> impl IntoView
{
    nav()
        .class(ClassName::TOOLBAR_LINKS)
        .attr("aria-label", t!("explorer.toolbar.site"))
        .child((
            a().href(HOME_URL).child(t!("explorer.toolbar.website")),
            a().href(MANUAL_URL).child(t!("explorer.toolbar.manual")),
        ))
}

/// The button that opens and shuts the menu of narrow screens.
///
/// # Arguments
///
/// * `open` - Whether the menu is open.
///
/// # Returns
///
/// The view.
fn menu_button(open: RwSignal<bool>) -> impl IntoView
{
    let label = move || {
        tern!(open.get(), t!("explorer.toolbar.menu_shut"), t!("explorer.toolbar.menu"))
    };
    button()
        .class(format!("{} {}", Controls::ICON_BUTTON, ClassName::MENU_BUTTON))
        .class((Controls::ICON_BUTTON_ON, move || open.get()))
        .r#type("button")
        .attr("aria-controls", MENU_ID)
        .attr("aria-expanded", move || open.get().to_string())
        .attr("aria-label", label)
        .title(label)
        .on(ev::click, move |_| open.update(|o| *o = !*o))
        .child(move || tern!(open.get(), cross().into_any(), menu().into_any()))
}

/// The menu of narrow screens: the links to the website and the manual, then
/// rows that open about and settings. A pick shuts it, and so does Escape.
///
/// # Arguments
///
/// * `open` - Whether the menu is open.
/// * `about` - Whether the about modal shows.
/// * `settings` - Whether the settings modal shows.
///
/// # Returns
///
/// The view.
fn menu_panel(
    open: RwSignal<bool>, about: RwSignal<bool>, settings: RwSignal<bool>,
) -> impl IntoView
{
    let _ = use_event_listener(use_window(), ev::keydown, move |e| {
        do_if!(e.key() == "Escape", open.set(false));
    });
    let link = move |href: &'static str, label: String| {
        a().href(href).on(ev::click, move |_| open.set(false)).child(label)
    };
    let row = move |modal: RwSignal<bool>, icon: AnyView, label: String| {
        let show = move |_| {
            open.set(false);
            modal.set(true);
        };
        button().r#type("button").on(ev::click, show).child((icon, span().child(label)))
    };
    let links = nav().class(ClassName::MENU_LINKS).child((
        link(HOME_URL, t!("explorer.toolbar.website").to_string()),
        link(MANUAL_URL, t!("explorer.toolbar.manual").to_string()),
    ));
    let rows = div().class(ClassName::MENU_ROWS).child((
        row(about, info().into_any(), t!("explorer.toolbar.about").to_string()),
        row(settings, gear().into_any(), t!("explorer.toolbar.settings").to_string()),
    ));
    div()
        .id(MENU_ID)
        .class(ClassName::MENU)
        .class((ClassName::MENU_OPEN, move || open.get()))
        .attr("inert", move || (!open.get()).then_some(""))
        .child((links, rows))
}

/// Button that turns debug mode on or off: on, the pane shows each phase of
/// the pipeline and the run view is a debugger.
///
/// # Returns
///
/// The view.
fn debug_switch() -> impl IntoView
{
    let state = use_state();
    let on = state.debug_mode;
    let title = move || {
        tern!(
            on.get(),
            t!("explorer.toolbar.debug_off"),
            t!("explorer.toolbar.debug_on")
        )
    };
    button()
        .class(Controls::ICON_BUTTON)
        .class((Controls::ICON_BUTTON_ON, move || on.get()))
        .attr(TOUR, "debug")
        .attr("aria-pressed", move || on.get().to_string())
        .attr("aria-label", title)
        .title(title)
        .on(ev::click, move |_| state.set_debug_mode.set(!on.get_untracked()))
        .child(bug())
}

/// Button that shows or hides the editor pane.
///
/// # Returns
///
/// The view.
fn editor_switch() -> impl IntoView
{
    let state = use_state();
    let open = state.editor_open;
    let title = move || {
        tern!(
            open.get(),
            t!("explorer.toolbar.editor_hide"),
            t!("explorer.toolbar.editor_show")
        )
    };
    button()
        .class(Controls::ICON_BUTTON)
        .class((Controls::ICON_BUTTON_ON, move || open.get()))
        .attr("aria-pressed", move || open.get().to_string())
        .attr("aria-label", title)
        .title(title)
        .on(ev::click, move |_| state.set_editor_open.set(!open.get_untracked()))
        .child(left_pane())
}

/// Picker that loads a sample program.
///
/// # Returns
///
/// The view.
fn sample_picker() -> impl IntoView
{
    let state = use_state();
    let picked = RwSignal::new(String::new());
    let on_pick = Callback::new(move |path: String| {
        let Some(sample) = find_sample(&path)
        else
        {
            return;
        };
        picked.set(path);
        state.load(sample.text.to_string());
    });
    let choices = GROUPS.iter().flat_map(|group| {
        in_group(group).map(move |s| {
            Choice::builder().value(s.path()).label(s.name).group(*group).build()
        })
    });
    let picker = Dropdown::builder()
        .choices(choices.collect())
        .value(picked)
        .on_pick(on_pick)
        .label(t!("explorer.toolbar.sample"))
        .placeholder(t!("explorer.toolbar.sample_pick"))
        .build();
    span()
        .class(ClassName::CONTROL)
        .attr(TOUR, "samples")
        .child((span().child(t!("explorer.toolbar.sample")), dropdown(picker)))
}
