use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_scoped_style::scss_file;
use zaka_web_ui_common::controls::scroll_bar::Scroller;
use zaka_web_ui_common::controls::scroll_bar::scroll_bar;
use zaka_web_ui_common::sheet;

/// The rule that hides the scroll bar of the browser on the page.
static SHEET: ComponentStyle = ComponentStyle::global_css(
    "site-scroll-bar",
    scss_file!("src/components/scroll_bar/mod.sass"),
);

/// The sheet of the scroll bar of the page. It mounts outside the island, as
/// a global sheet renders on the server alone.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn scroll_bar_sheet() -> impl IntoView { sheet(&SHEET) }

/// The scroll bar of the page, drawn over it, so it takes no room and each
/// page keeps its width. An island: it follows the page in the browser.
///
/// # Returns
///
/// The view.
#[island]
pub fn PageScrollBar() -> impl IntoView { scroll_bar(Scroller::Page) }
