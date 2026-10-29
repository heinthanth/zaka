use leptos::html::div;
use leptos::html::h3;
use leptos::html::li;
use leptos::html::ul;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::components::check_view::skipped;
use crate::components::table::ClassName as Table;
use crate::state::AppState;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/borrow_check_view/mod.sass");

/// Sheet of the borrow check view.
static SHEET: ComponentStyle =
    ComponentStyle::global_css("zaka-borrow-check-view", STYLE_SHEET);

/// The ownership check output: one line for each function with what it
/// frees, resizes or moves through; or a note when the check did not run.
///
/// # Returns
///
/// The view.
pub fn borrow_check_view() -> impl IntoView
{
    let state = use_state();
    let ran = Memo::new(move |_| state.compiled.with(|c| c.borrowchecked));
    let content = move || {
        let title = t!("explorer.borrow_check.skipped_title");
        let note = t!("explorer.borrow_check.skipped_note");
        tern!(
            ran.get(),
            digests_view(state).into_any(),
            skipped(title, note).into_any()
        )
    };
    let view = div()
        .class((ClassName::BORROW_CHECK_VIEW_SKIPPED, move || !ran.get()))
        .child(content);
    (sheet(&SHEET), view)
}

/// The digests under their title, one line each.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn digests_view(state: AppState) -> impl IntoView
{
    let title =
        h3().class(Table::TABLE_TITLE).child(t!("explorer.borrow_check.digests"));
    let lines = move || {
        let digests = state.compiled.with(|c| c.digests.clone());
        let line = |d: String| li().class(ClassName::DIGEST).child(d);
        digests.into_iter().map(line).collect_view()
    };
    (title, ul().class(ClassName::DIGEST_LIST).child(lines))
}
