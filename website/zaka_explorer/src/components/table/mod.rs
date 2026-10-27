use std::cell::RefCell;
use std::collections::HashMap;

use codee::string::FromToStringCodec;
use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::div;
use leptos::html::span;
use leptos::html::td;
use leptos::html::th;
use leptos::html::thead;
use leptos::html::tr;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_use_event_listener::UseEventListenerOptions;
use leptos_use_event_listener::use_event_listener_with_options;
use leptos_use_on_click_outside::on_click_outside;
use leptos_use_storage::use_local_storage;
use leptos_use_window::use_window;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use zaka_web_ui_common::controls::ClassName as Controls;
use zaka_web_ui_common::icon;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::compile::spans::Range;
use crate::state::AppState;
use crate::state::columns::drag_edge;
use crate::state::columns::fit_widths;
use crate::state::columns::is_hidden;
use crate::state::columns::next_shown;
use crate::state::columns::parse_widths;
use crate::state::columns::shown_count;
use crate::state::columns::shown_width;
use crate::state::columns::toggle_column;
use crate::state::columns::widths_list;

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

/// A box around a phase table that scrolls left and right when the table is
/// wider than the pane, so the titles and the other tables of a view remain
/// in place.
///
/// # Arguments
///
/// * `table` - The table.
///
/// # Returns
///
/// The view.
pub fn table_box(table: impl IntoView + 'static) -> impl IntoView
{
    div().class(ClassName::TABLE_BOX).child(table)
}

/// A column head: locale ids of the label and the tooltip around its CSS
/// class; blank strings mean none.
pub type Column = (&'static str, &'static str, &'static str);

/// Local storage name of the hidden columns of a table; the store name of
/// the table follows it.
const HIDDEN_STORE: &str = "zaka-explorer-hidden-columns";

/// Local storage name of the column widths of a table; the store name of the
/// table follows it.
const WIDTH_STORE: &str = "zaka-explorer-column-widths";

/// Width in px a hidden column takes when the widths of its table freeze; it
/// has no width of its own to measure.
const UNSEEN_WIDTH: f64 = 96.0;

/// Attribute on the root element while a column is dragged; the page sheet
/// holds the resize cursor and stops text selection under it.
const RESIZING: &str = "data-resizing";

/// Least room in px between the column list and the edges of the window.
const EDGE: f64 = 8.0;

/// Room in px the column list takes: its width, and the height of each
/// choice and of its frame, to keep it inside the window.
const LIST_SIZE: (f64, f64, f64) = (200.0, 31.0, 10.0);

/// The head row of a phase table. A right-click on it opens a list of the
/// columns that hides and shows them, and a grip between two heads moves the
/// edge between their columns. The head carries the hidden columns in
/// `data-hide`, and the sheet hides their cells; once an edge is dragged,
/// `data-sized` gives the table fixed widths that fill its box.
///
/// # Arguments
///
/// * `store` - Name the table keeps its hidden columns and widths under; tables
///   of one kind share it.
/// * `columns` - The columns.
///
/// # Returns
///
/// The view.
pub fn table_head(store: &str, columns: &[Column]) -> impl IntoView + use<>
{
    let (hidden, set_hidden) = stored_list(format!("{HIDDEN_STORE}-{store}"));
    let (widths, set_widths) = stored_list(format!("{WIDTH_STORE}-{store}"));
    let picker = Picker {
        open: RwSignal::new(false),
        spot: RwSignal::new(String::new()),
        list: NodeRef::new(),
        head: NodeRef::new(),
        drag: RwSignal::new(None),
        start: StoredValue::new(Vec::new()),
        hidden,
        set_hidden,
        widths,
        set_widths,
        count: columns.len(),
    };
    let cells = columns.iter().enumerate().map(|(i, &(label, class, title))| {
        let title = (!title.is_empty()).then(|| t!(title.to_string()));
        let width = move || picker.width(i).map(|w| format!("width: {w}px"));
        th().class(class)
            .title(title)
            .attr("style", width)
            .child((t!(label.to_string()), column_grip(picker, i + 1)))
    });
    let labels = columns.iter().map(|&(label, _, _)| label).collect();
    let pick = th().class(ClassName::COL_PICK).child(column_list(picker, labels));
    thead()
        .node_ref(picker.head)
        .attr("data-hide", hidden)
        .attr("data-sized", move || picker.sized().then_some(""))
        .title(t!("explorer.col.pick"))
        .on(ev::contextmenu, move |e: ev::MouseEvent| {
            e.prevent_default();
            picker.open_at((f64::from(e.client_x()), f64::from(e.client_y())));
        })
        .child(tr().child((cells.collect_view(), pick)))
}

thread_local! {
    /// The lists in local storage the table heads read and write, under
    /// their storage names, each with the owner that keeps it.
    static STORED_LISTS: RefCell<HashMap<String, (Owner, StoredList)>> =
        RefCell::new(HashMap::new());
}

/// A list in local storage, and how to set it.
type StoredList = (Signal<String>, WriteSignal<String>);

/// A list in local storage, made once per storage name under an owner of its
/// own. The heads of the tables come and go as the views redraw, often while
/// a write from one has not reached the storage; the lists outlive them, and
/// a table of the same kind finds its list again.
///
/// # Arguments
///
/// * `name` - The storage name.
///
/// # Returns
///
/// The list, and how to set it.
fn stored_list(name: String) -> StoredList
{
    STORED_LISTS.with_borrow_mut(|lists| {
        let (_, list) = lists.entry(name.clone()).or_insert_with(|| {
            let owner = Owner::new_root(None);
            let list = owner.with(|| {
                let (list, set_list, _) =
                    use_local_storage::<String, FromToStringCodec>(name);
                (list, set_list)
            });
            (owner, list)
        });
        *list
    })
}

/// The state the head of one table shares between its cells, its grips, the
/// column list and their handlers.
#[derive(Clone, Copy)]
struct Picker
{
    /// Whether the list shows.
    open: RwSignal<bool>,
    /// Placement of the list: its inline rules.
    spot: RwSignal<String>,
    /// The list, while it shows.
    list: NodeRef<html::Div>,
    /// The head, whose cells give the widths to freeze.
    head: NodeRef<html::Thead>,
    /// The edge a grip drags: the columns left and right of it, from 1, and
    /// where the pointer started.
    drag: RwSignal<Option<(usize, usize, f64)>>,
    /// The widths when the drag started.
    start: StoredValue<Vec<f64>>,
    /// The hidden list of the table.
    hidden: Signal<String>,
    /// Sets the hidden list.
    set_hidden: WriteSignal<String>,
    /// The width list of the table.
    widths: Signal<String>,
    /// Sets the width list.
    set_widths: WriteSignal<String>,
    /// The number of columns.
    count: usize,
}

impl Picker
{
    /// Open the list at a point, moved in from the edges of the window so
    /// it shows whole, and put the focus on its first choice.
    ///
    /// # Arguments
    ///
    /// * `point` - The point: its distances from the left and the top edge.
    fn open_at(&self, point: (f64, f64))
    {
        let (across, down) = point;
        let size = |n: Result<JsValue, JsValue>| n.ok().and_then(|n| n.as_f64());
        let width = size(window().inner_width()).unwrap_or(0.0);
        let height = size(window().inner_height()).unwrap_or(0.0);
        let (list_width, row, frame) = LIST_SIZE;
        let list_height = self.count as f64 * row + frame;
        let left = across.min(width - list_width - EDGE).max(EDGE);
        let low = down + list_height > height - EDGE;
        let top = tern!(low, (down - list_height).max(EDGE), down);
        self.spot.set(format!("top: {top}px; left: {left}px;"));
        self.open.set(true);
        let list = self.list;
        request_animation_frame(move || {
            let first = list.get_untracked().and_then(|l| l.first_element_child());
            let first = first.and_then(|f| f.dyn_into::<web_sys::HtmlElement>().ok());
            do_if!(let Some(first) = first, _ = first.focus());
        });
    }

    /// Close the list.
    fn hide(&self) { self.open.set(false) }

    /// Hide a column, or show it again. With fixed widths, the shown columns
    /// scale to the width the shown ones took before, so the table still
    /// fills its box.
    ///
    /// # Arguments
    ///
    /// * `n` - The column number, from 1.
    fn toggle(&self, n: usize)
    {
        let before = self.hidden.get_untracked();
        let list = toggle_column(&before, n, self.count);
        do_if!(self.widths_untracked().is_some(), {
            let widths = self.measure();
            let fitted = fit_widths(&widths, &list, shown_width(&widths, &before));
            self.set_widths.set(widths_list(&fitted));
        });
        self.set_hidden.set(list);
    }

    /// Whether the columns have fixed widths.
    ///
    /// # Returns
    ///
    /// `true` once an edge was dragged.
    fn sized(&self) -> bool
    {
        self.widths.with(|list| parse_widths(list, self.count).is_some())
    }

    /// The fixed widths, read without tracking.
    ///
    /// # Returns
    ///
    /// The widths in px, or `None` while the columns take natural widths.
    fn widths_untracked(&self) -> Option<Vec<f64>>
    {
        self.widths.with_untracked(|list| parse_widths(list, self.count))
    }

    /// The fixed width of a column.
    ///
    /// # Arguments
    ///
    /// * `i` - The column index, from 0.
    ///
    /// # Returns
    ///
    /// The width in px, or `None` while the column takes a natural width.
    fn width(&self, i: usize) -> Option<f64>
    {
        let widths = self.widths.with(|list| parse_widths(list, self.count));
        widths.and_then(|w| w.get(i).copied())
    }

    /// The widths the cells of the head take now; a hidden column keeps its
    /// fixed width, or takes [`UNSEEN_WIDTH`].
    ///
    /// # Returns
    ///
    /// The widths in px.
    fn measure(&self) -> Vec<f64>
    {
        let fixed = self.widths_untracked();
        let cells =
            self.head.get_untracked().and_then(|h| h.query_selector_all("th").ok());
        let width = |i: usize| {
            let cell = cells.as_ref().and_then(|c| c.item(i as u32));
            let cell = cell.and_then(|c| c.dyn_into::<web_sys::Element>().ok());
            let width = cell.map_or(0.0, |c| c.get_bounding_client_rect().width());
            let kept = fixed.as_ref().and_then(|f| f.get(i).copied());
            tern!(width > 0.0, width, kept.unwrap_or(UNSEEN_WIDTH))
        };
        (0..self.count).map(width).collect()
    }

    /// Start to drag the right edge of a column: freeze the widths the
    /// columns take now, and hold the pointer. The last shown column has no
    /// edge to drag.
    ///
    /// # Arguments
    ///
    /// * `n` - The column number, from 1.
    /// * `e` - The press.
    fn grab(&self, n: usize, e: &ev::PointerEvent)
    {
        ret_if!(e.button() != 0);
        let Some(next) =
            self.hidden.with_untracked(|list| next_shown(list, n, self.count))
        else
        {
            return;
        };
        e.prevent_default();
        e.stop_propagation();
        let widths = self.measure();
        self.set_widths.set(widths_list(&widths));
        self.start.set_value(widths);
        self.drag.set(Some((n, next, f64::from(e.client_x()))));
        let grip = e.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok());
        do_if!(let Some(grip) = grip, _ = grip.set_pointer_capture(e.pointer_id()));
        let root = document().document_element();
        do_if!(let Some(root) = root, _ = root.set_attribute(RESIZING, "vertical"));
    }

    /// Follow the pointer while an edge is dragged: the column on its left
    /// grows as much as the one on its right gives up.
    ///
    /// # Arguments
    ///
    /// * `e` - The move.
    fn follow(&self, e: &ev::PointerEvent)
    {
        let Some((n, next, from)) = self.drag.get_untracked()
        else
        {
            return;
        };
        let shift = f64::from(e.client_x()) - from;
        let widths = self.start.with_value(|w| drag_edge(w, n, next, shift));
        self.set_widths.set(widths_list(&widths));
    }

    /// End a drag.
    fn release(&self)
    {
        ret_if!(self.drag.get_untracked().is_none());
        self.drag.set(None);
        let root = document().document_element();
        do_if!(let Some(root) = root, _ = root.remove_attribute(RESIZING));
    }

    /// Give the columns back their natural widths.
    fn reset_widths(&self) { self.set_widths.set(String::new()) }
}

/// The grip on the right edge of a column head, which drags the edge between
/// the column and the next shown one; the last shown column has none.
///
/// # Arguments
///
/// * `picker` - The head.
/// * `n` - The column number, from 1.
///
/// # Returns
///
/// The view.
fn column_grip(picker: Picker, n: usize) -> impl IntoView
{
    let on = move || picker.drag.get().is_some_and(|(at, _, _)| at == n);
    let last =
        move || picker.hidden.with(|list| next_shown(list, n, picker.count).is_none());
    span()
        .class(ClassName::COL_GRIP)
        .class((ClassName::COL_GRIP_ON, on))
        .class((ClassName::COL_GRIP_OFF, last))
        .attr("aria-hidden", "true")
        .on(ev::pointerdown, move |e| picker.grab(n, &e))
        .on(ev::pointermove, move |e| picker.follow(&e))
        .on(ev::pointerup, move |_| picker.release())
        .on(ev::pointercancel, move |_| picker.release())
        .on(ev::click, |e| e.stop_propagation())
}

/// The column list while it shows: a choice per column, with a check while
/// the column shows. A click hides or shows a column; the last shown one
/// remains. Escape, a click outside, a scroll or a resize closes the list.
///
/// # Arguments
///
/// * `picker` - The picker.
/// * `labels` - Locale ids of the column labels.
///
/// # Returns
///
/// The view.
fn column_list(picker: Picker, labels: Vec<&'static str>) -> impl IntoView
{
    let _ = on_click_outside(picker.list, move |_| picker.hide());
    close_on_move(picker);
    let labels = StoredValue::new(labels);
    move || picker.open.get().then(|| list_view(picker, labels.get_value()))
}

/// Close the list while the page scrolls or the window changes size, since
/// the list would no longer sit where it opened.
///
/// # Arguments
///
/// * `picker` - The picker.
fn close_on_move(picker: Picker)
{
    let options = UseEventListenerOptions::default().capture(true);
    let close = move || do_if!(picker.open.get_untracked(), picker.hide());
    let _ = use_event_listener_with_options(
        use_window(),
        ev::scroll,
        move |_| close(),
        options,
    );
    let _ = use_event_listener_with_options(
        use_window(),
        ev::resize,
        move |_| close(),
        options,
    );
}

/// The list of the columns.
///
/// # Arguments
///
/// * `picker` - The picker.
/// * `labels` - Locale ids of the column labels.
///
/// # Returns
///
/// The view.
fn list_view(picker: Picker, labels: Vec<&'static str>) -> impl IntoView
{
    let rows = labels.into_iter().enumerate();
    let rows = rows.map(|(i, label)| choice_view(picker, i + 1, label)).collect_view();
    let reset = button()
        .class(format!("{} {}", Controls::DROPDOWN_CHOICE, ClassName::COL_CHOICE))
        .r#type("button")
        .attr("role", "menuitem")
        .attr("disabled", move || !picker.sized())
        .on(ev::click, move |_| picker.reset_widths())
        .child(t!("explorer.col.reset_widths"));
    let rule = div().class(ClassName::COL_RULE).attr("role", "separator");
    // A smooth scroller on the page takes the wheel; the mark leaves it to
    // the list.
    div()
        .class(format!("{} {}", Controls::DROPDOWN_LIST, ClassName::COL_LIST))
        .node_ref(picker.list)
        .attr("role", "menu")
        .on(ev::keydown, move |e| do_if!(e.key() == "Escape", picker.hide()))
        .attr("style", picker.spot.get_untracked())
        .attr("data-lenis-prevent", "")
        .child((rows, rule, reset))
}

/// One column of the list, with a check while it shows; the last shown
/// column cannot be hidden.
///
/// # Arguments
///
/// * `picker` - The picker.
/// * `n` - The column number, from 1.
/// * `label` - Locale id of its label.
///
/// # Returns
///
/// The view.
fn choice_view(picker: Picker, n: usize, label: &'static str) -> impl IntoView
{
    let shown = move || !is_hidden(&picker.hidden.get(), n);
    let last = move || shown() && shown_count(&picker.hidden.get(), picker.count) == 1;
    button()
        .class(format!("{} {}", Controls::DROPDOWN_CHOICE, ClassName::COL_CHOICE))
        .r#type("button")
        .attr("role", "menuitemcheckbox")
        .attr("aria-checked", move || shown().to_string())
        .attr("disabled", last)
        .on(ev::click, move |_| picker.toggle(n))
        .child((
            span().class(Controls::DROPDOWN_LABEL).child(t!(label.to_string())),
            move || shown().then(icon::check),
        ))
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
