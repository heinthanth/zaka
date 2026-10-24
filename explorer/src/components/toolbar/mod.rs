use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::div;
use leptos::html::header;
use leptos::html::img;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::scss_file;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::compile::samples::GROUPS;
use crate::compile::samples::find_sample;
use crate::compile::samples::in_group;
use crate::components::dropdown::Choice;
use crate::components::dropdown::Dropdown;
use crate::components::dropdown::dropdown;
use crate::components::icon::left_pane;
use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::components::settings::settings_menu;
use crate::state::use_state;

/// Sheet of the toolbar.
static SHEET: ScopedSheet =
    ScopedSheet::new("zaka-toolbar", scss_file!("src/components/toolbar/mod.sass"));

/// The logo for light themes, as `index.html` copies it.
const LOGO_LIGHT: &str = "brand/bloom-white-light.svg";

/// The logo for dark themes, with lighter outlines.
const LOGO_DARK: &str = "brand/bloom-white-dark.svg";

/// The bar above the panes: the brand on the left; samples, the editor switch
/// and settings on the right.
///
/// # Returns
///
/// The view.
pub fn toolbar() -> impl IntoView
{
    let state = use_state();
    let logo = img()
        .class("brand-logo")
        .attr("src", move || tern!(state.is_dark(), LOGO_DARK, LOGO_LIGHT))
        .attr("alt", "")
        .attr("aria-hidden", "true");
    let brand = div()
        .class("brand")
        .child((logo, span().class("brand-name").child(t!("explorer.toolbar.brand"))));
    let root = NodeRef::<html::Header>::new();
    let view = header().class("toolbar").node_ref(root).child((
        brand,
        span().class("toolbar-gap"),
        sample_picker(),
        div().class("toolbar-actions").child((editor_switch(), settings_menu())),
    ));
    scoped(&SHEET, root, view)
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
        .class("icon-button")
        .class(("icon-button-on", move || open.get()))
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
    span().class("control").child((
        span().class("control-label").child(t!("explorer.toolbar.sample")),
        dropdown(picker),
    ))
}
