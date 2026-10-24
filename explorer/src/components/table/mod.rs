use leptos::ev;
use leptos::html::td;
use leptos::html::th;
use leptos::html::thead;
use leptos::html::tr;
use leptos::prelude::*;
use zakac_i18n::t;

use crate::compile::spans::Range;
use crate::state::AppState;

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
    let pos = leptos::html::span().class("span-pos").child(format!("{line}:{col}"));
    let bounds = format!("{}..{}", range.lo, range.hi);
    td().class("cell-span")
        .child((pos, leptos::html::span().class("span-range").child(bounds)))
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
pub fn cell(class: &'static str, content: impl IntoView + 'static) -> impl IntoView
{
    td().class(class).child(content)
}
