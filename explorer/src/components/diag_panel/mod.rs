use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::div;
use leptos::html::p;
use leptos::html::pre;
use leptos::html::section;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::scss_file;
use zakac_diagnostics::Level;
use zakac_i18n::t;
use zakac_reporter::render::Paint;
use zakac_reporter::render::Piece;
use zakac_rust_sugars::tern;

use crate::compile::DiagRow;
use crate::compile::spans::Range;
use crate::components::icon::chevron;
use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::state::AppState;
use crate::state::use_state;

/// Sheet of the diagnostics panel.
static SHEET: ScopedSheet = ScopedSheet::new(
    "zaka-diag-panel",
    scss_file!("src/components/diag_panel/mod.sass"),
);

/// The problems panel: the diagnostics of all phases, as the CLI prints
/// them. Its header opens and shuts it and counts the errors and warnings.
///
/// # Returns
///
/// The view.
pub fn diag_panel() -> impl IntoView
{
    let state = use_state();
    let open = state.problems_open;
    let list = move || open.get().then(|| diag_list(state));
    let root = NodeRef::<html::Section>::new();
    let view = section()
        .class("diag-panel")
        .class(("diag-panel-shut", move || !open.get()))
        .node_ref(root)
        .child((diag_head(state), list));
    scoped(&SHEET, root, view)
}

/// The header: a button with a chevron, the title and the counts.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn diag_head(state: AppState) -> impl IntoView
{
    let open = state.problems_open;
    let errors = move || state.compiled.with(|c| c.count(Level::Error));
    let warnings = move || state.compiled.with(|c| c.count(Level::Warning));
    let counts = (
        span()
            .class("count count-error")
            .class(("count-zero", move || errors() == 0))
            .title(t!("explorer.problems.errors"))
            .child(errors),
        span()
            .class("count count-warning")
            .class(("count-zero", move || warnings() == 0))
            .title(t!("explorer.problems.warnings"))
            .child(warnings),
    );
    button()
        .class("diag-head")
        .attr("aria-expanded", move || open.get().to_string())
        .title(move || {
            tern!(
                open.get(),
                t!("explorer.problems.hide"),
                t!("explorer.problems.show")
            )
        })
        .on(ev::click, move |_| state.set_problems_open.set(!open.get_untracked()))
        .child((
            chevron(),
            span().class("diag-title").child(t!("explorer.problems.title")),
            counts,
        ))
}

/// The list of diagnostics, or a note when there are none.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn diag_list(state: AppState) -> impl IntoView
{
    let clean = move || {
        let none = state.compiled.with(|c| c.diagnostics.is_empty());
        none.then(|| p().class("diag-clean").child(t!("explorer.problems.none")))
    };
    let items = move || {
        state.compiled.with(|c| {
            c.diagnostics.iter().map(|d| diag_item(state, d.clone())).collect_view()
        })
    };
    div().class("diag-list").child((clean, items))
}

/// One diagnostic; hover points at its span and a click selects it.
///
/// # Arguments
///
/// * `state` - The state.
/// * `diag` - The diagnostic.
///
/// # Returns
///
/// The view.
fn diag_item(state: AppState, diag: DiagRow) -> impl IntoView
{
    let range = diag.range;
    let level: &'static str = diag.level.into();
    let phase: &'static str = diag.phase.into();
    let pieces = diag.pieces.into_iter().map(piece_view).collect_view();
    let tag = span().class("diag-phase").child(phase);
    div()
        .class(format!("diag-item diag-{level}"))
        .class(("diag-linked", range.is_some()))
        .on(ev::mouseenter, move |_| state.point(range))
        .on(ev::mouseleave, move |_| state.point(None))
        .on(ev::click, move |_| do_select(state, range))
        .child((tag, pre().class("diag-text").child(pieces)))
}

/// Select a range in the editor, when there is one.
///
/// # Arguments
///
/// * `state` - The state.
/// * `range` - The range.
fn do_select(state: AppState, range: Option<Range>)
{
    let Some(range) = range
    else
    {
        return;
    };
    state.select(range);
}

/// One painted piece of the CLI rendering.
///
/// # Arguments
///
/// * `piece` - The piece.
///
/// # Returns
///
/// The view.
fn piece_view(piece: Piece) -> impl IntoView
{
    span().class(paint_class(piece.paint)).child(piece.text)
}

/// CSS classes of a paint: `paint-<form>`, plus `paint-<level>` when the
/// paint carries a level. Both names come from strum.
///
/// # Arguments
///
/// * `paint` - The paint.
///
/// # Returns
///
/// The classes.
fn paint_class(paint: Paint) -> String
{
    let form: &'static str = paint.into();
    let level = paint_level(paint).map(<&'static str>::from);
    level.map_or_else(|| format!("paint-{form}"), |l| format!("paint-{form} paint-{l}"))
}

/// The level a paint carries.
///
/// # Arguments
///
/// * `paint` - The paint.
///
/// # Returns
///
/// The level, or `None` for plain text and the gutter.
fn paint_level(paint: Paint) -> Option<Level>
{
    match paint
    {
        Paint::Tag(level) | Paint::Marked(level) | Paint::Carets(level) => Some(level),
        Paint::Plain | Paint::Gutter => None,
    }
}
