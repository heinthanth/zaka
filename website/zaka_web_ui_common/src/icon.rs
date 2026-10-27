use leptos::prelude::*;
use leptos::svg::path;
use leptos::svg::svg;

use crate::controls::ClassName;

/// Outline of a gear, on a 24 unit grid.
const GEAR: [&str; 2] = [
    "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 \
     0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 \
     1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 \
     2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 \
     0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 \
     2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 \
     1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 \
     1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z",
    "M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0z",
];

/// A window with its left pane marked off and shaded, on a 24 unit grid.
const LEFT_PANE: [&str; 3] = [
    "M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z",
    "M9 3v18",
    "M6 7v10",
];

/// A window with its middle pane marked off and shaded, on a 24 unit grid.
const MIDDLE_PANE: [&str; 4] = [
    "M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z",
    "M9 3v18",
    "M15 3v18",
    "M12 7v10",
];

/// A window with its right pane marked off and shaded, on a 24 unit grid.
const RIGHT_PANE: [&str; 3] = [
    "M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z",
    "M15 3v18",
    "M18 7v10",
];

/// A window with its bottom pane marked off and shaded, on a 24 unit grid.
const BOTTOM_PANE: [&str; 3] = [
    "M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z",
    "M3 15h18",
    "M7 18h10",
];

/// Three sliders on their tracks, on a 24 unit grid.
const SLIDERS: [&str; 9] = [
    "M4 6h9", "M17 6h3", "M15 4v4", "M4 12h3", "M11 12h9", "M9 10v4", "M4 18h7",
    "M15 18h5", "M13 16v4",
];

/// A circle with an exclamation mark, on a 24 unit grid.
const ALERT: [&str; 3] =
    ["M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0z", "M12 8v4", "M12 16h.01"];

/// A circle with the letter i, on a 24 unit grid.
const INFO: [&str; 3] =
    ["M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0z", "M12 16v-4", "M12 8h.01"];

/// A globe with a meridian and the equator, on a 24 unit grid.
const GLOBE: [&str; 3] = [
    "M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0z",
    "M2 12h20",
    "M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 \
     0 1 4-10z",
];

/// A check mark, on a 24 unit grid.
const CHECK: [&str; 1] = ["M20 6 9 17l-5-5"];

/// A cross, on a 24 unit grid.
const CROSS: [&str; 2] = ["M18 6 6 18", "m6 6 12 12"];

/// A book of color swatches fanned open, on a 24 unit grid.
const SWATCHES: [&str; 4] = [
    "M11 17a4 4 0 0 1-8 0V5a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2Z",
    "M16.7 13H19a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2H7",
    "M7 17h.01",
    "m11 8 2.3-2.3a2.4 2.4 0 0 1 3.404.004L18.6 7.6a2.4 2.4 0 0 1 .026 3.434L9.9 19.8",
];

/// Three lines, one over the next, on a 24 unit grid.
const MENU: [&str; 3] = ["M4 7h16", "M4 12h16", "M4 17h16"];

/// A chevron that points down, on a 24 unit grid.
const CHEVRON: [&str; 1] = ["m6 9 6 6 6-6"];

/// A square, on a 24 unit grid.
const STOP: [&str; 1] =
    ["M7 5h10a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2z"];

/// A triangle that points right with a small clock at its lower right, on
/// a 24 unit grid.
const TIMED_START: [&str; 3] = [
    "M4 3v13l10.5-6.5z",
    "M22.5 17.5a4.5 4.5 0 1 1-9 0 4.5 4.5 0 0 1 9 0z",
    "M18 15.5v2l1.4 1.2",
];

/// A bug, on a 24 unit grid.
const BUG: [&str; 11] = [
    "m8 2 1.88 1.88",
    "M14.12 3.88 16 2",
    "M9 7.13v-1a3 3 0 1 1 6 0v1",
    "M12 20c-3.3 0-6-2.7-6-6v-3a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v3c0 3.3-2.7 6-6 6",
    "M12 20v-9",
    "M6.53 9C4.6 8.8 3 7.1 3 5",
    "M6 13H2",
    "M3 21c0-2.1 1.7-3.9 3.8-4",
    "M20.97 5c0 2.1-1.6 3.8-3.5 4",
    "M22 13h-4",
    "M17.2 17c2.1.1 3.8 1.9 3.8 4",
];

/// Two bars, on a 24 unit grid.
const PAUSE: [&str; 2] = ["M8 5v14", "M16 5v14"];

/// A triangle that points right, on a 24 unit grid.
const START: [&str; 1] = ["M7 4v16l13-8z"];

/// An arrow down onto a dot, on a 24 unit grid.
const STEP: [&str; 3] = ["M12 3v11", "m7 9 5 5 5-5", "M12 20h.01"];

/// An arc over a dot, on a 24 unit grid.
const STEP_OVER: [&str; 3] =
    ["M3 15a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3l3 2.7", "M21 5v6h-6", "M12 19h.01"];

/// An arrow up from a line, on a 24 unit grid.
const STEP_OUT: [&str; 3] = ["M12 16V4", "m7 9 5-5 5 5", "M5 20h14"];

/// An arrow round a circle, back to its start, on a 24 unit grid.
const RESTART: [&str; 2] =
    ["M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8", "M3 3v5h5"];

/// A line icon drawn with the current text color.
///
/// # Arguments
///
/// * `paths` - The path data, on a 24 unit grid.
///
/// # Returns
///
/// The view.
fn line_icon(paths: &'static [&'static str]) -> impl IntoView
{
    let strokes = paths.iter().map(|d| path().attr("d", *d)).collect_view();
    svg()
        .attr("class", ClassName::ICON)
        .attr("viewBox", "0 0 24 24")
        .attr("fill", "none")
        .attr("stroke", "currentColor")
        .attr("stroke-width", "2")
        .attr("stroke-linecap", "round")
        .attr("stroke-linejoin", "round")
        .attr("aria-hidden", "true")
        .child(strokes)
}

/// The settings gear.
///
/// # Returns
///
/// The view.
pub fn gear() -> impl IntoView { line_icon(&GEAR) }

/// A window with its left pane marked off, for the editor pane switch.
///
/// # Returns
///
/// The view.
pub fn left_pane() -> impl IntoView { line_icon(&LEFT_PANE) }

/// Three sliders, for the compiler options.
///
/// # Returns
///
/// The view.
pub fn sliders() -> impl IntoView { line_icon(&SLIDERS) }

/// A window with its middle pane marked off, for the stage pane switch.
///
/// # Returns
///
/// The view.
pub fn middle_pane() -> impl IntoView { line_icon(&MIDDLE_PANE) }

/// A window with its right pane marked off, for the dock switch.
///
/// # Returns
///
/// The view.
pub fn right_pane() -> impl IntoView { line_icon(&RIGHT_PANE) }

/// A window with its bottom pane marked off, for the bottom panel switch.
///
/// # Returns
///
/// The view.
pub fn bottom_pane() -> impl IntoView { line_icon(&BOTTOM_PANE) }

/// A circle with an exclamation mark, for a stage that did not run.
///
/// # Returns
///
/// The view.
pub fn alert() -> impl IntoView { line_icon(&ALERT) }

/// A circle with the letter i, for the about button.
///
/// # Returns
///
/// The view.
pub fn info() -> impl IntoView { line_icon(&INFO) }

/// A check mark, for the picked choice of a list.
///
/// # Returns
///
/// The view.
pub fn check() -> impl IntoView { line_icon(&CHECK) }

/// A cross, for close buttons.
///
/// # Returns
///
/// The view.
pub fn cross() -> impl IntoView { line_icon(&CROSS) }

/// A book of color swatches, the mark of the theme.
///
/// # Returns
///
/// The view.
pub fn swatches() -> impl IntoView { line_icon(&SWATCHES) }

/// Three lines, the button of a menu.
///
/// # Returns
///
/// The view.
pub fn menu() -> impl IntoView { line_icon(&MENU) }

/// A chevron that points down; rotate it with CSS for other sides.
///
/// # Returns
///
/// The view.
pub fn chevron() -> impl IntoView { line_icon(&CHEVRON) }

/// A square, to stop the program.
///
/// # Returns
///
/// The view.
pub fn stop() -> impl IntoView { line_icon(&STOP) }

/// A triangle with a small clock, to start the timed stepping.
///
/// # Returns
///
/// The view.
pub fn timed_start() -> impl IntoView { line_icon(&TIMED_START) }

/// A bug, for the debug mode switch.
///
/// # Returns
///
/// The view.
pub fn bug() -> impl IntoView { line_icon(&BUG) }

/// Two bars, to pause the timed stepping.
///
/// # Returns
///
/// The view.
pub fn pause() -> impl IntoView { line_icon(&PAUSE) }

/// A triangle that points right, to run on to the next break point.
///
/// # Returns
///
/// The view.
pub fn start() -> impl IntoView { line_icon(&START) }

/// An arrow down onto a dot, to run one op.
///
/// # Returns
///
/// The view.
pub fn step() -> impl IntoView { line_icon(&STEP) }

/// An arc over a dot, to run one op or a whole call.
///
/// # Returns
///
/// The view.
pub fn step_over() -> impl IntoView { line_icon(&STEP_OVER) }

/// An arrow up from a line, to run to the end of the call.
///
/// # Returns
///
/// The view.
pub fn step_out() -> impl IntoView { line_icon(&STEP_OUT) }

/// An arrow round a circle, to start again.
///
/// # Returns
///
/// The view.
pub fn restart() -> impl IntoView { line_icon(&RESTART) }

/// A globe, for the language picker.
///
/// # Returns
///
/// The view.
pub fn globe() -> impl IntoView { line_icon(&GLOBE) }
