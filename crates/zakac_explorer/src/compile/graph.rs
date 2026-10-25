use bon::Builder;
use zakac_rust_sugars::tern;

use crate::compile::spans::Range;
use crate::compile::tree::TreeNode;

/// Height of a node box, in px.
pub const NODE_HEIGHT: f64 = 28.0;

/// Room between two rows of nodes, in px.
pub const ROW_GAP: f64 = 36.0;

/// Room between two sibling subtrees, in px.
pub const SIBLING_GAP: f64 = 14.0;

/// Width of one char of node text at the graph font size, in px; a monospace
/// guess, so long Burmese text gets a little more room than it needs.
pub const CHAR_WIDTH: f64 = 7.4;

/// Room on the left and right of the text in a node, in px.
pub const NODE_PAD: f64 = 10.0;

/// Room around the whole graph, in px.
pub const MARGIN: f64 = 16.0;

/// Most chars of source text a node shows.
const VALUE_CHARS: usize = 16;

/// One node of the graph view, placed.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct GraphNode
{
    /// The node form, such as `decl` or `int`.
    pub label: &'static str,
    /// The source text shown after the label: the value of a literal, or the
    /// name of a name node; blank for others.
    #[builder(default)]
    pub value: String,
    /// Whether the node is a literal.
    pub literal: bool,
    /// The source region in the original text.
    pub range: Range,
    /// The left edge.
    pub x: f64,
    /// The top edge.
    pub y: f64,
    /// The width of the box.
    pub width: f64,
    /// Index of the parent node; `None` for the root.
    pub parent: Option<usize>,
}

impl GraphNode
{
    /// The middle of the box along the x axis.
    ///
    /// # Returns
    ///
    /// The x coordinate.
    pub fn center(&self) -> f64 { self.x + self.width / 2.0 }
}

/// The placed nodes of a tree and the room they take.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Graph
{
    /// The nodes, parents before children.
    pub nodes: Vec<GraphNode>,
    /// The width of the whole graph.
    pub width: f64,
    /// The height of the whole graph.
    pub height: f64,
}

/// Widths of a subtree: of its own box and of the room it takes.
struct Measure
{
    /// Width of the box of the node.
    width: f64,
    /// Width of the room of the subtree: the box or the children side by
    /// side, whichever is wider.
    room: f64,
    /// Measures of the children.
    children: Vec<Measure>,
}

/// Place the nodes of a tree top-down: each row a level deeper, the children
/// of a node side by side under it, and each node centered over its
/// children.
///
/// # Arguments
///
/// * `tree` - The root of the tree view.
///
/// # Returns
///
/// The graph.
pub fn graph(tree: &TreeNode) -> Graph
{
    let measure = measure(tree);
    let mut nodes = Vec::new();
    place(tree, &measure, MARGIN, 0, None, &mut nodes);
    let bottom = nodes.iter().map(|n| n.y + NODE_HEIGHT).fold(0.0, f64::max);
    Graph { nodes, width: measure.room + MARGIN * 2.0, height: bottom + MARGIN }
}

/// The source text a node shows: literals and the nodes that name things.
///
/// # Arguments
///
/// * `node` - The node.
///
/// # Returns
///
/// The text, cut to [`VALUE_CHARS`]; blank for other nodes.
fn value_of(node: &TreeNode) -> String
{
    let named =
        matches!(node.label, "ident" | "name" | "kind" | "read" | "slot" | "fn");
    let shown = node.is_literal() || named;
    let cut: String = node.detail.chars().take(VALUE_CHARS).collect();
    let more = node.detail.chars().count() > VALUE_CHARS;
    let cut = tern!(more, format!("{cut}…"), cut);
    tern!(shown, cut, String::new())
}

/// Width of the box of a node.
///
/// # Arguments
///
/// * `label` - The label.
/// * `value` - The source text it shows.
///
/// # Returns
///
/// The width in px.
fn box_width(label: &str, value: &str) -> f64
{
    let gap = usize::from(!value.is_empty());
    let chars = label.chars().count() + gap + value.chars().count();
    NODE_PAD * 2.0 + chars as f64 * CHAR_WIDTH
}

/// Measure a subtree, bottom-up.
///
/// # Arguments
///
/// * `node` - The root of the subtree.
///
/// # Returns
///
/// The measure.
fn measure(node: &TreeNode) -> Measure
{
    let width = box_width(node.label, &value_of(node));
    let children: Vec<Measure> = node.children.iter().map(measure).collect();
    let room = width.max(row_width(&children));
    Measure { width, room, children }
}

/// Width of subtrees side by side.
///
/// # Arguments
///
/// * `row` - Their measures.
///
/// # Returns
///
/// The width in px, gaps included.
fn row_width(row: &[Measure]) -> f64
{
    let gaps = row.len().saturating_sub(1) as f64 * SIBLING_GAP;
    row.iter().map(|m| m.room).sum::<f64>() + gaps
}

/// Place a subtree in the room that starts at `left`.
///
/// # Arguments
///
/// * `node` - The root of the subtree.
/// * `measure` - Its measure.
/// * `left` - The left edge of its room.
/// * `depth` - Its row.
/// * `parent` - Index of its parent.
/// * `out` - The placed nodes.
fn place(
    node: &TreeNode, measure: &Measure, left: f64, depth: usize, parent: Option<usize>,
    out: &mut Vec<GraphNode>,
)
{
    let idx = out.len();
    let placed = GraphNode::builder()
        .label(node.label)
        .value(value_of(node))
        .literal(node.is_literal())
        .range(node.range)
        .x(left + (measure.room - measure.width) / 2.0)
        .y(MARGIN + depth as f64 * (NODE_HEIGHT + ROW_GAP))
        .width(measure.width)
        .maybe_parent(parent)
        .build();
    out.push(placed);
    let start = left + (measure.room - row_width(&measure.children)) / 2.0;
    let pairs = node.children.iter().zip(&measure.children);
    pairs.fold(start, |at, (child, m)| {
        place(child, m, at, depth + 1, Some(idx), out);
        at + m.room + SIBLING_GAP
    });
}

/// Smallest zoom of the graph view.
pub const MIN_ZOOM: f64 = 0.2;

/// Largest zoom of the graph view.
pub const MAX_ZOOM: f64 = 3.0;

/// How the graph view shows the graph: the zoom, and where the origin of the
/// graph sits in the viewport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View
{
    /// The zoom; 1 is the natural size.
    pub zoom: f64,
    /// The origin along the x axis, in viewport px.
    pub x: f64,
    /// The origin along the y axis, in viewport px.
    pub y: f64,
}

impl Default for View
{
    /// The natural size, at the top left.
    ///
    /// # Returns
    ///
    /// The view.
    fn default() -> Self { View { zoom: 1.0, x: 0.0, y: 0.0 } }
}

impl View
{
    /// Zoom by a factor around a point of the viewport; the graph point under
    /// it holds still. The zoom stays within [`MIN_ZOOM`] and [`MAX_ZOOM`].
    ///
    /// # Arguments
    ///
    /// * `factor` - The factor; above 1 zooms in.
    /// * `px` - The point along the x axis, in viewport px.
    /// * `py` - The point along the y axis, in viewport px.
    ///
    /// # Returns
    ///
    /// The new view.
    pub fn zoom_at(self, factor: f64, px: f64, py: f64) -> View
    {
        let zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let k = zoom / self.zoom;
        View { zoom, x: px - (px - self.x) * k, y: py - (py - self.y) * k }
    }

    /// Move the graph.
    ///
    /// # Arguments
    ///
    /// * `dx` - The move along the x axis, in viewport px.
    /// * `dy` - The move along the y axis, in viewport px.
    ///
    /// # Returns
    ///
    /// The new view.
    pub fn moved(self, dx: f64, dy: f64) -> View
    {
        View { x: self.x + dx, y: self.y + dy, ..self }
    }

    /// The view that shows a whole graph in a viewport, centered, no larger
    /// than the natural size.
    ///
    /// # Arguments
    ///
    /// * `graph` - The graph.
    /// * `width` - Width of the viewport.
    /// * `height` - Height of the viewport.
    ///
    /// # Returns
    ///
    /// The view.
    pub fn fit(graph: &Graph, width: f64, height: f64) -> View
    {
        let room = |space: f64, size: f64| tern!(size > 0.0, space / size, 1.0);
        let zoom = room(width, graph.width).min(room(height, graph.height));
        let zoom = zoom.clamp(MIN_ZOOM, 1.0);
        let x = (width - graph.width * zoom) / 2.0;
        let y = ((height - graph.height * zoom) / 2.0).max(0.0);
        View { zoom, x, y }
    }

    /// The view a graph opens with: the root at the top middle at the
    /// natural size, or the whole graph when it fits at that size.
    ///
    /// # Arguments
    ///
    /// * `graph` - The graph.
    /// * `width` - Width of the viewport.
    /// * `height` - Height of the viewport.
    ///
    /// # Returns
    ///
    /// The view.
    pub fn focus_root(graph: &Graph, width: f64, height: f64) -> View
    {
        let fits = graph.width <= width && graph.height <= height;
        let Some(root) = graph.nodes.first().filter(|_| !fits)
        else
        {
            return View::fit(graph, width, height);
        };
        View { zoom: 1.0, x: width / 2.0 - root.center(), y: 0.0 }
    }
}
