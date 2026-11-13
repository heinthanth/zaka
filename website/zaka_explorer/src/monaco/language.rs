//! The Zaka language in Monaco: hover, completion, going to a declaration,
//! finding uses, renaming and the outline. Each one runs the code of
//! `zakac lsp` on the text of the editor, in the page.

use std::cell::Cell;
use std::cell::RefCell;
use std::path::Path;

use bon::Builder;
use gen_lsp_types::CompletionItem;
use gen_lsp_types::CompletionItemKind;
use gen_lsp_types::Contents;
use gen_lsp_types::DocumentSymbol;
use gen_lsp_types::InlayHint;
use gen_lsp_types::Label;
use gen_lsp_types::Location;
use gen_lsp_types::Position;
use gen_lsp_types::Range;
use gen_lsp_types::SymbolKind;
use js_sys::Array;
use js_sys::Object;
use js_sys::Reflect;
use serde::Serialize;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use zakac_lexer::token::TokenKind;
use zakac_lsp::check::Buffers;
use zakac_lsp::check::FileCheck;
use zakac_lsp::check::check_file;
use zakac_lsp::check::file_uri;
use zakac_lsp::complete::completions;
use zakac_lsp::hints::call_hints;
use zakac_lsp::hover::hover;
use zakac_lsp::names::definition;
use zakac_lsp::names::nothing_to_rename;
use zakac_lsp::names::references;
use zakac_lsp::names::rename;
use zakac_lsp::names::rename_range;
use zakac_lsp::outline::outline;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use super::Pos;
use super::TextModel;
use super::to_js;

/// The id of the Zaka language in Monaco.
pub(super) const LANGUAGE: &str = "zaka";

/// The path the explorer gives its one file.
const FILE: &str = "/explorer.zaka";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["monaco", "languages"], js_name = register)]
    fn register(info: &JsValue);
    #[wasm_bindgen(js_namespace = ["monaco", "languages"], js_name = setLanguageConfiguration)]
    fn set_language_setup(language: &str, setup: &JsValue);
    #[wasm_bindgen(js_namespace = ["monaco", "languages"], js_name = registerHoverProvider)]
    fn add_hover(language: &str, provider: &Object);
    #[wasm_bindgen(js_namespace = ["monaco", "languages"], js_name = registerCompletionItemProvider)]
    fn add_completion(language: &str, provider: &Object);
    #[wasm_bindgen(js_namespace = ["monaco", "languages"], js_name = registerDefinitionProvider)]
    fn add_definition(language: &str, provider: &Object);
    #[wasm_bindgen(js_namespace = ["monaco", "languages"], js_name = registerReferenceProvider)]
    fn add_references(language: &str, provider: &Object);
    #[wasm_bindgen(js_namespace = ["monaco", "languages"], js_name = registerRenameProvider)]
    fn add_rename(language: &str, provider: &Object);
    #[wasm_bindgen(js_namespace = ["monaco", "languages"], js_name = registerDocumentSymbolProvider)]
    fn add_outline(language: &str, provider: &Object);
    #[wasm_bindgen(js_namespace = ["monaco", "languages"], js_name = registerInlayHintsProvider)]
    fn add_hints(language: &str, provider: &Object);
}

thread_local! {
    /// Whether the language is in Monaco; Monaco is global to the page.
    static ADDED: Cell<bool> = const { Cell::new(false) };
    /// The last text checked and its check, so the requests on one text share
    /// one check.
    static LAST: RefCell<Option<(String, FileCheck)>> = const { RefCell::new(None) };
}

/// Add the Zaka language to Monaco, with its comment rules and each
/// provider, once per page.
pub(super) fn add_language()
{
    ret_if!(ADDED.with(|added| added.replace(true)));
    let info = LanguageInfo::builder()
        .id(LANGUAGE)
        .extensions([".zaka", ".zk"])
        .aliases(["Zaka", "စံကား"])
        .build();
    register(&to_js(&info));
    set_language_setup(LANGUAGE, &to_js(&LanguageSetup::builder().build()));
    add_hover(LANGUAGE, &provider(&[("provideHover", at_pos(provide_hover))]));
    let complete = at_pos(provide_completions);
    add_completion(LANGUAGE, &provider(&[("provideCompletionItems", complete)]));
    let define = at_pos(provide_definition);
    add_definition(LANGUAGE, &provider(&[("provideDefinition", define)]));
    let uses = at_pos(provide_references);
    add_references(LANGUAGE, &provider(&[("provideReferences", uses)]));
    let edits =
        Closure::<dyn Fn(TextModel, Pos, String) -> JsValue>::new(provide_rename);
    let place = at_pos(resolve_rename);
    #[rustfmt::skip]
    add_rename(LANGUAGE, &provider(&[("provideRenameEdits", edits.into_js_value()),
        ("resolveRenameLocation", place)]));
    let nodes = Closure::<dyn Fn(TextModel) -> JsValue>::new(provide_outline);
    add_outline(
        LANGUAGE,
        &provider(&[("provideDocumentSymbols", nodes.into_js_value())]),
    );
    let hints = Closure::<dyn Fn(TextModel) -> JsValue>::new(provide_hints);
    add_hints(LANGUAGE, &provider(&[("provideInlayHints", hints.into_js_value())]));
}

/// The inlay hints of the text: one before each argument a call can change
/// or takes over, as the language server gives them.
///
/// # Arguments
///
/// * `model` - The model.
///
/// # Returns
///
/// The hint list, with the `dispose` Monaco calls when it is done with it.
fn provide_hints(model: TextModel) -> JsValue
{
    let found = with_check(&model, call_hints);
    let hint = |h: InlayHint| {
        let label = match h.label
        {
            Label::String(text) => text,
            Label::InlayHintLabelPartList(parts) =>
            {
                parts.into_iter().map(|p| p.value).collect()
            }
        };
        let at = HintPos::builder()
            .line_number(h.position.line + 1)
            .column(h.position.character + 1)
            .build();
        HintOut::builder()
            .position(at)
            .label(label)
            .kind(PARAMETER_HINT)
            .padding_right(true)
            .build()
    };
    let out = to_js(
        &HintList::builder().hints(found.into_iter().map(hint).collect()).build(),
    );
    let done = Closure::<dyn Fn()>::new(|| ()).into_js_value();
    _ = Reflect::set(&out, &JsValue::from_str("dispose"), &done);
    out
}

/// The kind Monaco gives a hint that names a parameter.
const PARAMETER_HINT: u8 = 2;

/// A list of inlay hints, as Monaco takes it.
#[derive(Debug, Clone, Serialize, Builder)]
struct HintList
{
    /// The hints.
    hints: Vec<HintOut>,
}

/// One inlay hint, as Monaco takes it.
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct HintOut
{
    /// Where it stands.
    position: HintPos,
    /// Its text.
    label: String,
    /// What it names: a parameter.
    kind: u8,
    /// Whether a space follows it.
    padding_right: bool,
}

/// A position of lines and columns, both from 1, as Monaco takes it.
#[derive(Debug, Clone, Copy, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct HintPos
{
    /// The line.
    line_number: u32,
    /// The column.
    column: u32,
}

/// A provider function of a model and a position, kept for the life of the
/// page.
///
/// # Arguments
///
/// * `f` - The function.
///
/// # Returns
///
/// The JavaScript function.
fn at_pos(f: fn(TextModel, Pos) -> JsValue) -> JsValue
{
    Closure::<dyn Fn(TextModel, Pos) -> JsValue>::new(f).into_js_value()
}

/// A provider object with its functions.
///
/// # Arguments
///
/// * `fns` - Each function with its name.
///
/// # Returns
///
/// The object.
fn provider(fns: &[(&str, JsValue)]) -> Object
{
    let object = Object::new();
    do_for!((name, f) in fns, _ = Reflect::set(&object, &JsValue::from_str(name), f));
    object
}

/// Run `f` on the check of the text of a model, checking the text once.
///
/// # Arguments
///
/// * `model` - The model.
/// * `f` - What to do with the check.
///
/// # Returns
///
/// What `f` gives.
fn with_check<R>(model: &TextModel, f: impl FnOnce(&FileCheck) -> R) -> R
{
    let text = model.value();
    LAST.with(|last| {
        let mut last = last.borrow_mut();
        let stale = last.as_ref().is_none_or(|(seen, _)| *seen != text);
        do_if!(stale, *last = Some((text.clone(), check(&text))));
        f(&last.as_ref().expect("the check is in place").1)
    })
}

/// Check the text of the explorer as one file.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The check.
fn check(text: &str) -> FileCheck
{
    let path = Path::new(FILE);
    let uri = file_uri(path).expect("the path of the explorer has an address");
    check_file(&uri, path, text, Buffers::default())
}

/// The LSP position of a Monaco position, whose lines and columns count from 1.
///
/// # Arguments
///
/// * `pos` - The Monaco position.
///
/// # Returns
///
/// The LSP position.
fn lsp_pos(pos: &Pos) -> Position
{
    Position { line: pos.line() - 1, character: pos.column() - 1 }
}

/// The hover of the expression at a position.
///
/// # Arguments
///
/// * `model` - The model.
/// * `pos` - The position.
///
/// # Returns
///
/// The hover, or `null`.
fn provide_hover(model: TextModel, pos: Pos) -> JsValue
{
    let found = with_check(&model, |file| hover(file, lsp_pos(&pos)));
    let Some(found) = found
    else
    {
        return JsValue::NULL;
    };
    let Contents::MarkupContent(content) = found.contents
    else
    {
        return JsValue::NULL;
    };
    let range = found.range.map(|r| EditorRange::of(&r));
    let text = Markdown::builder().value(content.value).build();
    to_js(&HoverOut::builder().maybe_range(range).contents(vec![text]).build())
}

/// The completions at a position, over the word written up to it.
///
/// # Arguments
///
/// * `model` - The model.
/// * `pos` - The position.
///
/// # Returns
///
/// The suggestions.
fn provide_completions(model: TextModel, pos: Pos) -> JsValue
{
    let at = lsp_pos(&pos);
    let found = with_check(&model, |file| {
        let range = EditorRange::of(&Range { start: word_start(file, at), end: at });
        let suggest = |c: CompletionItem| Suggestion::of(c, range);
        completions(Some(file), at).into_iter().map(suggest).collect()
    });
    to_js(&Suggestions::builder().suggestions(found).build())
}

/// Where the word written up to a position starts: the start of the name or
/// reserved word that ends there, else the position itself.
///
/// # Arguments
///
/// * `file` - The check.
/// * `at` - The position.
///
/// # Returns
///
/// The start.
fn word_start(file: &FileCheck, at: Position) -> Position
{
    let off = file.lines.offset(at);
    let words = file.lexed.tokens.iter();
    let words = words.filter(|t| matches!(t.kind, TokenKind::Ident | TokenKind::Kw(_)));
    let ranges = words.map(|t| file.root_range(&t.span));
    let found = ranges.into_iter().find(|r| r.lo < off && r.hi == off);
    found.map_or(at, |r| file.lines.position(r.lo))
}

/// The declaration of the name at a position, in this file.
///
/// # Arguments
///
/// * `model` - The model.
/// * `pos` - The position.
///
/// # Returns
///
/// The location, or `null`.
fn provide_definition(model: TextModel, pos: Pos) -> JsValue
{
    let found = with_check(&model, |file| {
        let decl = definition(file, lsp_pos(&pos))?;
        here(&model, file, &decl)
    });
    found.unwrap_or(JsValue::NULL)
}

/// Each place of the name at a position, in this file.
///
/// # Arguments
///
/// * `model` - The model.
/// * `pos` - The position.
///
/// # Returns
///
/// The locations.
fn provide_references(model: TextModel, pos: Pos) -> JsValue
{
    let places = with_check(&model, |file| {
        let all = references(std::iter::once(file), file, lsp_pos(&pos), true);
        all.iter().filter_map(|l| here(&model, file, l)).collect::<Array>()
    });
    places.into()
}

/// A location of this file as Monaco takes it; a place in another file, such
/// as a module of the stdlib, has no model here.
///
/// # Arguments
///
/// * `model` - The model.
/// * `file` - The check.
/// * `place` - The location.
///
/// # Returns
///
/// The location, or `None` for another file.
fn here(model: &TextModel, file: &FileCheck, place: &Location) -> Option<JsValue>
{
    ret_if!(place.uri != file.uri, None);
    let object = Object::new();
    _ = Reflect::set(&object, &"uri".into(), &model.uri());
    _ = Reflect::set(&object, &"range".into(), &to_js(&EditorRange::of(&place.range)));
    Some(object.into())
}

/// The name a rename at a position changes, or the reason there is none.
///
/// # Arguments
///
/// * `model` - The model.
/// * `pos` - The position.
///
/// # Returns
///
/// The range and text of the name, or the reason.
fn resolve_rename(model: TextModel, pos: Pos) -> JsValue
{
    let found = with_check(&model, |file| {
        let range = rename_range(file, lsp_pos(&pos))?;
        let (lo, hi) = (file.lines.offset(range.start), file.lines.offset(range.end));
        let text = file.text.get(lo as usize..hi as usize)?.to_string();
        Some(RenamePlace::builder().range(EditorRange::of(&range)).text(text).build())
    });
    let refused =
        || RenamePlace::builder().range(EditorRange::at(&pos)).text(String::new());
    let refused = || refused().reject_reason(nothing_to_rename()).build();
    to_js(&found.unwrap_or_else(refused))
}

/// The edits of a rename at a position.
///
/// # Arguments
///
/// * `model` - The model.
/// * `pos` - The position.
/// * `name` - The new name.
///
/// # Returns
///
/// The edits, or the reason the rename is refused.
fn provide_rename(model: TextModel, pos: Pos, name: String) -> JsValue
{
    let found: Result<Vec<gen_lsp_types::TextEdit>, String> =
        with_check(&model, |file| {
            let edit = rename(std::iter::once(file), file, lsp_pos(&pos), &name)?;
            let changes = edit.changes.unwrap_or_default();
            Ok(changes.get(&file.uri).cloned().unwrap_or_default())
        });
    let edits = Array::new();
    let reason = found.as_ref().err().cloned();
    for change in found.unwrap_or_default()
    {
        let text = EditOut::builder()
            .range(EditorRange::of(&change.range))
            .text(change.new_text);
        let one = Object::new();
        _ = Reflect::set(&one, &"resource".into(), &model.uri());
        _ = Reflect::set(&one, &"textEdit".into(), &to_js(&text.build()));
        edits.push(&one);
    }
    let out = Object::new();
    _ = Reflect::set(&out, &"edits".into(), &edits);
    do_if!(let Some(text) = reason, _ = Reflect::set(&out, &"rejectReason".into(), &text.into()));
    out.into()
}

/// The outline of the text.
///
/// # Arguments
///
/// * `model` - The model.
///
/// # Returns
///
/// The outline nodes.
fn provide_outline(model: TextModel) -> JsValue
{
    let nodes = with_check(&model, outline);
    let nodes: Vec<OutlineNode> = nodes.iter().map(OutlineNode::of).collect();
    to_js(&nodes)
}

/// The language as Monaco registers it.
#[derive(Serialize, Builder)]
struct LanguageInfo
{
    /// The id.
    id: &'static str,
    /// The file name endings.
    extensions: [&'static str; 2],
    /// The names shown for it.
    aliases: [&'static str; 2],
}

/// The comment and bracket rules of the language.
#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct LanguageSetup
{
    /// The line comment.
    #[builder(default = LineComment::builder().build())]
    comments: LineComment,
    /// The brackets.
    #[builder(default = [["(", ")"]])]
    brackets: [[&'static str; 2]; 1],
}

/// The line comment of the language.
#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct LineComment
{
    /// The marker.
    #[builder(default = "//")]
    line_comment: &'static str,
}

/// A range of lines and columns, both from 1, as Monaco takes it.
#[derive(Debug, Clone, Copy, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct EditorRange
{
    /// The line of the start.
    start_line_number: u32,
    /// The column of the start.
    start_column: u32,
    /// The line of the end.
    end_line_number: u32,
    /// The column of the end.
    end_column: u32,
}

impl EditorRange
{
    /// The Monaco range of an LSP range.
    ///
    /// # Arguments
    ///
    /// * `r` - The LSP range.
    ///
    /// # Returns
    ///
    /// The range.
    fn of(r: &Range) -> Self
    {
        #[rustfmt::skip]
        return EditorRange::builder().start_line_number(r.start.line + 1)
            .start_column(r.start.character + 1).end_line_number(r.end.line + 1)
            .end_column(r.end.character + 1).build();
    }

    /// The blank range at a Monaco position.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position.
    ///
    /// # Returns
    ///
    /// The range.
    fn at(pos: &Pos) -> Self
    {
        #[rustfmt::skip]
        return EditorRange::builder().start_line_number(pos.line()).start_column(pos.column())
            .end_line_number(pos.line()).end_column(pos.column()).build();
    }
}

/// A hover as Monaco takes it.
#[derive(Serialize, Builder)]
struct HoverOut
{
    /// The range it is about.
    #[serde(skip_serializing_if = "Option::is_none")]
    range: Option<EditorRange>,
    /// The text.
    contents: Vec<Markdown>,
}

/// A Markdown text.
#[derive(Serialize, Builder)]
struct Markdown
{
    /// The text.
    value: String,
}

/// The suggestions of a completion.
#[derive(Serialize, Builder)]
struct Suggestions
{
    /// The suggestions.
    suggestions: Vec<Suggestion>,
}

/// One suggestion of a completion.
#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct Suggestion
{
    /// The text shown.
    label: String,
    /// What it is, as Monaco numbers kinds.
    kind: u32,
    /// The text written in place of the range.
    insert_text: String,
    /// The word the suggestion replaces.
    range: EditorRange,
    /// A note beside it.
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

impl Suggestion
{
    /// The suggestion of an LSP completion item.
    ///
    /// # Arguments
    ///
    /// * `c` - The item.
    /// * `range` - The word it replaces.
    ///
    /// # Returns
    ///
    /// The suggestion.
    fn of(c: CompletionItem, range: EditorRange) -> Self
    {
        #[rustfmt::skip]
        return Suggestion::builder().insert_text(c.label.clone()).label(c.label)
            .kind(suggestion_kind(c.kind)).range(range).maybe_detail(c.detail).build();
    }
}

/// Monaco's number for an LSP completion kind.
///
/// # Arguments
///
/// * `kind` - The LSP kind.
///
/// # Returns
///
/// The number.
fn suggestion_kind(kind: Option<CompletionItemKind>) -> u32
{
    match kind
    {
        Some(CompletionItemKind::Function) => 1,
        Some(CompletionItemKind::Field) => 3,
        Some(CompletionItemKind::Variable) => 4,
        Some(CompletionItemKind::Struct) => 6,
        Some(CompletionItemKind::Constant) => 14,
        Some(CompletionItemKind::Enum) => 15,
        Some(CompletionItemKind::EnumMember) => 16,
        Some(CompletionItemKind::Keyword) => 17,
        Some(CompletionItemKind::TypeParameter) => 24,
        _ => 18,
    }
}

/// The edit of a rename in one place.
#[derive(Serialize, Builder)]
struct EditOut
{
    /// The range.
    range: EditorRange,
    /// The new text.
    text: String,
}

/// The name a rename changes, or the reason there is none.
#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct RenamePlace
{
    /// The range of the name.
    range: EditorRange,
    /// The name.
    text: String,
    /// The reason the rename is refused.
    #[serde(skip_serializing_if = "Option::is_none")]
    reject_reason: Option<String>,
}

/// One node of the outline as Monaco takes it.
#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct OutlineNode
{
    /// The name.
    name: String,
    /// A note beside it.
    detail: String,
    /// What it is, as Monaco numbers kinds.
    kind: u32,
    /// Tags; none.
    tags: Vec<u32>,
    /// The whole declaration.
    range: EditorRange,
    /// The name in it.
    selection_range: EditorRange,
    /// The nodes inside it.
    children: Vec<OutlineNode>,
}

impl OutlineNode
{
    /// The node of an LSP outline node.
    ///
    /// # Arguments
    ///
    /// * `node` - The LSP node.
    ///
    /// # Returns
    ///
    /// The node.
    fn of(node: &DocumentSymbol) -> Self
    {
        let children = node.children.iter().flatten().map(OutlineNode::of).collect();
        #[rustfmt::skip]
        return OutlineNode::builder().name(node.name.clone()).detail(String::new())
            .kind(outline_kind(&node.kind)).tags(Vec::new()).range(EditorRange::of(&node.range))
            .selection_range(EditorRange::of(&node.selection_range)).children(children).build();
    }
}

/// Monaco's number for an LSP outline kind: the LSP numbers count from 1,
/// Monaco's from 0.
///
/// # Arguments
///
/// * `kind` - The LSP kind.
///
/// # Returns
///
/// The number.
fn outline_kind(kind: &SymbolKind) -> u32
{
    match kind
    {
        SymbolKind::Enum => 9,
        SymbolKind::Function => 11,
        SymbolKind::Constant => 13,
        SymbolKind::Field => 7,
        SymbolKind::EnumMember => 21,
        SymbolKind::Struct => 22,
        SymbolKind::TypeParameter => 25,
        _ => 12,
    }
}
