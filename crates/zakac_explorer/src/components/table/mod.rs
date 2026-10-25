use leptos::ev;
use leptos::html::td;
use leptos::html::th;
use leptos::html::thead;
use leptos::html::tr;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;

use crate::compile::spans::Range;
use crate::state::AppState;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/table/mod.sass");

/// Rules of the parts the views share: notes, titles, phase tables, the ops
/// of a function and the card of a phase that did not run.
static SHEET: ComponentStyle = ComponentStyle::global_css("zaka-table", STYLE_SHEET);

/// Mount the sheet of the shared parts.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn table_sheet() -> impl IntoView { sheet(&SHEET) }

/// A column head: locale ids of the label and the tooltip around its CSS
/// class; blank strings mean none.
pub type Column = (&'static str, &'static str, &'static str);

/// The head row of a phase table.
///
/// # Arguments
///
/// * `columns` - The columns.
///
/// # Returns
///
/// The view.
pub fn table_head(columns: &[Column]) -> impl IntoView + use<>
{
    let cells = columns.iter().map(|&(label, class, title)| {
        let title = (!title.is_empty()).then(|| t!(title.to_string()));
        th().class(class).title(title).child(t!(label.to_string()))
    });
    thead().child(tr().child(cells.collect_view()))
}

/// A row that points at its range on hover and selects it on click.
///
/// # Arguments
///
/// * `state` - The state.
/// * `range` - The source range of the row.
/// * `cells` - The cells.
///
/// # Returns
///
/// The view.
pub fn linked_row(
    state: AppState, range: Range, cells: impl IntoView + 'static,
) -> impl IntoView
{
    tr().on(ev::mouseenter, move |_| state.point(Some(range)))
        .on(ev::mouseleave, move |_| state.point(None))
        .on(ev::click, move |_| state.select(range))
        .child(cells)
}

/// Cell with `line:col` and the original u8 range.
///
/// # Arguments
///
/// * `line` - 1-based line.
/// * `col` - 1-based column.
/// * `range` - The range.
///
/// # Returns
///
/// The view.
pub fn span_cell(line: usize, col: usize, range: Range) -> impl IntoView
{
    let pos =
        leptos::html::span().class(ClassName::SPAN_POS).child(format!("{line}:{col}"));
    let bounds = format!("{}..{}", range.lo, range.hi);
    td().class(ClassName::CELL_SPAN)
        .child((pos, leptos::html::span().class(ClassName::SPAN_RANGE).child(bounds)))
}

/// A plain cell.
///
/// # Arguments
///
/// * `class` - The CSS class.
/// * `content` - The content.
///
/// # Returns
///
/// The view.
pub fn cell<C>(class: &str, content: C) -> impl IntoView + use<C>
where
    C: IntoView + 'static,
{
    td().class(class.to_string()).child(content)
}
