use std::borrow::Cow;

use leptos::html::div;
use leptos::html::p;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::icon::alert;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::compile::TreeSource;
use crate::components::table::ClassName as Table;
use crate::components::tree_view::tree_view;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/check_view/mod.sass");

/// Sheet of the check view.
static SHEET: ComponentStyle =
    ComponentStyle::global_css("zaka-check-view", STYLE_SHEET);

/// The check output: the tree with the kind of each expression, or a note
/// when the check did not run.
///
/// # Returns
///
/// The view.
pub fn check_view() -> impl IntoView
{
    let state = use_state();
    let checked = move || state.compiled.with(|c| c.checked);
    let content = move || {
        let kinds = (tree_view(|| TreeSource::Ast, true),);
        let title = t!("explorer.check.skipped_title");
        let note = t!("explorer.check.skipped_note");
        tern!(checked(), kinds.into_any(), skipped(title, note).into_any())
    };
    let view =
        div().class((ClassName::CHECK_VIEW_SKIPPED, move || !checked())).child(content);
    (sheet(&SHEET), view)
}

/// The note when a phase did not run: an icon, the reason and the fix, in
/// the middle of the pane. The sheet of the view that holds it includes the
/// `skipped-card` mixin.
///
/// # Arguments
///
/// * `title` - The reason.
/// * `note` - The fix.
///
/// # Returns
///
/// The view.
pub fn skipped(title: Cow<'static, str>, note: Cow<'static, str>) -> impl IntoView
{
    div().class(Table::SKIPPED).attr("role", "status").child((
        span().class(Table::SKIPPED_ICON).child(alert()),
        p().class(Table::SKIPPED_TITLE).child(title),
        p().class(Table::SKIPPED_NOTE).child(note),
    ))
}
