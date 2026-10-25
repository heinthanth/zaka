use bon::Builder;
use codee::string::FromToStringCodec;
use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::div;
use leptos::html::h3;
use leptos::html::nav;
use leptos::html::p;
use leptos::html::pre;
use leptos::html::section;
use leptos::html::span;
use leptos::html::table;
use leptos::html::tbody;
use leptos::html::td;
use leptos::html::tr;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_use_event_listener::use_event_listener;
use leptos_use_interval_fn::UseIntervalFnOptions;
use leptos_use_interval_fn::use_interval_fn_with_options;
use leptos_use_storage::UseStorageOptions;
use leptos_use_storage::use_local_storage_with_options;
use leptos_use_window::use_window;
use strum::IntoEnumIterator;
use web_sys::ScrollIntoViewOptions;
use web_sys::ScrollLogicalPosition;
use zaka_web_ui_common::controls::ClassName as Controls;
use zaka_web_ui_common::controls::dropdown::Choice;
use zaka_web_ui_common::controls::dropdown::Dropdown;
use zaka_web_ui_common::controls::dropdown::dropdown;
use zaka_web_ui_common::icon;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::compile::asm::AsmFunc;
use crate::compile::spans::Range;
use crate::components::asm_table::Gutter;
use crate::components::asm_table::asm_table;
use crate::components::asm_table::op_row_id;
use crate::components::check_view::skipped;
use crate::components::splitter::Axis;
use crate::components::splitter::Split;
use crate::components::splitter::splitter;
use crate::components::table::ClassName as Table;
use crate::components::table::Column;
use crate::components::table::cell;
use crate::components::table::table_head;
use crate::debug::FrameBlock;
use crate::debug::Registers;
use crate::debug::RunStatus;
use crate::debug::Session;
use crate::debug::Speed;
use crate::debug::ValueRow;
use crate::state::AppState;
use crate::state::split::DEFAULT_RUN_CODE_SHARE;
use crate::state::split::DEFAULT_RUN_STATE_SHARE;
use crate::state::split::share_fr;
use crate::state::tour::TOUR;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/run_view/mod.sass");

/// Sheet of the run view.
static SHEET: ComponentStyle = ComponentStyle::global_css("zaka-run-view", STYLE_SHEET);

/// The most ops a batch of a run takes; one batch runs a frame.
const BATCH: u32 = 50_000;

/// The time between two batches of a run, in ms.
const BATCH_MS: u64 = 16;

/// Local storage name of the ops share of the run view width.
const CODE_SHARE_STORE: &str = "zaka-explorer-run-code-share";

/// Local storage name of the machine state share of the side column.
const STATE_SHARE_STORE: &str = "zaka-explorer-run-state-share";

/// Columns of the stack.
const STORE_COLUMNS: [Column; 5] = [
    ("explorer.col.addr", Table::CELL_NUM, "explorer.col.addr_note"),
    ("explorer.col.slot", Table::CELL_NUM, ""),
    ("explorer.col.name", "", ""),
    ("explorer.col.kind", "", ""),
    ("explorer.col.value", "", ""),
];

/// Columns of the globals.
const VALUE_COLUMNS: [Column; 4] = [
    ("explorer.col.slot", Table::CELL_NUM, ""),
    ("explorer.col.name", "", ""),
    ("explorer.col.kind", "", ""),
    ("explorer.col.value", "", ""),
];

/// The run of the generated ops: its output, or in debug mode a debugger
/// with controls to step, step over, step out, go on to a break point,
/// start again and step on a timer; the ops of the call it shows with the
/// next op marked; and the machine state and the output. Or a note when
/// code generation did not run.
///
/// # Returns
///
/// The view.
pub fn run_view() -> impl IntoView
{
    let state = use_state();
    let emitted = Memo::new(move |_| state.compiled.with(|c| c.emitted));
    let content = move || {
        let title = t!("explorer.run.skipped_title");
        let note = t!("explorer.run.skipped_note");
        tern!(emitted.get(), runner(state).into_any(), skipped(title, note).into_any())
    };
    let view = div()
        .class(ClassName::RUN_VIEW)
        .class((ClassName::RUN_VIEW_SKIPPED, move || !emitted.get()))
        .child(content);
    (sheet(&SHEET), view)
}

/// What drives a run, which the bar shares in both modes.
#[derive(Clone, Copy, Builder)]
struct Drive
{
    /// Whether debug mode is on.
    debug: Signal<bool>,
    /// Turn debug mode on or off.
    set_debug: WriteSignal<bool>,
    /// Whether ops run in batches, one batch a frame.
    going: RwSignal<bool>,
    /// Whether the timed stepping is on.
    auto: RwSignal<bool>,
}

impl Drive
{
    /// Halt the batches and the timed stepping.
    fn halt(self)
    {
        self.going.set(false);
        self.auto.set(false);
    }
}

/// The output alone, or the debugger in debug mode; the mode switch sits at
/// the left of the bar in both, after go and stop outside debug mode.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn runner(state: AppState) -> impl IntoView
{
    let debug = state.run_debug;
    let drive = Drive::builder()
        .debug(debug)
        .set_debug(state.set_run_debug)
        .going(RwSignal::new(false))
        .auto(state.auto_step)
        .build();
    wire_going(state, drive);
    follow_mode(state, drive);
    move || {
        let on = debug.get();
        let switch = debug_switch(drive).into_any();
        tern!(
            on,
            debugger(state, drive, switch).into_any(),
            plain_run(state, drive, switch).into_any()
        )
    }
}

/// Run the ops in batches while a run goes on, one batch a frame, so the
/// page keeps responding and the run can be stopped. Outside debug mode a
/// new session starts to run on its own; in debug mode new ops halt the
/// run.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
fn wire_going(state: AppState, drive: Drive)
{
    let batch = move || {
        let breaks = drive.debug.get_untracked();
        let run = |s: &mut Session| s.run_for(BATCH, breaks);
        let go_on = state.debug.try_update(run).unwrap_or(false);
        do_if!(!go_on, drive.going.set(false));
    };
    let options = UseIntervalFnOptions::default().immediate(false);
    let timer = use_interval_fn_with_options(batch, BATCH_MS, options);
    let (resume, pause) = (timer.resume, timer.pause);
    Effect::new(move |_| tern!(drive.going.get(), resume(), pause()));
    Effect::new(move |_| {
        let fresh = state.debug.with(|s| s.is_live() && s.status() == RunStatus::Start);
        let idle = !drive.debug.get_untracked() && !drive.going.get_untracked();
        do_if!(fresh && idle, drive.going.set(true));
    });
    // New ops halt the run, but the first pass is the mount, not new ops.
    Effect::new(move |was: Option<()>| {
        state.compiled.track();
        do_if!(was.is_some() && drive.debug.get_untracked(), drive.halt());
    });
}

/// Act on a switch of debug mode, from the switch or the tour. Turning it
/// on goes back to the first op; turning it off runs on to the end, so the
/// output is whole.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
fn follow_mode(state: AppState, drive: Drive)
{
    Effect::new(move |was: Option<bool>| {
        let on = drive.debug.get();
        ret_if!(was.is_none_or(|w| w == on), on);
        drive.halt();
        do_if!(on, state.debug.update(Session::reset));
        let go_on = !on && state.debug.with_untracked(Session::running);
        do_if!(go_on, drive.going.set(true));
        on
    });
}

/// Run the program. Outside debug mode it runs again from the first op; in
/// debug mode it goes on to the next break point, from the first op once
/// the run has ended.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
fn go(state: AppState, drive: Drive)
{
    drive.auto.set(false);
    let ended = state.debug.with_untracked(|s| !s.running());
    do_if!(ended || !drive.debug.get_untracked(), state.debug.update(Session::reset));
    drive.going.set(true);
}

/// Stop the program before its end and go back to the first op.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
fn stop(state: AppState, drive: Drive)
{
    drive.halt();
    state.debug.update(Session::stop);
}

/// The go button: it runs the program, or in debug mode goes on to the
/// next break point.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
///
/// # Returns
///
/// The view.
fn go_button(state: AppState, drive: Drive) -> impl IntoView
{
    let going = drive.going;
    let name = move || {
        let debug = drive.debug.get();
        tern!(debug, t!("explorer.run.resume"), t!("explorer.run.go"))
    };
    button()
        .class(format!("{} {}", Controls::ICON_BUTTON, ClassName::RUN_GO))
        .title(name)
        .attr("aria-label", name)
        .attr("disabled", move || going.get().then_some(""))
        .on(ev::click, move |_| go(state, drive))
        .child(icon::start())
}

/// The stop button; it can be pressed while a run goes on.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
///
/// # Returns
///
/// The view.
fn stop_button(state: AppState, drive: Drive) -> impl IntoView
{
    let paused = move || state.debug.with(|s| s.status() == RunStatus::Paused);
    let can_stop = move || drive.going.get() || drive.auto.get() || paused();
    let name = t!("explorer.run.stop");
    button()
        .class(format!("{} {}", Controls::ICON_BUTTON, ClassName::RUN_STOP))
        .title(name.clone())
        .attr("aria-label", name)
        .attr("disabled", move || (!can_stop()).then_some(""))
        .on(ev::click, move |_| stop(state, drive))
        .child(icon::stop())
}

/// The switch of debug mode; [`follow_mode`] acts on it.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
///
/// # Returns
///
/// The view.
fn debug_switch(drive: Drive) -> impl IntoView
{
    let debug = drive.debug;
    let flip = move |_| drive.set_debug.set(!debug.get_untracked());
    button()
        .class(ClassName::RUN_DEBUG)
        .attr(TOUR, "debug")
        .class((ClassName::RUN_DEBUG_ON, move || debug.get()))
        .title(t!("explorer.run.debug_note"))
        .attr("aria-pressed", move || debug.get().to_string())
        .on(ev::click, flip)
        .child((icon::bug(), span().child(t!("explorer.run.debug"))))
}

/// The status chip: running while ops run on a timer, else where the
/// session stands.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
///
/// # Returns
///
/// The view.
fn status_chip(state: AppState, drive: Drive) -> impl IntoView
{
    let status = move || {
        let on = drive.going.get() || drive.auto.get();
        tern!(on, RunStatus::Running, state.debug.with(Session::status))
    };
    let class = move || format!("{} {}", ClassName::RUN_STATUS, status_class(status()));
    span().class(class).attr("role", "status").child(move || status().label())
}

/// The class of the status chip in a status.
///
/// # Arguments
///
/// * `status` - The status of the run.
///
/// # Returns
///
/// The class, blank before the first op.
fn status_class(status: RunStatus) -> &'static str
{
    match status
    {
        RunStatus::Start => "",
        RunStatus::Running => ClassName::RUN_STATUS_RUNNING,
        RunStatus::Paused => ClassName::RUN_STATUS_PAUSED,
        RunStatus::Stopped => ClassName::RUN_STATUS_STOPPED,
        RunStatus::Finished => ClassName::RUN_STATUS_FINISHED,
        RunStatus::Trapped => ClassName::RUN_STATUS_TRAPPED,
    }
}

/// The bar with go, stop, the mode switch and the status over the
/// output.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
/// * `switch` - The mode switch.
///
/// # Returns
///
/// The view.
fn plain_run(state: AppState, drive: Drive, switch: AnyView) -> impl IntoView
{
    let head = (go_button(state, drive), stop_button(state, drive), switch);
    let bar = nav()
        .class(ClassName::RUN_CONTROLS)
        .attr(TOUR, "controls")
        .attr("role", "toolbar")
        .attr("aria-label", t!("explorer.run.label"))
        .child((head, status_chip(state, drive)));
    (bar, out_pane(state, true))
}

/// The controls, the ops and the panels.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
/// * `switch` - The mode switch.
///
/// # Returns
///
/// The view.
fn debugger(state: AppState, drive: Drive, switch: AnyView) -> impl IntoView
{
    let func = Memo::new(move |_| state.debug.with(Session::viewed_func));
    let marked = Memo::new(move |_| state.debug.with(Session::marked_pc));
    let speed = RwSignal::new(Speed::default());
    wire_auto(state, drive.auto, speed);
    wire_shortcuts(state, drive);
    follow_pc(state, func, marked);
    (controls(state, drive, switch, speed), tiles(state, func, marked))
}

/// The panes as tiles: the ops on the left; the machine state over the
/// output on the right; splitters between them.
///
/// # Arguments
///
/// * `state` - The state.
/// * `func` - The function the ops pane shows.
/// * `marked` - The marked op.
///
/// # Returns
///
/// The view.
fn tiles(
    state: AppState, func: Memo<Option<u32>>, marked: Memo<Option<u32>>,
) -> impl IntoView
{
    let grid = NodeRef::<html::Div>::new();
    let column = NodeRef::<html::Div>::new();
    let (code_share, set_code_share) =
        stored_share(CODE_SHARE_STORE, DEFAULT_RUN_CODE_SHARE);
    let (state_share, set_state_share) =
        stored_share(STATE_SHARE_STORE, DEFAULT_RUN_STATE_SHARE);
    let between = Split::builder()
        .axis(Axis::Columns)
        .container(grid)
        .share(code_share)
        .set_share(set_code_share)
        .initial(DEFAULT_RUN_CODE_SHARE)
        .label("explorer.run.resize_code")
        .build();
    let above = Split::builder()
        .axis(Axis::Rows)
        .container(column)
        .share(state_share)
        .set_share(set_state_share)
        .initial(DEFAULT_RUN_STATE_SHARE)
        .label("explorer.run.resize_output")
        .build();
    let side = div()
        .class(ClassName::RUN_COLUMN)
        .node_ref(column)
        .style(("--first-share", move || share_fr(state_share.get())))
        .style(("--second-share", move || share_fr(1.0 - state_share.get())))
        .child((state_pane(state), splitter(above), out_pane(state, false)));
    div()
        .class(ClassName::RUN_TILES)
        .node_ref(grid)
        .style(("--first-share", move || share_fr(code_share.get())))
        .style(("--second-share", move || share_fr(1.0 - code_share.get())))
        .child((code_pane(state, func, marked), splitter(between), side))
}

/// A split share kept in local storage.
///
/// # Arguments
///
/// * `store` - The local storage name.
/// * `initial` - The share before one is kept.
///
/// # Returns
///
/// The share and its setter.
fn stored_share(store: &'static str, initial: f64) -> (Signal<f64>, WriteSignal<f64>)
{
    let options = UseStorageOptions::default().initial_value(initial);
    let (share, set_share, _) =
        use_local_storage_with_options::<f64, FromToStringCodec>(store, options);
    (share, set_share)
}

/// Run a command on the session; it halts the run.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
/// * `act` - The command.
fn command(state: AppState, drive: Drive, act: fn(&mut Session))
{
    drive.halt();
    state.debug.update(act);
}

/// The bar of controls: the mode switch; go on to a break point, stop and
/// start again; the step buttons; the timed stepping, the speed picker,
/// the status and the op count.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
/// * `switch` - The mode switch.
/// * `speed` - The speed of the timed stepping.
///
/// # Returns
///
/// The view.
fn controls(
    state: AppState, drive: Drive, switch: AnyView, speed: RwSignal<Speed>,
) -> impl IntoView
{
    let running: Signal<bool> =
        Memo::new(move |_| state.debug.with(Session::running)).into();
    let act = move |f: fn(&mut Session)| move || command(state, drive, f);
    let ends = (
        go_button(state, drive),
        stop_button(state, drive),
        control(
            "explorer.run.reset",
            icon::restart(),
            Signal::stored(true),
            act(Session::reset),
        ),
    );
    let buttons = (
        control("explorer.run.step", icon::step(), running, act(Session::step)),
        control(
            "explorer.run.step_over",
            icon::step_over(),
            running,
            act(Session::step_over),
        ),
        control(
            "explorer.run.step_out",
            icon::step_out(),
            running,
            act(Session::step_out),
        ),
        rule(),
        auto_button(drive.auto, running),
    );
    let count =
        move || t!("explorer.run.steps", count = state.debug.with(Session::steps));
    nav()
        .class(ClassName::RUN_CONTROLS)
        .attr(TOUR, "controls")
        .attr("role", "toolbar")
        .attr("aria-label", t!("explorer.run.label"))
        .child((
            switch,
            rule(),
            ends,
            rule(),
            buttons,
            speed_picker(speed),
            status_chip(state, drive),
            span().class(ClassName::RUN_STEPS).child(count),
        ))
}

/// A rule between groups of the bar.
///
/// # Returns
///
/// The view.
fn rule() -> impl IntoView
{
    span().class(ClassName::RUN_RULE).attr("aria-hidden", "true")
}

/// One button of the bar.
///
/// # Arguments
///
/// * `id` - Locale id of its name, which shows as a tooltip.
/// * `art` - The icon.
/// * `enabled` - Whether it can be pressed.
/// * `on_press` - What a press does.
///
/// # Returns
///
/// The view.
fn control(
    id: &'static str, art: impl IntoView + 'static, enabled: Signal<bool>,
    on_press: impl Fn() + 'static,
) -> impl IntoView
{
    button()
        .class(Controls::ICON_BUTTON)
        .title(t!(id.to_string()))
        .attr("aria-label", t!(id.to_string()))
        .attr("disabled", move || (!enabled.get()).then_some(""))
        .on(ev::click, move |_| on_press())
        .child(art)
}

/// The button that starts and pauses the timed stepping.
///
/// # Arguments
///
/// * `auto` - Whether the timed stepping is on.
/// * `running` - Whether ops remain to run.
///
/// # Returns
///
/// The view.
fn auto_button(auto: RwSignal<bool>, running: Signal<bool>) -> impl IntoView
{
    let name = move || {
        tern!(auto.get(), t!("explorer.run.auto_pause"), t!("explorer.run.auto_start"))
    };
    let art = move || {
        tern!(auto.get(), icon::pause().into_any(), icon::timed_start().into_any())
    };
    button()
        .class(Controls::ICON_BUTTON)
        .class((Controls::ICON_BUTTON_ON, move || auto.get()))
        .title(name)
        .attr("aria-label", name)
        .attr("aria-pressed", move || auto.get().to_string())
        .attr("disabled", move || (!running.get()).then_some(""))
        .on(ev::click, move |_| auto.update(|on| *on = !*on))
        .child(art)
}

/// The picker of the speed of the timed stepping.
///
/// # Arguments
///
/// * `speed` - The speed.
///
/// # Returns
///
/// The view.
fn speed_picker(speed: RwSignal<Speed>) -> impl IntoView
{
    let choice = |s: Speed| {
        let value: &'static str = s.into();
        Choice::builder().value(value).label(s.label()).build()
    };
    let value = Signal::derive(move || <&'static str>::from(speed.get()).to_string());
    let on_pick =
        Callback::new(move |v: String| speed.set(v.parse().unwrap_or_default()));
    let spec = Dropdown::builder()
        .choices(Speed::iter().map(choice).collect())
        .value(value)
        .on_pick(on_pick)
        .label(t!("explorer.run.speed"))
        .build();
    div()
        .class(ClassName::RUN_SPEED)
        .title(t!("explorer.run.speed"))
        .child(dropdown(spec))
}

/// Step on a timer while the timed stepping is on; it turns off at the end
/// of the run, at a break point, when the ops change and when the debugger
/// closes. It can be on before the debugger opens, as the tour sets it.
///
/// # Arguments
///
/// * `state` - The state.
/// * `auto` - Whether the timed stepping is on.
/// * `speed` - The speed.
fn wire_auto(state: AppState, auto: RwSignal<bool>, speed: RwSignal<Speed>)
{
    let tick = move || {
        let go_on = state.debug.try_update(Session::tick).unwrap_or(false);
        do_if!(!go_on, auto.set(false));
    };
    let pace = Signal::derive(move || speed.get().millis());
    let options = UseIntervalFnOptions::default().immediate(false);
    let timer = use_interval_fn_with_options(tick, pace, options);
    let (resume, pause) = (timer.resume, timer.pause);
    Effect::new(move |_| tern!(auto.get(), resume(), pause()));
    Effect::new(move |was: Option<()>| {
        state.compiled.track();
        do_if!(was.is_some(), auto.set(false));
    });
    on_cleanup(move || auto.set(false));
}

/// Debugger shortcuts while the view is open: F5 goes on to a break point,
/// F10 steps over, F11 steps and Shift+F11 steps out.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
fn wire_shortcuts(state: AppState, drive: Drive)
{
    let _ = use_event_listener(use_window(), ev::keydown, move |e| {
        on_press(state, drive, &e)
    });
}

/// Run the command of a debugger shortcut.
///
/// # Arguments
///
/// * `state` - The state.
/// * `drive` - What drives the run.
/// * `e` - The press.
fn on_press(state: AppState, drive: Drive, e: &ev::KeyboardEvent)
{
    let act: Option<fn(&mut Session)> = match (e.key().as_str(), e.shift_key())
    {
        ("F5", false) => None,
        ("F10", false) => Some(Session::step_over),
        ("F11", false) => Some(Session::step),
        ("F11", true) => Some(Session::step_out),
        _ => return,
    };
    e.prevent_default();
    match act
    {
        Some(f) => command(state, drive, f),
        None => go(state, drive),
    }
}

/// Keep the marked op in view and point the editor at its source. A view
/// that stops pointing hands the pointer back to the marked op.
///
/// # Arguments
///
/// * `state` - The state.
/// * `func` - The function the ops pane shows.
/// * `marked` - The marked op.
fn follow_pc(state: AppState, func: Memo<Option<u32>>, marked: Memo<Option<u32>>)
{
    let range = Memo::new(move |_| op_range(state, func.get()?, marked.get()?));
    Effect::new(move |_| {
        state.point(range.get());
        let (Some(f), Some(pc)) = (func.get(), marked.get())
        else
        {
            return;
        };
        scroll_to(&op_row_id(f, pc));
    });
    Effect::new(move |_| {
        ret_if!(state.pointer.get().is_some());
        do_if!(let Some(r) = range.get_untracked(), state.point(Some(r)));
    });
    on_cleanup(move || state.point(None));
}

/// The source region of an op.
///
/// # Arguments
///
/// * `state` - The state.
/// * `func` - The function index.
/// * `pc` - The op index.
///
/// # Returns
///
/// The region, or `None` for an op without one.
fn op_range(state: AppState, func: u32, pc: u32) -> Option<Range>
{
    state.compiled.with(|c| c.asm.get(func as usize)?.rows.get(pc as usize)?.range)
}

/// Bring an element in view, when it is there.
///
/// # Arguments
///
/// * `id` - The element id.
fn scroll_to(id: &str)
{
    let Some(row) = document().get_element_by_id(id)
    else
    {
        return;
    };
    let options = ScrollIntoViewOptions::new();
    options.set_block(ScrollLogicalPosition::Nearest);
    row.scroll_into_view_with_scroll_into_view_options(&options);
}

/// The ops of all functions, each with its break point gutter; the next op
/// of the call the panels show is marked.
///
/// # Arguments
///
/// * `state` - The state.
/// * `func` - The function of the call the panels show.
/// * `marked` - The marked op.
///
/// # Returns
///
/// The view.
fn code_pane(
    state: AppState, func: Memo<Option<u32>>, marked: Memo<Option<u32>>,
) -> impl IntoView
{
    let content = move || {
        let funcs = state.compiled.with(|c| c.asm.clone());
        let ops = move |asm: AsmFunc| func_ops(state, asm, func, marked);
        funcs.into_iter().map(ops).collect_view()
    };
    section()
        .class(format!("{} {}", ClassName::RUN_TILE, ClassName::RUN_CODE))
        .attr(TOUR, "ops")
        .child(content)
}

/// The ops of one function under its head.
///
/// # Arguments
///
/// * `state` - The state.
/// * `asm` - The function.
/// * `func` - The function of the call the panels show.
/// * `marked` - The marked op.
///
/// # Returns
///
/// The view.
fn func_ops(
    state: AppState, asm: AsmFunc, func: Memo<Option<u32>>, marked: Memo<Option<u32>>,
) -> impl IntoView
{
    let f = asm.index;
    let shown = move || func.get() == Some(f);
    let mark = Signal::derive(move || shown().then(|| marked.get()).flatten());
    let breaks = Memo::new(move |_| state.debug.with(|s| s.breaks_in(f)));
    let toggle = Callback::new(move |pc| state.debug.update(|s| s.toggle_break(f, pc)));
    let gutter = Gutter::builder().breaks(breaks.into()).toggle(toggle).build();
    let head = h3().class(ClassName::ASM_HEAD).child(asm.head());
    div()
        .class(ClassName::ASM_FUNC)
        .class((ClassName::ASM_FUNC_SHOWN, shown))
        .child((head, asm_table(state, asm, mark, Some(gutter))))
}

/// The registers, the stack with its frames and the globals.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn state_pane(state: AppState) -> impl IntoView
{
    let globals = move || {
        let rows = state.debug.with(Session::globals);
        (!rows.is_empty())
            .then(|| (tile_title("explorer.run.globals"), value_table('g', rows)))
    };
    section()
        .class(format!("{} {}", ClassName::RUN_TILE, ClassName::RUN_STATE))
        .attr(TOUR, "machine")
        .child((
            tile_title("explorer.run.registers"),
            registers_view(state),
            tile_title("explorer.run.frames"),
            frames_view(state),
            globals,
        ))
}

/// The output under its title.
///
/// # Arguments
///
/// * `state` - The state.
/// * `solo` - Whether it fills the view, outside debug mode.
///
/// # Returns
///
/// The view.
fn out_pane(state: AppState, solo: bool) -> impl IntoView
{
    section()
        .class(format!("{} {}", ClassName::RUN_TILE, ClassName::RUN_OUT))
        .attr(TOUR, "output")
        .class((ClassName::RUN_SOLO, solo))
        .child((tile_title("explorer.run.output"), output_view(state)))
}

/// The title of a part of a tile.
///
/// # Arguments
///
/// * `id` - Locale id of the title.
///
/// # Returns
///
/// The view.
fn tile_title(id: &'static str) -> impl IntoView
{
    h3().class(Table::TABLE_TITLE).child(t!(id.to_string()))
}

/// The registers: the program counter, the stack pointer and the base
/// pointer.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn registers_view(state: AppState) -> impl IntoView
{
    let regs = Memo::new(move |_| state.debug.with(Session::registers));
    let text = move |f: fn(&Registers) -> Option<String>| {
        move || regs.with(f).unwrap_or_else(|| "-".to_string())
    };
    let pc = text(|r| r.pc.map(|pc| pc.to_string()));
    let sp = text(|r| Some(r.sp.to_string()));
    let bp = text(|r| r.bp.map(|b| b.to_string()));
    div().class(ClassName::RUN_REGS).child((
        register("PC", "explorer.run.pc_note", pc),
        register("SP", "explorer.run.sp_note", sp),
        register("BP", "explorer.run.bp_note", bp),
    ))
}

/// One register: its name and its value.
///
/// # Arguments
///
/// * `name` - The name.
/// * `note` - Locale id of what it holds, which shows as a tooltip.
/// * `value` - The value.
///
/// # Returns
///
/// The view.
fn register(
    name: &'static str, note: &'static str,
    value: impl Fn() -> String + Send + Sync + 'static,
) -> impl IntoView
{
    div().class(ClassName::RUN_REG).title(t!(note.to_string())).child((
        span().class(ClassName::RUN_REG_NAME).child(name),
        span().class(ClassName::RUN_REG_VALUE).child(value),
    ))
}

/// The stack: the frame of each call in progress, innermost and highest
/// first. A frame holds the locals of its call and over them the values
/// the call pushed. The stack pointer marks the free slot over the top, the
/// base pointer the frame of the innermost call; a click shows that call.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn frames_view(state: AppState) -> impl IntoView
{
    move || {
        let blocks = state.debug.with(Session::frame_blocks);
        let note = p().class(Table::VIEW_NOTE).child(t!("explorer.run.no_frames"));
        ret_if!(blocks.is_empty(), note.into_any());
        let sp = state.debug.with(|s| s.registers().sp);
        let free = tr().class(ClassName::STACK_FREE).child((
            cell(Table::CELL_NUM, sp),
            td().attr("colspan", (STORE_COLUMNS.len() - 1).to_string())
                .child(span().class(ClassName::REG_TAG).child("sp")),
        ));
        let groups = blocks.into_iter().enumerate();
        let rows = groups.map(|(i, b)| frame_rows(state, b, i == 0)).collect_view();
        let head = table_head(&STORE_COLUMNS);
        let rows = tbody().child((free, rows));
        table().class(Table::PHASE_TABLE).child((head, rows)).into_any()
    }
}

/// The rows of one frame, highest first: its head row, the values its
/// call pushed, then its locals.
///
/// # Arguments
///
/// * `state` - The state.
/// * `block` - The frame.
/// * `top` - Whether it is the frame of the innermost call, which the base
///   pointer marks.
///
/// # Returns
///
/// The view.
fn frame_rows(state: AppState, block: FrameBlock, top: bool) -> impl IntoView
{
    let (call, start) = (block.call, block.start);
    let index = call.index;
    let viewed = move || state.debug.with(Session::viewed) == index;
    let view = move |_| state.debug.update(|s| s.view_frame(index));
    let tag = top.then(|| span().class(ClassName::REG_TAG).child("bp"));
    let meta = t!("explorer.run.frame_meta", pc = call.addr, base = start);
    let label = (
        span()
            .class(ClassName::FRAME_NAME)
            .child(format!("#{index} f{} {}", call.func, call.name)),
        tag,
        span().class(ClassName::FRAME_META).child(meta),
    );
    let head = tr()
        .class(ClassName::FRAME_ROW)
        .class((Table::ROW_ACTIVE, viewed))
        .title(t!("explorer.run.view_call"))
        .on(ev::click, view)
        .child(td().attr("colspan", STORE_COLUMNS.len().to_string()).child(label));
    let slot = move |at: usize, label: String, v: ValueRow| {
        tr().class((ClassName::ROW_VIEWED, viewed)).on(ev::click, view).child((
            cell(Table::CELL_NUM, at),
            cell(Table::CELL_NUM, label),
            cell(Table::CELL_LEXEME, v.name),
            cell(Table::CELL_KIND, v.kind),
            cell(Table::CELL_LEXEME, v.value),
        ))
    };
    let over = start + block.slots.len();
    let temp = move |v: ValueRow| {
        let i = v.index as usize;
        slot(over + i, format!("t{i}"), v)
    };
    let local = move |v: ValueRow| {
        let i = v.index as usize;
        slot(start + i, format!("l{i}"), v)
    };
    let temps = block.temps.into_iter().rev().map(temp).collect_view();
    let locals = block.slots.into_iter().rev().map(local).collect_view();
    (head, temps, locals)
}

/// A table of slots and their values.
///
/// # Arguments
///
/// * `prefix` - The slot prefix, `g` or `l`.
/// * `rows` - The slots.
///
/// # Returns
///
/// The view.
fn value_table(prefix: char, rows: Vec<ValueRow>) -> impl IntoView
{
    let row = move |v: ValueRow| {
        tr().child((
            cell(Table::CELL_NUM, format!("{prefix}{}", v.index)),
            cell(Table::CELL_LEXEME, v.name),
            cell(Table::CELL_KIND, v.kind),
            cell(Table::CELL_LEXEME, v.value),
        ))
    };
    let rows = rows.into_iter().map(row).collect_view();
    table()
        .class(Table::PHASE_TABLE)
        .child((table_head(&VALUE_COLUMNS), tbody().child(rows)))
}

/// What the program wrote so far, and the trap when the run stopped before
/// its end.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn output_view(state: AppState) -> impl IntoView
{
    let text = Memo::new(move |_| state.debug.with(Session::output));
    let blank = move || text.with(String::is_empty);
    let shown =
        move || tern!(blank(), t!("explorer.run.no_output").to_string(), text.get());
    let end = NodeRef::<html::Pre>::new();
    // Like a terminal, the output keeps its last line in view.
    Effect::new(move |_| {
        text.track();
        do_if!(let Some(el) = end.get(), el.set_scroll_top(el.scroll_height()));
    });
    let output = pre()
        .class(ClassName::RUN_OUTPUT)
        .class((ClassName::RUN_OUTPUT_BLANK, blank))
        .node_ref(end)
        .child(shown);
    let trap = move || {
        let trap = state.debug.with(Session::trap);
        trap.map(|t| p().class(ClassName::RUN_TRAP).attr("role", "status").child(t))
    };
    (output, trap)
}
