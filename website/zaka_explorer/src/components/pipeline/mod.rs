use leptos::ev;
use leptos::html::button;
use leptos::html::div;
use leptos::html::nav;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use strum::IntoEnumIterator;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use web_sys::ScrollIntoViewOptions;
use web_sys::ScrollLogicalPosition;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::compile::Phase;
use crate::compile::Status;
use crate::components::ast_view::ast_view;
use crate::components::check_view::check_view;
use crate::components::codegen_view::codegen_view;
use crate::components::lower_view::lower_view;
use crate::components::run_view::ops_view;
use crate::components::segments_view::segments_view;
use crate::components::tokens_view::tokens_view;
use crate::state::AppState;
use crate::state::tour::TOUR;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/pipeline/mod.sass");

/// Sheet of the pipeline and the stage view.
static SHEET: ComponentStyle = ComponentStyle::global_css("zaka-pipeline", STYLE_SHEET);

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
        Phase::Codegen => codegen_view().into_any(),
        Phase::Run => ops_view().into_any(),
    };
    let stage = move |phase: Phase| {
        let link = (phase as usize > 0).then(|| stage_link(state, phase));
        (link, stage_button(state, phase))
    };
    let stages = Phase::iter().map(stage).collect_view();
    follow_stage(state);
    let timeline = nav()
        .class(ClassName::PIPELINE)
        .attr(TOUR, "pipeline")
        .attr("role", "tablist")
        .attr("aria-label", t!("explorer.pipeline.label"))
        .on(ev::keydown, move |e| step_stage(state, &e))
        .child(stages);
    let view = div().class(ClassName::STAGE_VIEW).child((
        timeline,
        div().class(ClassName::STAGE_PANE).attr("role", "tabpanel").child(open),
    ));
    (sheet(&SHEET), view)
}

/// One stage of the timeline: its number, a status dot, its name and the
/// output it shows. Stages up to the open one count as passed, so the line
/// between them fills in; the halves of the line on both sides of the number
/// join the lines between the stages.
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
    let status = move || stage_status(state, phase);
    let is_open = move || state.stage.get() == phase;
    let passed = move || is_passed(state, phase);
    let dot = span()
        .class(move || format!("{} {}", ClassName::STAGE_DOT, status_class(status())))
        .title(move || {
            let word = status().label();
            t!("explorer.pipeline.status", phase = phase.label(), status = word)
        });
    let node = span().class(ClassName::STAGE_NODE).child((phase as usize + 1, dot));
    let track = span().class(ClassName::STAGE_TRACK).child(node);
    let text = span().class(ClassName::STAGE_TEXT).child((
        span().class(ClassName::STAGE_LABEL).child(phase.label()),
        span().class(ClassName::STAGE_NOTE).child(phase.note()),
    ));
    button()
        .id(stage_id(phase))
        .class(ClassName::STAGE)
        .class((ClassName::STAGE_OPEN, is_open))
        .class((ClassName::STAGE_PASSED, passed))
        .attr("role", "tab")
        .attr("aria-selected", move || is_open().to_string())
        .attr("tabindex", move || tern!(is_open(), "0", "-1"))
        .on(ev::click, move |_| state.stage.set(phase))
        .child((track, text))
}

/// The line into a stage from the one before it; it fills in once the stage
/// is passed. The lines take the width the stages leave, so the first and
/// last stages sit at the edges of the timeline.
///
/// # Arguments
///
/// * `state` - The state.
/// * `phase` - The phase of the stage the line leads to.
///
/// # Returns
///
/// The view.
fn stage_link(state: AppState, phase: Phase) -> impl IntoView
{
    span()
        .class(ClassName::STAGE_LINK)
        .class((ClassName::STAGE_LINK_PASSED, move || is_passed(state, phase)))
        .attr("aria-hidden", "true")
}

/// How the phase of a stage went: as compiled, and for the run a warning
/// once the session traps. The ops run when go or auto run starts them,
/// never to work out a status.
///
/// # Arguments
///
/// * `state` - The state.
/// * `phase` - The phase of the stage.
///
/// # Returns
///
/// The status.
fn stage_status(state: AppState, phase: Phase) -> Status
{
    let status = state.compiled.with(|c| c.status(phase));
    let trapped = phase == Phase::Run && state.debug.with(|s| s.trap().is_some());
    tern!(trapped && status == Status::Clean, Status::Warned, status)
}

/// Whether a stage comes up to the open one.
///
/// # Arguments
///
/// * `state` - The state.
/// * `phase` - The phase of the stage.
///
/// # Returns
///
/// `true` for the open stage and those before it.
fn is_passed(state: AppState, phase: Phase) -> bool
{
    phase as usize <= state.stage.get() as usize
}

/// Keep the open stage in view when the timeline scrolls on a narrow pane.
///
/// # Arguments
///
/// * `state` - The state.
fn follow_stage(state: AppState)
{
    Effect::new(move |_| {
        let id = stage_id(state.stage.get());
        let Some(el) = document().get_element_by_id(&id)
        else
        {
            return;
        };
        let options = ScrollIntoViewOptions::new();
        options.set_block(ScrollLogicalPosition::Nearest);
        options.set_inline(ScrollLogicalPosition::Nearest);
        el.scroll_into_view_with_scroll_into_view_options(&options);
    });
}

/// Element id of a stage button.
///
/// # Arguments
///
/// * `phase` - The phase.
///
/// # Returns
///
/// The class of the dot of a stage in a status.
///
/// # Arguments
///
/// * `status` - How the phase went.
///
/// # Returns
///
/// The class.
fn status_class(status: Status) -> &'static str
{
    match status
    {
        Status::Clean => ClassName::STAGE_CLEAN,
        Status::Warned => ClassName::STAGE_WARNED,
        Status::Failed => ClassName::STAGE_FAILED,
        Status::Skipped => ClassName::STAGE_SKIPPED,
    }
}

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
