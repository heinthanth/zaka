use bon::Builder;
use codee::string::FromToStringCodec;
use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::dialog;
use leptos::html::div;
use leptos::html::h2;
use leptos::html::p;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_use_element_bounding::use_element_bounding;
use leptos_use_storage::use_local_storage;
use leptos_use_window_size::use_window_size;
use web_sys::Element;
use zaka_web_ui_common::controls::modal::follow_open;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::compile::Phase;
use crate::compile::samples::find_sample;
use crate::debug::Session;
use crate::state::AppState;
use crate::state::tour::HOLE_PAD;
use crate::state::tour::Rect;
use crate::state::tour::Snapshot;
use crate::state::tour::Step;
use crate::state::tour::TOUR_SAMPLE;
use crate::state::tour::place;
use crate::state::tour::steps;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/tour/mod.sass");

/// Sheet of the tour.
static SHEET: ComponentStyle = ComponentStyle::global_css("zaka-tour", STYLE_SHEET);

/// Local storage name of the mark that the tour has opened once.
const SEEN_STORE: &str = "zaka-explorer-tour-seen";

/// Element id of the step title, which names the dialog.
const TITLE_ID: &str = "tour-title";

/// The element a step points at.
type Target = RwSignal<Option<Element>, LocalStorage>;

/// Where the tour stands.
#[derive(Clone, Copy, Builder)]
struct Walk
{
    /// The steps.
    steps: StoredValue<Vec<Step>>,
    /// Index of the open step.
    at: RwSignal<usize>,
    /// Whether the scene of the tour is set up: after the snapshot, before
    /// the restore.
    live: RwSignal<bool>,
    /// What the tour changes, as it was when it opened.
    saved: StoredValue<Option<Snapshot>>,
}

impl Walk
{
    /// The open step.
    ///
    /// # Returns
    ///
    /// The step.
    fn step(self) -> Step { self.steps.with_value(|s| s[self.at.get()]) }

    /// The number of steps.
    ///
    /// # Returns
    ///
    /// The count.
    fn count(self) -> usize { self.steps.with_value(Vec::len) }

    /// Whether the open step is the last one.
    ///
    /// # Returns
    ///
    /// `true` on the last step.
    fn at_end(self) -> bool { self.at.get() + 1 == self.count() }
}

/// The guided tour: a spotlight over the part a step points at and a card
/// with its text beside it. It walks through the page, each phase and the
/// debugger over a sample, then puts the text and the views back. It opens
/// on the first visit, and from the about panel.
///
/// # Returns
///
/// The view.
pub fn tour() -> impl IntoView
{
    let state = use_state();
    let walk = Walk::builder()
        .steps(StoredValue::new(steps()))
        .at(RwSignal::new(0))
        .live(RwSignal::new(false))
        .saved(StoredValue::new(None))
        .build();
    let (seen, set_seen, _) = use_local_storage::<bool, FromToStringCodec>(SEEN_STORE);
    open_first_time(state, seen);
    follow_tour(state, walk, set_seen);
    follow_scene(state, walk);
    let target: Target = RwSignal::new_local(None);
    follow_target(walk, target);
    let ring = ring_of(target);
    let host = NodeRef::<html::Dialog>::new();
    follow_open(state.tour, host);
    let view = dialog()
        .class(ClassName::TOUR)
        .node_ref(host)
        .attr("aria-labelledby", TITLE_ID)
        .on(ev::close, move |_| state.tour.set(false))
        .on(ev::keydown, move |e| on_press(state, walk, &e))
        .child((spotlight(ring), card(state, walk, ring)));
    (sheet(&SHEET), view)
}

/// Open the tour once the editor has loaded, when it has never opened.
///
/// # Arguments
///
/// * `state` - The state.
/// * `seen` - Whether the tour has opened before.
fn open_first_time(state: AppState, seen: Signal<bool>)
{
    Effect::new(move |_| {
        ret_if!(!state.mounted.get());
        ret_if!(seen.get_untracked() || state.tour.get_untracked());
        state.tour.set(true);
    });
}

/// Set up the tour as it opens and put things back as it closes.
///
/// # Arguments
///
/// * `state` - The state.
/// * `walk` - Where the tour stands.
/// * `set_seen` - Mark that the tour has opened.
fn follow_tour(state: AppState, walk: Walk, set_seen: WriteSignal<bool>)
{
    Effect::new(move |was: Option<bool>| {
        let on = state.tour.get();
        ret_if!(was.unwrap_or(false) == on, on);
        do_if!(on, set_seen.set(true), begin(state, walk));
        do_if!(!on, end(state, walk));
        on
    });
}

/// Keep what the tour changes, then load the sample on the first stage with
/// the editor open.
///
/// # Arguments
///
/// * `state` - The state.
/// * `walk` - Where the tour stands.
fn begin(state: AppState, walk: Walk)
{
    let snapshot = Snapshot::builder()
        .link(state.link())
        .editor_open(state.editor_open.get_untracked())
        .problems_open(state.problems_open.get_untracked())
        .debug_mode(state.debug_mode.get_untracked())
        .build();
    walk.saved.set_value(Some(snapshot));
    state.set_editor_open.set(true);
    state.stage.set(Phase::Segment);
    do_if!(let Some(sample) = find_sample(TOUR_SAMPLE), state.load(sample.text.into()));
    walk.at.set(0);
    walk.live.set(true);
}

/// Stop the timed stepping and put back what the tour changed.
///
/// # Arguments
///
/// * `state` - The state.
/// * `walk` - Where the tour stands.
fn end(state: AppState, walk: Walk)
{
    walk.live.set(false);
    state.auto_step.set(false);
    let Some(saved) = walk.saved.get_value()
    else
    {
        return;
    };
    walk.saved.set_value(None);
    state.set_debug_mode.set(saved.debug_mode);
    state.set_editor_open.set(saved.editor_open);
    state.set_problems_open.set(saved.problems_open);
    state.debug.update(Session::reset);
    state.open_link(saved.link);
}

/// Set up the scene of the open step: its stage, debug mode and the timed
/// stepping.
///
/// # Arguments
///
/// * `state` - The state.
/// * `walk` - Where the tour stands.
fn follow_scene(state: AppState, walk: Walk)
{
    Effect::new(move |_| {
        ret_if!(!walk.live.get());
        let step = walk.step();
        do_if!(let Some(phase) = step.stage, state.stage.set(phase));
        let flip = state.debug_mode.get_untracked() != step.debug;
        do_if!(flip, state.set_debug_mode.set(step.debug));
        state.auto_step.set(step.auto);
    });
}

/// Find the element the open step points at, a frame after the step opens,
/// once the views of its scene are in place.
///
/// # Arguments
///
/// * `walk` - Where the tour stands.
/// * `target` - The element.
fn follow_target(walk: Walk, target: Target)
{
    Effect::new(move |_| {
        let live = walk.live.get();
        let selector = walk.step().target.filter(|_| live);
        request_animation_frame(move || {
            let found = selector.and_then(|s| document().query_selector(s).ok()?);
            target.set(found);
        });
    });
}

/// The box of the spotlight: the target with some space around it, inside
/// the window.
///
/// # Arguments
///
/// * `target` - The element the open step points at.
///
/// # Returns
///
/// The box, or `None` when the step points at nothing.
fn ring_of(target: Target) -> Memo<Option<Rect>>
{
    let size = use_window_size();
    let bound = use_element_bounding(target);
    Memo::new(move |_| {
        ret_if!(target.with(Option::is_none), None);
        let rect = Rect::builder()
            .left(bound.left.get())
            .top(bound.top.get())
            .width(bound.width.get())
            .height(bound.height.get())
            .build();
        Some(rect.ring(HOLE_PAD, (size.width.get(), size.height.get())))
    })
}

/// The spotlight: a ring around the target whose shadow dims the rest of
/// the page. It glides from one target to the next; with none, it shrinks
/// to the middle and the whole page dims.
///
/// # Arguments
///
/// * `ring` - The box of the spotlight.
///
/// # Returns
///
/// The view.
fn spotlight(ring: Memo<Option<Rect>>) -> impl IntoView
{
    let px = move |side: fn(&Rect) -> f64, none: &'static str| {
        move || {
            ring.with(|r| r.as_ref().map(|r| format!("{}px", side(r))))
                .unwrap_or_else(|| none.to_string())
        }
    };
    div()
        .class(ClassName::TOUR_HOLE)
        .class((ClassName::TOUR_HOLE_NONE, move || ring.with(Option::is_none)))
        .attr("aria-hidden", "true")
        .style(("left", px(|r| r.left, "50%")))
        .style(("top", px(|r| r.top, "50%")))
        .style(("width", px(|r| r.width, "0px")))
        .style(("height", px(|r| r.height, "0px")))
}

/// The card of the open step: the progress, the count, the title and the
/// text over the buttons, placed beside the spotlight.
///
/// # Arguments
///
/// * `state` - The state.
/// * `walk` - Where the tour stands.
/// * `ring` - The box of the spotlight.
///
/// # Returns
///
/// The view.
fn card(state: AppState, walk: Walk, ring: Memo<Option<Rect>>) -> impl IntoView
{
    let size = use_window_size();
    let host = NodeRef::<html::Div>::new();
    let own = use_element_bounding(host);
    let spot = Memo::new(move |_| {
        let card = (own.width.get(), own.height.get());
        place(ring.get(), card, (size.width.get(), size.height.get()))
    });
    let count =
        move || t!("explorer.tour.count", at = walk.at.get() + 1, total = walk.count());
    let share = move || format!("{}%", (walk.at.get() + 1) * 100 / walk.count());
    let bar = div()
        .class(ClassName::TOUR_BAR)
        .attr("aria-hidden", "true")
        .child(span().class(ClassName::TOUR_FILL).style(("width", share)));
    let main = div().class(ClassName::TOUR_MAIN).attr("aria-live", "polite").child((
        span().class(ClassName::TOUR_COUNT).child(count),
        h2().id(TITLE_ID)
            .class(ClassName::TOUR_TITLE)
            .child(move || t!(walk.step().title.to_string())),
        p().class(ClassName::TOUR_TEXT).child(move || t!(walk.step().text.to_string())),
    ));
    div()
        .class(ClassName::TOUR_CARD)
        .node_ref(host)
        .style(("left", move || format!("{}px", spot.get().0)))
        .style(("top", move || format!("{}px", spot.get().1)))
        .child((bar, main, buttons(state, walk)))
}

/// The buttons of the card: skip on the left; back and next on the right.
/// Next reads done on the last step.
///
/// # Arguments
///
/// * `state` - The state.
/// * `walk` - Where the tour stands.
///
/// # Returns
///
/// The view.
fn buttons(state: AppState, walk: Walk) -> impl IntoView
{
    let skip = button()
        .class(ClassName::TOUR_SKIP)
        .on(ev::click, move |_| state.tour.set(false))
        .child(t!("explorer.tour.skip"));
    let back = button()
        .class(ClassName::TOUR_BACK)
        .attr("disabled", move || (walk.at.get() == 0).then_some(""))
        .on(ev::click, move |_| step_back(walk))
        .child(t!("explorer.tour.back"));
    let name = move || {
        tern!(walk.at_end(), t!("explorer.tour.done"), t!("explorer.tour.next"))
    };
    let next = button()
        .class(ClassName::TOUR_NEXT)
        .attr("autofocus", "")
        .on(ev::click, move |_| step_on(state, walk))
        .child(name);
    div().class(ClassName::TOUR_FOOT).child((
        skip,
        span().class(ClassName::TOUR_GAP),
        back,
        next,
    ))
}

/// Open the next step, or end the tour after the last one.
///
/// # Arguments
///
/// * `state` - The state.
/// * `walk` - Where the tour stands.
fn step_on(state: AppState, walk: Walk)
{
    let at = walk.at.get_untracked();
    ret_if!(at + 1 >= walk.count(), state.tour.set(false));
    walk.at.set(at + 1);
}

/// Open the step before, when there is one.
///
/// # Arguments
///
/// * `walk` - Where the tour stands.
fn step_back(walk: Walk) { walk.at.update(|at| *at = at.saturating_sub(1)) }

/// Move between the steps with the arrows.
///
/// # Arguments
///
/// * `state` - The state.
/// * `walk` - Where the tour stands.
/// * `e` - The press.
fn on_press(state: AppState, walk: Walk, e: &ev::KeyboardEvent)
{
    let ahead = match e.key().as_str()
    {
        "ArrowRight" => true,
        "ArrowLeft" => false,
        _ => return,
    };
    e.prevent_default();
    tern!(ahead, step_on(state, walk), step_back(walk));
}
