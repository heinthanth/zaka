use leptos::ev;
use leptos::html;
use leptos::html::div;
use leptos::html::h3;
use leptos::html::input;
use leptos::html::p;
use leptos::html::pre;
use leptos::html::section;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use web_sys::KeyboardEvent;
use web_sys::ScrollIntoViewOptions;
use web_sys::ScrollLogicalPosition;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::compile::asm::AsmFunc;
use crate::components::asm_table::Gutter;
use crate::components::asm_table::asm_table;
use crate::components::asm_table::op_row_id;
use crate::components::check_view::skipped;
use crate::components::debug_bar::run_bar;
use crate::components::table::ClassName as Table;
use crate::debug::Session;
use crate::state::AppState;
use crate::state::tour::TOUR;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/run_view/mod.sass");

/// Sheet of the run view.
static SHEET: ComponentStyle = ComponentStyle::global_css("zaka-run-view", STYLE_SHEET);

/// The run of the generated ops outside debug mode: go, stop and its
/// output. Or a note when code generation did not run.
///
/// # Returns
///
/// The view.
pub fn run_view() -> impl IntoView
{
    let state = use_state();
    let content = move |state: AppState| (run_bar(state), out_pane(state));
    ran_or_note(state, content)
}

/// The ops of the run in debug mode, each function with its break point
/// gutter and the next op of the call the dock shows marked. Or a note when
/// code generation did not run.
///
/// # Returns
///
/// The view.
pub fn ops_view() -> impl IntoView
{
    let state = use_state();
    ran_or_note(state, ops_pane)
}

/// A view of the run, or a note in its place when code generation did not
/// run.
///
/// # Arguments
///
/// * `state` - The state.
/// * `content` - The view of the run.
///
/// # Returns
///
/// The view.
fn ran_or_note<V>(state: AppState, content: fn(AppState) -> V) -> impl IntoView
where
    V: IntoView + 'static,
{
    let emitted = Memo::new(move |_| state.compiled.with(|c| c.emitted));
    let shown = move || {
        let title = t!("explorer.run.skipped_title");
        let note = t!("explorer.run.skipped_note");
        tern!(emitted.get(), content(state).into_any(), skipped(title, note).into_any())
    };
    let view = div()
        .class(ClassName::RUN_VIEW)
        .class((ClassName::RUN_VIEW_SKIPPED, move || !emitted.get()))
        .child(shown);
    (sheet(&SHEET), view)
}

/// The ops of all functions, kept with the marked op in view.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn ops_pane(state: AppState) -> impl IntoView
{
    let func = Memo::new(move |_| state.debug.with(Session::viewed_func));
    let marked = Memo::new(move |_| state.debug.with(Session::marked_pc));
    Effect::new(move |_| {
        let (Some(f), Some(pc)) = (func.get(), marked.get())
        else
        {
            return;
        };
        scroll_to(&op_row_id(f, pc));
    });
    let content = move || {
        let funcs = state.compiled.with(|c| c.asm.clone());
        let ops = move |asm: AsmFunc| func_ops(state, asm, func, marked);
        funcs.into_iter().map(ops).collect_view()
    };
    section().class(ClassName::RUN_CODE).attr(TOUR, "ops").child(content)
}

/// Bring an element in view, when it is there.
///
/// # Arguments
///
/// * `id` - The element id.
pub fn scroll_to(id: &str)
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

/// The ops of one function under its head.
///
/// # Arguments
///
/// * `state` - The state.
/// * `asm` - The function.
/// * `func` - The function of the call the dock shows.
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
    let toggle = Callback::new(move |pc| state.toggle_op(f, pc));
    let gutter = Gutter::builder().breaks(breaks.into()).toggle(toggle).build();
    let head = h3().class(ClassName::ASM_HEAD).child(asm.head());
    div()
        .class(ClassName::ASM_FUNC)
        .class((ClassName::ASM_FUNC_SHOWN, shown))
        .child((head, asm_table(state, asm, mark, Some(gutter))))
}

/// The output under its title, filling the view outside debug mode.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn out_pane(state: AppState) -> impl IntoView
{
    let title = h3().class(Table::TABLE_TITLE).child(t!("explorer.run.output"));
    section()
        .class(ClassName::RUN_OUT)
        .attr(TOUR, "output")
        .child((title, output_view(state)))
}

/// What the program wrote so far, with the lines entered for it, and the trap
/// when the run stopped before its end. While the run waits for standard
/// input, a line to type it in follows the output, as at a terminal.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
pub fn output_view(state: AppState) -> impl IntoView
{
    let text = Memo::new(move |_| state.debug.with(Session::output));
    let waiting = Memo::new(move |_| state.debug.with(Session::waiting));
    let blank = move || text.with(String::is_empty) && !waiting.get();
    let shown =
        move || tern!(blank(), t!("explorer.run.no_output").to_string(), text.get());
    let end = NodeRef::<html::Pre>::new();
    // Like a terminal, the output keeps its last line in view.
    Effect::new(move |_| {
        text.track();
        waiting.track();
        do_if!(let Some(el) = end.get(), el.set_scroll_top(el.scroll_height()));
    });
    let output = pre()
        .class(ClassName::RUN_OUTPUT)
        .class((ClassName::RUN_OUTPUT_BLANK, blank))
        .node_ref(end)
        .child((shown, move || waiting.get().then(|| input_line(state))));
    let trap = move || {
        let trap = state.debug.with(Session::trap);
        trap.map(|t| p().class(ClassName::RUN_TRAP).attr("role", "status").child(t))
    };
    (sheet(&SHEET), output, trap)
}

/// The line where the reader enters standard input while the run waits: Enter
/// gives the program the line, and Ctrl+D ends the input; then the run goes
/// on. It takes the focus when it shows.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn input_line(state: AppState) -> impl IntoView
{
    let field = NodeRef::<html::Input>::new();
    Effect::new(move |_| do_if!(let Some(el) = field.get(), _ = el.focus()));
    let pressed = move |e: KeyboardEvent| {
        let Some(el) = field.get_untracked()
        else
        {
            return;
        };
        let enter = e.key() == "Enter";
        let end = e.ctrl_key() && e.key().eq_ignore_ascii_case("d");
        ret_if!(!enter && !end);
        e.prevent_default();
        let line = el.value();
        el.set_value("");
        state.debug.update(|s| tern!(enter, s.give_input(&line), s.close_input()));
        state.going.set(true);
    };
    input()
        .r#type("text")
        .class(ClassName::RUN_INPUT)
        .attr("aria-label", t!("explorer.run.input"))
        .title(t!("explorer.run.input_hint"))
        .attr("autocomplete", "off")
        .attr("spellcheck", "false")
        .node_ref(field)
        .on(ev::keydown, pressed)
}
