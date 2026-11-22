//! The rewrites the settings ask for: the speech form of each reserved word,
//! the digits of number literals, and the order of the imports at the head of
//! a file.

use std::borrow::Cow;

use zakac_ast::stmt::ImportStmt;
use zakac_ast::stmt::Stmt;
use zakac_lexer::spoken::spoken;
use zakac_lexer::token::CommentKind;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::TokenKind;
use zakac_parser::load::InPlace;
use zakac_parser::load::is_std;
use zakac_rust_sugars::ret_if;
use zakac_span::source_file::SourceFile;
use zakac_span::span::Span;

use crate::format::line_at;
use crate::format::relative;
use crate::options::Digits;
use crate::options::FmtOptions;
use crate::options::SpeechMode;

/// A word to write in place of the text between two offsets of the
/// normalized text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Swap
{
    pub lo: usize,
    pub hi: usize,
    pub text: String,
}

/// The words the speech form and the digits rewrite, in the order of the
/// text.
///
/// # Arguments
///
/// * `file` - The file.
/// * `lexed` - Its tokens, with the spellings of its reserved words.
/// * `parsed` - The file as parsed: the spoken words it read as their second
///   word, and the spellings of its glued ends.
/// * `options` - The settings.
///
/// # Returns
///
/// The swaps.
pub fn swaps(
    file: &SourceFile, lexed: &LexOutput, parsed: &InPlace, options: &FmtOptions,
) -> Vec<Swap>
{
    let mut found = speech_swaps(file, lexed, parsed, options.speech_mode);
    found.extend(digit_swaps(file, lexed, options.digits));
    found.sort_by_key(|s| s.lo);
    found
}

/// The reserved words to write in the other speech form.
///
/// # Arguments
///
/// * `file` - The file.
/// * `lexed` - Its tokens, with the spellings of its reserved words.
/// * `parsed` - The file as parsed.
/// * `mode` - The form to write.
///
/// # Returns
///
/// The swaps.
fn speech_swaps(
    file: &SourceFile, lexed: &LexOutput, parsed: &InPlace, mode: SpeechMode,
) -> Vec<Swap>
{
    let at = |span: Span| (relative(file, span.lo), relative(file, span.hi));
    let swap = |span: Span, text: &str| {
        let (lo, hi) = at(span);
        Swap { lo, hi, text: text.to_string() }
    };
    // A spoken word the parser read as its second word takes the written
    // spelling of that word: `ရှိတဲ့` as `ရှိသည့်`.
    let written = |span: &Span, word: &'static str| {
        let (lo, hi) = at(*span);
        let also = spoken(&file.src[lo..hi]).and_then(|s| s.also).map(|(_, w)| w);
        also_or(parsed.also_taken.contains(span), also, word)
    };
    match mode
    {
        SpeechMode::Written =>
        {
            let each = lexed.spoken_words.iter().chain(&parsed.glued_ends.spoken);
            each.map(|(span, word)| swap(*span, written(span, word))).collect()
        }
        SpeechMode::Spoken =>
        {
            let each = lexed.written_words.iter().chain(&parsed.glued_ends.written);
            each.map(|(span, word)| swap(*span, word)).collect()
        }
    }
}

/// The second word a spoken word stands for, when the parser read it so, or
/// its first word.
///
/// # Arguments
///
/// * `taken` - Whether the parser read the second word.
/// * `also` - The written spelling of the second word.
/// * `word` - The written spelling of the first word.
///
/// # Returns
///
/// The spelling.
fn also_or(taken: bool, also: Option<&'static str>, word: &'static str)
-> &'static str
{
    also.filter(|_| taken).unwrap_or(word)
}

/// The number literals to write in the other script. A literal with letters
/// (a base above ten, an exponent) keeps its digits.
///
/// # Arguments
///
/// * `file` - The file.
/// * `lexed` - Its tokens.
/// * `digits` - The script to write.
///
/// # Returns
///
/// The swaps.
fn digit_swaps(file: &SourceFile, lexed: &LexOutput, digits: Digits) -> Vec<Swap>
{
    ret_if!(digits == Digits::Preserve, Vec::new());
    let numbers =
        lexed.tokens.iter().filter(|t| matches!(t.kind, TokenKind::Number(_)));
    let swap = |lo: usize, hi: usize| {
        let text = &file.src[lo..hi];
        let plain = text.chars().all(|c| c == '.' || digit_value(c).is_some());
        let new = in_script(text, digits);
        (plain && new != text).then_some(Swap { lo, hi, text: new })
    };
    let at = |span: Span| (relative(file, span.lo), relative(file, span.hi));
    numbers.map(|t| at(t.span)).filter_map(|(lo, hi)| swap(lo, hi)).collect()
}

/// The value of a digit of either script.
///
/// # Arguments
///
/// * `c` - The character.
///
/// # Returns
///
/// The value, or `None` for a character that is no digit.
fn digit_value(c: char) -> Option<u32>
{
    let burmese = ('\u{1040}'..='\u{1049}').contains(&c).then(|| c as u32 - 0x1040);
    burmese.or_else(|| c.to_digit(10).filter(|_| c.is_ascii_digit()))
}

/// A number literal with its digits in a script.
///
/// # Arguments
///
/// * `text` - The literal.
/// * `digits` - The script.
///
/// # Returns
///
/// The literal.
pub(crate) fn in_script(text: &str, digits: Digits) -> String
{
    let base = zero_of(digits);
    let each =
        |c: char| digit_value(c).and_then(|d| char::from_u32(base + d)).unwrap_or(c);
    text.chars().map(each).collect()
}

/// The code point of the digit zero of a script.
///
/// # Arguments
///
/// * `digits` - The script.
///
/// # Returns
///
/// The code point.
fn zero_of(digits: Digits) -> u32
{
    match digits
    {
        Digits::Burmese => 0x1040,
        _ => '0' as u32,
    }
}

/// One import at the head of a file, with the lines it takes: its own lines,
/// and the `///` comments just above it.
#[derive(Debug, Clone)]
struct Chunk
{
    first: usize,
    last: usize,
    /// The stdlib first, then the others, each group in the order of its path.
    key: (bool, String),
}

/// The text with each run of imports at the head of the file sorted: the
/// stdlib first, then the others, each group in the order of its path. A
/// blank line or a comment on a line of its own ends a run, and a run with
/// two statements on one line is left as it is.
///
/// # Arguments
///
/// * `file` - The file.
/// * `original` - The text as read.
/// * `lexed` - Its tokens and comments.
/// * `stmts` - Its statements as parsed.
///
/// # Returns
///
/// The text, or `None` when no import moves.
pub fn sort_imports(
    file: &SourceFile, original: &str, lexed: &LexOutput, stmts: &[Stmt],
) -> Option<String>
{
    let lines: Vec<&str> = original.split_inclusive('\n').collect();
    // The lines of the text as read must be the lines of the normalized text.
    let breaks = |t: &str| t.matches('\n').count();
    ret_if!(breaks(original) != breaks(&file.src) || !original.ends_with('\n'), None);
    let docs = doc_lines(file, lexed);
    let head =
        stmts.iter().take_while(|s| matches!(s, Stmt::Import(_) | Stmt::Share(_)));
    let chunks: Vec<Option<Chunk>> = head.map(|s| chunk_of(file, s, &docs)).collect();
    // Two statements on one line keep the order of the whole head.
    let placed: Vec<&Chunk> = chunks.iter().flatten().collect();
    ret_if!(placed.windows(2).any(|w| w[0].last >= w[1].first), None);
    let mut out: Vec<&str> = lines.clone();
    let mut moved = false;
    for run in runs(&chunks)
    {
        let mut sorted = run.clone();
        sorted.sort_by(|a, b| a.key.cmp(&b.key));
        let order = sorted.iter().flat_map(|c| lines[c.first..=c.last].iter().copied());
        let start = run[0].first;
        out.splice(start..start + order.clone().count(), order);
        moved |= sorted.iter().zip(&run).any(|(a, b)| a.first != b.first);
    }
    moved.then(|| out.concat())
}

/// The lines that hold a `///` comment and nothing else.
///
/// # Arguments
///
/// * `file` - The file.
/// * `lexed` - Its tokens and comments.
///
/// # Returns
///
/// The lines.
fn doc_lines(file: &SourceFile, lexed: &LexOutput) -> Vec<usize>
{
    let line = |span: Span| line_at(file, relative(file, span.lo));
    let taken: Vec<usize> = lexed.tokens.iter().map(|t| line(t.span)).collect();
    let docs = lexed.comments.iter().filter(|c| c.kind == CommentKind::DocSlash);
    docs.map(|c| line(c.span)).filter(|l| !taken.contains(l)).collect()
}

/// The lines of a statement at the head of a file, when it is an import.
///
/// # Arguments
///
/// * `file` - The file.
/// * `stmt` - The statement.
/// * `docs` - The lines that hold a `///` comment alone.
///
/// # Returns
///
/// The chunk, or `None` for a share, which ends a run.
fn chunk_of(file: &SourceFile, stmt: &Stmt, docs: &[usize]) -> Option<Chunk>
{
    let Stmt::Import(n) = stmt
    else
    {
        return None;
    };
    let line = |pos| line_at(file, relative(file, pos));
    let last = line(n.span.hi);
    let mut first = line(n.span.lo);
    while first > 0 && docs.contains(&(first - 1))
    {
        first -= 1;
    }
    Some(Chunk { first, last, key: key_of(n) })
}

/// The key an import sorts by.
///
/// # Arguments
///
/// * `n` - The import.
///
/// # Returns
///
/// `false` for the stdlib, then the path.
fn key_of(n: &ImportStmt) -> (bool, String) { (!is_std(n), n.path.lit.value.clone()) }

/// The runs of imports on lines next to each other, each with more
/// than one import.
///
/// # Arguments
///
/// * `chunks` - The statements at the head of the file; `None` for a share.
///
/// # Returns
///
/// The runs.
fn runs(chunks: &[Option<Chunk>]) -> Vec<Vec<Chunk>>
{
    let mut found: Vec<Vec<Chunk>> = Vec::new();
    let mut open: Vec<Chunk> = Vec::new();
    for chunk in chunks
    {
        let follows = |c: &Chunk| open.last().is_some_and(|p| p.last + 1 == c.first);
        match chunk
        {
            Some(c) if open.is_empty() || follows(c) => open.push(c.clone()),
            Some(c) => found.push(std::mem::replace(&mut open, vec![c.clone()])),
            None => found.push(std::mem::take(&mut open)),
        }
    }
    found.push(open);
    found.retain(|r| r.len() > 1);
    found
}

/// The text between two offsets of the normalized text, with the swaps that
/// fall inside it written in.
///
/// # Arguments
///
/// * `src` - The normalized text.
/// * `lo` - The first offset.
/// * `hi` - The offset past the end.
/// * `swaps` - The swaps, in the order of the text.
///
/// # Returns
///
/// The text.
pub fn swapped<'a>(src: &'a str, lo: usize, hi: usize, swaps: &[Swap]) -> Cow<'a, str>
{
    let inside: Vec<&Swap> =
        swaps.iter().filter(|s| lo <= s.lo && s.hi <= hi).collect();
    ret_if!(inside.is_empty(), Cow::Borrowed(&src[lo..hi]));
    let mut out = String::new();
    let end = inside.iter().fold(lo, |at, s| {
        out.push_str(&src[at..s.lo]);
        out.push_str(&s.text);
        s.hi
    });
    out.push_str(&src[end..hi]);
    Cow::Owned(out)
}
