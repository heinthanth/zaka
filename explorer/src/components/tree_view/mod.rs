use leptos::ev;
use leptos::html;
use leptos::html::details;
use leptos::html::div;
use leptos::html::span;
use leptos::html::summary;
use leptos::prelude::*;
use leptos_scoped_style::scss_file;
use zakac_i18n::t;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::compile::TreeSource;
use crate::compile::tree::TreeNode;
use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::state::AppState;
use crate::state::use_state;

/// Sheet of the tree view.
static SHEET: ScopedSheet =
    ScopedSheet::new("zaka-tree-view", scss_file!("src/components/tree_view/mod.sass"));

/// Tree levels open when the view first shows.
const OPEN_LEVELS: usize = 3;

/// The program tree; hover points at a node in the editor and a click on a
/// label selects it.
///
/// # Arguments
///
/// * `source` - Which tree to show.
/// * `kinds` - Show the kinds of the expressions.
///
/// # Returns
///
/// The view.
pub fn tree_view(
    source: impl Fn() -> TreeSource + Copy + Send + Sync + 'static, kinds: bool,
) -> impl IntoView
{
    let state = use_state();
    let tree = move || {
        let pick = source();
        state.compiled.with(|c| node_view(state, pick.tree(c).clone(), 0, kinds))
    };
    let root = NodeRef::<html::Div>::new();
    scoped(&SHEET, root, div().class("tree").node_ref(root).child(tree))
}

/// View of one tree node and its children.
///
/// # Arguments
///
/// * `state` - The state.
/// * `node` - The node.
/// * `depth` - How deep the node sits; the top levels start open.
/// * `kinds` - Show the kinds.
///
/// # Returns
///
/// The view.
fn node_view(state: AppState, node: TreeNode, depth: usize, kinds: bool) -> AnyView
{
    let range = node.range;
    let head = node_head(state, &node, kinds);
    let hover = move |_| state.point(Some(range));
    let leave = move |_| state.point(None);
    ret_if!(
        node.children.is_empty(),
        div()
            .class("tree-leaf")
            .on(ev::mouseenter, hover)
            .on(ev::mouseleave, leave)
            .child(head)
            .into_any()
    );
    let children =
        node.children.into_iter().map(|c| node_view(state, c, depth + 1, kinds));
    let line =
        summary().on(ev::mouseenter, hover).on(ev::mouseleave, leave).child(head);
    details()
        .class("tree-node")
        .open(depth < OPEN_LEVELS)
        .child((line, div().class("tree-children").child(children.collect_view())))
        .into_any()
}

/// Label, source text and, when asked, kind of a node; a click on the label
/// selects the node in the editor.
///
/// # Arguments
///
/// * `state` - The state.
/// * `node` - The node.
/// * `kinds` - Show the kind.
///
/// # Returns
///
/// The view.
fn node_head(state: AppState, node: &TreeNode, kinds: bool) -> impl IntoView + use<>
{
    let range = node.range;
    let pick = move |ev: ev::MouseEvent| {
        ev.prevent_default();
        state.select(range);
    };
    let label = span()
        .class("tree-label")
        .title(t!("explorer.tree.select"))
        .on(ev::click, pick)
        .child(node.label);
    let id = node.id.map(|i| span().class("tree-id").child(format!("#{i}")));
    let value = format!(" tree-value tree-value-{}", node.label);
    let class =
        format!("tree-detail{}", tern!(node.is_literal(), value, String::new()));
    let detail = span().class(class).child(node.detail.clone());
    let kind = node.kind.clone().filter(|_| kinds);
    let kind = kind.map(|k| span().class("tree-kind").child(k));
    (label, id, detail, kind)
}
