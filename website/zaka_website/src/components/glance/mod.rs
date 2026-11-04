use leptos::html::div;
use leptos::html::h2;
use leptos::html::li;
use leptos::html::ol;
use leptos::html::p;
use leptos::html::pre;
use leptos::html::section;
use leptos::html::span;
use leptos::prelude::*;
use leptos::svg::path;
use leptos::svg::svg;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::paint::painted;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;

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

/// One line of the program, with the words its label points at.
#[derive(Clone, Copy)]
struct Line
{
    /// The line of Zaka.
    code: &'static str,
    /// The words of the line the label points at; the first match is marked.
    pin: &'static str,
    /// Locale id of the label; `None` for a line with no note.
    label: Option<&'static str>,
}

/// The program, line after line: its start block adds the numbers from 1 to
/// 10, prints the total with a quote mark where a plain value wants an object
/// mark, and tells whether the total is more than 50.
const LINES: [Line; 7] = [
    Line {
        code: "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        pin: "ပရိုဂရမ်သည်",
        label: Some("website.glance.start"),
    },
    Line {
        code: "ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။",
        pin: "ကိန်းရှင်",
        label: Some("website.glance.variable"),
    },
    Line {
        code: "၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ စုစုပေါင်း ကို စုစုပေါင်း + ကိန်း ဟု ထားပါ။",
        pin: "ရေတွက်၍",
        label: Some("website.glance.count"),
    },
    Line {
        code: "စုစုပေါင်း ဟု ပြသပါ။", pin: "ဟု", label: Some("website.glance.verb")
    },
    Line {
        code: "အကယ်၍ စုစုပေါင်း သည် ၅၀ ထက် ကြီးလျှင် \"များသည်\" ဟု ပြသပါ။",
        pin: "သည် ၅၀ ထက်",
        label: Some("website.glance.particles"),
    },
    Line {
        code: "သို့မဟုတ်ပါက \"နည်းသည်\" ဟု ပြသပါ။",
        pin: "။",
        label: Some("website.glance.stop"),
    },
    Line {
        code: "ပရိုဂရမ် ပြီးဆုံးပါပြီး။", pin: "", label: None
    },
];

/// The program the glance shows, as one text.
///
/// # Returns
///
/// The source.
pub fn source() -> String { LINES.map(|l| l.code).join("\n") }

/// The glance: one short program with a handwritten note pinned to a part of
/// each line with an arrow, then what it prints and the warning the compiler
/// gives.
///
/// # Returns
///
/// The view.
pub fn glance() -> impl IntoView
{
    let run = run_panels(&source());
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
/// words marked, the note under it, and the arrow from the note to the
/// words, which [`crate::notes`] draws once the page knows where each
/// one sits.
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
    let note = line.label.map(|label| {
        (
            p().class(ClassName::GLANCE_LABEL)
                .lang("en")
                .attr("data-side", "below")
                .child(t!(label.to_string())),
            svg()
                .class(ClassName::GLANCE_ARROW)
                .attr("aria-hidden", "true")
                .child(path().class(ClassName::GLANCE_PEN)),
        )
    });
    li().class(ClassName::GLANCE_LINE).child((code_text(before, pinned, after), note))
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

/// The run of the glance: two panels, what the program printed and what the
/// compiler warned, both from the run the build makes.
///
/// # Arguments
///
/// * `source` - The program.
///
/// # Returns
///
/// The view.
fn run_panels(source: &str) -> impl IntoView + use<>
{
    let ran = run_program(source);
    let report = ran.report.into_iter().map(stroke).collect_view();
    div().class(ClassName::GLANCE_RUN).child((
        panel(t!("website.sandbox.output").to_string(), ran.printed),
        panel(t!("website.glance.warning").to_string(), report),
    ))
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
fn panel(name: String, content: impl IntoView) -> impl IntoView
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
