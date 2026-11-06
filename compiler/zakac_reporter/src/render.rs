use bon::Builder;
use strum::IntoStaticStr;
use termcolor::Color;
use termcolor::ColorSpec;
use termcolor::WriteColor;
use zakac_burmese_alignment::flow::Part;
use zakac_burmese_alignment::terminal::Drawing;
use zakac_burmese_alignment::terminal::detect;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Label;
use zakac_diagnostics::Level;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

use crate::width::Carets;
use crate::width::TAB_CELLS;
use crate::width::carets_setting;
use crate::width::str_cells;
use crate::writer::write_text;

/// How a piece of rendered text is painted on a color terminal; its static
/// name is the snake case variant name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Paint
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
    /// Text a row repeats from the line above so a front end sets it out
    /// the same; it is not drawn. Under [`Carets::Mirror`] alone.
    Hidden,
    /// The span text that a row of `mark` stands over: the front end sets
    /// the text out hidden and draws the marks across its width. Under
    /// [`Carets::Mirror`] alone.
    Under
    {
        /// The diagnostic level, for the color of the marks.
        level: Level,
        /// The mark: `^` under the main span, `-` under a label.
        mark: char,
    },
}

/// Name of the setting that picks the caret rows of terminal output.
pub const CARETS_ENV: &str = "ZAKAC_CARETS";

/// A piece of rendered text with its paint.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Piece
{
    /// How to paint the text.
    pub paint: Paint,
    /// The text; it can end in a line break.
    #[builder(into)]
    pub text: String,
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

/// Render a diagnostic to a string without color. Without color the span has
/// no other mark, so the caret rows are drawn, one cell per code point.
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
    let pieces = pieces(sm, diag, Carets::Codepoint);
    pieces.into_iter().map(|piece| piece.text).collect()
}

/// Render a diagnostic to painted pieces, for a front end that does its own
/// coloring.
///
/// # Arguments
///
/// * `sm` - The source map that holds the file of the diagnostic.
/// * `diag` - The diagnostic to render.
/// * `carets` - Whether to draw the rows under spans, and how to count cells.
///
/// # Returns
///
/// The pieces in output order; with [`Carets::Codepoint`] their texts joined
/// give [`format_diagnostic`].
pub fn diagnostic_pieces(
    sm: &SourceMap, diag: &Diagnostic, carets: Carets,
) -> Vec<Piece>
{
    pieces(sm, diag, carets)
}

/// Write a diagnostic, with color when the writer supports it.
///
/// With color, the rows under spans follow [`terminal_carets`]; without color
/// they are drawn as in [`format_diagnostic`].
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
    let carets = tern!(color, terminal_carets(), Carets::Codepoint);
    write_pieces(writer, &pieces(sm, diag, carets))
}

/// Write the pieces of a diagnostic, painted when the writer supports color.
///
/// The marked text is underlined when the pieces hold no caret or dash row,
/// so that the span keeps a mark of its own; with the rows it is colored
/// alone.
///
/// # Arguments
///
/// * `writer` - The writer to write to.
/// * `pieces` - The pieces, such as [`diagnostic_pieces`] gives.
///
/// # Returns
///
/// The result of the write operation.
pub fn write_pieces<W>(writer: &mut W, pieces: &[Piece]) -> std::io::Result<()>
where
    W: WriteColor + ?Sized,
{
    let color = writer.supports_color();
    let underline = !pieces
        .iter()
        .any(|piece| matches!(piece.paint, Paint::Carets(_) | Paint::Under { .. }));
    pieces.iter().try_for_each(|piece| write_piece(writer, piece, color, underline))
}

/// The caret setting of terminal output: the value of [`CARETS_ENV`] when it
/// is set; else the setting that fits the terminal, when a check of
/// `zakac_burmese_alignment` knows it, and [`Carets::Off`] when none does.
///
/// # Returns
///
/// The setting.
pub fn terminal_carets() -> Carets
{
    let setting = std::env::var(CARETS_ENV).ok();
    setting.map_or_else(detected_carets, |text| carets_setting(Some(&text)))
}

/// The caret setting that fits the terminal standard error goes to.
///
/// # Returns
///
/// The setting for how the terminal draws Burmese text, or [`Carets::Off`]
/// when no check knows the terminal.
fn detected_carets() -> Carets
{
    let Some(terminal) = detect()
    else
    {
        return Carets::Off;
    };
    match terminal.drawing
    {
        Drawing::Codepoint => Carets::Codepoint,
        Drawing::Cluster => Carets::Cluster,
        Drawing::Flow(_) => Carets::Flow,
    }
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
/// * `underline` - Whether marked text is underlined.
///
/// # Returns
///
/// The result of the write operation.
fn write_piece<W>(
    writer: &mut W, piece: &Piece, color: bool, underline: bool,
) -> std::io::Result<()>
where
    W: WriteColor + ?Sized,
{
    let spec = color_spec(piece.paint, underline);
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
/// * `underline` - Whether marked text is underlined.
///
/// # Returns
///
/// The color spec, or `None` for plain text.
fn color_spec(paint: Paint, underline: bool) -> Option<ColorSpec>
{
    match paint
    {
        Paint::Plain | Paint::Hidden => None,
        Paint::Gutter => Some(spec(Color::Blue, false, false)),
        Paint::Tag(level) | Paint::Carets(level) | Paint::Under { level, .. } =>
        {
            Some(spec(level_color(level), true, false))
        }
        Paint::Marked(level) => Some(spec(level_color(level), true, underline)),
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
/// * `carets` - Whether to draw the rows under spans, and how to count cells.
///
/// # Returns
///
/// The pieces in output order.
fn pieces(sm: &SourceMap, diag: &Diagnostic, carets: Carets) -> Vec<Piece>
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
    let labelled: Vec<(&Label, Excerpt)> = diag
        .labels
        .iter()
        .filter(|label| !label.span.is_placeholder())
        .map(|label| (label, self::excerpt(sm, &label.span)))
        .collect();
    let width = labelled
        .iter()
        .map(|(_, each)| each.line)
        .chain([excerpt.line])
        .map(|line| line.to_string().len())
        .max()
        .unwrap_or(1);
    let pad = " ".repeat(width);
    out.extend(location_pieces(&excerpt, &pad));
    out.extend(source_pieces(&excerpt, &pad, diag.level));
    out.extend(caret_pieces(&excerpt, &pad, diag.level, carets));
    do_for!(
        (label, each) in &labelled,
        out.extend(label_pieces(each, &pad, &label.text, carets))
    );
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
/// * `pad` - Spaces as wide as the widest line number.
/// * `level` - The diagnostic level, for the mark color.
///
/// # Returns
///
/// The source line pieces.
fn source_pieces(excerpt: &Excerpt, pad: &str, level: Level) -> Vec<Piece>
{
    vec![
        piece(Paint::Gutter, gutter_text(excerpt, pad)),
        piece(Paint::Plain, excerpt.before.clone()),
        piece(Paint::Marked(level), excerpt.marked.clone()),
        piece(Paint::Plain, format!("{}\n", excerpt.after)),
    ]
}

/// Pieces of the caret row under the span.
///
/// The caret column and length come from [`reach`].
///
/// # Arguments
///
/// * `excerpt` - The source excerpt.
/// * `pad` - Spaces as wide as the line number.
/// * `level` - The diagnostic level, for the caret color.
/// * `carets` - How to count cells; [`Carets::Off`] leaves the row out.
///
/// # Returns
///
/// The caret row pieces, or none.
fn caret_pieces(
    excerpt: &Excerpt, pad: &str, level: Level, carets: Carets,
) -> Vec<Piece>
{
    ret_if!(carets == Carets::Off, Vec::new());
    let mut out = vec![piece(Paint::Gutter, format!("{pad} | "))];
    out.extend(mark_pieces(excerpt, pad, level, '^', carets));
    out.push(piece(Paint::Plain, "\n"));
    out
}

/// Pieces of a row of marks under a span, after the gutter.
///
/// # Arguments
///
/// * `excerpt` - The source excerpt.
/// * `pad` - Spaces as wide as the widest line number.
/// * `level` - The level, for the color of the marks.
/// * `mark` - The mark: `^` or `-`.
/// * `carets` - How to count cells; under [`Carets::Mirror`] the row repeats
///   the text before the span, hidden, and the span text under the marks.
///
/// # Returns
///
/// The indent and the marks.
fn mark_pieces(
    excerpt: &Excerpt, pad: &str, level: Level, mark: char, carets: Carets,
) -> Vec<Piece>
{
    let under = Paint::Under { level, mark };
    // a blank span still gets one mark, as wide as a space
    let marked = tern!(excerpt.marked.is_empty(), " ", excerpt.marked.as_str());
    ret_if!(
        carets == Carets::Mirror,
        vec![piece(Paint::Hidden, excerpt.before.clone()), piece(under, marked)]
    );
    let (indent, width) = reach(excerpt, pad, carets);
    vec![
        piece(Paint::Plain, " ".repeat(indent)),
        piece(Paint::Carets(level), mark.to_string().repeat(width)),
    ]
}

/// The text of the gutter before a source line: the line number and `|`.
///
/// # Arguments
///
/// * `excerpt` - The source excerpt.
/// * `pad` - Spaces as wide as the widest line number.
///
/// # Returns
///
/// The gutter text.
fn gutter_text(excerpt: &Excerpt, pad: &str) -> String
{
    format!("{:>1$} | ", excerpt.line, pad.len())
}

/// The cells of the row under the span: where it starts after the gutter,
/// and how wide it is.
///
/// Under [`Carets::Flow`] the row is placed as the terminal flows the source
/// line; elsewhere, and when the terminal does not flow Burmese text or the
/// line wraps, the cells come from [`str_cells`].
///
/// # Arguments
///
/// * `excerpt` - The source excerpt.
/// * `pad` - Spaces as wide as the widest line number.
/// * `carets` - How to count cells.
///
/// # Returns
///
/// The indent and the width, in cells; the width is at least one.
fn reach(excerpt: &Excerpt, pad: &str, carets: Carets) -> (usize, usize)
{
    let before = str_cells(&excerpt.before, carets);
    let cells = (before, str_cells(&excerpt.marked, carets).max(1));
    ret_if!(carets != Carets::Flow, cells);
    let gutter = gutter_text(excerpt, pad);
    let parts = [
        Part::builder().text(&gutter).build(),
        Part::builder().text(&excerpt.before).build(),
        Part::builder().text(&excerpt.marked).bold(true).build(),
        Part::builder().text(&excerpt.after).build(),
    ];
    let found = detect().and_then(|terminal| terminal.reach(&parts, 2));
    found.map_or(cells, |at| (at.start.saturating_sub(gutter.len()), at.width))
}

/// Pieces of one label: a blank gutter line, the numbered source line, and a
/// dash row under the labelled region that ends in the label text.
///
/// # Arguments
///
/// * `excerpt` - The source excerpt of the label's span.
/// * `pad` - Spaces as wide as the widest line number.
/// * `text` - The label text.
/// * `carets` - How to count cells; [`Carets::Off`] leaves the dashes out, so
///   the row holds the label text alone.
///
/// # Returns
///
/// The label pieces.
fn label_pieces(excerpt: &Excerpt, pad: &str, text: &str, carets: Carets)
-> Vec<Piece>
{
    let mut out = vec![piece(Paint::Gutter, format!("{pad} |\n"))];
    out.extend(source_pieces(excerpt, pad, Level::Note));
    out.push(piece(Paint::Gutter, format!("{pad} | ")));
    out.extend(dash_pieces(excerpt, pad, carets));
    out.push(piece(Paint::Plain, format!("{text}\n")));
    out
}

/// Pieces of the dashes under a labelled region, with a space after them.
///
/// # Arguments
///
/// * `excerpt` - The source excerpt of the label's span.
/// * `pad` - Spaces as wide as the widest line number.
/// * `carets` - How to count cells; [`Carets::Off`] gives no dashes.
///
/// # Returns
///
/// The indent and the dashes, or none.
fn dash_pieces(excerpt: &Excerpt, pad: &str, carets: Carets) -> Vec<Piece>
{
    ret_if!(carets == Carets::Off, Vec::new());
    let mut out = mark_pieces(excerpt, pad, Level::Note, '-', carets);
    out.push(piece(Paint::Plain, " "));
    out
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
