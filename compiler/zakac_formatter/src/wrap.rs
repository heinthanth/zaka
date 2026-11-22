//! The wrap of a line still too wide once the declarations are laid out: a
//! statement line breaks where it reads best that keeps the line within the
//! widest line. The places, best first:
//!
//! 1. the end of a clause: after a reserved word that ends in `၍`, `ကာ`, `ပြီး`
//!    or `သည်`, after `ဖြစ်၍`, `လျှင်` and the `သည်` after a topic, and before the
//!    opener of a block;
//! 2. after a `၊` between the items of a list;
//! 3. a space after a reserved word or a closing mark, but not one that leaves
//!    the last word of the statement alone on its line.
//!
//! Within the best kind of place that has one, the last place that keeps the
//! line within the widest line wins; when none of them does, the first one.
//! The line breaks the author wrote remain, a line of a declaration the
//! formatter laid out is left to it, and a comment after code keeps its line
//! and takes no part in the width.

use std::collections::HashMap;

use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;

use crate::breaks::Gap;
use crate::breaks::Ruler;
use crate::breaks::Word;
use crate::breaks::line_width;

/// The kinds of places a line breaks at, best first.
const KINDS: [Place; 3] = [Place::Clause, Place::List, Place::Space];

/// A kind of place a line breaks at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Place
{
    Clause,
    List,
    Space,
}

/// Add the breaks that wrap each line too wide.
///
/// # Arguments
///
/// * `ruler` - The pieces and the widths.
/// * `gaps` - The gaps decided so far; the breaks of the wrap join them.
/// * `owned` - The pieces of each declaration the formatter laid out, as the
///   index of the first and the last.
/// * `starts_line` - Whether a piece starts a line, with the gaps so far.
pub fn wrap(
    ruler: &Ruler<'_>, gaps: &mut HashMap<usize, Gap>, owned: &[(usize, usize)],
    starts_line: &dyn Fn(usize, &HashMap<usize, Gap>) -> bool,
)
{
    let words = ruler.words;
    let starts: Vec<usize> =
        (0..words.len()).filter(|&i| i == 0 || starts_line(i, gaps)).collect();
    let ends = starts.iter().skip(1).copied().chain(std::iter::once(words.len()));
    let lines: Vec<(usize, usize)> = starts.iter().copied().zip(ends).collect();
    let inside =
        |(s, e): &(usize, usize)| owned.iter().any(|&(a, b)| *s <= b && a < *e);
    for &(start, end) in lines.iter().filter(|l| !inside(l))
    {
        let breaks = wrap_line(ruler, start, code_end(ruler, start, end), false);
        gaps.extend(breaks.into_iter().map(|i| (i, Gap::Break)));
    }
}

/// The end of the code of a line: before the comment that ends it.
///
/// # Arguments
///
/// * `ruler` - The pieces.
/// * `start` - The index of the first piece of the line.
/// * `end` - The index past its last piece.
///
/// # Returns
///
/// The index past the last piece of code.
fn code_end(ruler: &Ruler<'_>, start: usize, end: usize) -> usize
{
    let note = end > start + 1 && ruler.words[end - 1].comment;
    end - usize::from(note)
}

/// The breaks of one line.
///
/// # Arguments
///
/// * `ruler` - The pieces and the widths.
/// * `start` - The index of the first piece of the line.
/// * `end` - The index past its last piece of code.
/// * `made` - Whether the formatter made the line, so it starts with the space
///   of a line it makes.
///
/// # Returns
///
/// The indices of the pieces a break goes before.
pub fn wrap_line(ruler: &Ruler<'_>, start: usize, end: usize, made: bool)
-> Vec<usize>
{
    // The first part of a line the formatter made counts as a later part.
    let first = start + usize::from(made) * ruler.words.len();
    let mut found = Vec::new();
    let mut from = start;
    while width(ruler, first, from, end) > ruler.max_width
    {
        let Some(at) = best(ruler, first, from, end)
        else
        {
            break;
        };
        found.push(at);
        from = at;
    }
    found
}

/// The width of the pieces from `from` to `end` on one line: the space at
/// its start, the space of the statement for its first line and the space of
/// a line the formatter makes for the others.
///
/// # Arguments
///
/// * `ruler` - The pieces and the widths.
/// * `start` - The index of the first piece of the line as written.
/// * `from` - The index of the first piece of this part.
/// * `end` - The index past its last piece.
///
/// # Returns
///
/// The width.
fn width(ruler: &Ruler<'_>, start: usize, from: usize, end: usize) -> usize
{
    let lead = match from == start
    {
        true => (ruler.lead)(from),
        false => (ruler.made)(from),
    };
    lead + line_width(&ruler.words[from..end])
}

/// The best place to break a line part that is too wide.
///
/// # Arguments
///
/// * `ruler` - The pieces and the widths.
/// * `start` - The index of the first piece of the line as written.
/// * `from` - The index of the first piece of the part.
/// * `end` - The index past its last piece.
///
/// # Returns
///
/// The index of the piece to break before, or `None` for a part with no
/// space to break at.
fn best(ruler: &Ruler<'_>, start: usize, from: usize, end: usize) -> Option<usize>
{
    let fits = |i: &usize| width(ruler, start, from, *i) <= ruler.max_width;
    let places = |kind: Place| -> Vec<usize> {
        let open = |i: &usize| !ruler.words[*i].joined && !ruler.words[*i].comment;
        (from + 1..end).filter(open).filter(|&i| is_place(ruler, i, kind)).collect()
    };
    let all: Vec<Vec<usize>> = KINDS.iter().map(|&k| places(k)).collect();
    let fitting = all.iter().find_map(|p| p.iter().copied().rfind(fits));
    ret_if!(fitting.is_some(), fitting);
    all.iter().find_map(|p| p.first().copied())
}

/// Whether a line can break before a piece at a kind of place.
///
/// # Arguments
///
/// * `ruler` - The pieces.
/// * `at` - The index of the piece.
/// * `kind` - The kind of place.
///
/// # Returns
///
/// `true` when it can.
fn is_place(ruler: &Ruler<'_>, at: usize, kind: Place) -> bool
{
    let words = ruler.words;
    let before = &words[at - 1];
    match kind
    {
        Place::Clause =>
        {
            ends_clause(before.kind, before.text) || opens_block(words[at].kind)
        }
        Place::List => before.kind == Some(TokenKind::Punct(Punct::ListSep)),
        // A space break comes after a reserved word or a closing mark, so a
        // value keeps the particle after it, and never leaves the last word
        // alone on its line.
        Place::Space => after_word(before.kind) && !last_word(words, at),
    }
}

/// Whether a token ends a clause: a reserved word that ends in `၍`, `ကာ`,
/// `ပြီး` or `သည်`, `ဖြစ်၍` in each of its spellings, `လျှင်`, or the `သည်`
/// after a topic.
///
/// # Arguments
///
/// * `kind` - The kind of the token.
/// * `text` - Its text.
///
/// # Returns
///
/// `true` for such a token.
fn ends_clause(kind: Option<TokenKind>, text: &str) -> bool
{
    let Some(TokenKind::Kw(kw)) = kind
    else
    {
        return false;
    };
    let joint = ["၍", "ကာ", "ပြီး", "သည်"].iter().any(|end| text.ends_with(end));
    joint || matches!(kw, Kw::And | Kw::Then | Kw::Thi)
}

/// Whether a token opens a block.
///
/// # Arguments
///
/// * `kind` - The kind of the token.
///
/// # Returns
///
/// `true` for an opener.
fn opens_block(kind: Option<TokenKind>) -> bool
{
    let open = [Kw::BlockOpen, Kw::UnsafeBlockOpen, Kw::FnOpenRet];
    matches!(kind, Some(TokenKind::Kw(kw)) if open.contains(&kw))
}

/// Whether a space break can come after a token: a reserved word, or a mark
/// other than an opening one.
///
/// # Arguments
///
/// * `kind` - The kind of the token.
///
/// # Returns
///
/// `true` when it can.
fn after_word(kind: Option<TokenKind>) -> bool
{
    let opens = [Punct::LParen, Punct::LBracket];
    match kind
    {
        Some(TokenKind::Kw(_)) => true,
        Some(TokenKind::Punct(p)) => !opens.contains(&p),
        _ => false,
    }
}

/// Whether a piece starts the last word of its statement: nothing but pieces
/// joined to it stand between it and the stop.
///
/// # Arguments
///
/// * `words` - The pieces of the file.
/// * `at` - The index of the piece.
///
/// # Returns
///
/// `true` for the last word.
fn last_word(words: &[Word<'_>], at: usize) -> bool
{
    let joined = words[at + 1..].iter().take_while(|w| w.joined).count();
    at + 1 + joined >= end_of(words, at)
}

/// The index past the last piece of the statement that holds a piece: past
/// the stop that ends it, or past the last piece.
///
/// # Arguments
///
/// * `words` - The pieces of the file.
/// * `at` - The index of the piece.
///
/// # Returns
///
/// The index.
fn end_of(words: &[Word<'_>], at: usize) -> usize
{
    let stop = |w: &Word<'_>| w.kind == Some(TokenKind::Punct(Punct::FullStop));
    words[at..].iter().position(stop).map_or(words.len(), |n| at + n + 1)
}
