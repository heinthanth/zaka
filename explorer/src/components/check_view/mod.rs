use std::borrow::Cow;

use leptos::html;
use leptos::html::div;
use leptos::html::p;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::scss_file;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::compile::TreeSource;
use crate::components::icon::alert;
use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::components::tree_view::tree_view;
use crate::state::use_state;

/// Sheet of the check view.
static SHEET: ScopedSheet = ScopedSheet::new(
    "zaka-check-view",
    scss_file!("src/components/check_view/mod.sass"),
);

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
        let kinds = (
            p().class("view-note").child(t!("explorer.check.kinds_note")),
            tree_view(|| TreeSource::Ast, true),
        );
        let title = t!("explorer.check.skipped_title");
        let note = t!("explorer.check.skipped_note");
        tern!(checked(), kinds.into_any(), skipped(title, note).into_any())
    };
    let root = NodeRef::<html::Div>::new();
    let view = div()
        .class("check-view")
        .class(("check-view-skipped", move || !checked()))
        .node_ref(root)
        .child(content);
    scoped(&SHEET, root, view)
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
    div().class("skipped").attr("role", "status").child((
        span().class("skipped-icon").child(alert()),
        p().class("skipped-title").child(title),
        p().class("skipped-note").child(note),
    ))
}
