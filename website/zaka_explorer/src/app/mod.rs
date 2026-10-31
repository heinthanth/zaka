use leptos::html;
use leptos::html::a;
use leptos::html::div;
use leptos::html::img;
use leptos::html::p;
use leptos::html::section;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_use_event_listener::use_event_listener;
use leptos_use_window::use_window;
use zaka_web_ui_common::controls::ClassName as Controls;
use zaka_web_ui_common::controls::scroll_bar::Scroller;
use zaka_web_ui_common::controls::scroll_bar::scroll_bar;
use zaka_web_ui_common::shared_sheets;
use zaka_web_ui_common::sheet;
use zaka_web_ui_common::site::EXPLORER_FLUSH_EVENT;
use zakac_diagnostics::Level;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

use crate::assets::BLOSSOMS_PHOTO;
use crate::components::bottom_panel::bottom_panel;
use crate::components::debug_bar::debug_bar;
use crate::components::debug_bar::wire_going;
use crate::components::debug_dock::debug_dock;
use crate::components::editor::editor_pane;
use crate::components::pipeline::pipeline;
use crate::components::run_view::run_view;
use crate::components::splitter::Axis;
use crate::components::splitter::Split;
use crate::components::splitter::splitter;
use crate::components::table::table_sheet;
use crate::components::toolbar::toolbar;
use crate::components::tour::tour;
use crate::state::AppState;
use crate::state::PanelTab;
use crate::state::permalink;
use crate::state::provide_state;
use crate::state::split::DEFAULT_DOCK_SHARE;
use crate::state::split::DEFAULT_EDITOR_SHARE;
use crate::state::split::DEFAULT_STAGE_SHARE;
use crate::state::split::ERROR_STAGE_SHARE;
use crate::state::split::share_fr;
use crate::state::tour::TOUR;
use crate::state::url;
use crate::theme::theme_sheet;

/// The page of the plumeria photo the website and the blank canvas share.
const PHOTO_PAGE: &str = "https://unsplash.com/photos/FIN5CJoBJmI";

/// Who took the plumeria photo.
const PHOTO_AUTHOR: &str = "Antonio Gabriel";

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/app/mod.sass");

/// Sheet of the page shell.
static SHEET: ComponentStyle = ComponentStyle::global_css("zaka-app", STYLE_SHEET);

/// The whole page: the page-wide sheets, then the app.
///
/// # Returns
///
/// The view.
pub fn app() -> impl IntoView { (app_sheets(), app_shell(false)) }

/// The page-wide sheets: the shared tokens and controls, the theme, the
/// tables and the page shell. A host that renders the page ahead of time
/// mounts them outside its island, as a global sheet renders on the server
/// alone.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn app_sheets() -> impl IntoView
{
    (shared_sheets(), theme_sheet(), table_sheet(), sheet(&SHEET))
}

/// The app: toolbar, then the editor on the left and the stage view over the
/// bottom panel on the right, with splitters between them; the pane switches
/// show and hide each of them. Outside debug mode the stage view holds the
/// run and the bottom panel the problems. In debug mode the debugger bar sits
/// under the toolbar, the pipeline takes the place of the run with the dock
/// of the VM state next to it, and the bottom panel holds the output too. The
/// guided tour sits over it all.
///
/// # Arguments
///
/// * `on_site` - Whether the explorer sits in a page of the website, under its
///   header, which then names the site and links to its pages in place of the
///   toolbar.
///
/// # Returns
///
/// The view.
pub fn app_shell(on_site: bool) -> impl IntoView
{
    let state = provide_state();
    follow_hash(state);
    follow_flush(state);
    follow_errors(state);
    wire_going(state);
    mark_lang();
    let panes = NodeRef::<html::Div>::new();
    let column = NodeRef::<html::Div>::new();
    let debug = state.debug_mode;
    // Outside debug mode the run is the right pane, so its switch shows it.
    let stage_shown = Signal::derive(move || {
        tern!(debug.get(), state.stage_open.get(), state.dock_open.get())
    });
    let dock_shown = Signal::derive(move || debug.get() && state.dock_open.get());
    let row_shown = Signal::derive(move || stage_shown.get() || dock_shown.get());
    let panel_shown = state.problems_open;
    let both = Signal::derive(move || row_shown.get() && panel_shown.get());
    let column_shown = Signal::derive(move || row_shown.get() || panel_shown.get());
    let editor_split =
        Signal::derive(move || state.editor_open.get() && column_shown.get());
    let between = Split::builder()
        .axis(Axis::Columns)
        .container(panes)
        .share(state.editor_share)
        .set_share(state.set_editor_share)
        .initial(DEFAULT_EDITOR_SHARE)
        .label("explorer.app.resize_editor")
        .enabled(editor_split)
        .build();
    let above = Split::builder()
        .axis(Axis::Rows)
        .container(column)
        .share(state.stage_share)
        .set_share(state.set_stage_share)
        .initial(DEFAULT_STAGE_SHARE)
        .label("explorer.app.resize_problems")
        .enabled(both)
        .build();
    let left = section()
        .class(format!("{} {}", ClassName::PANEL, ClassName::PANEL_EDITOR))
        .attr(TOUR, "editor")
        .child(editor_pane());
    let problems = section()
        .class(format!("{} {}", ClassName::PANEL, ClassName::PANEL_PROBLEMS))
        .child(bottom_panel());
    let right = div()
        .class(ClassName::COLUMN)
        .class((ClassName::COLUMN_TOP, move || row_shown.get() && !panel_shown.get()))
        .class((ClassName::COLUMN_LOW, move || !row_shown.get() && panel_shown.get()))
        .node_ref(column)
        .style(("--first-share", move || share_fr(state.stage_share.get())))
        .style(("--second-share", move || share_fr(1.0 - state.stage_share.get())))
        .child((stage_row(state, stage_shown, dock_shown), splitter(above), problems));
    // On narrow screens the panes stack and scroll, under a bar of their own
    // that takes no room.
    let main = div()
        .class(format!("{} {}", ClassName::PANES, Controls::SCROLL_OWN))
        .class((ClassName::PANES_SHUT, move || !state.editor_open.get()))
        .class((ClassName::PANES_SOLO, move || !column_shown.get()))
        .class((ClassName::PANES_BLANK, move || {
            !state.editor_open.get() && !column_shown.get()
        }))
        .attr("role", "main")
        .node_ref(panes)
        .style(("--first-share", move || share_fr(state.editor_share.get())))
        .style(("--second-share", move || share_fr(1.0 - state.editor_share.get())))
        .child((left, splitter(between), right, blank_canvas()));
    let bar = move || debug.get().then(debug_bar);
    let scroll = scroll_bar(Scroller::Element(panes));
    div().class(ClassName::APP_SHELL).child((
        toolbar(on_site),
        bar,
        main,
        scroll,
        tour(),
    ))
}

/// The stage view, and in debug mode the dock next to it with a splitter
/// between them; the pane switches hide each of them.
///
/// # Arguments
///
/// * `state` - The state.
/// * `stage_shown` - Whether the stage view shows.
/// * `dock_shown` - Whether the dock shows.
///
/// # Returns
///
/// The view.
fn stage_row(
    state: AppState, stage_shown: Signal<bool>, dock_shown: Signal<bool>,
) -> impl IntoView
{
    let row = NodeRef::<html::Div>::new();
    let debug = state.debug_mode;
    let split = Signal::derive(move || stage_shown.get() && dock_shown.get());
    let beside = Split::builder()
        .axis(Axis::Columns)
        .container(row)
        .share(state.dock_share)
        .set_share(state.set_dock_share)
        .initial(DEFAULT_DOCK_SHARE)
        .label("explorer.app.resize_dock")
        .enabled(split)
        .build();
    let stage = section()
        .class(format!("{} {}", ClassName::PANEL, ClassName::PANEL_STAGE))
        .class((ClassName::PANEL_OFF, move || !stage_shown.get()))
        .attr(TOUR, "stage")
        .child(move || {
            let run = || div().class(ClassName::RUN_PANE).child(run_view());
            tern!(debug.get(), pipeline().into_any(), run().into_any())
        });
    let dock = section()
        .class(format!("{} {}", ClassName::PANEL, ClassName::PANEL_DOCK))
        .class((ClassName::PANEL_OFF, move || !dock_shown.get()))
        .child(move || debug.get().then(debug_dock));
    div()
        .class(ClassName::STAGE_ROW)
        .class((ClassName::STAGE_ROW_SPLIT, split))
        .class((ClassName::STAGE_ROW_DOCK, move || {
            dock_shown.get() && !stage_shown.get()
        }))
        .node_ref(row)
        .style(("--first-share", move || share_fr(state.dock_share.get())))
        .style(("--second-share", move || share_fr(1.0 - state.dock_share.get())))
        .child((stage, splitter(beside), dock))
}

/// The page under the panes when the switches hide them all: the plumeria
/// photo of the website behind a veil of the canvas color, a note on how to
/// bring a pane back, and the credit of the photo.
///
/// # Returns
///
/// The view.
fn blank_canvas() -> impl IntoView
{
    let photo = img()
        .class(ClassName::BLANK_PHOTO)
        .src(BLOSSOMS_PHOTO)
        .alt("")
        .attr("aria-hidden", "true")
        .attr("loading", "lazy");
    let note = div().class(ClassName::BLANK_NOTE).child((
        p().class(ClassName::BLANK_TITLE).child(t!("explorer.app.blank_title")),
        p().class(ClassName::BLANK_TEXT).child(t!("explorer.app.blank_text")),
    ));
    let credit = p().class(ClassName::BLANK_CREDIT).child(
        a().href(PHOTO_PAGE)
            .attr("target", "_blank")
            .attr("rel", "noopener")
            .child(t!("explorer.app.blank_credit", author = PHOTO_AUTHOR)),
    );
    div().class(ClassName::BLANK_CANVAS).child((photo, note, credit))
}

/// Open the problems tab when errors show up, at half the column or more.
/// It acts as the errors come, not while the errors stand, so the panel can
/// still be shut or resized.
///
/// # Arguments
///
/// * `state` - The state.
fn follow_errors(state: AppState)
{
    let failed = Memo::new(move |_| state.compiled.with(|c| c.count(Level::Error) > 0));
    Effect::new(move |was: Option<bool>| {
        let now = failed.get();
        do_if!(now && was != Some(true), show_problems(state));
        now
    });
}

/// Open the problems tab and give it at least half the column.
///
/// # Arguments
///
/// * `state` - The state.
fn show_problems(state: AppState)
{
    state.panel_tab.set(PanelTab::Problems);
    state.set_problems_open.set(true);
    let share = state.stage_share.get_untracked();
    do_if!(share > ERROR_STAGE_SHARE, state.set_stage_share.set(ERROR_STAGE_SHARE));
}

/// Tag the page with the active locale, for screen readers and fonts.
fn mark_lang()
{
    let page = document().document_element();
    do_if!(let Some(page) = page, {
        let _ = page.set_attribute("lang", &zakac_i18n::locale());
    });
}

/// Write the link of the latest edits to the URL when the header of the
/// website page asks, right before it reloads the page for a new language.
///
/// # Arguments
///
/// * `state` - The state.
fn follow_flush(state: AppState)
{
    let asks = leptos::ev::Custom::<web_sys::Event>::new(EXPLORER_FLUSH_EVENT);
    let _ = use_event_listener(use_window(), asks, move |_| state.flush_link());
}

/// Load a link that is pasted or edited in the address bar.
///
/// # Arguments
///
/// * `state` - The state.
fn follow_hash(state: AppState)
{
    let _ = use_event_listener(use_window(), leptos::ev::hashchange, move |_| {
        let linked = url::read_hash().and_then(|hash| permalink::decode(&hash));
        do_if!(let Some(link) = linked, state.open_link(link));
    });
}
