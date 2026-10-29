use bon::Builder;
use leptos::html::span;
use leptos::prelude::*;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;
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
    let tokens = live.map(|t| (t.span, token_class(&t.kind)));
    let comments = lexed.comments.iter().map(|c| (c.span, COMMENT_CLASS));
    let mut marks: Vec<(Span, &'static str)> = tokens.chain(comments).collect();
    marks.sort_by_key(|(span, _)| span.lo.as_u32());
    let base = file.start_pos.as_u32() as usize;
    let text = file.src.as_str();
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

/// CSS class that colors a token kind.
///
/// # Arguments
///
/// * `kind` - The token kind.
///
/// # Returns
///
/// The class.
pub fn token_class(kind: &TokenKind) -> &'static str
{
    match kind
    {
        TokenKind::Ident => "tok-ident",
        TokenKind::RawIdent => "tok-raw",
        TokenKind::Number(_) => "tok-number",
        TokenKind::Char(_) => "tok-char",
        TokenKind::Str(_) => "tok-str",
        TokenKind::Kw(kw) => kw_class(*kw),
        TokenKind::Punct(punct) => punct_class(*punct),
        TokenKind::Eof => "tok-plain",
    }
}

/// CSS class that colors a reserved word, from its role in a sentence:
/// control word, statement verb, operator, particle, kind noun or truth
/// value.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// The class.
pub fn kw_class(kw: Kw) -> &'static str
{
    match kw
    {
        Kw::If
        | Kw::Then
        | Kw::ThenIs
        | Kw::ElseIf
        | Kw::Else
        | Kw::While
        | Kw::WhileIs
        | Kw::Counting
        | Kw::Increasing
        | Kw::Decreasing
        | Kw::Loop
        | Kw::NextIteration
        | Kw::BlockOpen
        | Kw::FnOpen
        | Kw::FnOpenRet
        | Kw::BlockClose
        | Kw::Following
        | Kw::WillDo
        | Kw::DoAnd
        | Kw::Tasks
        | Kw::Done
        | Kw::Accepting
        | Kw::Being
        | Kw::WhenDone
        | Kw::Function
        | Kw::Const
        | Kw::Var
        | Kw::Mem
        | Kw::TakeSpace
        | Kw::At
        | Kw::AtAddress
        | Kw::SoThat
        | Kw::AllocTarget
        | Kw::ResizeHead
        | Kw::FreeHead => "tok-kw",
        Kw::Print
        | Kw::Assign
        | Kw::Return
        | Kw::WillReturn
        | Kw::Call
        | Kw::Let
        | Kw::Stop
        | Kw::Skip
        | Kw::ReturnValue
        | Kw::WillKeep
        | Kw::WillResize
        | Kw::WillRelease => "tok-verb",
        Kw::Sum
        | Kw::Difference
        | Kw::Product
        | Kw::Equal
        | Kw::NotEqual
        | Kw::Greater
        | Kw::NotGreater
        | Kw::Less
        | Kw::NotLess
        | Kw::Not
        | Kw::And
        | Kw::Or
        | Kw::Negate
        | Kw::ShiftLeft
        | Kw::ShiftRight
        | Kw::DivideHead
        | Kw::QuotientTail
        | Kw::RemainderTail
        | Kw::BitwiseHead
        | Kw::MeetTail
        | Kw::UnionTail
        | Kw::DifferTail
        | Kw::InvertTail
        | Kw::ShiftHead
        | Kw::ConvertHead
        | Kw::CallHead
        | Kw::Divide
        | Kw::Remainder
        | Kw::BitAnd
        | Kw::BitOr
        | Kw::BitXor
        | Kw::BitNot
        | Kw::Shift
        | Kw::Convert
        | Kw::CallResult
        | Kw::SizeOf
        | Kw::Cell
        | Kw::CellCount
        | Kw::Ordinal => "tok-op",
        Kw::Thi
        | Kw::Ko
        | Kw::Hnint
        | Kw::Hpjint
        | Kw::Htet
        | Kw::Auk
        | Kw::Hu
        | Kw::As
        | Kw::Pa
        | Kw::ListKo
        | Kw::ListWith
        | Kw::From
        | Kw::To
        | Kw::Step
        | Kw::CountAs
        | Kw::ListOf
        | Kw::Having => "tok-particle",
        Kw::IntKind
        | Kw::FloatKind
        | Kw::CharKind
        | Kw::BoolKind
        | Kw::BoolKindLoan
        | Kw::Signed
        | Kw::Unsigned
        | Kw::Bit
        | Kw::Base
        | Kw::Value
        | Kw::Result
        | Kw::Note
        | Kw::Address
        | Kw::RawAddress
        | Kw::Borrowed
        | Kw::Lend
        | Kw::Octets
        | Kw::Items
        | Kw::List => "tok-kind",
        Kw::True | Kw::False | Kw::Null | Kw::BlankList => "tok-bool",
    }
}

/// CSS class that colors a punctuation token: operators stand out,
/// separators and brackets recede.
///
/// # Arguments
///
/// * `punct` - The punctuation.
///
/// # Returns
///
/// The class.
pub fn punct_class(punct: Punct) -> &'static str
{
    let delim = matches!(
        punct,
        Punct::ListSep | Punct::FullStop | Punct::Ei | Punct::LParen | Punct::RParen
    );
    tern!(delim, "tok-delim", "tok-op")
}
