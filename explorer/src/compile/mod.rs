use std::borrow::Cow;

use bon::Builder;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoStaticStr;
use zakac_ast::s_expr::ToSexpr;
use zakac_burmese_segmenter::segmenter::SegmenterOptions;
use zakac_burmese_segmenter::segmenter::segment_at;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Level;
use zakac_diagnostics::has_errors;
use zakac_i18n::t;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::token::Kw;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::Particle;
use zakac_lexer::token::Punct;
use zakac_lexer::token::StrPart;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_naturalness::NaturalnessOptions;
use zakac_naturalness::check;
use zakac_parser::parser::parse;
use zakac_reporter::render::Piece;
use zakac_reporter::render::diagnostic_pieces;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_semcheck::semcheck::SemcheckOutput;
use zakac_semcheck::semcheck::semcheck;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

pub mod graph;
pub mod s_expr;
pub mod samples;
pub mod spans;
pub mod tree;

use crate::compile::spans::OffsetTable;
use crate::compile::spans::Range;
use crate::compile::spans::find_in;
use crate::compile::spans::to_original;
use crate::compile::tree::TreeNode;
use crate::compile::tree::build_tree;

/// File name shown in diagnostics.
pub const FILE_NAME: &str = "explorer.zaka";

/// Platform bit counts the check accepts.
pub const PLATFORM_BITS: [u16; 5] = [8, 16, 32, 64, 128];

/// Options of a compile run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct Options
{
    /// Bit count of the platform-sized integer kinds.
    #[builder(default = 64)]
    pub platform_bits: u16,
}

impl Default for Options
{
    /// A 64-bit platform.
    ///
    /// # Returns
    ///
    /// The options.
    fn default() -> Self { Options::builder().build() }
}

/// A compiler phase. It parses from its static name in upper or lower case,
/// as links write it.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    EnumIter,
    IntoStaticStr,
    EnumString
)]
#[strum(ascii_case_insensitive)]
pub enum Phase
{
    /// Pre-lexical segmentation.
    Segment,
    /// The lexer.
    #[default]
    Lex,
    /// The parser.
    Parse,
    /// The kind check.
    Check,
}

impl Phase
{
    /// Stage label in the pipeline.
    ///
    /// # Returns
    ///
    /// The label.
    pub fn label(self) -> Cow<'static, str>
    {
        match self
        {
            Phase::Segment => t!("explorer.phase.segment"),
            Phase::Lex => t!("explorer.phase.lex"),
            Phase::Parse => t!("explorer.phase.parse"),
            Phase::Check => t!("explorer.phase.check"),
        }
    }

    /// The output the stage shows, under its label.
    ///
    /// # Returns
    ///
    /// The note.
    pub fn note(self) -> Cow<'static, str>
    {
        match self
        {
            Phase::Segment => t!("explorer.phase.segment_note"),
            Phase::Lex => t!("explorer.phase.lex_note"),
            Phase::Parse => t!("explorer.phase.parse_note"),
            Phase::Check => t!("explorer.phase.check_note"),
        }
    }
}

/// How a phase went; its static name is a CSS class suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Status
{
    /// No diagnostics.
    Clean,
    /// Warnings, but no errors.
    Warned,
    /// At least one error.
    Failed,
    /// The phase did not run.
    Skipped,
}

impl Status
{
    /// The word for it in the stage tooltip.
    ///
    /// # Returns
    ///
    /// The word.
    pub fn label(self) -> Cow<'static, str>
    {
        match self
        {
            Status::Clean => t!("explorer.status.clean"),
            Status::Warned => t!("explorer.status.warned"),
            Status::Failed => t!("explorer.status.failed"),
            Status::Skipped => t!("explorer.status.skipped"),
        }
    }
}

/// A pre-lexical segment.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct SegmentRow
{
    /// The segment kind.
    pub kind: &'static str,
    /// The text.
    pub lexeme: String,
    /// The source region.
    pub range: Range,
    /// 1-based line.
    pub line: usize,
    /// 1-based column, in chars.
    pub col: usize,
}

/// A token of the lexer.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct TokenRow
{
    /// The token form, such as `kw` or `ident`.
    pub kind: &'static str,
    /// The reserved word, operator, number shape or decoded value.
    #[builder(default)]
    pub detail: String,
    /// The absorbed sentence-final particle, blank for none.
    #[builder(default)]
    pub particle: String,
    /// No whitespace before the token.
    pub joined: bool,
    /// The text the lexer saw.
    pub lexeme: String,
    /// The source region.
    pub range: Range,
    /// 1-based line.
    pub line: usize,
    /// 1-based column, in chars.
    pub col: usize,
    /// CSS class that colors the token in the editor.
    pub class: &'static str,
}

/// A comment of the lexer.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct CommentRow
{
    /// The marker form.
    pub kind: &'static str,
    /// The text, marker included.
    pub text: String,
    /// The source region.
    pub range: Range,
    /// 1-based line.
    pub line: usize,
}

/// What a piece of a string literal is; its static name is a CSS class
/// suffix and the tooltip of the piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum PartKind
{
    /// Plain text.
    Text,
    /// An escape such as `\n`.
    Escape,
    /// A `{...}` hole, kept for later substitution.
    Hole,
}

/// A piece of a decoded string literal, as the strings table shows it.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct StrPiece
{
    /// What the piece is.
    pub kind: PartKind,
    /// The piece as it reads: the text, the escape as written, or the hole in
    /// braces.
    #[builder(into)]
    pub text: String,
}

/// A decoded string literal.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct StrRow
{
    /// Index in the string table.
    pub idx: usize,
    /// The decoded value in pieces; the view draws escapes and holes as
    /// chips, so a newline reads `\n`.
    pub value: Vec<StrPiece>,
    /// The literal as written in the source, quotes included.
    pub source: String,
    /// The source region of the literal.
    pub range: Range,
}

/// A diagnostic of a phase.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct DiagRow
{
    /// The phase that reported it.
    pub phase: Phase,
    /// How serious it is.
    pub level: Level,
    /// The code, such as `ZK0301`.
    pub code: String,
    /// The message.
    pub message: String,
    /// The hint, when there is one.
    pub help: Option<String>,
    /// The source region; `None` for a placeholder span.
    pub range: Option<Range>,
    /// The CLI rendering, in painted pieces.
    pub pieces: Vec<Piece>,
}

/// What the views show for one source text.
#[derive(Debug, Clone, PartialEq, Default, Builder)]
pub struct Compiled
{
    /// The text that was compiled.
    pub source: String,
    /// UTF-8 to UTF-16 offsets of `source`.
    pub table: OffsetTable,
    /// Segments, in source order.
    pub segments: Vec<SegmentRow>,
    /// Tokens without the end marker, in source order.
    pub tokens: Vec<TokenRow>,
    /// Comments, in source order.
    pub comments: Vec<CommentRow>,
    /// Decoded string literals.
    pub strings: Vec<StrRow>,
    /// The tree view of the program.
    pub tree: TreeNode,
    /// The S-expression form of the program.
    pub sexpr: String,
    /// Diagnostics of all phases.
    pub diagnostics: Vec<DiagRow>,
    /// Whether the check ran; it is left out after lexer or parser errors.
    pub checked: bool,
}

impl Compiled
{
    /// The token ranges, for lookups with
    /// [`crate::compile::spans::find_range`].
    ///
    /// # Returns
    ///
    /// The ranges, in source order.
    pub fn token_ranges(&self) -> Vec<Range>
    {
        self.tokens.iter().map(|t| t.range).collect()
    }

    /// The token under a UTF-16 offset of the editor.
    ///
    /// # Arguments
    ///
    /// * `off16` - The UTF-16 offset.
    ///
    /// # Returns
    ///
    /// The token index, or `None` between tokens.
    pub fn token_at(&self, off16: u32) -> Option<usize>
    {
        let off = self.table.to_utf8(off16);
        find_in(&self.tokens, off, |t| t.range)
    }

    /// Count the diagnostics of a level.
    ///
    /// # Arguments
    ///
    /// * `level` - The level.
    ///
    /// # Returns
    ///
    /// The count.
    pub fn count(&self, level: Level) -> usize
    {
        self.diagnostics.iter().filter(|d| d.level == level).count()
    }

    /// How a phase went: the worst level among its diagnostics.
    ///
    /// # Arguments
    ///
    /// * `phase` - The phase.
    ///
    /// # Returns
    ///
    /// The status; the check is skipped after lexer or parser errors.
    pub fn status(&self, phase: Phase) -> Status
    {
        ret_if!(phase == Phase::Check && !self.checked, Status::Skipped);
        let own = self.diagnostics.iter().filter(|d| d.phase == phase);
        let levels: Vec<Level> = own.map(|d| d.level).collect();
        ret_if!(levels.contains(&Level::Error), Status::Failed);
        tern!(levels.contains(&Level::Warning), Status::Warned, Status::Clean)
    }
}

/// The source map, file and text a compile run works on.
#[derive(Builder)]
struct Unit<'a>
{
    /// The source map that holds the file.
    sm: &'a SourceMap,
    /// The file.
    file: &'a SourceFile,
    /// The original text.
    original: &'a str,
}

impl Unit<'_>
{
    /// Range of a span in the original text.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// The range.
    fn range(&self, span: &Span) -> Range
    {
        to_original(self.file, self.original, span)
    }

    /// Line and column of the start of a span.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// The 1-based line and 1-based column.
    fn line_col(&self, span: &Span) -> (usize, usize)
    {
        let relative = self.file.relative_position(&span.lo);
        let (line, col) = self.file.lookup_file_pos(&relative);
        (line, col.as_u32() as usize + 1)
    }

    /// Normalized source text of a span.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// The text.
    fn text(&self, span: &Span) -> String
    {
        let lo = self.file.relative_position(&span.lo);
        let hi = self.file.relative_position(&span.hi);
        self.file.snippet(&lo, &hi).unwrap_or("").to_string()
    }
}

/// Run all phases over `source`.
///
/// # Arguments
///
/// * `source` - The text the user wrote.
/// * `options` - The options.
///
/// # Returns
///
/// The rows and trees of all views.
pub fn compile(source: &str, options: &Options) -> Compiled
{
    let mut sm = SourceMap::new();
    let name = FileName::Custom(FILE_NAME.to_string());
    let file = sm.new_source_file(name, source.to_string());
    let unit = Unit::builder().sm(&sm).file(&file).original(source).build();

    let lexed = lex(&file, &LexerOptions::default());
    let parsed = parse(&file, &lexed);
    let checked = !has_errors(&lexed.diagnostics) && !has_errors(&parsed.diagnostics);
    let check_options =
        SemcheckOptions::builder().platform_bits(options.platform_bits).build();
    let idiom = NaturalnessOptions::default();
    let mut idiom = tern!(checked, check(&parsed.program, &idiom), Vec::new());
    let out = tern!(
        checked,
        semcheck(&parsed.program, &check_options),
        SemcheckOutput::default()
    );
    idiom.extend(out.diagnostics.iter().cloned());
    let tagged = [
        (Phase::Lex, &lexed.diagnostics),
        (Phase::Parse, &parsed.diagnostics),
        (Phase::Check, &idiom),
    ];
    let diagnostics = tagged
        .iter()
        .flat_map(|(phase, diags)| diags.iter().map(|d| diag_row(&unit, *phase, d)))
        .collect();

    Compiled::builder()
        .source(source.to_string())
        .table(OffsetTable::new(source))
        .segments(segment_rows(&unit))
        .tokens(token_rows(&unit, &lexed.tokens))
        .comments(comment_rows(&unit, &lexed))
        .strings(string_rows(&unit, &lexed))
        .tree(build_tree(&file, source, &parsed.program, &out.kinds))
        .sexpr(parsed.program.to_sexpr())
        .diagnostics(diagnostics)
        .checked(checked)
        .build()
}

/// Rows of the segments view.
///
/// # Arguments
///
/// * `unit` - The compile unit.
///
/// # Returns
///
/// The rows.
fn segment_rows(unit: &Unit) -> Vec<SegmentRow>
{
    let options = SegmenterOptions::builder().coalesce_non_burmese_chars(false).build();
    let segments = segment_at(&unit.file.src, unit.file.start_pos, &options);
    let row = |seg: &zakac_burmese_segmenter::segmenter::PreLexicalToken| {
        let (line, col) = unit.line_col(&seg.span);
        SegmentRow::builder()
            .kind(seg.kind.into())
            .lexeme(seg.lexeme.to_string())
            .range(unit.range(&seg.span))
            .line(line)
            .col(col)
            .build()
    };
    segments.iter().map(row).collect()
}

/// Rows of the tokens view, without the end marker.
///
/// # Arguments
///
/// * `unit` - The compile unit.
/// * `tokens` - The lexer tokens.
///
/// # Returns
///
/// The rows.
fn token_rows(unit: &Unit, tokens: &[Token]) -> Vec<TokenRow>
{
    let live = tokens.iter().filter(|t| t.kind != TokenKind::Eof);
    live.map(|t| token_row(unit, t)).collect()
}

/// Row of one token.
///
/// # Arguments
///
/// * `unit` - The compile unit.
/// * `token` - The token.
///
/// # Returns
///
/// The row.
fn token_row(unit: &Unit, token: &Token) -> TokenRow
{
    let (line, col) = unit.line_col(&token.span);
    TokenRow::builder()
        .kind(token.kind.into())
        .detail(token_detail(&token.kind))
        .particle(particle_name(token.particle))
        .joined(token.joined)
        .lexeme(unit.text(&token.span))
        .range(unit.range(&token.span))
        .line(line)
        .col(col)
        .class(token_class(&token.kind))
        .build()
}

/// Detail of a token kind: the reserved word, operator, number shape or
/// decoded value. The form name comes from strum.
///
/// # Arguments
///
/// * `kind` - The token kind.
///
/// # Returns
///
/// The detail text, blank for identifiers.
pub fn token_detail(kind: &TokenKind) -> String
{
    match kind
    {
        TokenKind::Number(shape) => number_detail(shape),
        TokenKind::Char(ch) => format!("{ch:?}"),
        TokenKind::Str(idx) => format!("#{idx}"),
        TokenKind::Kw(kw) => format!("{kw:?}"),
        TokenKind::Punct(punct) => punct.as_ref().to_string(),
        TokenKind::Ident | TokenKind::RawIdent | TokenKind::Eof => String::new(),
    }
}

/// Detail text of a number shape.
///
/// # Arguments
///
/// * `shape` - The shape.
///
/// # Returns
///
/// The script and the parts that are present.
fn number_detail(shape: &zakac_lexer::token::NumberShape) -> String
{
    let flags = [
        (shape.has_fraction, "fraction"),
        (shape.has_exponent, "exponent"),
        (shape.has_letters, "letters"),
    ];
    let parts = flags.iter().filter(|(on, _)| *on).map(|(_, name)| *name);
    let script = format!("{:?}", shape.script).to_lowercase();
    std::iter::once(script.as_str()).chain(parts).collect::<Vec<_>>().join(" ")
}

/// Name of a particle; blank for none.
///
/// # Arguments
///
/// * `particle` - The particle.
///
/// # Returns
///
/// The name.
fn particle_name(particle: Particle) -> String
{
    tern!(particle == Particle::None, String::new(), format!("{particle:?}"))
}

/// CSS class that colors a token kind in the editor.
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

/// CSS class that colors a reserved word in the editor, from its role in a
/// sentence: control word, statement verb, operator, particle, kind noun or
/// truth value.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// The class.
fn kw_class(kw: Kw) -> &'static str
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
        | Kw::Var => "tok-kw",
        Kw::Print
        | Kw::Assign
        | Kw::Return
        | Kw::WillReturn
        | Kw::Call
        | Kw::Let
        | Kw::Stop
        | Kw::Skip
        | Kw::ReturnValue => "tok-verb",
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
        | Kw::CallResult => "tok-op",
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
        | Kw::CountAs => "tok-particle",
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
        | Kw::Note => "tok-kind",
        Kw::True | Kw::False => "tok-bool",
    }
}

/// CSS class that colors a punctuation token in the editor: operators stand
/// out, separators and brackets recede.
///
/// # Arguments
///
/// * `punct` - The punctuation.
///
/// # Returns
///
/// The class.
fn punct_class(punct: Punct) -> &'static str
{
    let delim = matches!(
        punct,
        Punct::ListSep | Punct::FullStop | Punct::Ei | Punct::LParen | Punct::RParen
    );
    tern!(delim, "tok-delim", "tok-op")
}

/// Rows of the comments table.
///
/// # Arguments
///
/// * `unit` - The compile unit.
/// * `lexed` - The lexer output.
///
/// # Returns
///
/// The rows.
fn comment_rows(unit: &Unit, lexed: &LexOutput) -> Vec<CommentRow>
{
    let row = |c: &zakac_lexer::token::Comment| {
        CommentRow::builder()
            .kind(c.kind.into())
            .text(unit.text(&c.span))
            .range(unit.range(&c.span))
            .line(unit.line_col(&c.span).0)
            .build()
    };
    lexed.comments.iter().map(row).collect()
}

/// Rows of the strings table; each takes its source text and region from
/// its token.
///
/// # Arguments
///
/// * `unit` - The compile unit.
/// * `lexed` - The lexer output.
///
/// # Returns
///
/// The rows.
fn string_rows(unit: &Unit, lexed: &LexOutput) -> Vec<StrRow>
{
    let literal = |t: &Token| match t.kind
    {
        TokenKind::Str(idx) => Some((idx as usize, t.span)),
        _ => None,
    };
    let spans: Vec<(usize, Span)> = lexed.tokens.iter().filter_map(literal).collect();
    let row = |(idx, lit): (usize, &zakac_lexer::token::StrLit)| {
        let span = spans.iter().find(|(i, _)| *i == idx).map(|(_, s)| *s);
        StrRow::builder()
            .idx(idx)
            .value(lit.parts.iter().map(str_part).collect())
            .source(span.map(|s| unit.text(&s)).unwrap_or_default())
            .range(span.map(|s| unit.range(&s)).unwrap_or_default())
            .build()
    };
    lexed.strings.iter().enumerate().map(row).collect()
}

/// A string piece as the table shows it. Text keeps its chars as they are;
/// Rust debug quoting would spell the Burmese marks as `\u{...}`.
///
/// # Arguments
///
/// * `part` - The piece.
///
/// # Returns
///
/// The piece.
fn str_part(part: &StrPart) -> StrPiece
{
    let (kind, text) = match part
    {
        StrPart::Text(text) => (PartKind::Text, text.clone()),
        StrPart::Escape(ch) => (PartKind::Escape, escape_controls(&ch.to_string())),
        StrPart::Hole(text) => (PartKind::Hole, format!("{{{text}}}")),
    };
    StrPiece::builder().kind(kind).text(text).build()
}

/// A string with its control chars written as escapes: `\n`, `\t`, `\r`
/// and `\0`, else `\u{...}`.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The text; other chars, Burmese marks included, stand as they are.
pub fn escape_controls(text: &str) -> String
{
    let shown = |c: char| match c
    {
        '\n' => "\\n".to_string(),
        '\t' => "\\t".to_string(),
        '\r' => "\\r".to_string(),
        '\0' => "\\0".to_string(),
        c if c.is_control() => format!("\\u{{{:x}}}", u32::from(c)),
        c => c.to_string(),
    };
    text.chars().map(shown).collect()
}

/// Row of a diagnostic.
///
/// # Arguments
///
/// * `unit` - The compile unit.
/// * `phase` - The phase that reported it.
/// * `diag` - The diagnostic.
///
/// # Returns
///
/// The row.
fn diag_row(unit: &Unit, phase: Phase, diag: &Diagnostic) -> DiagRow
{
    let range = (!diag.span.is_placeholder()).then(|| unit.range(&diag.span));
    DiagRow::builder()
        .phase(phase)
        .level(diag.level)
        .code(diag.code.to_string())
        .message(diag.message.clone())
        .maybe_help(diag.help.clone())
        .maybe_range(range)
        .pieces(diagnostic_pieces(unit.sm, diag))
        .build()
}
