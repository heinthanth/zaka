use bon::Builder;
use termcolor::Color;
use termcolor::ColorSpec;
use termcolor::WriteColor;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Level;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

use crate::width::TAB_CELLS;
use crate::width::str_cells;
use crate::writer::write_text;

/// How a piece of rendered text is painted on a color terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Paint
{
    /// No color.
    Plain,
    /// The `error[code]` tag.
    Tag(Level),
    /// Line numbers, `-->`, `|` and `=`.
    Gutter,
    /// The source text the span covers.
    Marked(Level),
    /// The `^^^` row under the span.
    Carets(Level),
}

/// A piece of rendered text with its paint.
#[derive(Builder)]
struct Piece
{
    /// How to paint the text.
    paint: Paint,
    /// The text; it can end in a line break.
    #[builder(into)]
    text: String,
}

/// One source line split around the span, with tabs expanded.
#[derive(Builder)]
struct Excerpt
{
    /// File name as shown to the user.
    path: String,
    /// 1-based line number.
    line: usize,
    /// 1-based column, in characters.
    col: usize,
    /// Text before the span.
    before: String,
    /// Text the span covers on this line.
    marked: String,
    /// Text after the span.
    after: String,
}

/// Render a diagnostic to a string without color.
///
/// # Arguments
///
/// * `sm` - The source map the span belongs to.
/// * `diag` - The diagnostic to render.
///
/// # Returns
///
/// The rendered diagnostic, ending in a line break.
pub fn format_diagnostic(sm: &SourceMap, diag: &Diagnostic) -> String
{
    pieces(sm, diag).into_iter().map(|piece| piece.text).collect()
}

/// Write a diagnostic, with color when the writer supports it.
///
/// # Arguments
///
/// * `writer` - The writer to write to.
/// * `sm` - The source map the span belongs to.
/// * `diag` - The diagnostic to write.
///
/// # Returns
///
/// The result of the write operation.
pub fn write_diagnostic<W>(
    writer: &mut W, sm: &SourceMap, diag: &Diagnostic,
) -> std::io::Result<()>
where
    W: WriteColor + ?Sized,
{
    let color = writer.supports_color();
    pieces(sm, diag).iter().try_for_each(|piece| write_piece(writer, piece, color))
}

/// Write diagnostics with a blank line between two of them.
///
/// # Arguments
///
/// * `writer` - The writer to write to.
/// * `sm` - The source map the spans belong to.
/// * `diags` - The diagnostics to write.
///
/// # Returns
///
/// The result of the write operation.
pub fn write_diagnostics<W>(
    writer: &mut W, sm: &SourceMap, diags: &[Diagnostic],
) -> std::io::Result<()>
where
    W: WriteColor + ?Sized,
{
    for (idx, diag) in diags.iter().enumerate()
    {
        write_gap(writer, idx)?;
        write_diagnostic(writer, sm, diag)?;
    }
    Ok(())
}

/// Write the blank line that goes between two diagnostics.
///
/// # Arguments
///
/// * `writer` - The writer to write to.
/// * `idx` - Index of the diagnostic about to be written.
///
/// # Returns
///
/// The result of the write operation.
fn write_gap<W>(writer: &mut W, idx: usize) -> std::io::Result<()>
where
    W: WriteColor + ?Sized,
{
    tern!(idx == 0, Ok(()), write_text(writer, "\n"))
}

/// Write one piece, painting it when color is on.
///
/// # Arguments
///
/// * `writer` - The writer to write to.
/// * `piece` - The piece to write.
/// * `color` - Whether to paint.
///
/// # Returns
///
/// The result of the write operation.
fn write_piece<W>(writer: &mut W, piece: &Piece, color: bool) -> std::io::Result<()>
where
    W: WriteColor + ?Sized,
{
    let spec = color_spec(piece.paint);
    ret_if!(!color || spec.is_none(), write_text(writer, &piece.text));
    writer.set_color(&spec.unwrap())?;
    write_text(writer, &piece.text)?;
    writer.reset()
}

/// Map a paint to a terminal color spec.
///
/// # Arguments
///
/// * `paint` - The paint to map.
///
/// # Returns
///
/// The color spec, or `None` for plain text.
fn color_spec(paint: Paint) -> Option<ColorSpec>
{
    match paint
    {
        Paint::Plain => None,
        Paint::Gutter => Some(spec(Color::Blue, false, false)),
        Paint::Tag(level) | Paint::Carets(level) =>
        {
            Some(spec(level_color(level), true, false))
        }
        Paint::Marked(level) => Some(spec(level_color(level), true, true)),
    }
}

/// Build a color spec.
///
/// # Arguments
///
/// * `color` - The foreground color.
/// * `bold` - Whether the text is bold.
/// * `underline` - Whether the text is underlined.
///
/// # Returns
///
/// The color spec.
fn spec(color: Color, bold: bool, underline: bool) -> ColorSpec
{
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(color)).set_bold(bold).set_underline(underline).set_intense(true);
    spec
}

/// Color of a diagnostic level.
///
/// # Arguments
///
/// * `level` - The level.
///
/// # Returns
///
/// The terminal color for the level.
fn level_color(level: Level) -> Color
{
    match level
    {
        Level::Error => Color::Red,
        Level::Warning => Color::Yellow,
        Level::Note => Color::Green,
    }
}

/// Create a piece.
///
/// # Arguments
///
/// * `paint` - How to paint the text.
/// * `text` - The text.
///
/// # Returns
///
/// The piece.
fn piece(paint: Paint, text: impl Into<String>) -> Piece
{
    Piece::builder().paint(paint).text(text).build()
}

/// Build the painted pieces of a diagnostic.
///
/// ```text
/// error[code]: message
///  --> file:3:12
///   |
/// 3 | before marked after
///   |        ^^^^^^
///   = help: hint
/// ```
///
/// # Arguments
///
/// * `sm` - The source map the span belongs to.
/// * `diag` - The diagnostic to render.
///
/// # Returns
///
/// The pieces in output order.
fn pieces(sm: &SourceMap, diag: &Diagnostic) -> Vec<Piece>
{
    let mut out = header_pieces(diag);
    if diag.span.is_placeholder()
    {
        out.push(piece(Paint::Gutter, " --> "));
        out.push(piece(Paint::Plain, "<PLACEHOLDER>\n"));
        out.extend(help_pieces(diag, ""));
        return out;
    }

    let excerpt = excerpt(sm, &diag.span);
    let pad = " ".repeat(excerpt.line.to_string().len());
    out.extend(location_pieces(&excerpt, &pad));
    out.extend(source_pieces(&excerpt, diag.level));
    out.extend(caret_pieces(&excerpt, &pad, diag.level));
    out.extend(help_pieces(diag, &pad));
    out
}

/// Pieces of the `error[code]: message` line.
///
/// # Arguments
///
/// * `diag` - The diagnostic.
///
/// # Returns
///
/// The header pieces.
fn header_pieces(diag: &Diagnostic) -> Vec<Piece>
{
    let tag = format!("{}[{}]", diag.level.as_ref(), diag.code);
    vec![
        piece(Paint::Tag(diag.level), tag),
        piece(Paint::Plain, format!(": {}\n", diag.message)),
    ]
}

/// Pieces of the ` --> file:line:col` line and the blank gutter line.
///
/// # Arguments
///
/// * `excerpt` - The source excerpt.
/// * `pad` - Spaces as wide as the line number.
///
/// # Returns
///
/// The location pieces.
fn location_pieces(excerpt: &Excerpt, pad: &str) -> Vec<Piece>
{
    let loc = format!("{}:{}:{}\n", excerpt.path, excerpt.line, excerpt.col);
    vec![
        piece(Paint::Gutter, format!("{pad}--> ")),
        piece(Paint::Plain, loc),
        piece(Paint::Gutter, format!("{pad} |\n")),
    ]
}

/// Pieces of the numbered source line, with the span marked.
///
/// # Arguments
///
/// * `excerpt` - The source excerpt.
/// * `level` - The diagnostic level, for the mark color.
///
/// # Returns
///
/// The source line pieces.
fn source_pieces(excerpt: &Excerpt, level: Level) -> Vec<Piece>
{
    vec![
        piece(Paint::Gutter, format!("{} | ", excerpt.line)),
        piece(Paint::Plain, excerpt.before.clone()),
        piece(Paint::Marked(level), excerpt.marked.clone()),
        piece(Paint::Plain, format!("{}\n", excerpt.after)),
    ]
}

/// Pieces of the caret row under the span.
///
/// The caret column and length come from [`str_cells`], which is the one
/// place to tune Burmese cell counts.
///
/// # Arguments
///
/// * `excerpt` - The source excerpt.
/// * `pad` - Spaces as wide as the line number.
/// * `level` - The diagnostic level, for the caret color.
///
/// # Returns
///
/// The caret row pieces.
fn caret_pieces(excerpt: &Excerpt, pad: &str, level: Level) -> Vec<Piece>
{
    let indent = " ".repeat(str_cells(&excerpt.before));
    let carets = "^".repeat(str_cells(&excerpt.marked).max(1));
    vec![
        piece(Paint::Gutter, format!("{pad} | ")),
        piece(Paint::Plain, indent),
        piece(Paint::Carets(level), carets),
        piece(Paint::Plain, "\n"),
    ]
}

/// Pieces of the `= help: ...` line, when the diagnostic has help text.
///
/// # Arguments
///
/// * `diag` - The diagnostic.
/// * `pad` - Spaces as wide as the line number.
///
/// # Returns
///
/// The help pieces, or none.
fn help_pieces(diag: &Diagnostic, pad: &str) -> Vec<Piece>
{
    let Some(help) = &diag.help
    else
    {
        return Vec::new();
    };
    vec![
        piece(Paint::Gutter, format!("{pad} = ")),
        piece(Paint::Plain, format!("help: {help}\n")),
    ]
}

/// Cut the source line that holds the start of `span` into three parts.
///
/// A span that crosses lines is cut at the end of its first line.
///
/// # Arguments
///
/// * `sm` - The source map the span belongs to.
/// * `span` - The span; it must not be the placeholder span.
///
/// # Returns
///
/// The excerpt.
fn excerpt(sm: &SourceMap, span: &Span) -> Excerpt
{
    let loc = sm.lookup_char_pos(&span.lo);
    let file = &loc.file;
    let line_idx = loc.line - 1;
    let line_lo = file.lines[line_idx].as_u32() as usize;
    let text = file.get_line(line_idx).unwrap_or("").trim_end_matches(['\n', '\r']);
    let line_hi = line_lo + text.len();

    let lo =
        (file.relative_position(&span.lo).as_u32() as usize).clamp(line_lo, line_hi);
    let hi = (file.relative_position(&span.hi).as_u32() as usize).clamp(lo, line_hi);
    Excerpt::builder()
        .path(file.name.virtual_path_string())
        .line(loc.line)
        .col(loc.col.as_u32() as usize + 1)
        .before(expand_tabs(&file.src[line_lo..lo]))
        .marked(expand_tabs(&file.src[lo..hi]))
        .after(expand_tabs(&file.src[hi..line_hi]))
        .build()
}

/// Replace tabs with spaces so the caret row lines up.
///
/// # Arguments
///
/// * `text` - The text to expand.
///
/// # Returns
///
/// The text with each tab replaced with [`TAB_CELLS`] spaces.
fn expand_tabs(text: &str) -> String { text.replace('\t', &" ".repeat(TAB_CELLS)) }
