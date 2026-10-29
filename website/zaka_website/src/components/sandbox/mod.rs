pub mod run;

use bon::Builder;
use leptos::ev;
use leptos::html::button;
use leptos::html::div;
use leptos::html::p;
use leptos::html::pre;
use leptos::html::span;
use leptos::html::textarea;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_vm::machine::Machine;
use zaka_web_ui_common::paint::painted;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use self::run::BATCH;
use self::run::Fence;
use self::run::FenceOptions;
use self::run::Stroke;
use self::run::Tone;
use self::run::advance;
use self::run::compile;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/sandbox/mod.sass");

/// Rules of the sandbox.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-sandbox", STYLE_SHEET);

/// The most characters of output a sandbox keeps; a program that prints
/// without end loses its oldest lines past it.
const OUTPUT_CAP: usize = 40_000;

/// The name of the language, on the bar of each sandbox.
const NAME: &str = "စံကား";

/// The sheet of the sandbox. It mounts outside the island, as a global sheet
/// renders on the server alone.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn sandbox_sheet() -> impl IntoView { sheet(&SHEET) }

/// What a sandbox shows and runs.
#[derive(Clone, Copy, Builder)]
struct Live
{
    /// The program, as the reader left it.
    code: RwSignal<String>,
    /// What the last run gave, or `None` before the first run.
    shown: RwSignal<Option<Vec<Stroke>>>,
    /// Whether a run goes on.
    running: RwSignal<bool>,
    /// The machine of the run that goes on.
    machine: StoredValue<Option<Machine>>,
    /// What the fence of the program asks of the compiler.
    options: FenceOptions,
}

/// A program of a Learn page that the reader can edit and run: a bar with the
/// run button, the program painted with the lexer under a clear text field,
/// and what it prints. Until the first run, the output is what its page
/// shows as its output.
/// The island loads late, on its own, so the compiler reaches the pages that
/// run code and no other.
///
/// # Arguments
///
/// * `source` - The program.
/// * `fence` - What its page promises the program does.
/// * `options` - What its fence asks of the compiler.
/// * `expected` - What its page shows as its output.
///
/// # Returns
///
/// The view.
#[island(lazy)]
pub fn Sandbox(
    source: String, fence: Fence, options: FenceOptions, expected: String,
) -> impl IntoView
{
    let live = Live::builder()
        .code(RwSignal::new(source.clone()))
        .shown(RwSignal::new(None))
        .running(RwSignal::new(false))
        .machine(StoredValue::new(None))
        .options(options)
        .build();
    // The page is read before the island starts; until then the field cannot
    // be edited and the buttons are off.
    let started = RwSignal::new(false);
    Effect::new(move |_| started.set(true));
    let field = NodeRef::<leptos::html::Textarea>::new();
    div()
        .class(ClassName::SANDBOX)
        .attr("data-running", move || live.running.get().then_some(""))
        .child((
            bar(live, &source, &fence, started, field),
            editor(live, &source, started, field),
            output(live, &fence, expected),
        ))
}

/// The bar: the name of the language and the codes the program shows, then
/// the reset and run buttons.
///
/// # Arguments
///
/// * `live` - What the sandbox shows and runs.
/// * `source` - The program as its page writes it.
/// * `fence` - What its page promises the program does.
/// * `started` - Whether the island has started.
/// * `field` - The text field.
///
/// # Returns
///
/// The view.
fn bar(
    live: Live, source: &str, fence: &Fence, started: RwSignal<bool>,
    field: NodeRef<leptos::html::Textarea>,
) -> impl IntoView + use<>
{
    let codes = fence
        .codes()
        .iter()
        .map(|c| span().class(ClassName::SANDBOX_CODE).child(c.clone()));
    let original = source.to_string();
    let edited = {
        let original = original.clone();
        move || live.code.with(|c| *c != original)
    };
    let reset = button()
        .class(ClassName::SANDBOX_RESET)
        .r#type("button")
        .attr("hidden", move || (!edited()).then_some(""))
        .on(ev::click, move |_| reset(live, &original, field))
        .child(t!("website.sandbox.reset"));
    let label = move || {
        tern!(live.running.get(), t!("website.sandbox.stop"), t!("website.sandbox.run"))
    };
    let run = button()
        .class(format!("button {}", ClassName::SANDBOX_RUN))
        .r#type("button")
        .attr("disabled", move || (!started.get()).then_some(""))
        .attr("title", t!("website.sandbox.run_hint"))
        .on(ev::click, move |_| toggle(live))
        .child(label);
    div().class(ClassName::SANDBOX_BAR).child((
        span().class(ClassName::SANDBOX_NAME).lang("my").child(NAME),
        codes.collect_view(),
        div().class(ClassName::SANDBOX_ACTIONS).child((reset, run)),
    ))
}

/// The editor: the program painted with the lexer, under a text field with
/// clear letters and the same metrics, so the reader edits the painted text.
/// The field grows with the program, as both share one cell of a grid.
///
/// # Arguments
///
/// * `live` - What the sandbox shows and runs.
/// * `source` - The program as its page writes it.
/// * `started` - Whether the island has started.
/// * `field` - The text field.
///
/// # Returns
///
/// The view.
fn editor(
    live: Live, source: &str, started: RwSignal<bool>,
    field: NodeRef<leptos::html::Textarea>,
) -> impl IntoView + use<>
{
    // The line break at the end holds room for a last line left blank.
    let paint = pre()
        .class(format!("painted {}", ClassName::SANDBOX_PAINT))
        .attr("aria-hidden", "true")
        .child((move || live.code.with(|c| painted(c)), "\n"));
    let input = textarea()
        .node_ref(field)
        .class(ClassName::SANDBOX_INPUT)
        .lang("my")
        .attr("aria-label", t!("website.sandbox.editor"))
        .attr("spellcheck", "false")
        .attr("autocapitalize", "off")
        .attr("autocomplete", "off")
        .attr("readonly", move || (!started.get()).then_some(""))
        .on(ev::input, move |e| live.code.set(event_target_value(&e)))
        .on(ev::keydown, move |e| run_on_enter(live, &e))
        .child(source.to_string());
    div().class(ClassName::SANDBOX_EDITOR).child((paint, input))
}

/// The output: what the last run gave, or what its page shows as the output
/// of the program, until the first run.
///
/// # Arguments
///
/// * `live` - What the sandbox shows and runs.
/// * `fence` - What its page promises the program does.
/// * `expected` - What its page shows as its output.
///
/// # Returns
///
/// The view.
fn output(live: Live, fence: &Fence, expected: String) -> impl IntoView + use<>
{
    let promised = match fence
    {
        Fence::Error(_) => t!("website.sandbox.expected_error"),
        Fence::Warning(_) => t!("website.sandbox.expected_warning"),
        Fence::Run | Fence::Stop => t!("website.sandbox.expected"),
    };
    let label = move || match (live.running.get(), live.shown.with(Option::is_some))
    {
        (true, _) => t!("website.sandbox.running"),
        (false, true) => t!("website.sandbox.output"),
        (false, false) => promised.clone(),
    };
    let text = move || {
        let Some(strokes) = live.shown.get()
        else
        {
            return expected.clone().into_any();
        };
        let blank = strokes.is_empty() && !live.running.get();
        ret_if!(
            blank,
            span()
                .class(ClassName::SANDBOX_QUIET)
                .child(t!("website.sandbox.nothing"))
                .into_any()
        );
        strokes.into_iter().map(stroke).collect_view().into_any()
    };
    div()
        .class(ClassName::SANDBOX_OUTPUT)
        .class((ClassName::SANDBOX_PROMISED, move || live.shown.with(Option::is_none)))
        .child((
            p().class(ClassName::SANDBOX_LABEL).child(label),
            pre()
                .class(ClassName::SANDBOX_TEXT)
                .lang("my")
                .attr("aria-live", "polite")
                .child(text),
        ))
}

/// One run of output, in its color.
///
/// # Arguments
///
/// * `stroke` - The run.
///
/// # Returns
///
/// The view.
fn stroke(stroke: Stroke) -> impl IntoView
{
    let class = match stroke.tone
    {
        Tone::Plain => "",
        Tone::Gutter => ClassName::SANDBOX_GUTTER,
        Tone::Error => ClassName::SANDBOX_ERROR,
        Tone::Warning => ClassName::SANDBOX_WARNING,
        Tone::Note => ClassName::SANDBOX_NOTE,
    };
    span().class(class).child(stroke.text)
}

/// Run the program on Control or Command with Enter.
///
/// # Arguments
///
/// * `live` - What the sandbox shows and runs.
/// * `e` - The event of the pressed button.
fn run_on_enter(live: Live, e: &ev::KeyboardEvent)
{
    ret_if!(e.key() != "Enter" || !(e.ctrl_key() || e.meta_key()));
    e.prevent_default();
    do_if!(!live.running.get_untracked(), start(live));
}

/// Start a run, or stop the run that goes on.
///
/// # Arguments
///
/// * `live` - What the sandbox shows and runs.
fn toggle(live: Live)
{
    ret_if!(!live.running.get_untracked(), start(live));
    live.running.set(false);
    live.machine.set_value(None);
    let note = format!("{}\n", t!("website.sandbox.halted"));
    push(live, Stroke::builder().tone(Tone::Note).text(note).build());
}

/// Put the program back as its page writes it, with no output.
///
/// # Arguments
///
/// * `live` - What the sandbox shows and runs.
/// * `source` - The program as its page writes it.
/// * `field` - The text field.
fn reset(live: Live, source: &str, field: NodeRef<leptos::html::Textarea>)
{
    live.running.set(false);
    live.machine.set_value(None);
    live.code.set(source.to_string());
    live.shown.set(None);
    do_if!(let Some(el) = field.get_untracked(), el.set_value(source));
}

/// Compile the program and run it, a batch of ops a frame.
///
/// # Arguments
///
/// * `live` - What the sandbox shows and runs.
fn start(live: Live)
{
    let compiled = live.code.with_untracked(|c| compile(c, &live.options));
    live.shown.set(Some(compiled.report));
    let Some(machine) = compiled.machine
    else
    {
        return;
    };
    live.machine.set_value(Some(machine));
    live.running.set(true);
    pump(live);
}

/// Run one batch of ops, show what it printed, and ask for the next frame
/// while the run goes on.
///
/// # Arguments
///
/// * `live` - What the sandbox shows and runs.
fn pump(live: Live)
{
    ret_if!(!live.running.get_untracked());
    let batch = live
        .machine
        .try_update_value(|m| m.as_mut().map(|m| (advance(m, BATCH), m.running())));
    let Some(Some(((printed, stopped), going))) = batch
    else
    {
        return live.running.set(false);
    };
    let plain = Stroke::builder().tone(Tone::Plain).text(printed).build();
    do_if!(!plain.text.is_empty(), push(live, plain));
    let error =
        |r: String| Stroke::builder().tone(Tone::Error).text(format!("{r}\n")).build();
    do_if!(let Some(reason) = stopped, push(live, error(reason)));
    ret_if!(going, request_animation_frame(move || pump(live)));
    live.running.set(false);
    live.machine.set_value(None);
}

/// Add a run to the output, joined to the last run when both are plain, and
/// drop the oldest output past [`OUTPUT_CAP`].
///
/// # Arguments
///
/// * `live` - What the sandbox shows and runs.
/// * `next` - The run.
fn push(live: Live, next: Stroke)
{
    live.shown.update(|shown| {
        let strokes = shown.get_or_insert_with(Vec::new);
        let joins = strokes
            .last()
            .is_some_and(|l| l.tone == Tone::Plain && next.tone == Tone::Plain);
        match strokes.last_mut().filter(|_| joins)
        {
            Some(last) => last.text.push_str(&next.text),
            None => strokes.push(next),
        }
        trim(strokes);
    });
}

/// Drop the oldest output past [`OUTPUT_CAP`], whole lines at a time.
///
/// # Arguments
///
/// * `strokes` - The output.
fn trim(strokes: &mut Vec<Stroke>)
{
    let total: usize = strokes.iter().map(|s| s.text.len()).sum();
    ret_if!(total <= OUTPUT_CAP);
    let mut over = total - OUTPUT_CAP;
    let mut whole = |len: usize| {
        let gone = len <= over;
        do_if!(gone, over -= len);
        gone
    };
    let gone =
        strokes.iter().map(|s| s.text.len()).take_while(|&len| whole(len)).count();
    strokes.drain(..gone);
    let Some(first) = strokes.first_mut()
    else
    {
        return;
    };
    let len = first.text.len();
    let from = (over..len).find(|&i| first.text.is_char_boundary(i)).unwrap_or(len);
    let cut = first.text[from..].find('\n').map_or(len, |at| from + at + 1);
    first.text.drain(..cut);
}
