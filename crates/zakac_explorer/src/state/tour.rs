use bon::Builder;

use crate::compile::Phase;
use crate::state::permalink::Link;

/// Path of the sample the tour walks through.
pub const TOUR_SAMPLE: &str = "programs/gcd";

/// Space in px between the card and the part it points at, and between the
/// card and the edges of the window.
pub const GAP: f64 = 12.0;

/// Space in px between a part and the ring of the spotlight around it.
pub const HOLE_PAD: f64 = 6.0;

/// One step of the tour: its text, the part it points at and the scene
/// it sets up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct Step
{
    /// Locale id of the title.
    pub title: &'static str,
    /// Locale id of the text.
    pub text: &'static str,
    /// CSS selector of the part it points at; `None` centers the card.
    pub target: Option<&'static str>,
    /// The stage it opens; `None` keeps the open one.
    pub stage: Option<Phase>,
    /// Whether debug mode is on.
    #[builder(default)]
    pub debug: bool,
    /// Whether the debugger steps on a timer.
    #[builder(default)]
    pub auto: bool,
}

/// Attribute that marks the element a step of the tour points at. The
/// targets of the steps match its value, not class names, which change with
/// the hash of a sheet.
pub const TOUR: &str = "data-tour";

/// The steps of the tour, in order: the page and the run, the debug mode
/// switch, each phase, then the debugger with its panes while it steps on a
/// timer.
///
/// # Returns
///
/// The steps.
pub fn steps() -> Vec<Step>
{
    let stage = |title, text, phase| {
        Step::builder()
            .title(title)
            .text(text)
            .target("[data-tour=stage]")
            .stage(phase)
            .debug(true)
    };
    let pane = |title, text, target| {
        Step::builder().title(title).text(text).target(target).debug(true)
    };
    vec![
        Step::builder()
            .title("explorer.tour.welcome")
            .text("explorer.tour.welcome_text")
            .build(),
        Step::builder()
            .title("explorer.tour.samples")
            .text("explorer.tour.samples_text")
            .target("[data-tour=samples]")
            .build(),
        Step::builder()
            .title("explorer.tour.editor")
            .text("explorer.tour.editor_text")
            .target("[data-tour=editor]")
            .build(),
        Step::builder()
            .title("explorer.tour.output")
            .text("explorer.tour.output_text")
            .target("[data-tour=stage]")
            .build(),
        Step::builder()
            .title("explorer.tour.debug")
            .text("explorer.tour.debug_text")
            .target("[data-tour=debug]")
            .build(),
        pane(
            "explorer.tour.pipeline",
            "explorer.tour.pipeline_text",
            "[data-tour=pipeline]",
        )
        .build(),
        stage("explorer.tour.segment", "explorer.tour.segment_text", Phase::Segment)
            .build(),
        stage("explorer.tour.lex", "explorer.tour.lex_text", Phase::Lex).build(),
        stage("explorer.tour.parse", "explorer.tour.parse_text", Phase::Parse).build(),
        stage("explorer.tour.check", "explorer.tour.check_text", Phase::Check).build(),
        stage("explorer.tour.lower", "explorer.tour.lower_text", Phase::Lower).build(),
        stage("explorer.tour.codegen", "explorer.tour.codegen_text", Phase::Codegen)
            .build(),
        stage("explorer.tour.run", "explorer.tour.run_text", Phase::Run).build(),
        pane(
            "explorer.tour.controls",
            "explorer.tour.controls_text",
            "[data-tour=controls]",
        )
        .build(),
        pane("explorer.tour.ops", "explorer.tour.ops_text", "[data-tour=ops]")
            .auto(true)
            .build(),
        pane(
            "explorer.tour.machine",
            "explorer.tour.machine_text",
            "[data-tour=machine]",
        )
        .auto(true)
        .build(),
        pane(
            "explorer.tour.settings",
            "explorer.tour.settings_text",
            "[data-tour=settings]",
        )
        .build(),
        Step::builder()
            .title("explorer.tour.finish")
            .text("explorer.tour.finish_text")
            .debug(true)
            .build(),
    ]
}

/// A box on the page, in px from the top left of the window.
#[derive(Debug, Clone, Copy, PartialEq, Builder)]
pub struct Rect
{
    /// Distance from the left edge.
    pub left: f64,
    /// Distance from the top edge.
    pub top: f64,
    /// The width.
    pub width: f64,
    /// The height.
    pub height: f64,
}

impl Rect
{
    /// The box grown on each side, then cut to the window so its ring
    /// remains in view.
    ///
    /// # Arguments
    ///
    /// * `pad` - The space to add on each side.
    /// * `view` - The width and height of the window.
    ///
    /// # Returns
    ///
    /// The new box.
    pub fn ring(self, pad: f64, view: (f64, f64)) -> Rect
    {
        let left = (self.left - pad).max(1.0);
        let top = (self.top - pad).max(1.0);
        let right = (self.left + self.width + pad).min(view.0 - 1.0);
        let bottom = (self.top + self.height + pad).min(view.1 - 1.0);
        Rect::builder()
            .left(left)
            .top(top)
            .width((right - left).max(0.0))
            .height((bottom - top).max(0.0))
            .build()
    }
}

/// Where the card goes: under the part it points at when it fits there,
/// else over it, else to its right, else to its left, so it leaves the part
/// in view; inside its bottom edge when it fits nowhere around it; centered
/// when it points at nothing. It never leaves the window.
///
/// # Arguments
///
/// * `target` - The box of the part, or `None`.
/// * `card` - The width and height of the card.
/// * `view` - The width and height of the window.
///
/// # Returns
///
/// The left and top of the card.
pub fn place(target: Option<Rect>, card: (f64, f64), view: (f64, f64)) -> (f64, f64)
{
    let (w, h) = card;
    let middle = ((view.0 - w) / 2.0, (view.1 - h) / 2.0);
    let Some(t) = target
    else
    {
        return keep_inside(middle, card, view);
    };
    let left = t.left + (t.width - w) / 2.0;
    let top = t.top + (t.height - h) / 2.0;
    let under = t.top + t.height + GAP;
    let over = t.top - GAP - h;
    let right = t.left + t.width + GAP;
    let before = t.left - GAP - w;
    let inside = (left, t.top + t.height - GAP - h);
    let around = [
        (under + h + GAP <= view.1, (left, under)),
        (over >= GAP, (left, over)),
        (right + w + GAP <= view.0, (right, top)),
        (before >= GAP, (before, top)),
    ];
    let spot = around.into_iter().find(|(fits, _)| *fits).map_or(inside, |(_, at)| at);
    keep_inside(spot, card, view)
}

/// Move a card inside the window, a gap from each edge.
///
/// # Arguments
///
/// * `at` - The left and top of the card.
/// * `card` - The width and height of the card.
/// * `view` - The width and height of the window.
///
/// # Returns
///
/// The left and top inside the window.
fn keep_inside(at: (f64, f64), card: (f64, f64), view: (f64, f64)) -> (f64, f64)
{
    let left = at.0.min(view.0 - card.0 - GAP).max(GAP);
    let top = at.1.min(view.1 - card.1 - GAP).max(GAP);
    (left, top)
}

/// What the tour changes, as it was when the tour opened, so it comes back
/// when the tour ends.
#[derive(Debug, Clone, Builder)]
pub struct Snapshot
{
    /// The text, the options, the stage and the views.
    pub link: Link,
    /// Whether the editor pane showed.
    pub editor_open: bool,
    /// Whether the problems panel was open.
    pub problems_open: bool,
    /// Whether debug mode was on.
    pub debug_mode: bool,
}
