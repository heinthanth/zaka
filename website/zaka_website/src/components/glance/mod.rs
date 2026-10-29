use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use leptos::ev;
use leptos::html::a;
use leptos::html::button;
use leptos::html::div;
use leptos::html::h2;
use leptos::html::li;
use leptos::html::ol;
use leptos::html::p;
use leptos::html::pre;
use leptos::html::section;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::paint::painted;
use zaka_web_ui_common::sheet;
use zaka_web_ui_common::site::EXPLORER_URL;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::components::sandbox::run::FenceOptions;
use crate::components::sandbox::run::STEP_CAP;
use crate::components::sandbox::run::Stroke;
use crate::components::sandbox::run::Tone;
use crate::components::sandbox::run::advance;
use crate::components::sandbox::run::compile;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/glance/mod.sass");

/// Rules of the glance.
static SHEET: ComponentStyle = ComponentStyle::global_css("site-glance", STYLE_SHEET);

/// How long a run keeps its panels clear before it shows them again, so the
/// reader sees that it ran.
const RUN_PAUSE_MS: u64 = 280;

/// One line of the program, with the words its label points at.
#[derive(Clone, Copy)]
struct Line
{
    /// The line of Zaka.
    code: &'static str,
    /// The words of the line the label points at; the first match is marked.
    pin: &'static str,
    /// Locale id of the label.
    label: &'static str,
}

/// The program, line after line: it adds the numbers from 1 to 10, prints the
/// total with a quote mark where a plain value wants an object mark, and
/// tells whether the total is more than 50.
const LINES: [Line; 5] = [
    Line {
        code: "ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။",
        pin: "ကိန်းရှင်",
        label: "website.glance.variable",
    },
    Line {
        code: "၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ စုစုပေါင်း ကို စုစုပေါင်း + ကိန်း ဟု ထားပါ။",
        pin: "ရေတွက်၍",
        label: "website.glance.count",
    },
    Line {
        code: "စုစုပေါင်း ဟု ပြသပါ။", pin: "ဟု", label: "website.glance.verb"
    },
    Line {
        code: "အကယ်၍ စုစုပေါင်း သည် ၅၀ ထက် ကြီးလျှင် \"များသည်\" ဟု ပြသပါ။",
        pin: "သည် ၅၀ ထက်",
        label: "website.glance.particles",
    },
    Line {
        code: "သို့မဟုတ်ပါက \"နည်းသည်\" ဟု ပြသပါ။", pin: "။", label: "website.glance.stop"
    },
];

/// The program the glance shows, as one text.
///
/// # Returns
///
/// The source.
pub fn source() -> String { LINES.map(|l| l.code).join("\n") }

/// The glance: one short program with a label pinned to a part of each line,
/// then what it prints and the warning the compiler gives, which a button
/// runs again in the browser.
///
/// # Returns
///
/// The view.
pub fn glance() -> impl IntoView
{
    let run = GlanceRun(GlanceRunProps::builder().source(source()).build());
    let about = div().class(ClassName::GLANCE_ABOUT).child((
        p().child(t!("website.glance.about")),
        p().child(t!("website.glance.about_run")),
    ));
    section().id("glance").class(ClassName::GLANCE).attr("data-field", "violet").child(
        (
            sheet(&SHEET),
            h2().class(ClassName::GLANCE_HEAD).child(t!("website.glance.head")),
            ol().class(format!("{} rise", ClassName::GLANCE_CODE))
                .lang("my")
                .child(LINES.map(row)),
            run,
            about,
        ),
    )
}

/// One line of the program: its number, the painted line with its pinned
/// words marked, and the label tied to them.
///
/// # Arguments
///
/// * `line` - The line.
///
/// # Returns
///
/// The view.
fn row(line: Line) -> impl IntoView
{
    let at = line.code.find(line.pin).unwrap_or_default();
    let (before, rest) = line.code.split_at(at);
    let (pinned, after) = rest.split_at(line.pin.len());
    li().class(ClassName::GLANCE_LINE).child((
        code_text(before, pinned, after),
        span().class(ClassName::GLANCE_TIE).attr("aria-hidden", "true"),
        p().class(ClassName::GLANCE_LABEL).lang("en").child(t!(line.label.to_string())),
    ))
}

/// A line of code cut in three, each part painted with the lexer; the middle
/// one is marked as the words the label points at.
///
/// # Arguments
///
/// * `before` - The text before the pinned words.
/// * `pinned` - The pinned words.
/// * `after` - The text after them.
///
/// # Returns
///
/// The view.
fn code_text(before: &str, pinned: &str, after: &str) -> impl IntoView
{
    span().class(format!("{} painted", ClassName::GLANCE_TEXT)).child((
        painted(before),
        span().class(ClassName::GLANCE_PIN).child(painted(pinned)),
        painted(after),
    ))
}

/// What one run of the program gives.
#[derive(Clone)]
struct Ran
{
    /// The diagnostics, as the command line renders them.
    report: Vec<Stroke>,
    /// What the program printed.
    printed: String,
}

/// Compile the program and run it to its end.
///
/// # Arguments
///
/// * `source` - The program.
///
/// # Returns
///
/// Its diagnostics and what it printed.
fn run_program(source: &str) -> Ran
{
    let compiled = compile(source, &FenceOptions::default());
    let printed =
        compiled.machine.map(|mut m| advance(&mut m, STEP_CAP).0).unwrap_or_default();
    Ran { report: compiled.report, printed }
}

/// The address of the explorer with the program open in it, in the short
/// link form the explorer reads with its default options.
///
/// # Arguments
///
/// * `source` - The program.
///
/// # Returns
///
/// The address.
fn explorer_link(source: &str) -> String
{
    let text = URL_SAFE_NO_PAD.encode(source.as_bytes());
    format!("{EXPLORER_URL}#2.64.{text}")
}

/// The run of the glance: the run button and the link into the explorer,
/// then two panels, what the program printed and what the compiler warned.
/// The page is built with the first run in place; the button runs the
/// program again in the browser.
///
/// # Arguments
///
/// * `source` - The program.
///
/// # Returns
///
/// The view.
#[island(lazy)]
pub fn GlanceRun(source: String) -> impl IntoView
{
    let ran = RwSignal::new(Some(run_program(&source)));
    let running = RwSignal::new(false);
    let started = RwSignal::new(false);
    Effect::new(move |_| started.set(true));
    let again = source.clone();
    let rerun = move |_| {
        running.set(true);
        ran.set(None);
        let again = again.clone();
        set_timeout(
            move || {
                ran.set(Some(run_program(&again)));
                running.set(false);
            },
            std::time::Duration::from_millis(RUN_PAUSE_MS),
        );
    };
    let label = move || {
        tern!(running.get(), t!("website.sandbox.running"), t!("website.sandbox.run"))
    };
    let bar = div().class(ClassName::GLANCE_BAR).child((
        button()
            .class("button")
            .attr("type", "button")
            .attr("disabled", move || (!started.get() || running.get()).then_some(""))
            .on(ev::click, rerun)
            .child(label),
        a().class(ClassName::GLANCE_EDIT)
            .href(explorer_link(&source))
            .child((t!("website.glance.edit"), " →")),
    ));
    let printed = move || ran.get().map(|r| r.printed).unwrap_or_default();
    let report = move || ran.get().map(|r| r.report).unwrap_or_default();
    let panels = div().class(ClassName::GLANCE_PANELS).child((
        panel(t!("website.sandbox.output").to_string(), move || printed().into_any()),
        panel(t!("website.glance.warning").to_string(), move || {
            report().into_iter().map(stroke).collect_view().into_any()
        }),
    ));
    div().class(ClassName::GLANCE_RUN).child((bar, panels))
}

/// One panel under the program: a name over a block of text.
///
/// # Arguments
///
/// * `name` - The name of the panel.
/// * `content` - What it holds.
///
/// # Returns
///
/// The view.
fn panel(name: String, content: impl Fn() -> AnyView + Send + 'static)
-> impl IntoView
{
    div().class(ClassName::GLANCE_PANEL).child((
        p().class(ClassName::GLANCE_PANEL_NAME).child(name),
        pre().class(ClassName::GLANCE_PANEL_TEXT).lang("my").child(content),
    ))
}

/// One run of compiler text, in its color.
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
        Tone::Gutter => ClassName::GLANCE_GUTTER,
        Tone::Error => ClassName::GLANCE_ERROR,
        Tone::Warning => ClassName::GLANCE_WARNING,
        Tone::Note => ClassName::GLANCE_NOTE,
    };
    span().class(class).child(stroke.text)
}
