use std::collections::HashMap;

use gen_lsp_types::Location;
use gen_lsp_types::Position;
use gen_lsp_types::TextEdit;
use gen_lsp_types::Uri;
use gen_lsp_types::WorkspaceEdit;
use zakac_i18n::t;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

use crate::check::FileCheck;

/// The use or declaration of a name under the cursor, with the declaration
/// it resolves to.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `pos` - The cursor.
///
/// # Returns
///
/// The name under the cursor and its declaration, or `None` when the cursor
/// is on no resolved name.
fn under(file: &FileCheck, pos: Position) -> Option<&(Location, Location)>
{
    let at = (pos.line, pos.character);
    let holds = |l: &Location| {
        let (lo, hi) = (l.range.start, l.range.end);
        l.uri == file.uri
            && (lo.line, lo.character) <= at
            && at <= (hi.line, hi.character)
    };
    file.links.iter().find(|(used, _)| holds(used))
}

/// The declaration of the name under the cursor.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `pos` - The cursor.
///
/// # Returns
///
/// The declared name, or `None` when the cursor is on no resolved name.
pub fn definition(file: &FileCheck, pos: Position) -> Option<Location>
{
    under(file, pos).map(|(_, decl)| decl.clone())
}

/// Each place of the name under the cursor: its uses in the checks of all
/// open files, each once, and its declaration when asked for.
///
/// # Arguments
///
/// * `checks` - The checks of the open files.
/// * `file` - The check of the file the cursor is in.
/// * `pos` - The cursor.
/// * `with_decl` - Whether the declaration is one of the places.
///
/// # Returns
///
/// The places, sorted; none when the cursor is on no resolved name.
pub fn references<'a>(
    checks: impl Iterator<Item = &'a FileCheck>, file: &FileCheck, pos: Position,
    with_decl: bool,
) -> Vec<Location>
{
    let Some((_, decl)) = under(file, pos)
    else
    {
        return Vec::new();
    };
    let links = checks.flat_map(|c| c.links.iter());
    let uses = links.filter(|(used, to)| to == decl && (with_decl || used != decl));
    let mut places: Vec<Location> = uses.map(|(used, _)| used.clone()).collect();
    places.sort_by_key(|l| {
        (l.uri.to_string(), l.range.start.line, l.range.start.character)
    });
    places.dedup();
    places
}

/// The range of the name a rename would change, when the cursor is on one.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `pos` - The cursor.
///
/// # Returns
///
/// The range of the name, or `None` when the cursor is on no resolved name.
pub fn rename_range(file: &FileCheck, pos: Position) -> Option<gen_lsp_types::Range>
{
    under(file, pos).map(|(used, _)| used.range)
}

/// Why a rename has nothing to change: the cursor is on no declared name.
///
/// # Returns
///
/// The reason, in the language of the messages.
pub fn nothing_to_rename() -> String { t!("lsp.no_name").to_string() }

/// The edits that rename the name under the cursor at its declaration and at
/// each use the checks of the open files found.
///
/// # Arguments
///
/// * `checks` - The checks of the open files.
/// * `file` - The check of the file the cursor is in.
/// * `pos` - The cursor.
/// * `name` - The new name.
///
/// # Returns
///
/// The edits, or the reason the rename is refused: the new name is not one
/// name, or the cursor is on no resolved name.
pub fn rename<'a>(
    checks: impl Iterator<Item = &'a FileCheck>, file: &FileCheck, pos: Position,
    name: &str,
) -> Result<WorkspaceEdit, String>
{
    ret_if!(!is_one_name(name), Err(t!("lsp.bad_name", name = name).to_string()));
    let places = references(checks, file, pos, true);
    ret_if!(places.is_empty(), Err(t!("lsp.no_name").to_string()));
    let mut changes: HashMap<Uri, Vec<TextEdit>> = HashMap::new();
    for place in places
    {
        let edit = TextEdit { range: place.range, new_text: name.to_string() };
        changes.entry(place.uri).or_default().push(edit);
    }
    Ok(WorkspaceEdit { changes: Some(changes), ..Default::default() })
}

/// Whether a text lexes as one name and nothing else, so a rename cannot
/// turn a name into a reserved word or into several words.
///
/// # Arguments
///
/// * `name` - The text.
///
/// # Returns
///
/// `true` for one name.
fn is_one_name(name: &str) -> bool
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("rename".into()), name.to_string());
    let lexed = lex(&file, &LexerOptions::default());
    let kinds: Vec<TokenKind> = lexed.tokens.iter().map(|t| t.kind).collect();
    lexed.diagnostics.is_empty() && kinds == [TokenKind::Ident, TokenKind::Eof]
}
