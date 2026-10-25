use leptos::ev;
use leptos::html::div;
use leptos::html::h3;
use leptos::html::span;
use leptos::html::table;
use leptos::html::tbody;
use leptos::html::tr;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use web_sys::ScrollIntoViewOptions;
use web_sys::ScrollLogicalPosition;
use zaka_web_ui_common::sheet;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

use crate::compile::CommentRow;
use crate::compile::PartKind;
use crate::compile::StrPiece;
use crate::compile::StrRow;
use crate::compile::TokenRow;
use crate::components::table::ClassName as Table;
use crate::components::table::Column;
use crate::components::table::cell;
use crate::components::table::linked_row;
use crate::components::table::span_cell;
use crate::components::table::table_head;
use crate::state::AppState;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/tokens_view/mod.sass");

/// Sheet of the tokens view.
static SHEET: ComponentStyle =
    ComponentStyle::global_css("zaka-tokens-view", STYLE_SHEET);

/// Columns of the tokens table.
const TOKEN_COLUMNS: [Column; 7] = [
    ("explorer.col.index", Table::CELL_NUM, ""),
    ("explorer.col.kind", "", ""),
    ("explorer.col.detail", "", ""),
    ("explorer.col.text", "", ""),
    ("explorer.col.particle", "", ""),
    ("explorer.col.joined", "", "explorer.col.joined_note"),
    ("explorer.col.span", "", ""),
];

/// Columns of the comments table.
const COMMENT_COLUMNS: [Column; 3] = [
    ("explorer.col.kind", "", ""),
    ("explorer.col.text", "", ""),
    ("explorer.col.line", "", ""),
];

/// Columns of the strings table.
const STRING_COLUMNS: [Column; 3] = [
    ("explorer.col.index", Table::CELL_NUM, ""),
    ("explorer.col.value", "", "explorer.col.value_note"),
    ("explorer.col.source", "", "explorer.col.source_note"),
];

/// Tables of the lexer output: tokens, comments and strings.
///
/// # Returns
///
/// The view.
pub fn tokens_view() -> impl IntoView
{
    let state = use_state();
    follow_focus(state);
    let rows = move || {
        state.compiled.with(|c| {
            let rows = c.tokens.iter().enumerate();
            rows.map(|(idx, row)| token_row(state, idx, row.clone())).collect_view()
        })
    };
    let tokens = table()
        .class(Table::PHASE_TABLE)
        .child((table_head(&TOKEN_COLUMNS), tbody().child(rows)));
    let view = div().class(ClassName::TOKENS_VIEW).child((
        tokens,
        comments_table(state),
        strings_table(state),
    ));
    (sheet(&SHEET), view)
}

/// Bring the row of the token the editor points at in view.
///
/// # Arguments
///
/// * `state` - The state.
fn follow_focus(state: AppState)
{
    Effect::new(move |_| {
        let Some(idx) = state.focus_token()
        else
        {
            return;
        };
        let Some(row) = document().get_element_by_id(&format!("tok-{idx}"))
        else
        {
            return;
        };
        let options = ScrollIntoViewOptions::new();
        options.set_block(ScrollLogicalPosition::Nearest);
        row.scroll_into_view_with_scroll_into_view_options(&options);
    });
}

/// One row of the tokens table; it also lights up when the editor points at
/// the token.
///
/// # Arguments
///
/// * `state` - The state.
/// * `idx` - The token index.
/// * `row` - The token.
///
/// # Returns
///
/// The view.
fn token_row(state: AppState, idx: usize, row: TokenRow) -> impl IntoView
{
    let range = row.range;
    let chip =
        span().class(format!("{} {}", ClassName::CHIP, row.class)).child(row.kind);
    let cells = (
        cell(Table::CELL_NUM, idx),
        cell(Table::CELL_KIND, chip),
        cell(Table::CELL_DETAIL, row.detail),
        cell(Table::CELL_LEXEME, row.lexeme),
        cell(Table::CELL_DETAIL, row.particle),
        cell(Table::CELL_NUM, tern!(row.joined, "✓", "")),
        span_cell(row.line, row.col, range),
    );
    tr().id(format!("tok-{idx}"))
        .class((Table::ROW_ACTIVE, move || state.focus_token() == Some(idx)))
        .on(ev::mouseenter, move |_| state.point(Some(range)))
        .on(ev::mouseleave, move |_| state.point(None))
        .on(ev::click, move |_| state.select(range))
        .child(cells)
}

/// Table of the comments, when there are some.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn comments_table(state: AppState) -> impl IntoView
{
    move || {
        state.compiled.with(|c| {
            let some = !c.comments.is_empty();
            let rows = c.comments.iter().map(|row| comment_row(state, row.clone()));
            let listing = tbody().child(rows.collect_view());
            let view = table()
                .class(Table::PHASE_TABLE)
                .child((table_head(&COMMENT_COLUMNS), listing));
            some.then(|| {
                (
                    h3().class(Table::TABLE_TITLE)
                        .child(t!("explorer.tokens.comments")),
                    view,
                )
            })
        })
    }
}

/// One row of the comments table.
///
/// # Arguments
///
/// * `state` - The state.
/// * `row` - The comment.
///
/// # Returns
///
/// The view.
fn comment_row(state: AppState, row: CommentRow) -> impl IntoView
{
    let cells = (
        cell(Table::CELL_KIND, row.kind),
        cell(Table::CELL_LEXEME, row.text),
        cell(Table::CELL_NUM, row.line),
    );
    linked_row(state, row.range, cells)
}

/// Table of the decoded string literals, when there are some.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn strings_table(state: AppState) -> impl IntoView
{
    move || {
        state.compiled.with(|c| {
            let some = !c.strings.is_empty();
            let rows = c.strings.iter().map(|row| string_row(state, row.clone()));
            let listing = tbody().child(rows.collect_view());
            let view = table()
                .class(Table::PHASE_TABLE)
                .child((table_head(&STRING_COLUMNS), listing));
            some.then(|| {
                (
                    h3().class(Table::TABLE_TITLE).child(t!("explorer.tokens.strings")),
                    view,
                )
            })
        })
    }
}

/// One row of the strings table; hover points at the literal in the editor
/// and a click selects it.
///
/// # Arguments
///
/// * `state` - The state.
/// * `row` - The string.
///
/// # Returns
///
/// The view.
fn string_row(state: AppState, row: StrRow) -> impl IntoView
{
    let pieces = row.value.into_iter().map(piece_view).collect_view();
    let cells = (
        cell(Table::CELL_NUM, row.idx),
        cell(Table::CELL_LEXEME, pieces),
        cell(&format!("{} {}", Table::CELL_DETAIL, Table::CELL_LEXEME), row.source),
    );
    linked_row(state, row.range, cells)
}

/// One piece of a string value: text as it is, an escape or a hole as a chip
/// named after its kind.
///
/// # Arguments
///
/// * `piece` - The piece.
///
/// # Returns
///
/// The view.
fn piece_view(piece: StrPiece) -> impl IntoView
{
    let kind: &'static str = piece.kind.into();
    let chip = piece.kind != PartKind::Text;
    let class = match piece.kind
    {
        PartKind::Text => String::new(),
        PartKind::Escape => format!("{} {}", ClassName::PART, ClassName::PART_ESCAPE),
        PartKind::Hole => format!("{} {}", ClassName::PART, ClassName::PART_HOLE),
    };
    span().class(class).title(tern!(chip, kind, "")).child(piece.text)
}
