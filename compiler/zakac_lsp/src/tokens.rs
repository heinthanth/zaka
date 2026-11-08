use gen_lsp_types::SemanticToken;
use zakac_lexer::role::TokenRole;
use zakac_lexer::role::token_role;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::tern;
use zakac_span::text_range::Range;

use crate::check::FileCheck;

/// The token kinds the server reports, in legend order. These are the
/// standard LSP names, so each editor theme colors them with no extra map.
pub const TOKEN_KINDS: [&str; 10] = [
    "keyword",
    "function",
    "operator",
    "modifier",
    "type",
    "enumMember",
    "variable",
    "number",
    "string",
    "comment",
];

/// The legend index of a role. Control words, verbs, operators, particles,
/// kind nouns and truth values take the standard kinds `keyword`,
/// `function`, `operator`, `modifier`, `type` and `enumMember`; separators
/// and brackets remain plain.
///
/// # Arguments
///
/// * `role` - The role.
///
/// # Returns
///
/// The index in [`TOKEN_KINDS`], or `None` for a token left plain.
pub fn legend_index(role: TokenRole) -> Option<u32>
{
    match role
    {
        TokenRole::Control => Some(0),
        TokenRole::Verb => Some(1),
        TokenRole::Op => Some(2),
        TokenRole::Particle => Some(3),
        TokenRole::Kind => Some(4),
        TokenRole::Truth => Some(5),
        TokenRole::Ident | TokenRole::Raw => Some(6),
        TokenRole::Number => Some(7),
        TokenRole::Char | TokenRole::Str => Some(8),
        TokenRole::Comment => Some(9),
        TokenRole::Delim | TokenRole::Plain => None,
    }
}

/// The colored tokens of the open file, encoded as LSP wants them: each one
/// on a single line, relative to the one before it.
///
/// # Arguments
///
/// * `file` - The check of the open file.
///
/// # Returns
///
/// The tokens.
pub fn semantic_tokens(file: &FileCheck) -> Vec<SemanticToken>
{
    let lexed = &file.lexed;
    let live = lexed.tokens.iter().filter(|t| t.kind != TokenKind::Eof);
    let tokens = live.map(|t| (t.span, token_role(&t.kind)));
    let comments = lexed.comments.iter().map(|c| (c.span, TokenRole::Comment));
    let mut marks: Vec<(Range, u32)> = tokens
        .chain(comments)
        .filter_map(|(span, role)| Some((file.root_range(&span), legend_index(role)?)))
        .collect();
    marks.sort_by_key(|(range, _)| range.lo);
    let pieces = marks.iter().flat_map(|&(range, kind)| {
        line_pieces(file, range).into_iter().map(move |piece| (piece, kind))
    });
    encode(file, pieces)
}

/// Cut a range at the line breaks it crosses, as LSP tokens cannot span
/// lines.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `range` - The range.
///
/// # Returns
///
/// The pieces that hold text, each on one line.
fn line_pieces(file: &FileCheck, range: Range) -> Vec<Range>
{
    let starts = file.lines.starts();
    let first = starts.partition_point(|&start| start <= range.lo) - 1;
    let last = starts.partition_point(|&start| start < range.hi).max(first + 1);
    (first..last)
        .map(|line| {
            let end = starts.get(line + 1).map_or(range.hi, |&next| next - 1);
            Range::new(range.lo.max(starts[line]), range.hi.min(end))
        })
        .filter(|piece| !piece.is_blank())
        .collect()
}

/// Encode one-line pieces in order as LSP tokens, each relative to the one
/// before it.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `pieces` - The pieces and their legend index, in order.
///
/// # Returns
///
/// The tokens.
fn encode(
    file: &FileCheck, pieces: impl Iterator<Item = (Range, u32)>,
) -> Vec<SemanticToken>
{
    let mut at = (0, 0);
    let mut out = Vec::new();
    for (piece, kind) in pieces
    {
        let start = file.lines.position(piece.lo);
        let end = file.lines.position(piece.hi);
        let delta_line = start.line - at.0;
        let delta_start = start.character - tern!(delta_line == 0, at.1, 0);
        out.push(SemanticToken {
            delta_line,
            delta_start,
            length: end.character - start.character,
            token_type: kind,
            token_modifiers_bitset: 0,
        });
        at = (start.line, start.character);
    }
    out
}
