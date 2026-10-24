use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::div;
use leptos::html::nav;
use leptos::html::pre;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::scss_file;
use strum::IntoEnumIterator;
use zakac_i18n::t;

use crate::compile::TreeSource;
use crate::compile::s_expr::Chunk;
use crate::compile::s_expr::SEXPR_WIDTH;
use crate::compile::s_expr::arrange;
use crate::components::graph_view::graph_view;
use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::components::tree_view::tree_view;
use crate::state::AppState;
use crate::state::settings::AstMode;
use crate::state::use_state;

/// Sheet of the AST view.
static SHEET: ScopedSheet =
    ScopedSheet::new("zaka-ast-view", scss_file!("src/components/ast_view/mod.sass"));

/// The parser output, in one of three forms that a switch at the top right
/// picks: the tree, the S-expression or the graph.
///
/// # Returns
///
/// The view.
pub fn ast_view() -> impl IntoView
{
    forms_view(|| TreeSource::Ast, false, ().into_any())
}

/// A tree in one of three forms that a switch at the top right picks: the
/// tree, the S-expression or the graph.
///
/// # Arguments
///
/// * `source` - Which tree to show.
/// * `kinds` - Show the kinds of the expressions in the tree form.
/// * `extra` - Controls shown before the switch.
///
/// # Returns
///
/// The view.
pub fn forms_view(
    source: impl Fn() -> TreeSource + Copy + Send + Sync + 'static, kinds: bool,
    extra: AnyView,
) -> impl IntoView
{
    let state = use_state();
    let modes = AstMode::iter().map(|m| mode_button(state, m)).collect_view();
    let switch = nav()
        .class("ast-modes")
        .attr("role", "tablist")
        .attr("aria-label", t!("explorer.ast.label"))
        .child(modes);
    let shown = move || match state.ast_mode.get()
    {
        AstMode::Tree => tree_view(source, kinds).into_any(),
        AstMode::Sexpr => sexpr_view(state, source).into_any(),
        AstMode::Graph => graph_view(source).into_any(),
    };
    let head = div().class("ast-head").child((extra, switch));
    let root = NodeRef::<html::Div>::new();
    let view = div()
        .class("ast-view")
        .class(("ast-view-graph", move || state.ast_mode.get() == AstMode::Graph))
        .node_ref(root)
        .child((head, shown));
    scoped(&SHEET, root, view)
}

/// One button of the switch.
///
/// # Arguments
///
/// * `state` - The state.
/// * `mode` - The form it shows.
///
/// # Returns
///
/// The view.
fn mode_button(state: AppState, mode: AstMode) -> impl IntoView
{
    let on = move || state.ast_mode.get() == mode;
    button()
        .class("ast-mode")
        .class(("ast-mode-on", on))
        .attr("role", "tab")
        .attr("aria-selected", move || on().to_string())
        .on(ev::click, move |_| state.set_ast_mode.set(mode))
        .child(mode.label())
}

/// The S-expression form, indented as Lisp code is.
///
/// # Arguments
///
/// * `state` - The state.
/// * `source` - Which tree to show.
///
/// # Returns
///
/// The view.
fn sexpr_view(
    state: AppState, source: impl Fn() -> TreeSource + Copy + Send + Sync + 'static,
) -> impl IntoView
{
    let chunks = move || {
        let pick = source();
        let chunks = state.compiled.with(|c| arrange(pick.sexpr(c), SEXPR_WIDTH));
        chunks.into_iter().map(chunk_view).collect_view()
    };
    pre().class("sexpr").child(chunks)
}

/// One chunk of the S-expression, in the color of its ink.
///
/// # Arguments
///
/// * `chunk` - The chunk.
///
/// # Returns
///
/// The view.
fn chunk_view(chunk: Chunk) -> impl IntoView
{
    let ink: &'static str = chunk.ink.into();
    span().class(format!("ink-{ink}")).child(chunk.text)
}
