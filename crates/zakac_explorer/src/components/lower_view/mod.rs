use leptos::ev;
use leptos::html::button;
use leptos::html::div;
use leptos::html::nav;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use strum::IntoEnumIterator;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::compile::TreeSource;
use crate::components::ast_view::ClassName as AstView;
use crate::components::ast_view::forms_view;
use crate::components::check_view::skipped;
use crate::state::AppState;
use crate::state::settings::AstMode;
use crate::state::settings::LowerMode;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/lower_view/mod.sass");

/// Sheet of the lowering view.
static SHEET: ComponentStyle =
    ComponentStyle::global_css("zaka-lower-view", STYLE_SHEET);

/// The IR, as lowered or after folding, in the forms of the AST view; or a
/// note when lowering did not run.
///
/// # Returns
///
/// The view.
pub fn lower_view() -> impl IntoView
{
    let state = use_state();
    let lowered = Memo::new(move |_| state.compiled.with(|c| c.lowered));
    let source = move || {
        let folded = state.lower_mode.get() == LowerMode::Folded;
        tern!(folded, TreeSource::Folded, TreeSource::Plain)
    };
    let content = move || {
        let title = t!("explorer.lower.skipped_title");
        let note = t!("explorer.lower.skipped_note");
        let forms = move || forms_view(source, true, fold_switch(state).into_any());
        tern!(lowered.get(), forms().into_any(), skipped(title, note).into_any())
    };
    // The graph and the note fill the pane; the other forms grow with their
    // content, so the pane keeps its padding below them.
    let graph = move || state.ast_mode.get() == AstMode::Graph;
    let view = div()
        .class((ClassName::LOWER_VIEW_FILL, move || !lowered.get() || graph()))
        .class((ClassName::LOWER_VIEW_SKIPPED, move || !lowered.get()))
        .child(content);
    (sheet(&SHEET), view)
}

/// The switch between the IR as lowered and after folding. It sits in the
/// head of the forms view and takes its look from there.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn fold_switch(state: AppState) -> impl IntoView
{
    let buttons = LowerMode::iter().map(|m| mode_button(state, m)).collect_view();
    nav()
        .class(AstView::AST_MODES)
        .attr("role", "tablist")
        .attr("aria-label", t!("explorer.lower.label"))
        .child(buttons)
}

/// One button of the switch.
///
/// # Arguments
///
/// * `state` - The state.
/// * `mode` - The IR it shows.
///
/// # Returns
///
/// The view.
fn mode_button(state: AppState, mode: LowerMode) -> impl IntoView
{
    let on = move || state.lower_mode.get() == mode;
    button()
        .class(AstView::AST_MODE)
        .class((AstView::AST_MODE_ON, on))
        .attr("role", "tab")
        .attr("aria-selected", move || on().to_string())
        .on(ev::click, move |_| state.set_lower_mode.set(mode))
        .child(mode.label())
}
