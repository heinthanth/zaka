use std::borrow::Cow;

use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::div;
use leptos::html::span;
use leptos::prelude::*;
use leptos::svg::g;
use leptos::svg::path;
use leptos::svg::rect;
use leptos::svg::svg;
use leptos::svg::text;
use leptos::svg::tspan;
use leptos_scoped_style::scss_file;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::compile::TreeSource;
use crate::compile::graph::CHAR_WIDTH;
use crate::compile::graph::Graph;
use crate::compile::graph::GraphNode;
use crate::compile::graph::NODE_HEIGHT;
use crate::compile::graph::NODE_PAD;
use crate::compile::graph::View;
use crate::compile::graph::graph;
use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::state::AppState;
use crate::state::use_state;

/// Sheet of the graph view.
static SHEET: ScopedSheet = ScopedSheet::new(
    "zaka-graph-view",
    scss_file!("src/components/graph_view/mod.sass"),
);

/// Zoom step of the buttons and of `+` and `-`.
const ZOOM_STEP: f64 = 1.2;

/// Zoom per px of wheel travel with Ctrl or Cmd held.
const WHEEL_ZOOM: f64 = 0.002;

/// The pan and zoom state of one graph view.
#[derive(Clone, Copy)]
struct Camera
{
    /// The viewport element.
    port: NodeRef<html::Div>,
    /// The placed graph.
    shape: Memo<Graph>,
    /// The zoom and the origin.
    view: RwSignal<View>,
    /// The last pointer position while a Ctrl-drag moves the graph.
    drag: RwSignal<Option<(f64, f64)>>,
}

impl Camera
{
    /// The size of the viewport.
    ///
    /// # Returns
    ///
    /// The width and height, or zero before the viewport is on the page.
    fn size(&self) -> (f64, f64)
    {
        let rect = self.port.get_untracked().map(|el| el.get_bounding_client_rect());
        rect.map_or((0.0, 0.0), |r| (r.width(), r.height()))
    }

    /// A point of the window in viewport px.
    ///
    /// # Arguments
    ///
    /// * `x` - Client x.
    /// * `y` - Client y.
    ///
    /// # Returns
    ///
    /// The point.
    fn local(&self, x: f64, y: f64) -> (f64, f64)
    {
        let rect = self.port.get_untracked().map(|el| el.get_bounding_client_rect());
        rect.map_or((x, y), |r| (x - r.left(), y - r.top()))
    }

    /// Zoom around the middle of the viewport.
    ///
    /// # Arguments
    ///
    /// * `factor` - The factor; above 1 zooms in.
    fn zoom(&self, factor: f64)
    {
        let (w, h) = self.size();
        self.view.update(|v| *v = v.zoom_at(factor, w / 2.0, h / 2.0));
    }

    /// Open on the root: the program node at the top middle, at the natural
    /// size.
    fn focus_root(&self)
    {
        let (w, h) = self.size();
        ret_if!(w <= 0.0 || h <= 0.0);
        self.view.set(View::focus_root(&self.shape.get_untracked(), w, h));
    }

    /// Show the whole graph.
    fn fit(&self)
    {
        let (w, h) = self.size();
        ret_if!(w <= 0.0 || h <= 0.0);
        self.view.set(View::fit(&self.shape.get_untracked(), w, h));
    }
}

/// The program tree drawn as a graph: boxes for the nodes, top-down, and
/// lines from each node to its children. Hover points at a node in the
/// editor and a click selects it. Ctrl or Cmd with the wheel zooms, Ctrl or
/// Cmd with a drag moves, the plain wheel pans, and the buttons, `+`, `-`
/// and `0` zoom and fit.
///
/// # Arguments
///
/// * `source` - Which tree to draw.
///
/// # Returns
///
/// The view.
pub fn graph_view(
    source: impl Fn() -> TreeSource + Copy + Send + Sync + 'static,
) -> impl IntoView
{
    let state = use_state();
    let shape = Memo::new(move |_| {
        let pick = source();
        state.compiled.with(|c| graph(pick.tree(c)))
    });
    let cam = Camera {
        port: NodeRef::new(),
        shape,
        view: RwSignal::new(View::default()),
        drag: RwSignal::new(None),
    };
    // Open on the root once the viewport has a size.
    Effect::new(move |_| {
        do_if!(
            cam.port.get().is_some(),
            request_animation_frame(move || cam.focus_root())
        )
    });
    let transform = move || {
        let v = cam.view.get();
        format!("translate({} {}) scale({})", v.x, v.y, v.zoom)
    };
    let content = move || cam.shape.with(|shape| drawing(state, shape));
    let picture = svg()
        .attr("class", "graph-svg")
        .attr("role", "img")
        .attr("aria-label", t!("explorer.graph.label"))
        .child(g().attr("transform", transform).child(content));
    let port = div()
        .class("graph")
        .class(("graph-dragging", move || cam.drag.get().is_some()))
        .node_ref(cam.port)
        .attr("tabindex", "0")
        .on(ev::wheel, move |e| on_wheel(cam, &e))
        .on(ev::pointerdown, move |e| start_drag(cam, &e))
        .on(ev::pointermove, move |e| follow_drag(cam, &e))
        .on(ev::pointerup, move |_| cam.drag.set(None))
        .on(ev::pointercancel, move |_| cam.drag.set(None))
        .on(ev::keydown, move |e| on_press(cam, &e))
        .child((picture, controls(cam), hint()));
    scoped(&SHEET, cam.port, port)
}

/// Zoom with Ctrl or Cmd and the wheel, around the pointer; pan with the
/// plain wheel.
///
/// # Arguments
///
/// * `cam` - The camera.
/// * `e` - The wheel event.
fn on_wheel(cam: Camera, e: &ev::WheelEvent)
{
    e.prevent_default();
    let (dx, dy) = (e.delta_x(), e.delta_y());
    ret_if!(
        !(e.ctrl_key() || e.meta_key()),
        cam.view.update(|v| *v = v.moved(-dx, -dy))
    );
    let (px, py) = cam.local(f64::from(e.client_x()), f64::from(e.client_y()));
    let factor = (-dy * WHEEL_ZOOM).exp();
    cam.view.update(|v| *v = v.zoom_at(factor, px, py));
}

/// Start a Ctrl-drag or Cmd-drag.
///
/// # Arguments
///
/// * `cam` - The camera.
/// * `e` - The press.
fn start_drag(cam: Camera, e: &ev::PointerEvent)
{
    ret_if!(!(e.ctrl_key() || e.meta_key()));
    e.prevent_default();
    do_if!(let Some(el) = cam.port.get_untracked(), { let _ = el.set_pointer_capture(e.pointer_id()); });
    cam.drag.set(Some((f64::from(e.client_x()), f64::from(e.client_y()))));
}

/// Move the graph with the pointer while a drag is on.
///
/// # Arguments
///
/// * `cam` - The camera.
/// * `e` - The move.
fn follow_drag(cam: Camera, e: &ev::PointerEvent)
{
    let Some((lx, ly)) = cam.drag.get_untracked()
    else
    {
        return;
    };
    let (x, y) = (f64::from(e.client_x()), f64::from(e.client_y()));
    cam.view.update(|v| *v = v.moved(x - lx, y - ly));
    cam.drag.set(Some((x, y)));
}

/// Zoom with `+` and `-` and fit with `0`.
///
/// # Arguments
///
/// * `cam` - The camera.
/// * `e` - The press.
fn on_press(cam: Camera, e: &ev::KeyboardEvent)
{
    match e.key().as_str()
    {
        "+" | "=" => cam.zoom(ZOOM_STEP),
        "-" | "_" => cam.zoom(1.0 / ZOOM_STEP),
        "0" => cam.fit(),
        _ => return,
    }
    e.prevent_default();
}

/// The zoom buttons and the zoom readout, at the bottom right.
///
/// # Arguments
///
/// * `cam` - The camera.
///
/// # Returns
///
/// The view.
fn controls(cam: Camera) -> impl IntoView
{
    let percent = move || format!("{:.0}%", cam.view.get().zoom * 100.0);
    let control = |text: Cow<'static, str>, id: &'static str| {
        button()
            .class("graph-control")
            .attr("aria-label", t!(id.to_string()))
            .title(t!(id.to_string()))
            .child(text)
    };
    div().class("graph-controls").child((
        control("-".into(), "explorer.graph.zoom_out")
            .on(ev::click, move |_| cam.zoom(1.0 / ZOOM_STEP)),
        span().class("graph-zoom").child(percent),
        control("+".into(), "explorer.graph.zoom_in")
            .on(ev::click, move |_| cam.zoom(ZOOM_STEP)),
        control(t!("explorer.graph.fit"), "explorer.graph.fit_note")
            .on(ev::click, move |_| cam.fit()),
    ))
}

/// How to move about, at the bottom left.
///
/// # Returns
///
/// The view.
fn hint() -> impl IntoView
{
    span().class("graph-hint").child("Ctrl/⌘ + scroll to zoom · Ctrl/⌘ + drag to move")
}

/// The lines and boxes of a graph.
///
/// # Arguments
///
/// * `state` - The state.
/// * `graph` - The placed nodes.
///
/// # Returns
///
/// The view.
fn drawing(state: AppState, graph: &Graph) -> impl IntoView + use<>
{
    let nodes = &graph.nodes;
    let lines = nodes.iter().filter_map(|n| n.parent.map(|p| line(&nodes[p], n)));
    let boxes = nodes.iter().cloned().map(|n| node_view(state, n));
    (
        g().attr("class", "graph-lines").child(lines.collect_view()),
        g().child(boxes.collect_view()),
    )
}

/// The curve from the bottom of a parent to the top of a child.
///
/// # Arguments
///
/// * `from` - The parent.
/// * `to` - The child.
///
/// # Returns
///
/// The view.
fn line(from: &GraphNode, to: &GraphNode) -> impl IntoView + use<>
{
    let (x1, y1) = (from.center(), from.y + NODE_HEIGHT);
    let (x2, y2) = (to.center(), to.y);
    let mid = (y1 + y2) / 2.0;
    path().attr("d", format!("M {x1} {y1} C {x1} {mid}, {x2} {mid}, {x2} {y2}"))
}

/// One node: a box with its label and, for literals and names, the source
/// text.
///
/// # Arguments
///
/// * `state` - The state.
/// * `node` - The node.
///
/// # Returns
///
/// The view.
fn node_view(state: AppState, node: GraphNode) -> impl IntoView
{
    let range = node.range;
    let tone = tern!(
        node.literal,
        format!("graph-value graph-value-{}", node.label),
        "graph-value".to_string()
    );
    let value = (!node.value.is_empty())
        .then(|| tspan().attr("class", tone).attr("dx", CHAR_WIDTH).child(node.value));
    let caption = text()
        .attr("x", node.x + NODE_PAD)
        .attr("y", node.y + NODE_HEIGHT / 2.0)
        .attr("dominant-baseline", "central")
        .child((tspan().attr("class", "graph-label").child(node.label), value));
    let frame = rect()
        .attr("x", node.x)
        .attr("y", node.y)
        .attr("width", node.width)
        .attr("height", NODE_HEIGHT)
        .attr("rx", 7);
    g().attr("class", "graph-node")
        .on(ev::mouseenter, move |_| state.point(Some(range)))
        .on(ev::mouseleave, move |_| state.point(None))
        // A Ctrl-click starts a drag, not a pick.
        .on(ev::click, move |e: ev::MouseEvent| {
            do_if!(!(e.ctrl_key() || e.meta_key()), state.select(range))
        })
        .child((frame, caption))
}
