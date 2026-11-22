//! The check of the formatter's output: it must lex to the same words, in
//! the same order, with the same comments.

use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::Particle;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_false;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

/// One token as the check compares it.
#[derive(Debug, PartialEq)]
struct Word<'a>
{
    kind: TokenKind,
    particle: Particle,
    joined: bool,
    text: &'a str,
}

/// Whether the output reads as the same words as the file.
///
/// # Arguments
///
/// * `file` - The file.
/// * `lexed` - Its tokens and comments.
/// * `out` - The text the formatter wrote.
///
/// # Returns
///
/// `true` when the two match.
pub fn same_words(file: &SourceFile, lexed: &LexOutput, out: &str) -> bool
{
    let mut map = SourceMap::new();
    let again =
        map.new_source_file(FileName::Custom("fmt".to_string()), out.to_string());
    let relexed = lex(&again, &LexerOptions::default());
    ret_false!(words(file, lexed).ne(words(&again, &relexed)));
    notes(file, lexed).eq(notes(&again, &relexed))
}

/// The tokens of a file as the check compares them.
///
/// # Arguments
///
/// * `file` - The file.
/// * `lexed` - Its tokens.
///
/// # Returns
///
/// The tokens.
fn words<'a>(
    file: &'a SourceFile, lexed: &'a LexOutput,
) -> impl Iterator<Item = Word<'a>>
{
    lexed.tokens.iter().map(move |t| Word {
        kind: t.kind,
        particle: t.particle,
        joined: t.joined,
        text: text_of(file, t.span),
    })
}

/// The comments of a file as the check compares them.
///
/// # Arguments
///
/// * `file` - The file.
/// * `lexed` - Its comments.
///
/// # Returns
///
/// The text of each comment, with no space at its end.
fn notes<'a>(
    file: &'a SourceFile, lexed: &'a LexOutput,
) -> impl Iterator<Item = &'a str>
{
    lexed.comments.iter().map(move |c| text_of(file, c.span).trim_end())
}

/// The normalized text of a span of a file.
///
/// # Arguments
///
/// * `file` - The file.
/// * `span` - The span.
///
/// # Returns
///
/// The text.
fn text_of(file: &SourceFile, span: Span) -> &str
{
    let lo = file.relative_position(&span.lo);
    let hi = file.relative_position(&span.hi);
    file.snippet(&lo, &hi).unwrap_or("")
}
