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

/// The program, line after line: its start block keeps the prices of two
/// fruits in a table, adds each row to a total, prints the total with a quote
/// mark where a plain value wants an object mark, and tells whether the total
/// is more than 1000.
const LINES: [&str; 8] = [
    "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
    "ကိန်းရှင် ဈေးနှုန်း သည် [\"ပန်းသီး\": ၅၀၀၊ \"သရက်သီး\": ၈၀၀] ဖြစ်ပါစေ။",
    "ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။",
    "ဈေးနှုန်း ၏ အတန်းတိုင်းကို နှုန်း အနေဖြင့် ယူ၍ စုစုပေါင်း ကို စုစုပေါင်း + နှုန်း ဟု ထားမည်။",
    "စုစုပေါင်း ဟု ပြသမည်။",
    "အကယ်၍ စုစုပေါင်း သည် ၁၀၀၀ ထက် ကြီးလျှင် \"ဈေးကြီးသည်\" ဟု ပြသမည်။",
    "သို့မဟုတ်ပါက \"ဈေးသက်သာသည်\" ဟု ပြသမည်။",
    "ပရိုဂရမ် ပြီးဆုံးပါပြီး။",
];

/// The program the glance shows, as one text.
///
/// # Returns
///
/// The source.
pub fn source() -> String { LINES.join("\n") }

/// The glance: one short program, then what it prints and the warning the
/// compiler gives.
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

/// One line of the program: its number and the line, painted with the lexer.
///
/// # Arguments
///
/// * `code` - The line.
///
/// # Returns
///
/// The view.
fn row(code: &str) -> impl IntoView
{
    li().class(ClassName::GLANCE_LINE).child(
        span()
            .class(format!("{} painted", ClassName::GLANCE_TEXT))
            .child(painted(code)),
    )
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
    let compiled = compile(source, &FenceOptions::default(), "");
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
