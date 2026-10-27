use leptos::ev;
use leptos::html::button;
use leptos::html::div;
use leptos::html::section;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::icon::chevron;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::components::diag_panel::diag_counts;
use crate::components::diag_panel::diag_view;
use crate::components::run_view::output_view;
use crate::state::AppState;
use crate::state::PanelTab;
use crate::state::tour::TOUR;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/bottom_panel/mod.sass");

/// Sheet of the bottom panel.
static SHEET: ComponentStyle =
    ComponentStyle::global_css("zaka-bottom-panel", STYLE_SHEET);

/// The panel under the stage view: a head with a chevron that hides it and
/// a tab for each part, then the open part. In debug mode it
/// has the output and the problems; outside it the problems alone, as the
/// output then sits in the stage view.
///
/// # Returns
///
/// The view.
pub fn bottom_panel() -> impl IntoView
{
    let state = use_state();
    let open = state.problems_open;
    let content = move || {
        let part = tern!(
            shown_tab(state) == PanelTab::Output,
            output_part().into_any(),
            diag_view().into_any()
        );
        open.get().then(|| div().class(ClassName::PANEL_CONTENT).child(part))
    };
    let view = section()
        .class(ClassName::BOTTOM_PANEL)
        .attr(TOUR, "panel")
        .child((panel_head(state), content));
    (sheet(&SHEET), view)
}

/// The tab the panel shows: the open one in debug mode, else the problems.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The tab.
fn shown_tab(state: AppState) -> PanelTab
{
    tern!(state.debug_mode.get(), state.panel_tab.get(), PanelTab::Problems)
}

/// The output of the run, with its own padding.
///
/// # Returns
///
/// The view.
fn output_part() -> impl IntoView
{
    let state = use_state();
    div().class(ClassName::OUTPUT_PART).attr(TOUR, "output").child(output_view(state))
}

/// The head: the chevron that hides the panel, then the tabs.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn panel_head(state: AppState) -> impl IntoView
{
    let open = state.problems_open;
    let name =
        move || tern!(open.get(), t!("explorer.panel.hide"), t!("explorer.panel.show"));
    let shut_button = button()
        .class(ClassName::PANEL_CHEVRON)
        .attr("aria-label", name)
        .title(name)
        .on(ev::click, move |_| state.set_problems_open.set(false))
        .child(chevron());
    let output = move || {
        state
            .debug_mode
            .get()
            .then(|| tab(state, PanelTab::Output, t!("explorer.run.output").into_any()))
    };
    let problems = (
        span().class(ClassName::TAB_TITLE).child(t!("explorer.problems.title")),
        diag_counts(),
    );
    let problems = tab(state, PanelTab::Problems, problems.into_any());
    div()
        .class(ClassName::PANEL_HEAD)
        .attr("role", "tablist")
        .attr("aria-label", t!("explorer.panel.label"))
        .child((shut_button, output, problems))
}

/// A tab of the head; a click turns the panel to it.
///
/// # Arguments
///
/// * `state` - The state.
/// * `which` - The tab.
/// * `label` - What the tab shows.
///
/// # Returns
///
/// The view.
fn tab(state: AppState, which: PanelTab, label: AnyView) -> impl IntoView
{
    let picked = move || shown_tab(state) == which;
    button()
        .class(ClassName::PANEL_TAB)
        .class((ClassName::PANEL_TAB_OPEN, picked))
        .attr("role", "tab")
        .attr("aria-selected", move || picked().to_string())
        .on(ev::click, move |_| state.panel_tab.set(which))
        .child(label)
}
