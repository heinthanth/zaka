use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::div;
use leptos::html::nav;
use leptos::prelude::*;
use leptos_scoped_style::scss_file;
use strum::IntoEnumIterator;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::compile::TreeSource;
use crate::components::ast_view::forms_view;
use crate::components::check_view::skipped;
use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::state::AppState;
use crate::state::settings::AstMode;
use crate::state::settings::LowerMode;
use crate::state::use_state;

/// Sheet of the lowering view.
static SHEET: ScopedSheet = ScopedSheet::new(
    "zaka-lower-view",
    scss_file!("src/components/lower_view/mod.sass"),
);

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
    let root = NodeRef::<html::Div>::new();
    let view = div()
        .class("lower-view")
        .class(("lower-view-fill", move || !lowered.get() || graph()))
        .class(("lower-view-skipped", move || !lowered.get()))
        .node_ref(root)
        .child(content);
    scoped(&SHEET, root, view)
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
        .class("ast-modes")
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
        .class("ast-mode")
        .class(("ast-mode-on", on))
        .attr("role", "tab")
        .attr("aria-selected", move || on().to_string())
        .on(ev::click, move |_| state.set_lower_mode.set(mode))
        .child(mode.label())
}
