use leptos::ev;
use leptos::html::button;
use leptos::html::div;
use leptos::html::header;
use leptos::html::img;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::controls::ClassName as Controls;
use zaka_web_ui_common::controls::dropdown::Choice;
use zaka_web_ui_common::controls::dropdown::Dropdown;
use zaka_web_ui_common::controls::dropdown::dropdown;
use zaka_web_ui_common::icon::left_pane;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::assets::LOGO_DARK;
use crate::assets::LOGO_LIGHT;
use crate::compile::samples::GROUPS;
use crate::compile::samples::find_sample;
use crate::compile::samples::in_group;
use crate::components::about::about_menu;
use crate::components::settings::settings_menu;
use crate::state::tour::TOUR;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/toolbar/mod.sass");

/// Sheet of the toolbar.
static SHEET: ComponentStyle = ComponentStyle::global_css("zaka-toolbar", STYLE_SHEET);

/// The bar above the panes: the brand on the left; samples, the editor
/// switch, about and settings on the right.
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
    let view = header().class(ClassName::TOOLBAR).child((
        brand,
        span().class(ClassName::TOOLBAR_GAP),
        sample_picker(),
        div().class(ClassName::TOOLBAR_ACTIONS).attr(TOUR, "settings").child((
            editor_switch(),
            about_menu(),
            settings_menu(),
        )),
    ));
    (sheet(&SHEET), view)
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
