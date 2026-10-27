use leptos::ev;
use leptos::html::button;
use leptos::html::div;
use leptos::html::nav;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_use_event_listener::use_event_listener;
use leptos_use_interval_fn::UseIntervalFnOptions;
use leptos_use_interval_fn::use_interval_fn_with_options;
use leptos_use_window::use_window;
use strum::IntoEnumIterator;
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

use crate::compile::Phase;
use crate::compile::spans::Range;
use crate::compile::timing::time_text;
use crate::components::settings::compiler_modal;
use crate::debug::RunStatus;
use crate::debug::Session;
use crate::debug::Speed;
use crate::debug::octets;
use crate::state::AppState;
use crate::state::PanelTab;
use crate::state::tour::TOUR;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/debug_bar/mod.sass");

/// Sheet of the run controls.
static SHEET: ComponentStyle =
    ComponentStyle::global_css("zaka-debug-bar", STYLE_SHEET);

/// The most ops a batch of a run takes; one batch runs a frame.
const BATCH: u32 = 50_000;

/// The time between two batches of a run, in ms.
const BATCH_MS: u64 = 16;

/// The bar of the debugger over the panes in debug mode: go on to a break
/// point, stop and start again; the step buttons; the timed stepping, the
/// speed picker, the status and the op count. While it shows, the debugger
/// shortcuts work, the timer steps and the editor points at the next op.
///
/// # Returns
///
/// The view.
pub fn debug_bar() -> impl IntoView
{
    let state = use_state();
    wire_auto(state);
    wire_shortcuts(state);
    follow_pc(state);
    let options = RwSignal::new(false);
    let bar = div().class(ClassName::DEBUG_BAR).child(controls(state, options));
    (sheet(&SHEET), bar, compiler_modal(options))
}

/// The bar with go, stop, the compiler options and the status, outside
/// debug mode.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
pub fn run_bar(state: AppState) -> impl IntoView
{
    let options = RwSignal::new(false);
    let head = (go_button(state), stop_button(state), rule(), options_button(options));
    let bar = nav()
        .class(ClassName::RUN_CONTROLS)
        .attr(TOUR, "controls")
        .attr("role", "toolbar")
        .attr("aria-label", t!("explorer.run.label"))
        .child((head, status_chip(state), run_chips(state)));
    (sheet(&SHEET), bar, compiler_modal(options))
}

/// Halt the batches and the timed stepping.
///
/// # Arguments
///
/// * `state` - The state.
fn halt(state: AppState)
{
    state.going.set(false);
    state.auto_step.set(false);
}

/// Run the ops in batches while a run goes on, one batch a frame, so the
/// page keeps responding and the run can be stopped. Outside debug mode a
/// new session starts to run on its own when auto run is on, and a run the
/// debugger left paused goes on, so the output is whole; in debug mode new
/// ops halt the run.
///
/// # Arguments
///
/// * `state` - The state.
pub fn wire_going(state: AppState)
{
    let batch = move || {
        let breaks = state.debug_mode.get_untracked();
        let run = |s: &mut Session| s.run_for(BATCH, breaks);
        let go_on = state.debug.try_update(run).unwrap_or(false);
        do_if!(!go_on, state.going.set(false));
    };
    let options = UseIntervalFnOptions::default().immediate(false);
    let timer = use_interval_fn_with_options(batch, BATCH_MS, options);
    let (resume, pause) = (timer.resume, timer.pause);
    Effect::new(move |_| tern!(state.going.get(), resume(), pause()));
    Effect::new(move |_| {
        let auto = state.auto_run.get();
        let open = state.debug.with(|s| s.is_live() && opens(s.status(), auto));
        let idle = !state.debug_mode.get_untracked() && !state.going.get_untracked();
        do_if!(open && idle, state.going.set(true));
    });
    // New ops halt the run, but the first pass is the mount, not new ops.
    Effect::new(move |was: Option<()>| {
        state.compiled.track();
        do_if!(was.is_some() && state.debug_mode.get_untracked(), halt(state));
    });
}

/// Whether a session outside debug mode runs on without a press of go: a run
/// the debugger left paused does, and a new one when auto run is on.
///
/// # Arguments
///
/// * `status` - Where the session stands.
/// * `auto` - Whether auto run is on.
///
/// # Returns
///
/// `true` when the run goes on.
fn opens(status: RunStatus, auto: bool) -> bool
{
    status == RunStatus::Paused || (auto && status == RunStatus::Start)
}

/// Run the program. Outside debug mode it runs again from the first op; in
/// debug mode it goes on to the next break point, from the first op once
/// the run has ended, and the bottom panel turns to the output.
///
/// # Arguments
///
/// * `state` - The state.
fn go(state: AppState)
{
    state.auto_step.set(false);
    let debug = state.debug_mode.get_untracked();
    let ended = state.debug.with_untracked(|s| !s.running());
    do_if!(ended || !debug, state.debug.update(Session::reset));
    do_if!(debug, show_output(state));
    state.going.set(true);
}

/// Open the bottom panel on the output.
///
/// # Arguments
///
/// * `state` - The state.
fn show_output(state: AppState)
{
    state.panel_tab.set(PanelTab::Output);
    state.set_problems_open.set(true);
}

/// Stop the program before its end and go back to the first op.
///
/// # Arguments
///
/// * `state` - The state.
fn stop(state: AppState)
{
    halt(state);
    state.debug.update(Session::stop);
}

/// The go button: it runs the program, or in debug mode goes on to the
/// next break point.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn go_button(state: AppState) -> impl IntoView
{
    let name = move || {
        let debug = state.debug_mode.get();
        tern!(debug, t!("explorer.run.resume"), t!("explorer.run.go"))
    };
    let live = move || state.debug.with(Session::is_live);
    let blocked = move || state.going.get() || !live();
    button()
        .class(format!("{} {}", Controls::ICON_BUTTON, ClassName::RUN_GO))
        .attr(TOUR, "go")
        .title(name)
        .attr("aria-label", name)
        .attr("disabled", move || blocked().then_some(""))
        .on(ev::click, move |_| go(state))
        .child(icon::start())
}

/// The stop button; it can be pressed while a run goes on.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn stop_button(state: AppState) -> impl IntoView
{
    let paused = move || state.debug.with(|s| s.status() == RunStatus::Paused);
    let can_stop = move || state.going.get() || state.auto_step.get() || paused();
    let name = t!("explorer.run.stop");
    button()
        .class(format!("{} {}", Controls::ICON_BUTTON, ClassName::RUN_STOP))
        .attr(TOUR, "stop")
        .title(name.clone())
        .attr("aria-label", name)
        .attr("disabled", move || (!can_stop()).then_some(""))
        .on(ev::click, move |_| stop(state))
        .child(icon::stop())
}

/// The status chip: running while ops run on a timer, else where the
/// session stands.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn status_chip(state: AppState) -> impl IntoView
{
    let status = move || {
        let on = state.going.get() || state.auto_step.get();
        tern!(on, RunStatus::Running, state.debug.with(Session::status))
    };
    let class = move || format!("{} {}", ClassName::RUN_STATUS, status_class(status()));
    span()
        .class(class)
        .attr(TOUR, "status")
        .attr("role", "status")
        .child(move || status().label())
}

/// The chips after the status: the ops run, the times of the compile and of
/// the run, and the most heap and stack the run held at once.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn run_chips(state: AppState) -> impl IntoView
{
    let ops =
        move || t!("explorer.run.steps", count = state.debug.with(Session::steps));
    let compile_time = Memo::new(move |_| state.compiled.with(|c| c.times.total()));
    let run_time = Memo::new(move |_| state.debug.with(Session::run_time));
    let (compile_text, run_text) =
        (move || time_text(compile_time.get()), move || time_text(run_time.get()));
    let times = move || format!("{} / {}", compile_text(), run_text());
    let times_note = move || {
        t!("explorer.run.times_note", compile = compile_text(), run = run_text())
    };
    let usage = Memo::new(move |_| state.debug.with(Session::usage));
    let usage_text = move || {
        let (heap, stack) = usage.get();
        t!("explorer.run.usage", heap = octets(heap), stack = stack)
    };
    (
        span().class(ClassName::RUN_CHIP).child(ops),
        span().class(ClassName::RUN_CHIP).title(times_note).child(times),
        span()
            .class(format!("{} {}", ClassName::RUN_CHIP, ClassName::RUN_CHIP_USAGE))
            .title(t!("explorer.run.usage_note"))
            .child(usage_text),
    )
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

/// Run a command on the session; it halts the run.
///
/// # Arguments
///
/// * `state` - The state.
/// * `act` - The command.
fn command(state: AppState, act: fn(&mut Session))
{
    halt(state);
    state.debug.update(act);
}

/// The bar of controls: the run and step buttons, the timer, the compiler
/// options, then the status on the right.
///
/// # Arguments
///
/// * `state` - The state.
/// * `options` - Whether the compiler options show.
///
/// # Returns
///
/// The view.
fn controls(state: AppState, options: RwSignal<bool>) -> impl IntoView
{
    let running: Signal<bool> =
        Memo::new(move |_| state.debug.with(Session::running)).into();
    let act = move |f: fn(&mut Session)| move || command(state, f);
    let ends = (
        go_button(state),
        stop_button(state),
        control(
            "explorer.run.reset",
            "reset",
            icon::restart(),
            Signal::stored(true),
            act(Session::reset),
        ),
    );
    let buttons = (
        control("explorer.run.step", "step", icon::step(), running, act(Session::step)),
        control(
            "explorer.run.step_over",
            "step-over",
            icon::step_over(),
            running,
            act(Session::step_over),
        ),
        control(
            "explorer.run.step_out",
            "step-out",
            icon::step_out(),
            running,
            act(Session::step_out),
        ),
        rule(),
        auto_button(state.auto_step, running),
    );
    nav()
        .class(ClassName::RUN_CONTROLS)
        .attr(TOUR, "controls")
        .attr("role", "toolbar")
        .attr("aria-label", t!("explorer.run.label"))
        .child((
            ends,
            rule(),
            buttons,
            speed_picker(state.speed),
            rule(),
            options_button(options),
            status_chip(state),
            run_chips(state),
        ))
}

/// The button that opens the compiler options: its icon, and on wide
/// screens its name.
///
/// # Arguments
///
/// * `open` - Whether the compiler options show.
///
/// # Returns
///
/// The view.
fn options_button(open: RwSignal<bool>) -> impl IntoView
{
    let name = t!("explorer.run.options");
    let label = name.clone();
    button()
        .class(format!("{} {}", Controls::ICON_BUTTON, ClassName::RUN_OPTIONS))
        .class((Controls::ICON_BUTTON_ON, move || open.get()))
        .attr(TOUR, "options")
        .attr("aria-haspopup", "dialog")
        .attr("aria-label", name.clone())
        .title(name)
        .on(ev::click, move |_| open.set(true))
        .child((
            icon::sliders(),
            span().class(ClassName::RUN_OPTIONS_NAME).child(label),
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
/// * `tour` - Its name for the tour.
/// * `art` - The icon.
/// * `enabled` - Whether it can be pressed.
/// * `on_press` - What a press does.
///
/// # Returns
///
/// The view.
fn control(
    id: &'static str, tour: &'static str, art: impl IntoView + 'static,
    enabled: Signal<bool>, on_press: impl Fn() + 'static,
) -> impl IntoView
{
    button()
        .class(Controls::ICON_BUTTON)
        .attr(TOUR, tour)
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
        .attr(TOUR, "timer")
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
        .attr(TOUR, "speed")
        .title(t!("explorer.run.speed"))
        .child(dropdown(spec))
}

/// Step on a timer while the timed stepping is on; it turns off at the end
/// of the run, at a break point, when the ops change and when debug mode
/// turns off. It can be on before the bar shows, as the tour sets it.
///
/// # Arguments
///
/// * `state` - The state.
fn wire_auto(state: AppState)
{
    let auto = state.auto_step;
    let tick = move || {
        let go_on = state.debug.try_update(Session::tick).unwrap_or(false);
        do_if!(!go_on, auto.set(false));
    };
    let pace = Signal::derive(move || state.speed.get().millis());
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

/// Debugger shortcuts while debug mode is on: F5 goes on to a break point,
/// F10 steps over, F11 steps and Shift+F11 steps out.
///
/// # Arguments
///
/// * `state` - The state.
fn wire_shortcuts(state: AppState)
{
    let _ = use_event_listener(use_window(), ev::keydown, move |e| on_press(state, &e));
}

/// Run the command of a debugger shortcut.
///
/// # Arguments
///
/// * `state` - The state.
/// * `e` - The press.
fn on_press(state: AppState, e: &ev::KeyboardEvent)
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
        Some(f) => command(state, f),
        None => go(state),
    }
}

/// Point the editor at the source of the next op while the ops are open or
/// a run is in progress, so the stepping shows in the editor from each stage.
/// A view that stops pointing hands the pointer back to the next op.
///
/// # Arguments
///
/// * `state` - The state.
fn follow_pc(state: AppState)
{
    let range = Memo::new(move |_| {
        let on_ops = state.stage.get() == Phase::Run;
        let mid_run = state
            .debug
            .with(|s| matches!(s.status(), RunStatus::Paused | RunStatus::Trapped));
        ret_if!(!on_ops && !mid_run, None);
        let (func, pc) = state.debug.with(|s| (s.viewed_func(), s.marked_pc()));
        op_range(state, func?, pc?)
    });
    Effect::new(move |_| state.point(range.get()));
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
