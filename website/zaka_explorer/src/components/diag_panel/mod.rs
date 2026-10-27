use leptos::ev;
use leptos::html::div;
use leptos::html::p;
use leptos::html::pre;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_diagnostics::Level;
use zakac_i18n::t;
use zakac_reporter::render::Paint;
use zakac_reporter::render::Piece;
use zakac_rust_sugars::ret_if;

use crate::compile::DiagRow;
use crate::compile::spans::Range;
use crate::components::table::ClassName as Table;
use crate::state::AppState;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/diag_panel/mod.sass");

/// Sheet of the diagnostics panel.
static SHEET: ComponentStyle =
    ComponentStyle::global_css("zaka-diag-panel", STYLE_SHEET);

/// The error and warning counts of the problems tab.
///
/// # Returns
///
/// The view.
pub fn diag_counts() -> impl IntoView
{
    let state = use_state();
    let errors = move || state.compiled.with(|c| c.count(Level::Error));
    let warnings = move || state.compiled.with(|c| c.count(Level::Warning));
    let counts = (
        span()
            .class(format!("{} {}", ClassName::COUNT, ClassName::COUNT_ERROR))
            .class((ClassName::COUNT_ZERO, move || errors() == 0))
            .title(t!("explorer.problems.errors"))
            .child(errors),
        span()
            .class(format!("{} {}", ClassName::COUNT, ClassName::COUNT_WARNING))
            .class((ClassName::COUNT_ZERO, move || warnings() == 0))
            .title(t!("explorer.problems.warnings"))
            .child(warnings),
    );
    (sheet(&SHEET), counts)
}

/// The diagnostics of all phases, as the CLI prints them.
///
/// # Returns
///
/// The view.
pub fn diag_view() -> impl IntoView { (sheet(&SHEET), diag_list(use_state())) }

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
    let items = move || {
        let rows = state.compiled.with(|c| c.diagnostics.clone());
        let note = p().class(Table::VIEW_NOTE).child(t!("explorer.problems.none"));
        ret_if!(rows.is_empty(), note.into_any());
        rows.into_iter().map(|d| diag_item(state, d)).collect_view().into_any()
    };
    div().class(ClassName::DIAG_LIST).child(items)
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
    let phase: &'static str = diag.phase.into();
    let pieces = diag.pieces.into_iter().map(piece_view).collect_view();
    let tag = span().class(ClassName::DIAG_PHASE).child(phase);
    div()
        .class(format!("{} {}", ClassName::DIAG_ITEM, level_class(diag.level)))
        .class((ClassName::DIAG_LINKED, range.is_some()))
        .on(ev::mouseenter, move |_| state.point(range))
        .on(ev::mouseleave, move |_| state.point(None))
        .on(ev::click, move |_| do_select(state, range))
        .child((tag, pre().class(ClassName::DIAG_TEXT).child(pieces)))
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

/// The class of a diagnostic of a level.
///
/// # Arguments
///
/// * `level` - The level.
///
/// # Returns
///
/// The class.
fn level_class(level: Level) -> &'static str
{
    match level
    {
        Level::Error => ClassName::DIAG_ERROR,
        Level::Warning => ClassName::DIAG_WARNING,
        Level::Note => ClassName::DIAG_NOTE,
    }
}

/// The classes of a paint: one for its form, plus one for its level when the
/// paint carries a level. Plain text takes none.
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
    let form = match paint
    {
        Paint::Plain => "",
        Paint::Tag(_) => ClassName::PAINT_TAG,
        Paint::Gutter => ClassName::PAINT_GUTTER,
        Paint::Marked(_) => ClassName::PAINT_MARKED,
        Paint::Carets(_) => ClassName::PAINT_CARETS,
    };
    let level = paint_level(paint).map(|l| match l
    {
        Level::Error => ClassName::PAINT_ERROR,
        Level::Warning => ClassName::PAINT_WARNING,
        Level::Note => ClassName::PAINT_NOTE,
    });
    level.map_or_else(|| form.to_string(), |l| format!("{form} {l}"))
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
