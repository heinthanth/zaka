use leptos::prelude::document;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::DomRect;
use web_sys::Element;
use web_sys::ResizeObserver;
use zakac_rust_sugars::tern;

use crate::components::glance::ClassName as Glance;

/// Room between an arrow and the words or the note at its ends, in px.
const GAP: f64 = 6.0;

/// How far a note starts past the middle of its words, or past the end of
/// its line when it sits beside it, in px.
const LEAD: f64 = 44.0;

/// How far an arrow leaves the side of its note before it bends, in px.
const SWING: f64 = 32.0;

/// How far under its words an arrow from a note beside the line dips before
/// it turns up into them, in px.
const DIP: f64 = 22.0;

/// The farthest a note beside its line can sit from the words it points at,
/// in px; past it, the arrow would run under most of the line, so the note
/// goes under the line.
const MAX_SWEEP: f64 = 260.0;

/// The most room from the top of a note to the middle of its first line, in
/// px.
const NOTE_MIDDLE: f64 = 14.0;

/// Length of each stroke of an arrow head, in px.
const HEAD: f64 = 8.0;

/// Half the angle between the two strokes of an arrow head, in radians.
const SPREAD: f64 = 0.5;

/// A point in the box of a line of the glance, in px.
#[derive(Clone, Copy)]
struct Point
{
    /// From the left edge.
    left: f64,
    /// From the top edge.
    top: f64,
}

/// The curve of an arrow: where it starts, the two points it bends toward,
/// and where it ends.
struct Curve
{
    /// Where it leaves the note.
    start: Point,
    /// The point it bends toward first.
    out: Point,
    /// The point it bends toward last.
    into: Point,
    /// Where it meets the words.
    tip: Point,
}

/// Draw the arrows of the glance, from each note to the words it points at,
/// and draw them again each time the program changes size or the fonts come
/// in, since either moves the words. A page with no glance draws nothing.
pub fn start()
{
    let found = document().query_selector(&format!(".{}", Glance::GLANCE_CODE));
    let Ok(Some(code)) = found
    else
    {
        return;
    };
    let lines = code.clone();
    let redraw = Closure::<dyn FnMut(JsValue)>::new(move |_| draw_all(&lines));
    // The observer also fires once when it starts, which draws the first time.
    let _ = ResizeObserver::new(redraw.as_ref().unchecked_ref())
        .map(|observer| observer.observe(&code));
    let _ = document().fonts().ready().map(|loading| loading.then(&redraw));
    redraw.forget();
}

/// The first element under `root` with the class `class`.
///
/// # Arguments
///
/// * `root` - Where to look.
/// * `class` - The class.
///
/// # Returns
///
/// The element, if there is one.
fn find(root: &Element, class: &str) -> Option<Element>
{
    root.query_selector(&format!(".{class}")).ok().flatten()
}

/// The last box of an element on the page: the box of its last line when it
/// wraps.
///
/// # Arguments
///
/// * `element` - The element.
///
/// # Returns
///
/// The box, if the element shows.
fn last_box(element: &Element) -> Option<DomRect>
{
    let boxes = element.get_client_rects();
    boxes.get(boxes.length().checked_sub(1)?)
}

/// Draw the arrow of each line of the program.
///
/// # Arguments
///
/// * `code` - The list of lines.
fn draw_all(code: &Element)
{
    let Ok(lines) = code.query_selector_all(&format!(".{}", Glance::GLANCE_LINE))
    else
    {
        return;
    };
    (0..lines.length())
        .filter_map(|i| lines.item(i)?.dyn_into::<Element>().ok())
        .for_each(|line| {
            let _ = draw(&line);
        });
}

/// Place the note of one line, then draw its arrow to the words it points
/// at. The note sits beside the line, past its last word, when the line
/// takes one row, the row has the room and the words are near the end of the
/// line; else it sits in the gap under the line, just past the words. A
/// note beside the line sits level with its first row, so a line that wraps
/// would run under it.
/// The note keeps one width in both places, so the choice stands when the
/// page measures again.
///
/// # Arguments
///
/// * `line` - The line.
///
/// # Returns
///
/// `None` when a part of the line is missing.
fn draw(line: &Element) -> Option<()>
{
    let pin = find(line, Glance::GLANCE_PIN)?;
    let text = find(line, Glance::GLANCE_TEXT)?;
    let note = find(line, Glance::GLANCE_LABEL)?;
    let pen = find(line, Glance::GLANCE_PEN)?;
    let origin = line.get_bounding_client_rect();
    let words = last_box(&pin)?;
    let tip = Point {
        left: (words.left() + words.right()) / 2.0 - origin.left(),
        top: words.bottom() + GAP - origin.top(),
    };
    let column = text.get_bounding_client_rect();
    let column_left = column.left() - origin.left();
    let last = last_box(&text.last_element_child()?)?;
    let end = last.right() - origin.left();
    let width = note.get_bounding_client_rect().width();
    let one_row = last.top() - column.top() < GAP;
    let beside = one_row
        && end + LEAD + width <= column.right() - origin.left()
        && end + LEAD - tip.left <= MAX_SWEEP;
    let room = (column.width() - width).max(0.0);
    let shift = tern!(
        beside,
        end + LEAD - column_left,
        (tip.left + LEAD - column_left).clamp(0.0, room)
    );
    note.set_attribute("data-side", tern!(beside, "beside", "below")).ok()?;
    note.set_attribute("style", &format!("--shift: {shift:.1}px")).ok()?;
    let placed = note.get_bounding_client_rect();
    let curve = tern!(
        beside,
        from_beside(&placed, &origin, tip),
        from_below(&placed, &origin, tip)
    );
    pen.set_attribute("d", &arrow(&curve)).ok()
}

/// The curve from a note beside its line: out of the left edge of the note,
/// level with its first line, then under the words and up into them.
///
/// # Arguments
///
/// * `note` - The box of the note, on the page.
/// * `origin` - The box of the line, on the page.
/// * `tip` - Where the arrow ends.
///
/// # Returns
///
/// The curve.
fn from_beside(note: &DomRect, origin: &DomRect, tip: Point) -> Curve
{
    let top = note.top() + (note.height() / 2.0).min(NOTE_MIDDLE) - origin.top();
    let start = Point { left: note.left() - origin.left() - GAP, top };
    let out = Point { left: start.left - SWING, top: tip.top + DIP };
    // Close in under the words, so the arrow comes up into them at a steep
    // slant.
    let into = Point { left: tip.left + DIP / 2.0, top: tip.top + DIP };
    Curve { start, out, into, tip }
}

/// The curve from a note under its line. When the note starts past the
/// words, the arrow leaves its left edge, level with its first line, swings
/// out to the left and meets the words at a slant. When the line had no room
/// to move the note past them, it leaves the top edge of the note, just past
/// the words, and heads straight up to them.
///
/// # Arguments
///
/// * `note` - The box of the note, on the page.
/// * `origin` - The box of the line, on the page.
/// * `tip` - Where the arrow ends.
///
/// # Returns
///
/// The curve.
fn from_below(note: &DomRect, origin: &DomRect, tip: Point) -> Curve
{
    let left = note.left() - origin.left() - GAP;
    let (start, out) = if left > tip.left
    {
        let top = note.top() + (note.height() / 2.0).min(NOTE_MIDDLE) - origin.top();
        (Point { left, top }, Point { left: left - SWING, top })
    }
    else
    {
        let (near, far) = (note.left() - origin.left(), note.right() - origin.left());
        let across =
            (tip.left + LEAD / 2.0).clamp(near + GAP, (far - GAP).max(near + GAP));
        let edge = note.top() - 2.0 - origin.top();
        let out = Point { left: across, top: edge + (tip.top - edge) * 0.5 };
        (Point { left: across, top: edge }, out)
    };
    let into = Point {
        left: tip.left + (out.left - tip.left) * 0.35,
        top: tip.top + (start.top - tip.top) * 0.8,
    };
    Curve { start, out, into, tip }
}

/// The path of an arrow: its curve, then the two strokes of the head, which
/// point along the curve as it meets the words, so the head lines up with the
/// stroke.
///
/// # Arguments
///
/// * `curve` - The curve.
///
/// # Returns
///
/// The `d` of the path.
fn arrow(curve: &Curve) -> String
{
    let tip = curve.tip;
    let (across, down) = (tip.left - curve.into.left, tip.top - curve.into.top);
    let length = across.hypot(down).max(f64::EPSILON);
    let back = (-across / length, -down / length);
    let stroke = |turn: f64| Point {
        left: tip.left + HEAD * (back.0 * turn.cos() - back.1 * turn.sin()),
        top: tip.top + HEAD * (back.0 * turn.sin() + back.1 * turn.cos()),
    };
    let at = |p: Point| format!("{:.1} {:.1}", p.left, p.top);
    let (one, two) = (stroke(SPREAD), stroke(-SPREAD));
    format!(
        "M{}C{} {} {}M{}L{}L{}",
        at(curve.start),
        at(curve.out),
        at(curve.into),
        at(tip),
        at(one),
        at(tip),
        at(two)
    )
}
