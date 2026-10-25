use leptos::ev;
use leptos::html::details;
use leptos::html::div;
use leptos::html::span;
use leptos::html::summary;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::ret_if;

use crate::compile::TreeSource;
use crate::compile::tree::TreeNode;
use crate::state::AppState;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/tree_view/mod.sass");

/// Sheet of the tree view.
static SHEET: ComponentStyle =
    ComponentStyle::global_css("zaka-tree-view", STYLE_SHEET);

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
    (sheet(&SHEET), div().class(ClassName::TREE).child(tree))
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
            .class(ClassName::TREE_LEAF)
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
        .class(ClassName::TREE_NODE)
        .open(depth < OPEN_LEVELS)
        .child((
            line,
            div().class(ClassName::TREE_CHILDREN).child(children.collect_view()),
        ))
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
        .class(ClassName::TREE_LABEL)
        .title(t!("explorer.tree.select"))
        .on(ev::click, pick)
        .child(node.label);
    let id = node.id.map(|i| span().class(ClassName::TREE_ID).child(format!("#{i}")));
    let value = node.is_literal().then(|| value_class(&node.label));
    let class = value.map_or_else(
        || ClassName::TREE_DETAIL.to_string(),
        |v| format!("{} {} {v}", ClassName::TREE_DETAIL, ClassName::TREE_VALUE),
    );
    let detail = span().class(class).child(node.detail.clone());
    let kind = node.kind.clone().filter(|_| kinds);
    let kind = kind.map(|k| span().class(ClassName::TREE_KIND).child(k));
    (label, id, detail, kind)
}

/// The class of the value of a literal, in the color of its kind.
///
/// # Arguments
///
/// * `label` - The label of the literal node.
///
/// # Returns
///
/// The class, blank for a kind without a color.
fn value_class(label: &str) -> &'static str
{
    match label
    {
        "int" => ClassName::TREE_VALUE_INT,
        "float" => ClassName::TREE_VALUE_FLOAT,
        "str" => ClassName::TREE_VALUE_STR,
        "char" => ClassName::TREE_VALUE_CHAR,
        "bool" => ClassName::TREE_VALUE_BOOL,
        _ => "",
    }
}
