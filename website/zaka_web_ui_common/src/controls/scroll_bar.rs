use std::time::Duration;

use leptos::ev;
use leptos::html;
use leptos::html::div;
use leptos::prelude::*;
use leptos_use_event_listener::use_event_listener;
use leptos_use_window::use_window;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use super::ClassName;

/// The shortest the thumb gets, in px, so a long page leaves it a fair
/// target for the pointer.
const MIN_THUMB: f64 = 40.0;

/// How long the thumb remains after its box stops moving, in ms.
const LINGER_MS: u64 = 900;

/// The width of the track, in px, at the right edge of what scrolls.
const TRACK_WIDTH: f64 = 14.0;

/// What a scroll bar follows: the page, or a box that scrolls on its own and
/// takes `ClassName::SCROLL_OWN`.
#[derive(Debug, Clone, Copy)]
pub enum Scroller
{
    /// The page, in the window.
    Page,
    /// A box of the page.
    Element(NodeRef<html::Div>),
}

/// The thumb for a box that scrolls: where it sits and how tall it is.
///
/// # Arguments
///
/// * `view` - The height the box shows, in px.
/// * `page` - The height of its contents, in px.
/// * `scroll` - How far it has scrolled, in px.
///
/// # Returns
///
/// The top and the height of the thumb in px, from the top of the box, or
/// `None` when the contents fit the box.
pub fn thumb_for(view: f64, page: f64, scroll: f64) -> Option<(f64, f64)>
{
    ret_if!(page <= view + 1.0, None);
    let height = (view * view / page).clamp(MIN_THUMB.min(view), view);
    let done = (scroll / (page - view)).clamp(0.0, 1.0);
    Some((done * (view - height), height))
}

impl Scroller
{
    /// The height it shows, the height of its contents and how far it has
    /// scrolled, in px.
    ///
    /// # Returns
    ///
    /// The three, or `None` while the box is not on the page.
    fn heights(self) -> Option<(f64, f64, f64)>
    {
        let Scroller::Element(node) = self
        else
        {
            let view = window().inner_height().ok()?.as_f64()?;
            let page = document().document_element()?.scroll_height();
            return Some((view, f64::from(page), window().scroll_y().ok()?));
        };
        let el = node.get_untracked()?;
        let (view, page) = (el.client_height(), el.scroll_height());
        Some((f64::from(view), f64::from(page), f64::from(el.scroll_top())))
    }

    /// Where the track goes over a box: its top, its left and its height in
    /// px, from the window.
    ///
    /// # Returns
    ///
    /// The place, or `None` over the page, whose track the sheet places.
    fn track(self) -> Option<(f64, f64, f64)>
    {
        let Scroller::Element(node) = self
        else
        {
            return None;
        };
        let rect = node.get_untracked()?.get_bounding_client_rect();
        Some((rect.top(), rect.right() - TRACK_WIDTH, rect.height()))
    }

    /// Scroll to a place.
    ///
    /// # Arguments
    ///
    /// * `top` - How far from the top, in px.
    fn scroll_to(self, top: f64)
    {
        let Scroller::Element(node) = self
        else
        {
            return window().scroll_to_with_x_and_y(0.0, top);
        };
        do_if!(let Some(el) = node.get_untracked(), el.set_scroll_top(top as i32));
    }
}

/// A scroll bar drawn over what scrolls: a thumb at its right edge that shows
/// while it moves or the pointer rests on the edge, and that a drag moves. It
/// hides when the contents fit.
///
/// # Arguments
///
/// * `scroller` - What it follows.
///
/// # Returns
///
/// The view.
pub fn scroll_bar(scroller: Scroller) -> impl IntoView
{
    let thumb = RwSignal::new(None::<(f64, f64)>);
    let track = RwSignal::new(None::<(f64, f64, f64)>);
    let awake = RwSignal::new(false);
    let hovered = RwSignal::new(false);
    // The pointer and the scroll where a drag of the thumb began.
    let grab = RwSignal::new(None::<(f64, f64)>);
    let timer = StoredValue::new(None::<TimeoutHandle>);
    let measure = move || {
        thumb.set(scroller.heights().and_then(|(v, p, s)| thumb_for(v, p, s)));
        track.set(scroller.track());
    };
    let rest = move || {
        do_if!(let Some(t) = timer.get_value(), t.clear());
        let sleep = move || {
            let idle = !hovered.get_untracked() && grab.get_untracked().is_none();
            do_if!(idle, awake.set(false));
        };
        let wait = Duration::from_millis(LINGER_MS);
        timer.set_value(set_timeout_with_handle(sleep, wait).ok());
    };
    let wake = move || {
        measure();
        awake.set(true);
        rest();
    };
    Effect::new(move |_| measure());
    let _ = use_event_listener(use_window(), ev::resize, move |_| measure());
    match scroller
    {
        Scroller::Page =>
        {
            let _ = use_event_listener(use_window(), ev::scroll, move |_| wake());
        }
        Scroller::Element(node) =>
        {
            let _ = use_event_listener(node, ev::scroll, move |_| wake());
        }
    }
    let start = move |e: ev::PointerEvent| {
        e.prevent_default();
        let at = scroller.heights().map(|(_, _, s)| s).unwrap_or_default();
        grab.set(Some((f64::from(e.client_y()), at)));
        let target = event_target::<web_sys::Element>(&e);
        let _ = target.set_pointer_capture(e.pointer_id());
    };
    let follow = move |e: ev::PointerEvent| {
        let Some((from, at)) = grab.get_untracked()
        else
        {
            return;
        };
        let Some(((view, page, _), (_, height))) =
            scroller.heights().zip(thumb.get_untracked())
        else
        {
            return;
        };
        let rate = (page - view) / (view - height).max(1.0);
        scroller.scroll_to(at + (f64::from(e.client_y()) - from) * rate);
    };
    let release = move |_: ev::PointerEvent| {
        grab.set(None);
        rest();
    };
    let enter = move |_: ev::PointerEvent| {
        hovered.set(true);
        wake();
    };
    let leave = move |_: ev::PointerEvent| {
        hovered.set(false);
        rest();
    };
    let knob = div()
        .class(ClassName::SCROLL_THUMB)
        .style(("height", move || thumb.get().map(|(_, h)| format!("{h}px"))))
        .style(("translate", move || thumb.get().map(|(t, _)| format!("0 {t}px"))))
        .on(ev::pointerdown, start)
        .on(ev::pointermove, follow)
        .on(ev::pointerup, release)
        .on(ev::pointercancel, release);
    let place = move |pick: fn((f64, f64, f64)) -> f64| {
        move || track.get().map(|t| format!("{}px", pick(t)))
    };
    div()
        .class(ClassName::SCROLL_TRACK)
        .class((ClassName::SCROLL_NONE, move || thumb.get().is_none()))
        .class((ClassName::SCROLL_AWAKE, move || awake.get()))
        .class((ClassName::SCROLL_DRAG, move || grab.with(Option::is_some)))
        .style(("top", place(|t| t.0)))
        .style(("left", place(|t| t.1)))
        .style(("height", place(|t| t.2)))
        .attr("aria-hidden", "true")
        .on(ev::pointerenter, enter)
        .on(ev::pointerleave, leave)
        .child(knob)
}
