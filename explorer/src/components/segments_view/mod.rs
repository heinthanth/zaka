use leptos::html;
use leptos::html::div;
use leptos::html::table;
use leptos::html::tbody;
use leptos::prelude::*;
use leptos_scoped_style::scss_file;

use crate::compile::SegmentRow;
use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::components::table::Column;
use crate::components::table::cell;
use crate::components::table::linked_row;
use crate::components::table::span_cell;
use crate::components::table::table_head;
use crate::state::AppState;
use crate::state::use_state;

/// Sheet of the segments view.
static SHEET: ScopedSheet = ScopedSheet::new(
    "zaka-segments-view",
    scss_file!("src/components/segments_view/mod.sass"),
);

/// Columns of the segments table.
const COLUMNS: [Column; 4] = [
    ("explorer.col.index", "cell-num", ""),
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
    let grid =
        table().class("phase-table").child((table_head(&COLUMNS), tbody().child(rows)));
    let root = NodeRef::<html::Div>::new();
    scoped(&SHEET, root, div().class("segments-view").node_ref(root).child(grid))
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
        cell("cell-num", idx),
        cell("cell-kind", row.kind),
        cell("cell-lexeme", row.lexeme),
        span_cell(row.line, row.col, row.range),
    );
    linked_row(state, row.range, cells)
}
