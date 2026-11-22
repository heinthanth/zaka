//! The check of the formatter's output: it must lex to the same words, in
//! the same order, with the same comments. Each word of the output is the
//! word of the file with the rewrites swapped in; a rewrite of a reserved
//! word can change its reading alone to the second word a spoken spelling
//! stands for, and a rewrite of digits the script of a number alone.

use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::spoken::spoken;
use zakac_lexer::token::DigitScript;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::NumberShape;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_true;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

use crate::format::relative;
use crate::rewrite::Swap;
use crate::rewrite::swapped;

/// Whether the output reads as the same words as the file, with the
/// rewrites swapped in.
///
/// # Arguments
///
/// * `file` - The file.
/// * `lexed` - Its tokens and comments.
/// * `swaps` - The words the rewrites swap in.
/// * `out` - The text the formatter wrote.
///
/// # Returns
///
/// `true` when the two match.
pub fn same_words(
    file: &SourceFile, lexed: &LexOutput, swaps: &[Swap], out: &str,
) -> bool
{
    let mut map = SourceMap::new();
    let again =
        map.new_source_file(FileName::Custom("fmt".to_string()), out.to_string());
    let relexed = lex(&again, &LexerOptions::default());
    ret_false!(lexed.tokens.len() != relexed.tokens.len());
    let same = lexed.tokens.iter().zip(&relexed.tokens).all(|(a, b)| {
        let (lo, hi) = (relative(file, a.span.lo), relative(file, a.span.hi));
        let want = swapped(&file.src, lo, hi, swaps);
        let texts = (&file.src[lo..hi], text_of(&again, b.span));
        let kinds = same_kind(a.kind, b.kind, texts);
        let marks = a.particle == b.particle && a.joined == b.joined;
        kinds && marks && want == text_of(&again, b.span)
    });
    ret_false!(!same);
    notes(file, lexed).eq(notes(&again, &relexed))
}

/// Whether a token of the output has the kind of the token of the file: the
/// same kind, the second word a spoken spelling of either stands for, or a
/// number in the other script.
///
/// # Arguments
///
/// * `a` - The kind of the token of the file.
/// * `b` - The kind of the token of the output.
/// * `texts` - The text of each token.
///
/// # Returns
///
/// `true` when it has.
fn same_kind(a: TokenKind, b: TokenKind, texts: (&str, &str)) -> bool
{
    ret_true!(a == b);
    let also =
        |text| spoken(text).and_then(|s| s.also).map(|(kw, _)| TokenKind::Kw(kw));
    let shape = |s: NumberShape| NumberShape { script: DigitScript::Western, ..s };
    match (a, b)
    {
        (TokenKind::Number(p), TokenKind::Number(q)) => shape(p) == shape(q),
        _ => also(texts.0) == Some(b) || also(texts.1) == Some(a),
    }
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
