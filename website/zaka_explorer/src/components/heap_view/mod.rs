use std::borrow::Cow;

use leptos::ev;
use leptos::html::div;
use leptos::html::p;
use leptos::html::pre;
use leptos::html::span;
use leptos::html::table;
use leptos::html::tbody;
use leptos::html::td;
use leptos::html::tr;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_vm::mem::Origin;
use zaka_vm::trap::hex;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::components::run_view::scroll_to;
use crate::components::table::ClassName as Table;
use crate::components::table::Column;
use crate::components::table::cell;
use crate::components::table::table_head;
use crate::debug::BlockRow;
use crate::debug::BlockState;
use crate::debug::HeapShot;
use crate::debug::Session;
use crate::debug::octets;
use crate::state::AppState;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/heap_view/mod.sass");

/// Sheet of the heap view.
static SHEET: ComponentStyle =
    ComponentStyle::global_css("zaka-heap-view", STYLE_SHEET);

/// Columns of the blocks.
const BLOCK_COLUMNS: [Column; 4] = [
    ("explorer.col.addr", Table::CELL_NUM, ""),
    ("explorer.heap.size", Table::CELL_NUM, "explorer.heap.size_note"),
    ("explorer.heap.origin", "", ""),
    ("explorer.heap.state", "", ""),
];

/// The octets on one line of a dump.
const DUMP_WIDTH: usize = 16;

/// The blocks of the address space: a line of counts, a map of the blocks
/// in address order, and a list of them where a click opens the dump of a
/// block. A click on a pointer in the dock opens the block it leads into.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
pub fn heap_view(state: AppState) -> impl IntoView
{
    let shot = Memo::new(move |_| state.debug.with(Session::heap));
    let open = RwSignal::new(None::<u64>);
    wire_focus(state, shot, open);
    let blank = Memo::new(move |_| shot.with(|s| s.blocks.is_empty()));
    let content = move || {
        let note = p().class(Table::VIEW_NOTE).child(t!("explorer.heap.blank"));
        ret_if!(blank.get(), note.into_any());
        (heap_stats(shot), legend(), heap_map(shot, open), block_list(shot, open))
            .into_any()
    };
    (sheet(&SHEET), div().class(ClassName::HEAP_VIEW).child(content))
}

/// Open the block a clicked pointer leads into and bring it in view.
///
/// # Arguments
///
/// * `state` - The state.
/// * `shot` - The blocks.
/// * `open` - The start of the open block.
fn wire_focus(state: AppState, shot: Memo<HeapShot>, open: RwSignal<Option<u64>>)
{
    Effect::new(move |_| {
        let Some(addr) = state.heap_focus.get()
        else
        {
            return;
        };
        state.heap_focus.set(None);
        let start = shot.with_untracked(|s| s.block_of(addr));
        open.set(start);
        do_if!(let Some(s) = start, scroll_to(&block_id(s)));
    });
}

/// Element id of the row of a block.
///
/// # Arguments
///
/// * `start` - The start of the block.
///
/// # Returns
///
/// The id.
fn block_id(start: u64) -> String { format!("heap-block-{start}") }

/// The line of counts: the live heap octets of the cap, the blocks in use
/// and those in quarantine, and the blocks left out of the list.
///
/// # Arguments
///
/// * `shot` - The blocks.
///
/// # Returns
///
/// The view.
fn heap_stats(shot: Memo<HeapShot>) -> impl IntoView
{
    let text = move || {
        shot.with(|s| {
            t!(
                "explorer.heap.stats",
                used = octets(s.live_heap),
                cap = octets(s.cap),
                live = s.live,
                freed = s.freed
            )
        })
    };
    let omitted = move || {
        let left = shot.with(|s| s.omitted);
        (left > 0).then(|| {
            let text = t!("explorer.heap.omitted", count = left);
            span().class(ClassName::HEAP_OMITTED).child(text)
        })
    };
    p().class(ClassName::HEAP_STATS).child((text, omitted))
}

/// The legend of the colors of the map.
///
/// # Returns
///
/// The view.
fn legend() -> impl IntoView
{
    let entry = |class: &'static str, id: &'static str, note: &'static str| {
        span().class(ClassName::LEGEND_ENTRY).title(t!(note.to_string())).child((
            span().class(format!("{} {class}", ClassName::LEGEND_SWATCH)),
            t!(id.to_string()),
        ))
    };
    div().class(ClassName::HEAP_LEGEND).child((
        entry(
            ClassName::ORIGIN_HEAP,
            "explorer.heap.origin_heap",
            "explorer.heap.heap_note",
        ),
        entry(
            ClassName::ORIGIN_CELL,
            "explorer.heap.origin_cell",
            "explorer.heap.cell_note",
        ),
        entry(
            ClassName::ORIGIN_STATIC,
            "explorer.heap.origin_static",
            "explorer.heap.static_note",
        ),
        entry(
            ClassName::STATE_FREED,
            "explorer.heap.freed",
            "explorer.heap.freed_note",
        ),
        entry(
            ClassName::STATE_LEAKED,
            "explorer.heap.leaked",
            "explorer.heap.leaked_note",
        ),
    ))
}

/// The class of the color of an origin.
///
/// # Arguments
///
/// * `origin` - Where a block came from.
///
/// # Returns
///
/// The class.
fn origin_class(origin: Origin) -> &'static str
{
    match origin
    {
        Origin::Heap => ClassName::ORIGIN_HEAP,
        Origin::Cell => ClassName::ORIGIN_CELL,
        Origin::Static => ClassName::ORIGIN_STATIC,
    }
}

/// The class of a block state.
///
/// # Arguments
///
/// * `state` - Whether a block is in use.
///
/// # Returns
///
/// The class.
fn state_class(state: BlockState) -> &'static str
{
    match state
    {
        BlockState::Live => ClassName::STATE_LIVE,
        BlockState::Freed => ClassName::STATE_FREED,
        BlockState::Leaked => ClassName::STATE_LEAKED,
    }
}

/// Where a block came from, as text: a cell names the function that made
/// it.
///
/// # Arguments
///
/// * `b` - The block.
///
/// # Returns
///
/// The text.
fn origin_text(b: &BlockRow) -> Cow<'static, str>
{
    match (b.origin, &b.maker)
    {
        (Origin::Heap, _) => t!("explorer.heap.origin_heap"),
        (Origin::Static, _) => t!("explorer.heap.origin_static"),
        (Origin::Cell, Some(func)) => t!("explorer.heap.cell_of", func = func),
        (Origin::Cell, None) => t!("explorer.heap.origin_cell"),
    }
}

/// The text of a tooltip on a block: its start, size, origin and state.
///
/// # Arguments
///
/// * `b` - The block.
///
/// # Returns
///
/// The text.
fn block_tip(b: &BlockRow) -> String
{
    let (size, origin, state) = (octets(b.len), origin_text(b), b.state.label());
    format!("{} · {size} · {origin} · {state}", hex(b.start))
}

/// The map: one bar per block in address order, colored after its origin
/// and state. The gaps between blocks are left out, and the width of a bar
/// grows with the square root of its size, so a small block still shows
/// next to a large one. A click opens the block in the list.
///
/// # Arguments
///
/// * `shot` - The blocks.
/// * `open` - The start of the open block.
///
/// # Returns
///
/// The view.
fn heap_map(shot: Memo<HeapShot>, open: RwSignal<Option<u64>>) -> impl IntoView
{
    let bar = move |b: &BlockRow| {
        let start = b.start;
        let grow = (b.len as f64).sqrt().max(1.0);
        let class = format!(
            "{} {} {}",
            ClassName::MAP_BAR,
            origin_class(b.origin),
            state_class(b.state)
        );
        div()
            .class(class)
            .class((ClassName::MAP_BAR_OPEN, move || open.get() == Some(start)))
            .style(("flex-grow", grow.to_string()))
            .title(block_tip(b))
            .on(ev::click, move |_| toggle(open, start))
    };
    let bars = move || shot.with(|s| s.blocks.iter().map(bar).collect_view());
    div().class(ClassName::HEAP_MAP).attr("aria-hidden", "true").child(bars)
}

/// Open a block, or shut it when it is open.
///
/// # Arguments
///
/// * `open` - The start of the open block.
/// * `start` - The start of the block.
fn toggle(open: RwSignal<Option<u64>>, start: u64)
{
    open.update(|o| *o = tern!(*o == Some(start), None, Some(start)));
}

/// The list of blocks; the open one shows its dump under it.
///
/// # Arguments
///
/// * `shot` - The blocks.
/// * `open` - The start of the open block.
///
/// # Returns
///
/// The view.
fn block_list(shot: Memo<HeapShot>, open: RwSignal<Option<u64>>) -> impl IntoView
{
    let rows = move || {
        shot.with(|s| {
            s.blocks.iter().map(|b| block_rows(b.clone(), open)).collect_view()
        })
    };
    table()
        .class(format!("{} {}", Table::PHASE_TABLE, ClassName::BLOCK_TABLE))
        .child((table_head(&BLOCK_COLUMNS), tbody().child(rows)))
}

/// The row of a block, and its dump when it is open.
///
/// # Arguments
///
/// * `b` - The block.
/// * `open` - The start of the open block.
///
/// # Returns
///
/// The view.
fn block_rows(b: BlockRow, open: RwSignal<Option<u64>>) -> impl IntoView
{
    let start = b.start;
    let is_open = move || open.get() == Some(start);
    let badge = span()
        .class(format!("{} {}", ClassName::STATE_BADGE, state_class(b.state)))
        .child(b.state.label());
    let swatch = span().class(format!(
        "{} {}",
        ClassName::LEGEND_SWATCH,
        origin_class(b.origin)
    ));
    let row = tr()
        .id(block_id(start))
        .class((Table::ROW_ACTIVE, is_open))
        .attr("aria-expanded", move || is_open().to_string())
        .on(ev::click, move |_| toggle(open, start))
        .child((
            cell(Table::CELL_NUM, hex(start)),
            cell(Table::CELL_NUM, b.len),
            td().child((swatch, origin_text(&b))),
            td().child(badge),
        ));
    let dump = move || {
        is_open().then(|| {
            let colspan = BLOCK_COLUMNS.len().to_string();
            tr().class(ClassName::DUMP_ROW)
                .child(td().attr("colspan", colspan).child(dump_view(b.clone())))
        })
    };
    (row, dump)
}

/// The contents of a block in hex, 16 octets a line, each line led with its
/// address; a note tells when the dump is cut or the contents are gone.
///
/// # Arguments
///
/// * `b` - The block.
///
/// # Returns
///
/// The view.
fn dump_view(b: BlockRow) -> impl IntoView
{
    let line = |(i, chunk): (usize, &[u8])| {
        let at = hex(b.start + (i * DUMP_WIDTH) as u64);
        let pairs: Vec<String> = chunk.iter().map(|o| format!("{o:02x}")).collect();
        format!("{at}  {}", pairs.join(" "))
    };
    let lines: Vec<String> = b.data.chunks(DUMP_WIDTH).enumerate().map(line).collect();
    let note = dump_note(&b).map(|n| p().class(ClassName::DUMP_NOTE).child(n));
    let text = (!lines.is_empty())
        .then(|| pre().class(ClassName::DUMP).child(lines.join("\n")));
    (text, note)
}

/// The note under a dump.
///
/// # Arguments
///
/// * `b` - The block.
///
/// # Returns
///
/// The note, or `None` for a whole dump.
fn dump_note(b: &BlockRow) -> Option<Cow<'static, str>>
{
    ret_if!(b.state == BlockState::Freed, Some(t!("explorer.heap.dump_freed")));
    ret_if!(b.len == 0, Some(t!("explorer.heap.dump_zero")));
    let shown = b.data.len() as u64;
    (shown < b.len).then(|| t!("explorer.heap.dump_cut", shown = shown, len = b.len))
}
