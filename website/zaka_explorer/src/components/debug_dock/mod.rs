use codee::string::FromToStringCodec;
use leptos::ev;
use leptos::html;
use leptos::html::aside;
use leptos::html::button;
use leptos::html::div;
use leptos::html::h3;
use leptos::html::p;
use leptos::html::section;
use leptos::html::span;
use leptos::html::table;
use leptos::html::tbody;
use leptos::html::td;
use leptos::html::tr;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_use_storage::UseStorageOptions;
use leptos_use_storage::use_local_storage_with_options;
use strum::IntoStaticStr;
use zaka_vm::trap::hex;
use zaka_web_ui_common::icon::chevron;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::components::check_view::skipped;
use crate::components::heap_view::heap_view;
use crate::components::splitter::Axis;
use crate::components::splitter::Split;
use crate::components::splitter::splitter;
use crate::components::table::ClassName as Table;
use crate::components::table::Column;
use crate::components::table::cell;
use crate::components::table::table_head;
use crate::debug::FrameBlock;
use crate::debug::Registers;
use crate::debug::Session;
use crate::debug::ValueRow;
use crate::state::AppState;
use crate::state::split::DEFAULT_GLOBALS_SHARE;
use crate::state::split::DEFAULT_STACK_SHARE;
use crate::state::split::share_fr;
use crate::state::tour::TOUR;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/debug_dock/mod.sass");

/// Sheet of the debug dock.
static SHEET: ComponentStyle =
    ComponentStyle::global_css("zaka-debug-dock", STYLE_SHEET);

/// Columns of the stack.
const STORE_COLUMNS: [Column; 6] = [
    ("explorer.col.addr", Table::CELL_NUM, "explorer.col.addr_note"),
    ("explorer.col.slot", "", ""),
    ("explorer.col.name", "", ""),
    ("explorer.col.kind", "", ""),
    ("explorer.col.value", "", ""),
    ("explorer.col.block", "", "explorer.col.block_note"),
];

/// Columns of the globals.
const VALUE_COLUMNS: [Column; 5] = [
    ("explorer.col.slot", "", ""),
    ("explorer.col.name", "", ""),
    ("explorer.col.kind", "", ""),
    ("explorer.col.value", "", ""),
    ("explorer.col.block", "", "explorer.col.block_note"),
];

/// Local storage name of the stack share of the dock.
const STACK_SHARE_STORE: &str = "zaka-explorer-dock-stack-share";

/// Local storage name of the globals share of the panes under the stack.
const GLOBALS_SHARE_STORE: &str = "zaka-explorer-dock-globals-share";

/// Local storage name of whether a pane of the dock is open; the name of the
/// pane follows it.
const OPEN_STORE: &str = "zaka-explorer-dock-open";

/// Whether a pane of the dock is open, kept in local storage, and how to
/// open or shut it.
#[derive(Clone, Copy)]
struct PaneOpen
{
    /// Whether the pane is open.
    open: Signal<bool>,
    /// Opens or shuts the pane.
    set_open: WriteSignal<bool>,
}

/// A pane of the dock; its static name is its name for the tour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
enum Pane
{
    /// The stack with its frames.
    Stack,
    /// The global slots.
    Globals,
    /// The blocks of the address space.
    Heap,
}

impl Pane
{
    /// Locale id of the title.
    ///
    /// # Returns
    ///
    /// The id.
    fn title(self) -> &'static str
    {
        match self
        {
            Pane::Stack => "explorer.run.frames",
            Pane::Globals => "explorer.run.globals",
            Pane::Heap => "explorer.heap.title",
        }
    }
}

/// The dock of the debugger next to the stage view: the registers on top,
/// then the stack, the globals and the heap, each a pane of its own that
/// scrolls on its own, with splitters between them. Or a note when code
/// generation did not run.
///
/// # Returns
///
/// The view.
pub fn debug_dock() -> impl IntoView
{
    let state = use_state();
    let emitted = Memo::new(move |_| state.compiled.with(|c| c.emitted));
    let content = move || {
        let title = t!("explorer.run.skipped_title");
        let note = t!("explorer.run.skipped_note");
        tern!(emitted.get(), panes(state).into_any(), skipped(title, note).into_any())
    };
    let view = aside()
        .class(ClassName::DEBUG_DOCK)
        .class((ClassName::DEBUG_DOCK_SKIPPED, move || !emitted.get()))
        .attr(TOUR, "machine")
        .attr("aria-label", t!("explorer.dock.label"))
        .child(content);
    (sheet(&SHEET), view)
}

/// The registers over the panes: the stack over the globals over the heap.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn panes(state: AppState) -> impl IntoView
{
    let outer = NodeRef::<html::Div>::new();
    let inner = NodeRef::<html::Div>::new();
    let under = split_of(
        outer,
        STACK_SHARE_STORE,
        DEFAULT_STACK_SHARE,
        "explorer.dock.resize_stack",
    );
    let between = split_of(
        inner,
        GLOBALS_SHARE_STORE,
        DEFAULT_GLOBALS_SHARE,
        "explorer.dock.resize_globals",
    );
    let (stack, globals, heap) =
        (pane_open(Pane::Stack), pane_open(Pane::Globals), pane_open(Pane::Heap));
    let lower = stacked(
        inner,
        between,
        (globals.open, heap.open),
        (
            pane(Pane::Globals, globals, move || globals_view(state)),
            splitter(between),
            pane(Pane::Heap, heap, move || heap_view(state)),
        ),
    );
    let lower_open = Signal::derive(move || globals.open.get() || heap.open.get());
    let upper = stacked(
        outer,
        under,
        (stack.open, lower_open),
        (pane(Pane::Stack, stack, move || frames_view(state)), splitter(under), lower),
    );
    let regs = div()
        .class(ClassName::DOCK_REGS)
        .attr(TOUR, "registers")
        .title(t!("explorer.run.registers"))
        .child(registers_view(state));
    (regs, upper)
}

/// A split of two stacked panes, its share kept in local storage.
///
/// # Arguments
///
/// * `container` - The element whose height is split.
/// * `store` - The local storage name of the share.
/// * `initial` - The share before one is kept.
/// * `label` - Locale id of the name of the splitter.
///
/// # Returns
///
/// The split.
fn split_of(
    container: NodeRef<html::Div>, store: &'static str, initial: f64,
    label: &'static str,
) -> Split
{
    let options = UseStorageOptions::default().initial_value(initial);
    let (share, set_share, _) =
        use_local_storage_with_options::<f64, FromToStringCodec>(store, options);
    Split::builder()
        .axis(Axis::Rows)
        .container(container)
        .share(share)
        .set_share(set_share)
        .initial(initial)
        .label(label)
        .build()
}

/// Whether a pane of the dock is open, kept in local storage; a pane starts
/// open.
///
/// # Arguments
///
/// * `pane` - The pane.
///
/// # Returns
///
/// Whether it is open, and how to open or shut it.
fn pane_open(pane: Pane) -> PaneOpen
{
    let name: &'static str = pane.into();
    let options = UseStorageOptions::default().initial_value(true);
    let (open, set_open, _) = use_local_storage_with_options::<bool, FromToStringCodec>(
        format!("{OPEN_STORE}-{name}"),
        options,
    );
    PaneOpen { open, set_open }
}

/// The box of two stacked panes and the splitter between them, sized after
/// the share of the split. A shut pane takes the height of its title, and
/// the open one the rest.
///
/// # Arguments
///
/// * `container` - Its element.
/// * `split` - The split.
/// * `open` - Whether the first and the second pane are open.
/// * `children` - The panes and the splitter.
///
/// # Returns
///
/// The view.
fn stacked(
    container: NodeRef<html::Div>, split: Split, open: (Signal<bool>, Signal<bool>),
    children: impl IntoView + 'static,
) -> impl IntoView
{
    let share = split.share;
    let (first, second) = open;
    div()
        .class(ClassName::DOCK_STACK)
        .class((ClassName::DOCK_STACK_FIRST_SHUT, move || !first.get()))
        .class((ClassName::DOCK_STACK_SECOND_SHUT, move || !second.get()))
        .node_ref(container)
        .style(("--first-share", move || share_fr(share.get())))
        .style(("--second-share", move || share_fr(1.0 - share.get())))
        .child(children)
}

/// A pane of the dock: its title, then its content, which scrolls on its own.
///
/// # Arguments
///
/// * `pane` - The pane.
/// * `fold` - Whether it is open, and how to open or shut it.
/// * `content` - Makes the content; a shut pane leaves it out, so its tables do
///   not follow the run.
///
/// # Returns
///
/// The view.
fn pane<V>(
    pane: Pane, fold: PaneOpen, content: impl Fn() -> V + Send + Sync + 'static,
) -> impl IntoView
where
    V: IntoView + 'static,
{
    let open = fold.open;
    let toggle = button()
        .class(ClassName::PANE_TOGGLE)
        .r#type("button")
        .attr("aria-expanded", move || open.get().to_string())
        .on(ev::click, move |_| fold.set_open.set(!open.get_untracked()))
        .child((chevron(), span().child(t!(pane.title().to_string()))));
    let inside = move || {
        open.get().then(|| div().class(ClassName::PANE_CONTENT).child(content()))
    };
    section()
        .class(ClassName::DOCK_PANE)
        .class((ClassName::DOCK_PANE_SHUT, move || !open.get()))
        .attr(TOUR, <&'static str>::from(pane))
        .child((h3().class(ClassName::PANE_TITLE).child(toggle), inside))
}

/// The registers: the program counter, the stack pointer and the base
/// pointer.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn registers_view(state: AppState) -> impl IntoView
{
    let regs = Memo::new(move |_| state.debug.with(Session::registers));
    let text = move |f: fn(&Registers) -> Option<String>| {
        move || regs.with(f).unwrap_or_else(|| "-".to_string())
    };
    let pc = text(|r| r.pc.map(|pc| pc.to_string()));
    let sp = text(|r| Some(r.sp.to_string()));
    let bp = text(|r| r.bp.map(|b| b.to_string()));
    div().class(ClassName::RUN_REGS).child((
        register("PC", "explorer.run.pc_note", pc),
        register("SP", "explorer.run.sp_note", sp),
        register("BP", "explorer.run.bp_note", bp),
    ))
}

/// One register: its name and its value.
///
/// # Arguments
///
/// * `name` - The name.
/// * `note` - Locale id of what it holds, which shows as a tooltip.
/// * `value` - The value.
///
/// # Returns
///
/// The view.
fn register(
    name: &'static str, note: &'static str,
    value: impl Fn() -> String + Send + Sync + 'static,
) -> impl IntoView
{
    div().class(ClassName::RUN_REG).title(t!(note.to_string())).child((
        span().class(ClassName::RUN_REG_NAME).child(name),
        span().class(ClassName::RUN_REG_VALUE).child(value),
    ))
}

/// A note in place of a blank table.
///
/// # Arguments
///
/// * `id` - Locale id of the note.
///
/// # Returns
///
/// The view.
fn note(id: &'static str) -> impl IntoView
{
    p().class(Table::VIEW_NOTE).child(t!(id.to_string()))
}

/// The stack: the frame of each call in progress, innermost and highest
/// first. A frame holds the locals of its call and over them the values
/// the call pushed. The stack pointer marks the free slot over the top, the
/// base pointer the frame of the innermost call; a click shows that call.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn frames_view(state: AppState) -> impl IntoView
{
    move || {
        let blocks = state.debug.with(Session::frame_blocks);
        ret_if!(blocks.is_empty(), note("explorer.run.no_frames").into_any());
        let sp = state.debug.with(|s| s.registers().sp);
        let free = tr().class(ClassName::STACK_FREE).child((
            cell(Table::CELL_NUM, sp),
            td().attr("colspan", (STORE_COLUMNS.len() - 1).to_string())
                .child(span().class(ClassName::REG_TAG).child("sp")),
        ));
        let groups = blocks.into_iter().enumerate();
        let rows = groups.map(|(i, b)| frame_rows(state, b, i == 0)).collect_view();
        let head = table_head("stack", &STORE_COLUMNS);
        let rows = tbody().child((free, rows));
        table().class(Table::PHASE_TABLE).child((head, rows)).into_any()
    }
}

/// The rows of one frame, highest first: its head row, the values its
/// call pushed, then its locals.
///
/// # Arguments
///
/// * `state` - The state.
/// * `block` - The frame.
/// * `top` - Whether it is the frame of the innermost call, which the base
///   pointer marks.
///
/// # Returns
///
/// The view.
fn frame_rows(state: AppState, block: FrameBlock, top: bool) -> impl IntoView
{
    let (call, start) = (block.call, block.start);
    let index = call.index;
    let viewed = move || state.debug.with(Session::viewed) == index;
    let view = move |_| state.debug.update(|s| s.view_frame(index));
    let tag = top.then(|| span().class(ClassName::REG_TAG).child("bp"));
    let meta = t!("explorer.run.frame_meta", pc = call.addr, base = start);
    let label = (
        span()
            .class(ClassName::FRAME_NAME)
            .child(format!("#{index} f{} {}", call.func, call.name)),
        tag,
        span().class(ClassName::FRAME_META).child(meta),
    );
    let head = tr()
        .class(ClassName::FRAME_ROW)
        .class((Table::ROW_ACTIVE, viewed))
        .title(t!("explorer.run.view_call"))
        .on(ev::click, view)
        .child(td().attr("colspan", STORE_COLUMNS.len().to_string()).child(label));
    let slot = move |at: usize, label: String, v: ValueRow| {
        tr().class((ClassName::ROW_VIEWED, viewed)).on(ev::click, view).child((
            cell(Table::CELL_NUM, at),
            cell(Table::CELL_DETAIL, label),
            cell(Table::CELL_LEXEME, v.name.clone()),
            cell(Table::CELL_KIND, v.kind.clone()),
            value_cell(state, v.value, v.points_at),
            value_cell(state, v.home.map(hex).unwrap_or_default(), v.home),
        ))
    };
    let over = start + block.slots.len();
    let temp = move |v: ValueRow| {
        let i = v.index as usize;
        slot(over + i, format!("t{i}"), v)
    };
    let local = move |v: ValueRow| {
        let i = v.index as usize;
        slot(start + i, format!("l{i}"), v)
    };
    let temps = block.temps.into_iter().rev().map(temp).collect_view();
    let locals = block.slots.into_iter().rev().map(local).collect_view();
    (head, temps, locals)
}

/// The cell of a text that can lead to an address, such as a pointer
/// value or the block of a variable: with an address, a link that shows
/// its block in the heap.
///
/// # Arguments
///
/// * `state` - The state.
/// * `text` - The text.
/// * `addr` - The address it leads to, if one.
///
/// # Returns
///
/// The view.
fn value_cell(state: AppState, text: String, addr: Option<u64>) -> impl IntoView
{
    let Some(addr) = addr
    else
    {
        return cell(Table::CELL_LEXEME, text).into_any();
    };
    let link = button()
        .class(ClassName::POINTER_LINK)
        .title(t!("explorer.heap.show_block"))
        .on(ev::click, move |_| state.heap_focus.set(Some(addr)))
        .child(text);
    cell(Table::CELL_LEXEME, link).into_any()
}

/// The global slots, or a note when there are none.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn globals_view(state: AppState) -> impl IntoView
{
    move || {
        let rows = state.debug.with(Session::globals);
        ret_if!(rows.is_empty(), note("explorer.run.no_globals").into_any());
        let row = move |v: ValueRow| {
            tr().child((
                cell(Table::CELL_DETAIL, format!("g{}", v.index)),
                cell(Table::CELL_LEXEME, v.name.clone()),
                cell(Table::CELL_KIND, v.kind.clone()),
                value_cell(state, v.value, v.points_at),
                value_cell(state, v.home.map(hex).unwrap_or_default(), v.home),
            ))
        };
        let rows = rows.into_iter().map(row).collect_view();
        let head = table_head("globals", &VALUE_COLUMNS);
        table().class(Table::PHASE_TABLE).child((head, tbody().child(rows))).into_any()
    }
}
