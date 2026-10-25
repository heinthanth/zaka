use leptos::html::div;
use leptos::html::table;
use leptos::html::tbody;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;

use crate::compile::SegmentRow;
use crate::components::table::ClassName as Table;
use crate::components::table::Column;
use crate::components::table::cell;
use crate::components::table::linked_row;
use crate::components::table::span_cell;
use crate::components::table::table_box;
use crate::components::table::table_head;
use crate::state::AppState;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/segments_view/mod.sass");

/// Sheet of the segments view.
static SHEET: ComponentStyle =
    ComponentStyle::global_css("zaka-segments-view", STYLE_SHEET);

/// Columns of the segments table.
const COLUMNS: [Column; 4] = [
    ("explorer.col.index", Table::CELL_NUM, ""),
    ("explorer.col.kind", "", ""),
    ("explorer.col.text", "", ""),
    ("explorer.col.span", "", ""),
];

/// Table of the pre-lexical segments.
///
/// # Returns
///
/// The view.
pub fn segments_view() -> impl IntoView
{
    let state = use_state();
    let rows = move || {
        state.compiled.with(|c| {
            let rows = c.segments.iter().enumerate();
            rows.map(|(idx, row)| segment_row(state, idx, row.clone())).collect_view()
        })
    };
    let grid = table()
        .class(Table::PHASE_TABLE)
        .child((table_head(&COLUMNS), tbody().child(rows)));
    (sheet(&SHEET), div().child(table_box(grid)))
}

/// One row of the segments table.
///
/// # Arguments
///
/// * `state` - The state.
/// * `idx` - The segment index.
/// * `row` - The segment.
///
/// # Returns
///
/// The view.
fn segment_row(state: AppState, idx: usize, row: SegmentRow) -> impl IntoView
{
    let cells = (
        cell(Table::CELL_NUM, idx),
        cell(Table::CELL_KIND, row.kind),
        cell(Table::CELL_LEXEME, row.lexeme),
        span_cell(row.line, row.col, row.range),
    );
    linked_row(state, row.range, cells)
}
