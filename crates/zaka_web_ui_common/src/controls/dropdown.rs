use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use bon::Builder;
use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::div;
use leptos::html::span;
use leptos::prelude::*;
use leptos_use_event_listener::UseEventListenerOptions;
use leptos_use_event_listener::use_event_listener_with_options;
use leptos_use_on_click_outside::on_click_outside;
use leptos_use_window::use_window;
use web_sys::ScrollIntoViewOptions;
use web_sys::ScrollLogicalPosition;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::controls::ClassName;
use crate::icon::check;
use crate::icon::chevron;

/// Number of the next dropdown, for the element ids of its list.
static NEXT: AtomicUsize = AtomicUsize::new(0);

/// Most px the list grows to before it scrolls.
const LIST_MAX: f64 = 320.0;

/// Room in px between the trigger and the list, and between the list and the
/// edge of the window.
const GAP: f64 = 6.0;

/// One choice of a dropdown.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Choice
{
    /// The value it stands for.
    #[builder(into)]
    pub value: String,
    /// The text it shows.
    #[builder(into)]
    pub label: String,
    /// The group it is listed under; a header shows where the group changes.
    #[builder(into)]
    pub group: Option<String>,
}

/// What a dropdown offers and where its pick goes.
#[derive(Builder)]
pub struct Dropdown
{
    /// The choices, grouped in order.
    pub choices: Vec<Choice>,
    /// The value of the picked choice.
    #[builder(into)]
    pub value: Signal<String>,
    /// Called with the value of a choice when it is picked.
    pub on_pick: Callback<String>,
    /// Accessible name of the dropdown.
    #[builder(into)]
    pub label: String,
    /// Text the trigger shows while no choice holds the value.
    #[builder(default, into)]
    pub placeholder: String,
    /// Focus the trigger when the page or dialog around it opens.
    #[builder(default)]
    pub autofocus: bool,
    /// An icon to draw the trigger with, in place of the picked label; the
    /// label and the pick then name the trigger. The list
    /// lines up with the end of the trigger.
    pub icon: Option<fn() -> AnyView>,
}

/// How the trigger of a dropdown looks and what names it.
#[derive(Builder)]
struct Face
{
    /// Accessible name of the dropdown.
    label: String,
    /// Text while no choice holds the value.
    placeholder: String,
    /// Whether the trigger takes the focus when shown.
    autofocus: bool,
    /// The icon to draw the trigger with, if one.
    icon: Option<fn() -> AnyView>,
}

/// The state one dropdown shares between its trigger, its list and its
/// handlers.
#[derive(Clone, Copy)]
struct Ctl
{
    /// Whether the list shows.
    open: RwSignal<bool>,
    /// The highlighted choice, which Enter picks.
    active: RwSignal<usize>,
    /// Placement of the list: its inline rules, and whether it opens above.
    spot: RwSignal<(String, bool)>,
    /// The choices.
    choices: StoredValue<Vec<Choice>>,
    /// The value of the picked choice.
    value: Signal<String>,
    /// Called with a picked value.
    on_pick: Callback<String>,
    /// The trigger, which the list lines up with.
    trigger: NodeRef<html::Button>,
    /// Element id of the list; the choices add `-<index>`.
    list_id: StoredValue<String>,
    /// Whether the list lines up with the end of the trigger, not its start.
    end: bool,
}

impl Ctl
{
    /// Index of the picked choice.
    ///
    /// # Returns
    ///
    /// The index, or `None` while no choice holds the value.
    fn picked(&self) -> Option<usize>
    {
        let value = self.value.get_untracked();
        self.choices.with_value(|c| c.iter().position(|choice| choice.value == value))
    }

    /// Number of choices.
    ///
    /// # Returns
    ///
    /// The count.
    fn count(&self) -> usize { self.choices.with_value(Vec::len) }

    /// Element id of a choice.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the choice.
    ///
    /// # Returns
    ///
    /// The id.
    fn choice_id(&self, idx: usize) -> String
    {
        self.list_id.with_value(|id| format!("{id}-{idx}"))
    }

    /// Open the list next to the trigger, with the picked choice highlighted.
    fn show(&self)
    {
        self.place();
        self.active.set(self.picked().unwrap_or(0));
        self.open.set(true);
    }

    /// Close the list.
    fn hide(&self) { self.open.set(false) }

    /// Pick a choice and close the list.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the choice.
    fn pick(&self, idx: usize)
    {
        let value =
            self.choices.with_value(|c| c.get(idx).map(|choice| choice.value.clone()));
        do_if!(let Some(value) = value, self.on_pick.run(value));
        self.hide();
    }

    /// Move the highlight.
    ///
    /// # Arguments
    ///
    /// * `to` - Where to: a function of the current index and the count.
    fn move_to(&self, to: impl Fn(usize, usize) -> usize)
    {
        let count = self.count();
        ret_if!(count == 0);
        self.active.update(|at| *at = to(*at, count).min(count - 1));
    }

    /// Highlight the next choice whose label starts with a letter.
    ///
    /// # Arguments
    ///
    /// * `letter` - The letter.
    fn jump_to_letter(&self, letter: char)
    {
        let letter = letter.to_lowercase().to_string();
        let count = self.count();
        let from = self.active.get_untracked();
        let starts = |idx: &usize| {
            self.choices
                .with_value(|c| c[*idx].label.to_lowercase().starts_with(&letter))
        };
        let next = (1..=count).map(|step| (from + step) % count).find(starts);
        do_if!(let Some(next) = next, self.active.set(next));
    }

    /// Line the list up with the trigger: under it when there is room,
    /// else above it, no taller than the room on that side.
    fn place(&self)
    {
        let Some(trigger) = self.trigger.get_untracked()
        else
        {
            return;
        };
        let rect = trigger.get_bounding_client_rect();
        let window_height =
            window().inner_height().ok().and_then(|h| h.as_f64()).unwrap_or(0.0);
        let below = window_height - rect.bottom() - GAP * 2.0;
        let above = rect.top() - GAP * 2.0;
        let up = below < LIST_MAX && above > below;
        let room = tern!(up, above, below).min(LIST_MAX);
        let edge = tern!(
            up,
            format!("bottom: {}px;", window_height - rect.top() + GAP),
            format!("top: {}px;", rect.bottom() + GAP)
        );
        let window_width =
            window().inner_width().ok().and_then(|w| w.as_f64()).unwrap_or(0.0);
        let side = tern!(
            self.end,
            format!("right: {}px;", window_width - rect.right()),
            format!("left: {}px;", rect.left())
        );
        let style = format!(
            "{edge} {side} min-width: {}px; max-height: {room}px;",
            rect.width()
        );
        self.spot.set((style, up));
    }
}

/// A dropdown in place of the native select: a trigger that shows the
/// picked choice, and a list of choices under it or above it. The focus remains
/// on the trigger; the arrows, Home and End move the highlight, a letter
/// jumps to a choice, Enter or Space picks, and Escape or a click outside
/// closes the list. Its rules sit in the shared controls sheet.
///
/// # Arguments
///
/// * `spec` - What it offers and where its pick goes.
///
/// # Returns
///
/// The view.
pub fn dropdown(spec: Dropdown) -> impl IntoView
{
    let Dropdown { choices, value, on_pick, label, placeholder, autofocus, icon } =
        spec;
    let list_id = format!("dropdown-{}", NEXT.fetch_add(1, Ordering::Relaxed));
    let ctl = Ctl {
        open: RwSignal::new(false),
        active: RwSignal::new(0),
        spot: RwSignal::new((String::new(), false)),
        choices: StoredValue::new(choices),
        value,
        on_pick,
        trigger: NodeRef::new(),
        list_id: StoredValue::new(list_id),
        end: icon.is_some(),
    };
    let root = NodeRef::<html::Span>::new();
    let _ = on_click_outside(root, move |_| ctl.hide());
    follow_the_trigger(ctl);
    show_the_active(ctl);
    let list = move || ctl.open.get().then(|| list_view(ctl));
    span().class(ClassName::DROPDOWN).node_ref(root).child((
        trigger_view(
            ctl,
            Face::builder()
                .label(label)
                .placeholder(placeholder)
                .autofocus(autofocus)
                .maybe_icon(icon)
                .build(),
        ),
        list,
    ))
}

/// The label of the picked choice.
///
/// # Arguments
///
/// * `ctl` - The dropdown.
///
/// # Returns
///
/// The label, or `None` while no choice holds the value.
fn picked_label(ctl: Ctl) -> Option<String>
{
    let value = ctl.value.get();
    ctl.choices.with_value(|c| {
        c.iter().find(|choice| choice.value == value).map(|choice| choice.label.clone())
    })
}

/// The trigger: the picked label, or the placeholder, and a chevron; or an
/// icon and a chevron, whose name joins the label of the dropdown and the
/// pick.
///
/// # Arguments
///
/// * `ctl` - The dropdown.
/// * `face` - How the trigger looks and what names it.
///
/// # Returns
///
/// The view.
fn trigger_view(ctl: Ctl, face: Face) -> impl IntoView
{
    let Face { label, placeholder, autofocus, icon } = face;
    let shown = move || {
        let found = picked_label(ctl);
        let missing = found.is_none();
        span()
            .class(ClassName::DROPDOWN_VALUE)
            .class((ClassName::DROPDOWN_PLACEHOLDER, missing))
            .child(found.unwrap_or_else(|| placeholder.clone()))
    };
    let name = move || {
        let pick = picked_label(ctl).filter(|_| icon.is_some());
        pick.map_or_else(|| label.clone(), |p| format!("{label}: {p}"))
    };
    let drawn = icon.map_or_else(
        || (shown, chevron()).into_any(),
        |draw| (draw(), chevron()).into_any(),
    );
    let active_id = move || ctl.open.get().then(|| ctl.choice_id(ctl.active.get()));
    button()
        .class(ClassName::DROPDOWN_TRIGGER)
        .class((ClassName::DROPDOWN_TRIGGER_OPEN, move || ctl.open.get()))
        .class((ClassName::DROPDOWN_TRIGGER_ICON, icon.is_some()))
        .node_ref(ctl.trigger)
        .r#type("button")
        .attr("role", "combobox")
        .attr("aria-label", name.clone())
        .attr("title", move || icon.is_some().then(name.clone()))
        .attr("aria-haspopup", "listbox")
        .attr("aria-expanded", move || ctl.open.get().to_string())
        .attr("aria-controls", ctl.list_id.get_value())
        .attr("aria-activedescendant", active_id)
        .attr("autofocus", autofocus)
        .on(ev::click, move |_| tern!(ctl.open.get_untracked(), ctl.hide(), ctl.show()))
        .on(ev::keydown, move |e| on_press(ctl, &e))
        .child(drawn)
}

/// Answer a press on the trigger.
///
/// # Arguments
///
/// * `ctl` - The dropdown.
/// * `e` - The press.
fn on_press(ctl: Ctl, e: &ev::KeyboardEvent)
{
    let pressed = e.key();
    let open = ctl.open.get_untracked();
    let opens = matches!(pressed.as_str(), "ArrowDown" | "ArrowUp" | "Enter" | " ");
    ret_if!(!open && opens, e.prevent_default(), ctl.show());
    ret_if!(!open);
    // Tab moves on and closes the list; it keeps its own effect.
    ret_if!(pressed == "Tab", ctl.hide());
    let single = pressed.chars().count() == 1;
    let letter = pressed.chars().next().filter(|c| single && c.is_alphanumeric());
    match (pressed.as_str(), letter)
    {
        ("ArrowDown", _) => ctl.move_to(|at, _| at + 1),
        ("ArrowUp", _) => ctl.move_to(|at, _| at.saturating_sub(1)),
        ("Home" | "PageUp", _) => ctl.move_to(|_, _| 0),
        ("End" | "PageDown", _) => ctl.move_to(|_, count| count),
        ("Enter" | " ", _) => ctl.pick(ctl.active.get_untracked()),
        // Escape closes the list alone, not the dialog around it.
        ("Escape", _) => ctl.hide(),
        (_, Some(letter)) => ctl.jump_to_letter(letter),
        _ => return,
    }
    e.prevent_default();
    e.stop_propagation();
}

/// The list of choices, with a header where the group changes.
///
/// # Arguments
///
/// * `ctl` - The dropdown.
///
/// # Returns
///
/// The view.
fn list_view(ctl: Ctl) -> impl IntoView
{
    let rows = ctl.choices.with_value(|choices| {
        let groups = choices.iter().enumerate().map(|(idx, choice)| {
            let starts = idx == 0 || choices[idx - 1].group != choice.group;
            choice.group.clone().filter(|_| starts)
        });
        let heads: Vec<Option<String>> = groups.collect();
        choices
            .iter()
            .cloned()
            .enumerate()
            .zip(heads)
            .map(|((idx, choice), head)| {
                let head = head.map(|h| {
                    div()
                        .class(ClassName::DROPDOWN_GROUP)
                        .attr("role", "presentation")
                        .child(h)
                });
                (head, choice_view(ctl, idx, choice))
            })
            .collect_view()
    });
    let (style, up) = ctl.spot.get_untracked();
    // A smooth scroller on the page, such as Lenis on the website, takes the
    // wheel for the page; the mark leaves the wheel to the list.
    div()
        .id(ctl.list_id.get_value())
        .class(ClassName::DROPDOWN_LIST)
        .class((ClassName::DROPDOWN_LIST_UP, up))
        .attr("role", "listbox")
        .attr("style", style)
        .attr("data-lenis-prevent", "")
        .child(rows)
}

/// One choice of the list; the pointer highlights it and a click picks it.
///
/// # Arguments
///
/// * `ctl` - The dropdown.
/// * `idx` - Its index.
/// * `choice` - The choice.
///
/// # Returns
///
/// The view.
fn choice_view(ctl: Ctl, idx: usize, choice: Choice) -> impl IntoView
{
    let value = choice.value.clone();
    let picked = Memo::new(move |_| ctl.value.get() == value);
    let mark = move || picked.get().then(check);
    div()
        .id(ctl.choice_id(idx))
        .class(ClassName::DROPDOWN_CHOICE)
        .class((ClassName::DROPDOWN_CHOICE_ACTIVE, move || ctl.active.get() == idx))
        .class((ClassName::DROPDOWN_CHOICE_PICKED, picked))
        .attr("role", "option")
        .attr("aria-selected", move || picked.get().to_string())
        .on(ev::mousemove, move |_| {
            do_if!(ctl.active.get_untracked() != idx, ctl.active.set(idx))
        })
        // The focus remains on the trigger.
        .on(ev::mousedown, |e| e.prevent_default())
        .on(ev::click, move |_| ctl.pick(idx))
        .child((span().class(ClassName::DROPDOWN_LABEL).child(choice.label), mark))
}

/// Keep the list lined up with the trigger while the page or the dialog
/// around it scrolls or the window changes size.
///
/// # Arguments
///
/// * `ctl` - The dropdown.
fn follow_the_trigger(ctl: Ctl)
{
    let options = UseEventListenerOptions::default().capture(true);
    let replace = move || do_if!(ctl.open.get_untracked(), ctl.place());
    let _ = use_event_listener_with_options(
        use_window(),
        ev::scroll,
        move |_| replace(),
        options,
    );
    let _ = use_event_listener_with_options(
        use_window(),
        ev::resize,
        move |_| replace(),
        options,
    );
}

/// Bring the highlighted choice into view as the highlight moves.
///
/// # Arguments
///
/// * `ctl` - The dropdown.
fn show_the_active(ctl: Ctl)
{
    Effect::new(move |_| {
        let (open, idx) = (ctl.open.get(), ctl.active.get());
        ret_if!(!open);
        request_animation_frame(move || {
            let Some(el) = document().get_element_by_id(&ctl.choice_id(idx))
            else
            {
                return;
            };
            let options = ScrollIntoViewOptions::new();
            options.set_block(ScrollLogicalPosition::Nearest);
            el.scroll_into_view_with_scroll_into_view_options(&options);
        });
    });
}
