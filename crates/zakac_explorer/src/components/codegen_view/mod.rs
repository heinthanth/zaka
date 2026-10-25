use leptos::html::div;
use leptos::html::h3;
use leptos::html::table;
use leptos::html::tbody;
use leptos::html::tr;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use zaka_web_ui_common::sheet;
use zakac_codegen::listing::quote;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::compile::asm::AsmFunc;
use crate::components::asm_table::asm_table;
use crate::components::asm_table::slot_table;
use crate::components::check_view::skipped;
use crate::components::table::ClassName as Table;
use crate::components::table::Column;
use crate::components::table::cell;
use crate::components::table::table_box;
use crate::components::table::table_head;
use crate::state::AppState;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/codegen_view/mod.sass");

/// Sheet of the code generation view.
static SHEET: ComponentStyle =
    ComponentStyle::global_css("zaka-codegen-view", STYLE_SHEET);

/// Columns of the texts table.
const TEXT_COLUMNS: [Column; 2] =
    [("explorer.col.slot", Table::CELL_NUM, ""), ("explorer.col.value", "", "")];

/// The generated ops: the global slots, the pooled texts and each function
/// with its locals and ops; or a note when code generation did not run.
///
/// # Returns
///
/// The view.
pub fn codegen_view() -> impl IntoView
{
    let state = use_state();
    let emitted = Memo::new(move |_| state.compiled.with(|c| c.emitted));
    let content = move || {
        let title = t!("explorer.codegen.skipped_title");
        let note = t!("explorer.codegen.skipped_note");
        tern!(
            emitted.get(),
            ops_view(state).into_any(),
            skipped(title, note).into_any()
        )
    };
    let view = div()
        .class((ClassName::CODEGEN_VIEW_SKIPPED, move || !emitted.get()))
        .child(content);
    (sheet(&SHEET), view)
}

/// The globals, the texts and the functions.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn ops_view(state: AppState) -> impl IntoView
{
    let globals = move || {
        let rows = state.compiled.with(|c| c.globals.clone());
        (!rows.is_empty()).then(|| {
            let title =
                h3().class(Table::TABLE_TITLE).child(t!("explorer.codegen.globals"));
            (title, table_box(slot_table('g', rows)))
        })
    };
    let funcs = move || {
        let funcs = state.compiled.with(|c| c.asm.clone());
        funcs.into_iter().map(|f| func_view(state, f)).collect_view()
    };
    (globals, texts_view(state), funcs)
}

/// The pooled print texts, when there are some.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn texts_view(state: AppState) -> impl IntoView
{
    move || {
        let texts = state.compiled.with(|c| c.texts.clone());
        (!texts.is_empty()).then(|| {
            let row = |(i, text): (usize, String)| {
                tr().child((
                    cell(Table::CELL_NUM, format!("t{i}")),
                    cell(Table::CELL_LEXEME, quote(&text)),
                ))
            };
            let rows = texts.into_iter().enumerate().map(row).collect_view();
            let title =
                h3().class(Table::TABLE_TITLE).child(t!("explorer.codegen.texts"));
            let view = table()
                .class(Table::PHASE_TABLE)
                .child((table_head(&TEXT_COLUMNS), tbody().child(rows)));
            (title, table_box(view))
        })
    }
}

/// One function: its head, its locals and its ops.
///
/// # Arguments
///
/// * `state` - The state.
/// * `func` - The function.
///
/// # Returns
///
/// The view.
fn func_view(state: AppState, func: AsmFunc) -> impl IntoView
{
    let head = h3().class(ClassName::ASM_HEAD).child(func.head());
    let locals = func.locals.clone();
    let locals = (!locals.is_empty()).then(|| {
        let title = h3().class(Table::TABLE_TITLE).child(t!("explorer.codegen.locals"));
        (title, table_box(slot_table('l', locals)))
    });
    let ops = asm_table(state, func, Signal::stored(None), None);
    div().class(ClassName::ASM_FUNC).child((head, locals, table_box(ops)))
}
