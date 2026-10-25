use bon::Builder;
use leptos::ev;
use leptos::html::button;
use leptos::html::table;
use leptos::html::tbody;
use leptos::html::td;
use leptos::html::tr;
use leptos::prelude::*;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

use crate::compile::asm::AsmFunc;
use crate::compile::asm::OpRow;
use crate::compile::asm::SlotRow;
use crate::components::table::Column;
use crate::components::table::cell;
use crate::components::table::table_head;
use crate::state::AppState;

/// Columns of an asm table.
const OP_COLUMNS: [Column; 5] = [
    ("explorer.col.op_addr", "cell-num", "explorer.col.op_addr_note"),
    ("explorer.col.label", "", ""),
    ("explorer.col.op", "", ""),
    ("explorer.col.ref", "", ""),
    ("explorer.col.line", "cell-num", ""),
];

/// Columns of an asm table with a break point gutter.
const GUTTER_COLUMNS: [Column; 6] = [
    ("explorer.col.break", "cell-break", "explorer.col.break_note"),
    ("explorer.col.op_addr", "cell-num", "explorer.col.op_addr_note"),
    ("explorer.col.label", "", ""),
    ("explorer.col.op", "", ""),
    ("explorer.col.ref", "", ""),
    ("explorer.col.line", "cell-num", ""),
];

/// Columns of a slot table.
const SLOT_COLUMNS: [Column; 3] = [
    ("explorer.col.slot", "cell-num", ""),
    ("explorer.col.name", "", ""),
    ("explorer.col.kind", "", ""),
];

/// The break points of one function and how to set or clear one.
#[derive(Clone, Copy, Builder)]
pub struct Gutter
{
    /// The op indices with a break point.
    pub breaks: Signal<Vec<u32>>,
    /// Set or clear the break point at an op index.
    pub toggle: Callback<u32>,
}

/// The ops of one function as a table. Hover points at the source of an op
/// and a click selects it; the marked op stands out, and a gutter, when
/// given, sets break points.
///
/// # Arguments
///
/// * `state` - The state.
/// * `func` - The function.
/// * `marked` - The op index to mark, if one.
/// * `gutter` - The break points, if the table sets them.
///
/// # Returns
///
/// The view.
pub fn asm_table(
    state: AppState, func: AsmFunc, marked: Signal<Option<u32>>, gutter: Option<Gutter>,
) -> impl IntoView
{
    let index = func.index;
    let columns: &[Column] = tern!(gutter.is_some(), &GUTTER_COLUMNS, &OP_COLUMNS);
    let rows =
        func.rows.into_iter().map(|row| op_row(state, index, row, marked, gutter));
    table()
        .class("phase-table asm-table")
        .child((table_head(columns), tbody().child(rows.collect_view())))
}

/// One row of an asm table.
///
/// # Arguments
///
/// * `state` - The state.
/// * `func` - The function index.
/// * `row` - The op.
/// * `marked` - The op index to mark, if one.
/// * `gutter` - The break points, if the table sets them.
///
/// # Returns
///
/// The view.
fn op_row(
    state: AppState, func: u32, row: OpRow, marked: Signal<Option<u32>>,
    gutter: Option<Gutter>,
) -> impl IntoView
{
    let (pc, range) = (row.pc, row.range);
    let has_break = move || gutter.is_some_and(|g| g.breaks.with(|b| b.contains(&pc)));
    let mark = gutter.map(|g| td().class("cell-break").child(break_dot(g, pc)));
    let cells = (
        mark,
        cell("cell-num", row.addr),
        cell("cell-label", row.label.map(|n| format!("L{n}:"))),
        cell("cell-op", row.text),
        cell("cell-detail", row.note),
        cell("cell-num", row.line),
    );
    tr().id(op_row_id(func, pc))
        .class(("row-pc", move || marked.get() == Some(pc)))
        .class(("row-break", has_break))
        .on(ev::mouseenter, move |_| state.point(range))
        .on(ev::mouseleave, move |_| state.point(None))
        .on(ev::click, move |_| do_if!(let Some(r) = range, state.select(r)))
        .child(cells)
}

/// The dot of the break point gutter.
///
/// # Arguments
///
/// * `gutter` - The break points.
/// * `pc` - The op index of the row.
///
/// # Returns
///
/// The view.
fn break_dot(gutter: Gutter, pc: u32) -> impl IntoView
{
    let on = move || gutter.breaks.with(|b| b.contains(&pc));
    button()
        .class("break-dot")
        .class(("break-on", on))
        .attr("aria-pressed", move || on().to_string())
        .attr("aria-label", t!("explorer.col.break_note"))
        .title(t!("explorer.col.break_note"))
        .on(ev::click, move |e| {
            e.stop_propagation();
            gutter.toggle.run(pc);
        })
}

/// Element id of the row of an op.
///
/// # Arguments
///
/// * `func` - The function index.
/// * `pc` - The op index.
///
/// # Returns
///
/// The id.
pub fn op_row_id(func: u32, pc: u32) -> String { format!("op-{func}-{pc}") }

/// A table of slots: index, name and kind.
///
/// # Arguments
///
/// * `prefix` - The slot prefix, `g` or `l`.
/// * `slots` - The slots.
///
/// # Returns
///
/// The view.
pub fn slot_table(prefix: char, slots: Vec<SlotRow>) -> impl IntoView
{
    let row = move |s: SlotRow| {
        tr().child((
            cell("cell-num", format!("{prefix}{}", s.index)),
            cell("cell-lexeme", s.name),
            cell("cell-kind", s.kind),
        ))
    };
    let rows = slots.into_iter().map(row).collect_view();
    table().class("phase-table").child((table_head(&SLOT_COLUMNS), tbody().child(rows)))
}
