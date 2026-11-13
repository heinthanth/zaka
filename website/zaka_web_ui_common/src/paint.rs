use bon::Builder;
use leptos::html::span;
use leptos::prelude::*;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::role::TokenRole;
use zakac_lexer::role::role_with_text;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_if;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

/// Class of a comment.
pub const COMMENT_CLASS: &str = "tok-comment";

/// Class of the text between tokens.
pub const GAP_CLASS: &str = "tok-gap";

/// A run of source text and the class that colors it.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Piece
{
    /// The text.
    #[builder(into)]
    pub text: String,
    /// The CSS class.
    pub class: &'static str,
}

/// Cut a source text into colored pieces with the lexer: tokens in the color
/// of their role, comments, and the text between them. The pieces cover the
/// text the lexer saw, which is the source after Burmese normalization.
///
/// # Arguments
///
/// * `source` - The source text.
///
/// # Returns
///
/// The pieces, in order.
pub fn paint(source: &str) -> Vec<Piece>
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("paint.zaka".into()), source.into());
    let lexed = lex(&file, &LexerOptions::default());
    let live = lexed.tokens.iter().filter(|t| t.kind != TokenKind::Eof);
    let base = file.start_pos.as_u32() as usize;
    let text = file.src.as_str();
    let lexeme =
        |s: Span| &text[s.lo.as_u32() as usize - base..s.hi.as_u32() as usize - base];
    let tokens = live.map(|t| (t.span, token_class(&t.kind, lexeme(t.span))));
    let comments = lexed.comments.iter().map(|c| (c.span, COMMENT_CLASS));
    let mut marks: Vec<(Span, &'static str)> = tokens.chain(comments).collect();
    marks.sort_by_key(|(span, _)| span.lo.as_u32());
    let mut pieces = Vec::new();
    let end = marks.iter().fold(0, |at, (span, class)| {
        let range =
            (span.lo.as_u32() as usize - base, span.hi.as_u32() as usize - base);
        push_mark(&mut pieces, text, at, range, class)
    });
    push_gap(&mut pieces, &text[end..]);
    pieces
}

/// Add a mark and the text before it.
///
/// # Arguments
///
/// * `pieces` - Where the pieces go.
/// * `text` - The text the lexer saw.
/// * `at` - Where the last mark ended.
/// * `(lo, hi)` - The range of the mark in `text`.
/// * `class` - The class of the mark.
///
/// # Returns
///
/// Where this mark ends.
fn push_mark(
    pieces: &mut Vec<Piece>, text: &str, at: usize, (lo, hi): (usize, usize),
    class: &'static str,
) -> usize
{
    push_gap(pieces, &text[at.min(lo)..lo]);
    let (lo, hi) = (lo.max(at), hi.max(at));
    pieces.push(Piece::builder().text(&text[lo..hi]).class(class).build());
    hi
}

/// Add the text between two marks, when there is some.
///
/// # Arguments
///
/// * `pieces` - Where it goes.
/// * `gap` - The text.
fn push_gap(pieces: &mut Vec<Piece>, gap: &str)
{
    do_if!(
        !gap.is_empty(),
        pieces.push(Piece::builder().text(gap).class(GAP_CLASS).build())
    );
}

/// A painted source text: one `span` per piece.
///
/// # Arguments
///
/// * `source` - The source text.
///
/// # Returns
///
/// The view.
pub fn painted(source: &str) -> impl IntoView + use<>
{
    let view = |p: Piece| span().class(p.class).child(p.text);
    paint(source).into_iter().map(view).collect_view()
}

/// CSS class that colors a token: `tok-` and the name of its role.
///
/// # Arguments
///
/// * `kind` - The token kind.
/// * `text` - The token text, so a kind name such as `ကိန်းပြည့်` takes the color
///   of a kind.
///
/// # Returns
///
/// The class.
pub fn token_class(kind: &TokenKind, text: &str) -> &'static str
{
    match role_with_text(kind, text)
    {
        TokenRole::Control => "tok-kw",
        TokenRole::Verb => "tok-verb",
        TokenRole::Op => "tok-op",
        TokenRole::Particle => "tok-particle",
        TokenRole::Kind => "tok-kind",
        TokenRole::Truth => "tok-bool",
        TokenRole::Ident => "tok-ident",
        TokenRole::Raw => "tok-raw",
        TokenRole::Number => "tok-number",
        TokenRole::Char => "tok-char",
        TokenRole::Str => "tok-str",
        TokenRole::Delim => "tok-delim",
        TokenRole::Comment => COMMENT_CLASS,
        TokenRole::Plain => "tok-plain",
    }
}
