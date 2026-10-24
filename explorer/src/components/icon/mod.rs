use leptos::prelude::*;
use leptos::svg::path;
use leptos::svg::svg;

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

/// A window with its left pane marked off, on a 24 unit grid.
const LEFT_PANE: [&str; 2] = [
    "M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z",
    "M9 3v18",
];

/// A circle with an exclamation mark, on a 24 unit grid.
const ALERT: [&str; 3] =
    ["M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0z", "M12 8v4", "M12 16h.01"];

/// A check mark, on a 24 unit grid.
const CHECK: [&str; 1] = ["M20 6 9 17l-5-5"];

/// A cross, on a 24 unit grid.
const CROSS: [&str; 2] = ["M18 6 6 18", "m6 6 12 12"];

/// A chevron that points down, on a 24 unit grid.
const CHEVRON: [&str; 1] = ["m6 9 6 6 6-6"];

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
        .attr("class", "icon")
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

/// A circle with an exclamation mark, for a stage that did not run.
///
/// # Returns
///
/// The view.
pub fn alert() -> impl IntoView { line_icon(&ALERT) }

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

/// A chevron that points down; rotate it with CSS for other sides.
///
/// # Returns
///
/// The view.
pub fn chevron() -> impl IntoView { line_icon(&CHEVRON) }
