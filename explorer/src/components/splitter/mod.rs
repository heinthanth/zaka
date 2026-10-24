use bon::Builder;
use leptos::ev;
use leptos::html;
use leptos::html::div;
use leptos::prelude::*;
use leptos_scoped_style::scss_file;
use leptos_use_draggable::UseDraggableCallbackArgs;
use leptos_use_draggable::UseDraggableOptions;
use leptos_use_draggable::UseDraggableReturn;
use leptos_use_draggable::use_draggable_with_options;
use strum::IntoStaticStr;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::state::split::MAX_SPLIT;
use crate::state::split::MIN_SPLIT;
use crate::state::split::SPLIT_STEP;
use crate::state::split::clamp_split;
use crate::state::split::split_at;

/// Sheet of the splitter.
static SHEET: ScopedSheet =
    ScopedSheet::new("zaka-splitter", scss_file!("src/components/splitter/mod.sass"));

/// Attribute on the root element while a splitter is dragged; its value is
/// the orientation, so the page keeps the resize cursor.
const RESIZING: &str = "data-resizing";

/// How the panes on both sides of a splitter sit; the static name is the
/// ARIA orientation of the splitter itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
pub enum Axis
{
    /// Panes next to each other; the splitter is an upright bar.
    #[strum(serialize = "vertical")]
    Columns,
    /// Stacked panes; the splitter is a flat bar.
    #[strum(serialize = "horizontal")]
    Rows,
}

/// What a splitter splits and where the share is kept.
#[derive(Clone, Copy, Builder)]
pub struct Split
{
    /// How the panes sit.
    pub axis: Axis,
    /// The element whose length is split.
    pub container: NodeRef<html::Div>,
    /// Share of the first pane.
    pub share: Signal<f64>,
    /// Set the share of the first pane.
    pub set_share: WriteSignal<f64>,
    /// Share a double click goes back to.
    pub initial: f64,
    /// Locale id of the accessible name of the splitter.
    pub label: &'static str,
    /// Whether the splitter shows and takes drags.
    #[builder(default = Signal::stored(true))]
    pub enabled: Signal<bool>,
}

/// A bar between two panes that resizes them: drag it, move it with the
/// arrows, or double-click it to go back to the first share.
///
/// # Arguments
///
/// * `split` - What it splits.
///
/// # Returns
///
/// The view.
pub fn splitter(split: Split) -> impl IntoView
{
    let handle = NodeRef::<html::Div>::new();
    let options = UseDraggableOptions::default()
        .prevent_default(true)
        .on_move(move |args: UseDraggableCallbackArgs| follow_pointer(split, &args));
    let UseDraggableReturn { is_dragging, .. } =
        use_draggable_with_options(handle, options);
    let orientation: &'static str = split.axis.into();
    mark_resizing(is_dragging, orientation);
    let percent = move || (split.share.get() * 100.0).round().to_string();
    let bar = div()
        .class(format!("splitter splitter-{orientation}"))
        .class(("splitter-active", is_dragging))
        .class(("splitter-off", move || !split.enabled.get()))
        .node_ref(handle)
        .attr("role", "separator")
        .attr("tabindex", "0")
        .attr("aria-label", t!(split.label.to_string()))
        .attr("aria-orientation", orientation)
        .attr("aria-valuemin", (MIN_SPLIT * 100.0).to_string())
        .attr("aria-valuemax", (MAX_SPLIT * 100.0).to_string())
        .attr("aria-valuenow", percent)
        .attr("aria-disabled", move || (!split.enabled.get()).to_string())
        .title(t!("explorer.splitter.hint"))
        .on(ev::dblclick, move |_| split.set_share.set(split.initial))
        .on(ev::keydown, move |e| step_share(split, &e));
    scoped(&SHEET, handle, bar)
}

/// Set the share from the pointer position over the container.
///
/// # Arguments
///
/// * `split` - What it splits.
/// * `args` - The drag event.
fn follow_pointer(split: Split, args: &UseDraggableCallbackArgs)
{
    let Some(el) = split.container.get_untracked()
    else
    {
        return;
    };
    let rect = el.get_bounding_client_rect();
    let e = &args.event;
    let share = tern!(
        split.axis == Axis::Columns,
        split_at(f64::from(e.client_x()), rect.left(), rect.width()),
        split_at(f64::from(e.client_y()), rect.top(), rect.height())
    );
    split.set_share.set(share);
}

/// Move the share one step with an arrow press.
///
/// # Arguments
///
/// * `split` - What it splits.
/// * `e` - The press.
fn step_share(split: Split, e: &ev::KeyboardEvent)
{
    let step = match e.key().as_str()
    {
        "ArrowLeft" | "ArrowUp" => -SPLIT_STEP,
        "ArrowRight" | "ArrowDown" => SPLIT_STEP,
        _ => return,
    };
    e.prevent_default();
    split.set_share.set(clamp_split(split.share.get_untracked() + step));
}

/// Mark the root element while dragging, so the cursor holds and no text is
/// selected when the pointer leaves the bar.
///
/// # Arguments
///
/// * `is_dragging` - Whether the bar is dragged.
/// * `orientation` - The splitter orientation.
fn mark_resizing(is_dragging: Signal<bool>, orientation: &'static str)
{
    Effect::new(move |_| {
        let on = is_dragging.get();
        let Some(root) = document().document_element()
        else
        {
            return;
        };
        let _ = tern!(
            on,
            root.set_attribute(RESIZING, orientation),
            root.remove_attribute(RESIZING)
        );
    });
}
