use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::div;
use leptos::html::nav;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::scss_file;
use strum::IntoEnumIterator;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::compile::Phase;
use crate::components::ast_view::ast_view;
use crate::components::check_view::check_view;
use crate::components::lower_view::lower_view;
use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::components::segments_view::segments_view;
use crate::components::tokens_view::tokens_view;
use crate::state::AppState;
use crate::state::use_state;

/// Sheet of the pipeline and the stage view.
static SHEET: ScopedSheet =
    ScopedSheet::new("zaka-pipeline", scss_file!("src/components/pipeline/mod.sass"));

/// The compiler pipeline as a timeline of stages, and the view of the open
/// stage.
///
/// # Returns
///
/// The view.
pub fn pipeline() -> impl IntoView
{
    let state = use_state();
    let open = move || match state.stage.get()
    {
        Phase::Segment => segments_view().into_any(),
        Phase::Lex => tokens_view().into_any(),
        Phase::Parse => ast_view().into_any(),
        Phase::Check => check_view().into_any(),
        Phase::Lower => lower_view().into_any(),
    };
    let stages = Phase::iter().map(|phase| stage_button(state, phase)).collect_view();
    let timeline = nav()
        .class("pipeline")
        .attr("role", "tablist")
        .attr("aria-label", t!("explorer.pipeline.label"))
        .on(ev::keydown, move |e| step_stage(state, &e))
        .child(stages);
    let root = NodeRef::<html::Div>::new();
    let view = div().class("stage-view").node_ref(root).child((
        timeline,
        div().class("stage-pane").attr("role", "tabpanel").child(open),
    ));
    scoped(&SHEET, root, view)
}

/// One stage of the timeline: its number, a status dot, its name and the
/// output it shows. Stages up to the open one count as passed, so the line
/// between them fills in.
///
/// # Arguments
///
/// * `state` - The state.
/// * `phase` - The phase of the stage.
///
/// # Returns
///
/// The view.
fn stage_button(state: AppState, phase: Phase) -> impl IntoView
{
    let status = move || state.compiled.with(|c| c.status(phase));
    let class = move || -> &'static str { status().into() };
    let is_open = move || state.stage.get() == phase;
    let passed = move || phase as usize <= state.stage.get() as usize;
    let dot =
        span().class(move || format!("stage-dot stage-{}", class())).title(move || {
            let word = status().label();
            t!("explorer.pipeline.status", phase = phase.label(), status = word)
        });
    let node = span().class("stage-node").child((phase as usize + 1, dot));
    let text = span().class("stage-text").child((
        span().class("stage-label").child(phase.label()),
        span().class("stage-note").child(phase.note()),
    ));
    button()
        .id(stage_id(phase))
        .class("stage")
        .class(("stage-open", is_open))
        .class(("stage-passed", passed))
        .attr("role", "tab")
        .attr("aria-selected", move || is_open().to_string())
        .attr("tabindex", move || tern!(is_open(), "0", "-1"))
        .on(ev::click, move |_| state.stage.set(phase))
        .child((node, text))
}

/// Element id of a stage button.
///
/// # Arguments
///
/// * `phase` - The phase.
///
/// # Returns
///
/// The id.
fn stage_id(phase: Phase) -> String
{
    let label: &'static str = phase.into();
    format!("stage-{}", label.to_lowercase())
}

/// Open the stage before or after the open one with the arrows, and
/// move the focus with it.
///
/// # Arguments
///
/// * `state` - The state.
/// * `e` - The press.
fn step_stage(state: AppState, e: &ev::KeyboardEvent)
{
    let phases: Vec<Phase> = Phase::iter().collect();
    let at = state.stage.get_untracked() as usize;
    let next = match e.key().as_str()
    {
        "ArrowRight" => (at + 1).min(phases.len() - 1),
        "ArrowLeft" => at.saturating_sub(1),
        _ => return,
    };
    e.prevent_default();
    state.stage.set(phases[next]);
    let Some(el) = document().get_element_by_id(&stage_id(phases[next]))
    else
    {
        return;
    };
    let _ = el.unchecked_into::<HtmlElement>().focus();
}
