use leptos::html;
use leptos::html::div;
use leptos::html::section;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_use_event_listener::use_event_listener;
use leptos_use_window::use_window;
use zaka_web_ui_common::shared_sheets;
use zaka_web_ui_common::sheet;
use zakac_diagnostics::Level;
use zakac_rust_sugars::do_if;

use crate::components::diag_panel::diag_panel;
use crate::components::editor::editor_pane;
use crate::components::pipeline::pipeline;
use crate::components::splitter::Axis;
use crate::components::splitter::Split;
use crate::components::splitter::splitter;
use crate::components::table::table_sheet;
use crate::components::toolbar::toolbar;
use crate::components::tour::tour;
use crate::state::AppState;
use crate::state::permalink;
use crate::state::provide_state;
use crate::state::split::DEFAULT_EDITOR_SHARE;
use crate::state::split::DEFAULT_STAGE_SHARE;
use crate::state::split::ERROR_STAGE_SHARE;
use crate::state::split::share_fr;
use crate::state::tour::TOUR;
use crate::state::url;
use crate::theme::theme_sheet;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/app/mod.sass");

/// Sheet of the page shell.
static SHEET: ComponentStyle = ComponentStyle::global_css("zaka-app", STYLE_SHEET);

/// The whole page: toolbar, then the editor on the left and the pipeline
/// over the problems on the right, with splitters between them. The
/// problems panel shows while there are problems. The guided tour sits over
/// it all.
///
/// # Returns
///
/// The view.
pub fn app() -> impl IntoView
{
    let state = provide_state();
    follow_hash(state);
    follow_errors(state);
    mark_lang();
    let panes = NodeRef::<html::Div>::new();
    let column = NodeRef::<html::Div>::new();
    let clean = Memo::new(move |_| state.compiled.with(|c| c.diagnostics.is_empty()));
    let problems_shown =
        Signal::derive(move || state.problems_open.get() && !clean.get());
    let between = Split::builder()
        .axis(Axis::Columns)
        .container(panes)
        .share(state.editor_share)
        .set_share(state.set_editor_share)
        .initial(DEFAULT_EDITOR_SHARE)
        .label("explorer.app.resize_editor")
        .enabled(state.editor_open)
        .build();
    let above = Split::builder()
        .axis(Axis::Rows)
        .container(column)
        .share(state.stage_share)
        .set_share(state.set_stage_share)
        .initial(DEFAULT_STAGE_SHARE)
        .label("explorer.app.resize_problems")
        .enabled(problems_shown)
        .build();
    let left = section()
        .class(format!("{} {}", ClassName::PANEL, ClassName::PANEL_EDITOR))
        .attr(TOUR, "editor")
        .child(editor_pane());
    let stage = section()
        .class(format!("{} {}", ClassName::PANEL, ClassName::PANEL_STAGE))
        .attr(TOUR, "stage")
        .child(pipeline());
    let problems = section()
        .class(format!("{} {}", ClassName::PANEL, ClassName::PANEL_PROBLEMS))
        .child(diag_panel());
    let right = div()
        .class(ClassName::COLUMN)
        .class((ClassName::COLUMN_SHUT, move || !state.problems_open.get()))
        .class((ClassName::COLUMN_CLEAN, move || clean.get()))
        .node_ref(column)
        .style(("--first-share", move || share_fr(state.stage_share.get())))
        .style(("--second-share", move || share_fr(1.0 - state.stage_share.get())))
        .child((stage, splitter(above), problems));
    let main = div()
        .class(ClassName::PANES)
        .class((ClassName::PANES_SHUT, move || !state.editor_open.get()))
        .attr("role", "main")
        .node_ref(panes)
        .style(("--first-share", move || share_fr(state.editor_share.get())))
        .style(("--second-share", move || share_fr(1.0 - state.editor_share.get())))
        .child((left, splitter(between), right));
    let shell = div().class(ClassName::APP_SHELL).child((toolbar(), main, tour()));
    (shared_sheets(), theme_sheet(), table_sheet(), sheet(&SHEET), shell)
}

/// Open the problems panel when errors show up, at half the column or more.
/// It acts as the errors come, not while they stand, so the panel can still
/// be shut or resized.
///
/// # Arguments
///
/// * `state` - The state.
fn follow_errors(state: AppState)
{
    let failed = Memo::new(move |_| state.compiled.with(|c| c.count(Level::Error) > 0));
    Effect::new(move |was: Option<bool>| {
        let now = failed.get();
        do_if!(now && was != Some(true), show_problems(state));
        now
    });
}

/// Open the problems panel and give it at least half the column.
///
/// # Arguments
///
/// * `state` - The state.
fn show_problems(state: AppState)
{
    state.set_problems_open.set(true);
    let share = state.stage_share.get_untracked();
    do_if!(share > ERROR_STAGE_SHARE, state.set_stage_share.set(ERROR_STAGE_SHARE));
}

/// Tag the page with the active locale, for screen readers and fonts.
fn mark_lang()
{
    let page = document().document_element();
    do_if!(let Some(page) = page, {
        let _ = page.set_attribute("lang", &zakac_i18n::locale());
    });
}

/// Load a link that is pasted or edited in the address bar.
///
/// # Arguments
///
/// * `state` - The state.
fn follow_hash(state: AppState)
{
    let _ = use_event_listener(use_window(), leptos::ev::hashchange, move |_| {
        let linked = url::read_hash().and_then(|hash| permalink::decode(&hash));
        do_if!(let Some(link) = linked, state.open_link(link));
    });
}
