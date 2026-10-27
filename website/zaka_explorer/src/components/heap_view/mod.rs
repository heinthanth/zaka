use std::borrow::Cow;

use codee::string::FromToStringCodec;
use leptos::ev;
use leptos::html::div;
use leptos::html::label;
use leptos::html::p;
use leptos::html::pre;
use leptos::html::span;
use leptos::html::table;
use leptos::html::tbody;
use leptos::html::td;
use leptos::html::tr;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_use_storage::UseStorageOptions;
use leptos_use_storage::use_local_storage_with_options;
use zaka_vm::mem::Origin;
use zaka_web_ui_common::controls::settings::switch;
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
use crate::debug::CELL;
use crate::debug::HeapShot;
use crate::debug::Lane;
use crate::debug::MapPart;
use crate::debug::Session;
use crate::debug::cell_width;
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

/// Local storage name of whether the gaps show in the map and the list.
const GAPS_STORE: &str = "zaka-explorer-heap-gaps";

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
    let (gaps, set_gaps) = use_gaps();
    wire_focus(state, shot, open);
    let blank = Memo::new(move |_| shot.with(|s| s.blocks.is_empty()));
    let content = move || {
        let note = p().class(Table::VIEW_NOTE).child(t!("explorer.heap.blank"));
        ret_if!(blank.get(), note.into_any());
        let parts =
            (heap_stats(shot, gaps, set_gaps), legend(shot), heap_map(shot, open));
        (parts, block_list(shot, open, gaps)).into_any()
    };
    (sheet(&SHEET), div().class(ClassName::HEAP_VIEW).child(content))
}

/// Whether the list shows the gaps between the blocks, kept in local
/// storage; the gaps show at first. The map shows them each time.
///
/// # Returns
///
/// Whether the gaps show, and how to show or hide them.
pub fn use_gaps() -> (Signal<bool>, WriteSignal<bool>)
{
    let options = UseStorageOptions::default().initial_value(true);
    let (gaps, set_gaps, _) =
        use_local_storage_with_options::<bool, FromToStringCodec>(GAPS_STORE, options);
    (gaps, set_gaps)
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
        let start = shot.with_untracked(|s| s.row_of(addr));
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

/// The line of counts: the live heap octets of the cap, the stack octets in
/// use of the stack region, the blocks in use and those in quarantine, and
/// the blocks left out of the list; on its right, the switch that shows the
/// gaps.
///
/// # Arguments
///
/// * `shot` - The blocks.
/// * `gaps` - Whether the gaps show.
/// * `set_gaps` - Shows or hides them.
///
/// # Returns
///
/// The view.
fn heap_stats(
    shot: Memo<HeapShot>, gaps: Signal<bool>, set_gaps: WriteSignal<bool>,
) -> impl IntoView
{
    let text = move || {
        shot.with(|s| {
            t!(
                "explorer.heap.stats",
                used = octets(s.live_heap),
                cap = octets(s.cap),
                stack = octets(s.stack_used),
                stack_cap = octets(s.stack_cap),
                live = s.live,
                freed = s.freed
            )
        })
    };
    let toggle = label()
        .class(ClassName::GAP_SWITCH)
        .title(t!("explorer.heap.gaps_note"))
        .child((
            t!("explorer.heap.gaps"),
            switch(gaps, Signal::stored(false), set_gaps),
        ));
    let omitted = move || {
        let left = shot.with(|s| s.omitted);
        (left > 0).then(|| {
            let text = t!("explorer.heap.omitted", count = left);
            span().class(ClassName::HEAP_OMITTED).child(text)
        })
    };
    let counts = p().class(ClassName::HEAP_COUNTS).child((text, omitted));
    div().class(ClassName::HEAP_STATS).child((counts, toggle))
}

/// The legend of the colors of the map; the trap shows once the run
/// trapped at an address.
///
/// # Arguments
///
/// * `shot` - The blocks.
///
/// # Returns
///
/// The view.
fn legend(shot: Memo<HeapShot>) -> impl IntoView
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
            ClassName::ORIGIN_STACK,
            "explorer.heap.origin_stack",
            "explorer.heap.stack_note",
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
        entry(ClassName::SWATCH_GAP, "explorer.heap.gap", "explorer.heap.gap_note"),
        size_scale(),
        move || {
            let trapped = shot.with(|s| s.fault.is_some());
            let fault = || {
                entry(
                    ClassName::SWATCH_FAULT,
                    "explorer.heap.fault",
                    "explorer.heap.fault_note",
                )
            };
            trapped.then(fault)
        },
    ))
}

/// The scale of the widths of the cells, from a small block to a large one.
///
/// # Returns
///
/// The view.
fn size_scale() -> impl IntoView
{
    let swatch = |len: u64| {
        let class = format!("{} {}", ClassName::SCALE_CELL, ClassName::ORIGIN_HEAP);
        span()
            .class(class)
            .style(("width", format!("{}px", cell_width(len) * 10 / CELL)))
    };
    span()
        .class(ClassName::LEGEND_ENTRY)
        .title(t!("explorer.heap.size_scale_note"))
        .child((
            t!("explorer.heap.size_small"),
            [8, 64, 512].map(swatch),
            t!("explorer.heap.size_large"),
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
        Origin::Stack => ClassName::ORIGIN_STACK,
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

/// Where a block came from, as text: a stack or static block names the
/// variables it holds.
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
    b.owner.clone().map_or_else(|| origin_name(b.origin), Cow::Owned)
}

/// An address as `0x` and lowercase hex digits, padded with zeros to a
/// width, so the addresses of the panel line up.
///
/// # Arguments
///
/// * `addr` - The address.
/// * `digits` - The width, from [`HeapShot::addr_digits`].
///
/// # Returns
///
/// The text.
fn addr_text(addr: u64, digits: usize) -> String { format!("0x{addr:0digits$x}") }

/// The text of a tooltip on a block: its start, size, origin and state.
///
/// # Arguments
///
/// * `b` - The block.
/// * `digits` - The width of an address.
///
/// # Returns
///
/// The text.
fn block_tip(b: &BlockRow, digits: usize) -> String
{
    let (size, origin, state) = (octets(b.len), origin_text(b), b.state.label());
    format!("{} · {size} · {origin} · {state}", addr_text(b.start, digits))
}

/// The map, in the style of a contribution graph: a lane for the static
/// data, the stack and the heap, each a run of cells, one per block in
/// address order, that wraps to the width of the pane. A cell takes the
/// color of its origin and state, and a width that grows with the size of
/// its block (see [`cell_width`]).
/// The gap after a block, where an access traps, is a hatched seam that
/// wraps with its block. The part the run trapped in is ringed. A click on
/// a cell opens the block in the list.
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
    let lanes = move || {
        shot.with(|s| {
            let lane =
                |(lane, parts): (Lane, Vec<MapPart>)| lane_view(s, lane, parts, open);
            s.lanes().into_iter().map(lane).collect_view()
        })
    };
    div().class(ClassName::HEAP_MAP).attr("aria-hidden", "true").child(lanes)
}

/// One lane of the map: its name with its count of blocks, then its cells.
///
/// # Arguments
///
/// * `s` - The blocks.
/// * `lane` - The lane.
/// * `parts` - The parts of the lane.
/// * `open` - The start of the open block.
///
/// # Returns
///
/// The view.
fn lane_view(
    s: &HeapShot, lane: Lane, parts: Vec<MapPart>, open: RwSignal<Option<u64>>,
) -> impl IntoView + use<>
{
    let count = parts.iter().filter(|p| matches!(p, MapPart::Block(_))).count();
    let name = match lane
    {
        Lane::Data => t!("explorer.heap.lane_data", count = count),
        Lane::Stack => t!("explorer.heap.lane_stack", count = count),
    };
    let units = unit_views(s, &parts, open);
    (
        span().class(ClassName::LANE_NAME).child(name.into_owned()),
        div().class(ClassName::LANE_CELLS).child(units),
    )
}

/// The cells of a lane, each block with the seam of the gap after it, so the
/// two wrap as one. A block has no seam when the next one follows with no
/// gap, as nothing between them traps.
///
/// # Arguments
///
/// * `s` - The blocks.
/// * `parts` - The parts of the lane.
/// * `open` - The start of the open block.
///
/// # Returns
///
/// The views.
fn unit_views(
    s: &HeapShot, parts: &[MapPart], open: RwSignal<Option<u64>>,
) -> Vec<AnyView>
{
    let seam_after = |at: usize| match parts.get(at + 1)
    {
        Some(&MapPart::Gap { start, len }) =>
        {
            Some(seam(start, len, s.faults_in(start, len), s.addr_digits()))
        }
        _ => None,
    };
    let unit = |(at, part): (usize, &MapPart)| {
        let &MapPart::Block(i) = part
        else
        {
            return None;
        };
        let b = &s.blocks[i];
        let cells = block_cells(s, b, open);
        Some(
            span().class(ClassName::MAP_UNIT).child((cells, seam_after(at))).into_any(),
        )
    };
    parts.iter().enumerate().filter_map(unit).collect()
}

/// The cells of a block: one, or one for each variable of a packed block,
/// next to each other with no seam, as nothing between them traps.
///
/// # Arguments
///
/// * `s` - The blocks.
/// * `b` - The block.
/// * `open` - The start of the open row.
///
/// # Returns
///
/// The view.
fn block_cells(s: &HeapShot, b: &BlockRow, open: RwSignal<Option<u64>>) -> AnyView
{
    let digits = s.addr_digits();
    let cell = |r: &BlockRow| {
        cell_of(r, block_tip(r, digits), s.faults_in(r.start, r.len.max(1)), open)
    };
    let cells: Vec<_> = b.rows().iter().map(cell).collect();
    span().class(ClassName::MAP_PACK).child(cells).into_any()
}

/// A cell of the map: as wide as its row's octets, in the color of its
/// block. A click opens the row in the list.
///
/// # Arguments
///
/// * `b` - The row: a block, or a variable of a packed block.
/// * `tip` - Its tooltip.
/// * `fault` - Whether the run trapped in it.
/// * `open` - The start of the open block.
///
/// # Returns
///
/// The view.
fn cell_of(
    b: &BlockRow, tip: String, fault: bool, open: RwSignal<Option<u64>>,
) -> impl IntoView + use<>
{
    let start = b.start;
    #[rustfmt::skip]
    let class = format!("{} {} {}", ClassName::MAP_CELL, origin_class(b.origin),
        state_class(b.state));
    span()
        .class(class)
        .style(("width", format!("{}px", cell_width(b.len))))
        .class((ClassName::MAP_CELL_OPEN, move || open.get() == Some(start)))
        .class((ClassName::MAP_FAULT, fault))
        .title(tip)
        .on(ev::click, move |_| toggle(open, start))
}

/// The seam of the gap after a block.
///
/// # Arguments
///
/// * `start` - The first octet of the gap.
/// * `len` - Its octet count.
/// * `fault` - Whether the run trapped in it.
/// * `digits` - The width of an address.
///
/// # Returns
///
/// The view.
fn seam(start: u64, len: u64, fault: bool, digits: usize) -> impl IntoView
{
    let at = addr_text(start, digits);
    let tip = t!("explorer.heap.gap_tip", start = at, size = octets(len));
    span()
        .class(ClassName::MAP_GAP)
        .class((ClassName::MAP_FAULT, fault))
        .title(tip.into_owned())
}

/// The name of an origin.
///
/// # Arguments
///
/// * `origin` - Where blocks came from.
///
/// # Returns
///
/// The name.
fn origin_name(origin: Origin) -> Cow<'static, str>
{
    match origin
    {
        Origin::Heap => t!("explorer.heap.origin_heap"),
        Origin::Stack => t!("explorer.heap.origin_stack"),
        Origin::Static => t!("explorer.heap.origin_static"),
    }
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

/// The list of blocks, with a row for each gap while the gaps show; the
/// open block shows its dump under it.
///
/// # Arguments
///
/// * `shot` - The blocks.
/// * `open` - The start of the open block.
/// * `gaps` - Whether the gaps show.
///
/// # Returns
///
/// The view.
fn block_list(
    shot: Memo<HeapShot>, open: RwSignal<Option<u64>>, gaps: Signal<bool>,
) -> impl IntoView
{
    let row = move |s: &HeapShot, part: MapPart| {
        let digits = s.addr_digits();
        match part
        {
            MapPart::Block(i) =>
            {
                let rows = s.blocks[i].rows().into_iter();
                rows.map(|r| block_rows(r, open, digits)).collect_view().into_any()
            }
            MapPart::Gap { start, len } =>
            {
                gap_row(start, len, s.faults_in(start, len), digits).into_any()
            }
        }
    };
    let rows = move || {
        let show = gaps.get();
        shot.with(|s| {
            let parts = s
                .parts()
                .into_iter()
                .filter(|p| show || matches!(p, MapPart::Block(_)));
            parts.map(|p| row(s, p)).collect_view()
        })
    };
    table()
        .class(format!("{} {}", Table::PHASE_TABLE, ClassName::BLOCK_TABLE))
        .child((table_head("heap-blocks", &BLOCK_COLUMNS), tbody().child(rows)))
}

/// The row of a block, and its dump when it is open.
///
/// # Arguments
///
/// * `b` - The block.
/// * `open` - The start of the open block.
/// * `digits` - The width of an address.
///
/// # Returns
///
/// The view.
fn block_rows(b: BlockRow, open: RwSignal<Option<u64>>, digits: usize)
-> impl IntoView
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
            cell(Table::CELL_NUM, addr_text(start, digits)),
            cell(Table::CELL_NUM, b.len),
            td().child((swatch, origin_text(&b))),
            td().child(badge),
        ));
    let dump = move || {
        is_open().then(|| {
            let colspan = BLOCK_COLUMNS.len().to_string();
            tr().class(ClassName::DUMP_ROW).child(
                td().attr("colspan", colspan).child(dump_view(b.clone(), digits)),
            )
        })
    };
    (row, dump)
}

/// The row of a gap in the list: its start, size, a hatched swatch and a
/// badge that tells an access there traps; ringed when the run trapped in
/// it.
///
/// # Arguments
///
/// * `start` - The first octet.
/// * `len` - The octet count.
/// * `fault` - Whether the run trapped in it.
/// * `digits` - The width of an address.
///
/// # Returns
///
/// The view.
fn gap_row(start: u64, len: u64, fault: bool, digits: usize) -> impl IntoView
{
    let swatch =
        span().class(format!("{} {}", ClassName::LEGEND_SWATCH, ClassName::SWATCH_GAP));
    let badge = span()
        .class(format!("{} {}", ClassName::STATE_BADGE, ClassName::STATE_TRAPS))
        .child(t!("explorer.heap.gap_state"));
    tr().class(ClassName::GAP_ROW).class((ClassName::GAP_ROW_FAULT, fault)).child((
        cell(Table::CELL_NUM, addr_text(start, digits)),
        cell(Table::CELL_NUM, len),
        td().child((swatch, t!("explorer.heap.gap"))),
        td().child(badge),
    ))
}

/// The contents of a block in hex, 16 octets a line, each line led with its
/// address; a note tells when the dump is cut or the contents are gone.
///
/// # Arguments
///
/// * `b` - The block.
/// * `digits` - The width of an address.
///
/// # Returns
///
/// The view.
fn dump_view(b: BlockRow, digits: usize) -> impl IntoView
{
    let line = |(i, chunk): (usize, &[u8])| {
        let at = addr_text(b.start + (i * DUMP_WIDTH) as u64, digits);
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
