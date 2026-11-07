//! Move the statements that run at the top level of a program into a start
//! block, for tools and tests that hold programs written before the start
//! block was required.

use std::collections::BTreeSet;

use zakac_ast::stmt::DeclStmt;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

use crate::parser::parse;

/// The header of the start block that [`wrap_in_start`] writes.
const HEADER: &str = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n";

/// The closer of the start block that [`wrap_in_start`] writes.
const CLOSER: &str = "ပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";

/// A program put into a start block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrapped
{
    /// The new text.
    pub text: String,
    /// For each line of the new text, the line of the old text it comes from,
    /// counted from 0; `None` for the lines of the start block's header and
    /// closer.
    pub lines: Vec<Option<usize>>,
}

/// Put a program into a start block, as tests write it: a variable or
/// constant remains at the top level when it comes before the first statement
/// that runs or a function uses it, and the rest moves into a start block at
/// the end.
///
/// # Arguments
///
/// * `text` - The program.
///
/// # Returns
///
/// The new program, or `text` itself when it has a start block or does not
/// parse.
pub fn in_start_block(text: &str) -> String { in_start_block_lines(text).text }

/// [`in_start_block`], with the line each new line comes from.
///
/// # Arguments
///
/// * `text` - The program.
///
/// # Returns
///
/// The new program and its lines; `text` itself, line for line, when it has a
/// start block or does not parse.
pub fn in_start_block_lines(text: &str) -> Wrapped
{
    let wrapped = wrap_in_start(text, |_, seen, used| !seen || used);
    wrapped.unwrap_or_else(|| {
        let lines = (0..text.split('\n').count()).map(Some).collect();
        Wrapped { text: text.to_string(), lines }
    })
}

/// Move the statements that run at the top level of a program into a start
/// block at its end, each with the comments before it. Functions, structs,
/// imports and exported declarations remain in their places, and `keep`
/// tells, for each other variable or constant, whether it remains at the top
/// level.
/// A program with nothing to move gets a blank start block.
///
/// # Arguments
///
/// * `text` - The program.
/// * `keep` - Whether a declaration remains, given whether a statement that
///   runs comes before it and whether a function uses its name.
///
/// # Returns
///
/// The new program, or `None` when it has a start block or does not parse.
pub fn wrap_in_start(
    text: &str, keep: impl Fn(&DeclStmt, bool, bool) -> bool,
) -> Option<Wrapped>
{
    let mut sm = SourceMap::new();
    let file =
        sm.new_source_file(FileName::Custom("wrap.zaka".into()), text.to_string());
    let lexed = lex(&file, &LexerOptions::default());
    let parsed = parse(&file, &lexed);
    ret_if!(!lexed.diagnostics.is_empty() || !parsed.diagnostics.is_empty(), None);
    let stmts = &parsed.program.stmts;
    ret_if!(stmts.iter().any(|s| matches!(s, Stmt::Fn(f) if f.start)), None);
    let base = file.start_pos.as_u32() as usize;
    let range = |lo: u32, hi: u32| (lo as usize - base, hi as usize - base);
    let spans: Vec<(usize, usize)> = stmts
        .iter()
        .map(|s| range(s.span().lo.as_u32(), s.span().hi.as_u32()))
        .collect();
    let fns: Vec<(usize, usize)> = (0..stmts.len())
        .filter(|&i| matches!(stmts[i], Stmt::Fn(_)))
        .map(|i| spans[i])
        .collect();
    let in_fn = |(lo, hi): (usize, usize)| fns.iter().any(|f| f.0 <= lo && hi <= f.1);
    let names: BTreeSet<&str> = lexed
        .tokens
        .iter()
        .filter(|t| matches!(t.kind, TokenKind::Ident | TokenKind::RawIdent))
        .map(|t| range(t.span.lo.as_u32(), t.span.hi.as_u32()))
        .filter(|&r| in_fn(r))
        .map(|(lo, hi)| file.src[lo..hi].trim_matches('`'))
        .collect();
    let mut seen = false;
    let moves: Vec<bool> = stmts
        .iter()
        .map(|s| {
            let moved = moves(s, seen, &names, &keep);
            seen |= runs(s);
            moved
        })
        .collect();
    Some(write_out(&file.src, &spans, &moves))
}

/// Whether a top-level statement moves into the start block.
///
/// # Arguments
///
/// * `stmt` - The statement.
/// * `seen` - Whether a statement that runs comes before it.
/// * `names` - The names the functions use.
/// * `keep` - Whether a declaration remains.
///
/// # Returns
///
/// `true` for a statement that runs, and for a declaration `keep` lets go.
fn moves(
    stmt: &Stmt, seen: bool, names: &BTreeSet<&str>,
    keep: &impl Fn(&DeclStmt, bool, bool) -> bool,
) -> bool
{
    match stmt
    {
        Stmt::Fn(_)
        | Stmt::Struct(_)
        | Stmt::Enum(_)
        | Stmt::Alias(_)
        | Stmt::Import(_) => false,
        Stmt::Decl(d) =>
        {
            !d.exported && !keep(d, seen, names.contains(d.name.name.as_str()))
        }
        _ => true,
    }
}

/// Whether a top-level statement runs where it stands.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// `false` for a function, a struct, an enumeration, an import, a variable or
/// a constant.
fn runs(stmt: &Stmt) -> bool
{
    !matches!(
        stmt,
        Stmt::Fn(_)
            | Stmt::Struct(_)
            | Stmt::Enum(_)
            | Stmt::Alias(_)
            | Stmt::Import(_)
            | Stmt::Decl(_)
    )
}

/// The new text, built from slices of the old one and lines of its own.
#[derive(Default)]
struct Out
{
    /// The text so far.
    text: String,
    /// The old line of each new line started so far.
    lines: Vec<Option<usize>>,
}

impl Out
{
    /// Add a slice of the old text, or a text of its own when `from` is
    /// `None`.
    ///
    /// # Arguments
    ///
    /// * `piece` - The text to add.
    /// * `from` - Where it starts in the old text, and the old text.
    fn push(&mut self, piece: &str, from: Option<(usize, &str)>)
    {
        for (i, c) in piece.char_indices()
        {
            let starts_line = self.text.is_empty() || self.text.ends_with('\n');
            let line = |(at, src): (usize, &str)| src[..at + i].matches('\n').count();
            do_if!(starts_line, self.lines.push(from.map(line)));
            self.text.push(c);
        }
    }
}

/// Write the program again: the statements that remain, in their order, then
/// the start block with the ones that move.
///
/// # Arguments
///
/// * `src` - The text.
/// * `spans` - The octet range of each top-level statement.
/// * `moves` - Whether each one moves.
///
/// # Returns
///
/// The new text and its lines.
fn write_out(src: &str, spans: &[(usize, usize)], moves: &[bool]) -> Wrapped
{
    let ends: Vec<usize> = (0..spans.len()).map(|i| chunk_end(src, spans, i)).collect();
    let starts: Vec<usize> = std::iter::once(0).chain(ends.iter().copied()).collect();
    let kept: Vec<(usize, usize)> =
        (0..spans.len()).filter(|&i| !moves[i]).map(|i| (starts[i], ends[i])).collect();
    let moved: Vec<usize> = (0..spans.len()).filter(|&i| moves[i]).collect();
    let mut out = Out::default();
    let top_end = kept.iter().map(|&(from, to)| trim_end_at(src, from, to)).next_back();
    for (n, &(from, to)) in kept.iter().enumerate()
    {
        let to = tern!(n + 1 == kept.len(), top_end.unwrap_or(to), to);
        out.push(&src[from..to], Some((from, src)));
    }
    do_if!(!out.text.is_empty(), out.push("\n", None));
    let first = moved.first().map(|&i| (starts[i], spans[i].0));
    let cut = first
        .map(|(from, lo)| src[from..lo].rfind('\n').map_or(from, |n| from + n + 1));
    let lead = first.zip(cut).map(|((from, _), cut)| (from, cut));
    let gap = lead.is_none_or(|(from, cut)| !src[from..cut].starts_with('\n'));
    do_if!(!out.text.is_empty() && gap, out.push("\n", None));
    do_if!(let Some((from, cut)) = lead, out.push(&src[from..cut], Some((from, src))));
    out.push(HEADER, None);
    let pieces = moved.iter().enumerate().map(|(n, &i)| {
        let from = tern!(n == 0, cut.unwrap_or(starts[i]), starts[i]);
        (from, ends[i])
    });
    let pieces: Vec<(usize, usize)> = pieces.collect();
    let last_end = pieces.last().map(|&(from, to)| trim_end_at(src, from, to));
    for (n, &(from, to)) in pieces.iter().enumerate()
    {
        let to = tern!(n + 1 == pieces.len(), last_end.unwrap_or(to), to);
        out.push(&src[from..to], Some((from, src)));
    }
    do_if!(!pieces.is_empty(), out.push("\n", None));
    out.push(CLOSER, None);
    let tail_from = starts[spans.len()];
    let tail = src[tail_from..].trim_end();
    let tail_at =
        tail_from + (src[tail_from..].len() - src[tail_from..].trim_start().len());
    do_if!(!tail.is_empty(), out.push(tail.trim_start(), Some((tail_at, src))));
    do_if!(!tail.is_empty(), out.push("\n", None));
    Wrapped { text: out.text, lines: out.lines }
}

/// The end of a slice with the white space at its end left out.
///
/// # Arguments
///
/// * `src` - The text.
/// * `from` - The start of the slice.
/// * `to` - Its end.
///
/// # Returns
///
/// The new end.
fn trim_end_at(src: &str, from: usize, to: usize) -> usize
{
    from + src[from..to].trim_end().len()
}

/// Where the text of a top-level statement ends: past a comment and a line
/// break that follow it on its line, or at its own end when the next
/// statement starts on that line.
///
/// # Arguments
///
/// * `src` - The text.
/// * `spans` - The octet range of each top-level statement.
/// * `i` - The statement.
///
/// # Returns
///
/// The octet offset.
fn chunk_end(src: &str, spans: &[(usize, usize)], i: usize) -> usize
{
    let hi = spans[i].1;
    let next = spans.get(i + 1).map_or(src.len(), |s| s.0);
    let line_end = src[hi..].find('\n').map_or(src.len(), |n| hi + n + 1);
    tern!(line_end <= next, line_end, hi)
}
