//! Where a terminal that draws Burmese text at the natural width of its font
//! puts each part of a row. That width is often wider than the cells the
//! terminal gave the text, and the terminal flows the row to make room
//! (saturngod's Ghostty fork is one such terminal):
//!
//! - A row splits into runs: a run ends where the paint changes, and where the
//!   font changes between Burmese text and other text. A space after Burmese
//!   text remains in the Burmese run.
//! - A Burmese run is shaped as a whole and split into orthographic units: a
//!   consonant with its signs, its stacked consonants and the killed consonants
//!   that close it.
//! - The first unit of a word starts at the grid column of its first cell, or
//!   right after the text before it when that text is still being drawn there.
//!   The other units of the word follow at their natural spacing. A space is
//!   drawn at least one cell wide.
//! - A run starts at its grid column, or right after the run before it when
//!   that run is still being drawn there.
//! - When the row ends past the right edge of the window, the row from its
//!   first Burmese unit on is squeezed so that it ends at the edge.
//!
//! The caret row under a source line holds no Burmese text, so it remains on
//! the grid; [`reach`] tells which cells of that row sit under a part.

use bon::Builder;
use unicode_width::UnicodeWidthChar;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;

/// The virama: the consonant after it is stacked under the one before it.
const VIRAMA: char = '\u{1039}';
/// The asat: it kills the vowel of the consonant before it.
const ASAT: char = '\u{103A}';
/// The dot below, which can come between a consonant and its asat.
const DOT_BELOW: char = '\u{1037}';

/// A stretch of a row in one paint.
#[derive(Debug, Clone, Copy, Builder)]
pub struct Part<'a>
{
    /// The text, with no line break.
    pub text: &'a str,
    /// Whether the text is drawn bold.
    #[builder(default)]
    pub bold: bool,
}

/// The cell grid of the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Builder)]
pub struct Grid
{
    /// Width of one cell, in pixels.
    pub cell: f64,
    /// Number of columns in a row.
    pub cols: usize,
}

/// One drawn sign that a shaper gives back, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Builder)]
pub struct Drawn
{
    /// Octet offset, in the shaped text, of the character it comes from.
    pub at: usize,
    /// How far the pen moves after it.
    pub advance: f64,
    /// How far it is drawn from the pen.
    #[builder(default)]
    pub offset: f64,
}

/// A terminal that flows Burmese text: its grid, and the shaper of its
/// Burmese font.
#[derive(Builder)]
pub struct Flow
{
    /// The cell grid.
    pub grid: Grid,
    /// Shapes the Burmese runs as the terminal does.
    pub shaper: Box<dyn Shaper + Send + Sync>,
}

impl Flow
{
    /// Tell which cells of a caret row sit under a part of the row above.
    ///
    /// # Arguments
    ///
    /// * `parts` - The parts of the row, in order.
    /// * `marked` - Index of the part.
    ///
    /// # Returns
    ///
    /// The reach, or `None` when the row wraps.
    pub fn reach(&self, parts: &[Part], marked: usize) -> Option<Reach>
    {
        reach(self.shaper.as_ref(), self.grid, parts, marked)
    }
}

/// Shapes Burmese text with the font the terminal draws it with.
pub trait Shaper
{
    /// Shape a run of text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text of the run.
    /// * `bold` - Whether the run is drawn bold.
    ///
    /// # Returns
    ///
    /// The drawn signs in visual order.
    fn shape(&self, text: &str, bold: bool) -> Vec<Drawn>;
}

/// The cells of a caret row that sit under a part of the row above.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct Reach
{
    /// The first cell, counted from the start of the row.
    pub start: usize,
    /// The number of cells, at least one.
    pub width: usize,
}

/// One cell of the row: a character and the zero-width signs after it.
#[derive(Debug, Clone, Copy)]
struct Cell
{
    /// Index of the part that holds it.
    part: usize,
    /// Octet range of its text in the part.
    lo: usize,
    hi: usize,
    /// Its first column.
    col: usize,
    /// The columns it covers.
    cols: usize,
    /// Whether its first character is Burmese.
    burmese: bool,
    /// Whether it is a space.
    space: bool,
}

/// A run: cells in one paint and one font.
#[derive(Debug, Clone, Copy)]
struct Run
{
    /// Index of the part that holds it.
    part: usize,
    /// Range of its cells.
    first: usize,
    end: usize,
    /// Whether it is drawn with the Burmese font.
    burmese: bool,
}

/// Where a run is drawn, in pixels from the start of the row.
#[derive(Debug, Clone)]
struct Placed
{
    /// Index of the part that holds it.
    part: usize,
    /// Where it starts and ends.
    start: f64,
    end: f64,
    /// The units of a Burmese run, as start and end; none for other text.
    units: Vec<(f64, f64)>,
}

/// Whether `ch` is a Burmese (Myanmar script) character.
///
/// # Arguments
///
/// * `ch` - The character.
///
/// # Returns
///
/// `true` for a character of the Myanmar blocks.
pub fn is_burmese(ch: char) -> bool
{
    matches!(
        ch,
        '\u{1000}'..='\u{109F}'
            | '\u{A9E0}'..='\u{A9FF}'
            | '\u{AA60}'..='\u{AA7F}'
            | '\u{116D0}'..='\u{116FF}'
    )
}

/// Whether `ch` is a Burmese sign that joins the unit before it.
///
/// # Arguments
///
/// * `ch` - The character.
///
/// # Returns
///
/// `true` for a vowel sign, a medial, a tone mark or a similar sign.
fn is_sign(ch: char) -> bool
{
    matches!(
        ch,
        '\u{102B}'..='\u{103E}'
            | '\u{1056}'..='\u{1059}'
            | '\u{105E}'..='\u{1060}'
            | '\u{1062}'..='\u{1064}'
            | '\u{1067}'..='\u{106D}'
            | '\u{1071}'..='\u{1074}'
            | '\u{1082}'..='\u{108D}'
            | '\u{108F}'
            | '\u{109A}'..='\u{109D}'
            | '\u{A9E5}'
            | '\u{AA7B}'..='\u{AA7D}'
    )
}

/// Whether `ch` joins the text before it: a zero-width joiner or a variation
/// selector.
///
/// # Arguments
///
/// * `ch` - The character.
///
/// # Returns
///
/// `true` for a joiner.
fn is_joiner(ch: char) -> bool
{
    matches!(ch, '\u{200C}' | '\u{200D}' | '\u{FE00}'..='\u{FE0F}')
}

/// Whether the character at `idx` starts a new unit.
///
/// A killed consonant (a consonant, then an asat) closes the unit before
/// it, as `င်` closes `ကြောင်`; kinzi (`င်္`) belongs to the consonant after
/// it, so it starts a unit.
///
/// # Arguments
///
/// * `chars` - The characters of the run.
/// * `idx` - The index to check.
///
/// # Returns
///
/// `true` if a unit starts there.
pub fn starts_unit(chars: &[char], idx: usize) -> bool
{
    let ch = chars[idx];
    let Some(&prev) = idx.checked_sub(1).and_then(|before| chars.get(before))
    else
    {
        return true;
    };
    ret_true!(!is_burmese(prev));
    ret_if!(!is_burmese(ch), !is_joiner(ch));
    ret_false!(prev == VIRAMA || is_sign(ch));
    let rest = &chars[idx + 1..];
    let after_dots = rest.iter().position(|c| *c != DOT_BELOW).unwrap_or(rest.len());
    ret_true!(rest.get(after_dots) != Some(&ASAT));
    rest.get(after_dots + 1) == Some(&VIRAMA)
}

/// Find the cells of `marked` in the row and tell which cells of a caret row
/// sit under it.
///
/// # Arguments
///
/// * `shaper` - Shapes the Burmese runs.
/// * `grid` - The cell grid.
/// * `parts` - The parts of the row, in order.
/// * `marked` - Index of the part to reach.
///
/// # Returns
///
/// The reach, or `None` when the row is wider than the window and wraps.
pub fn reach(
    shaper: &(impl Shaper + ?Sized), grid: Grid, parts: &[Part], marked: usize,
) -> Option<Reach>
{
    let cells = cells(parts);
    let cols = cells.last().map_or(0, |cell| cell.col + cell.cols);
    ret_if!(cols > grid.cols, None);
    let mut placed = place(shaper, grid, parts, &cells);
    squeeze(&mut placed, grid);
    let (start, end) = span_of(&placed, &cells, grid, marked);
    let width = ((end - start) / grid.cell).round().max(1.0) as usize;
    Some(
        Reach::builder()
            .start((start / grid.cell).round() as usize)
            .width(width)
            .build(),
    )
}

/// Split the parts into cells; a zero-width sign joins the cell before it
/// in the same part.
///
/// # Arguments
///
/// * `parts` - The parts of the row.
///
/// # Returns
///
/// The cells in order.
fn cells(parts: &[Part]) -> Vec<Cell>
{
    let mut out: Vec<Cell> = Vec::new();
    let mut col = 0;
    for (part, (at, ch)) in parts
        .iter()
        .enumerate()
        .flat_map(|(idx, p)| p.text.char_indices().map(move |each| (idx, each)))
    {
        let cols = ch.width().unwrap_or(0);
        let joins = cols == 0 && out.last().is_some_and(|last| last.part == part);
        if joins
        {
            out.last_mut().unwrap().hi = at + ch.len_utf8();
            continue;
        }
        let cell = Cell {
            part,
            lo: at,
            hi: at + ch.len_utf8(),
            col,
            cols,
            burmese: is_burmese(ch),
            space: ch == ' ',
        };
        out.push(cell);
        col += cols;
    }
    out
}

/// Split the cells into runs.
///
/// # Arguments
///
/// * `cells` - The cells of the row.
///
/// # Returns
///
/// The runs in order.
fn runs(cells: &[Cell]) -> Vec<Run>
{
    let mut out: Vec<Run> = Vec::new();
    for (idx, cell) in cells.iter().enumerate()
    {
        let current = out.last().filter(|run| run.part == cell.part);
        let burmese =
            cell.burmese || (cell.space && current.is_some_and(|run| run.burmese));
        match out
            .last_mut()
            .filter(|run| run.part == cell.part && run.burmese == burmese)
        {
            Some(run) => run.end = idx + 1,
            None =>
            {
                out.push(Run { part: cell.part, first: idx, end: idx + 1, burmese })
            }
        }
    }
    out
}

/// Place each run of the row, before the squeeze.
///
/// # Arguments
///
/// * `shaper` - Shapes the Burmese runs.
/// * `grid` - The cell grid.
/// * `parts` - The parts of the row.
/// * `cells` - The cells of the row.
///
/// # Returns
///
/// Where each run is drawn.
fn place(
    shaper: &(impl Shaper + ?Sized), grid: Grid, parts: &[Part], cells: &[Cell],
) -> Vec<Placed>
{
    let mut end = 0.0_f64;
    let mut out = Vec::new();
    for run in runs(cells)
    {
        let first = cells[run.first];
        let grid_x = first.col as f64 * grid.cell;
        let base = grid_x + (end - grid_x).max(0.0);
        let mut placed = tern!(
            run.burmese,
            place_burmese(shaper, grid, parts[run.part], &cells[run.first..run.end]),
            place_plain(grid, &cells[run.first..run.end])
        );
        placed.start += base;
        placed.end += base;
        placed.units.iter_mut().for_each(|unit| *unit = (unit.0 + base, unit.1 + base));
        end = placed.end;
        out.push(placed);
    }
    out
}

/// Place a run of text in the main font: it covers its cells.
///
/// # Arguments
///
/// * `grid` - The cell grid.
/// * `cells` - The cells of the run.
///
/// # Returns
///
/// The run, from zero.
fn place_plain(grid: Grid, cells: &[Cell]) -> Placed
{
    let cols: usize = cells.iter().map(|cell| cell.cols).sum();
    Placed {
        part: cells[0].part,
        start: 0.0,
        end: cols as f64 * grid.cell,
        units: Vec::new(),
    }
}

/// Place a Burmese run, one unit at a time.
///
/// # Arguments
///
/// * `shaper` - Shapes the run.
/// * `grid` - The cell grid.
/// * `part` - The part that holds the run.
/// * `cells` - The cells of the run.
///
/// # Returns
///
/// The run, from zero.
fn place_burmese(
    shaper: &(impl Shaper + ?Sized), grid: Grid, part: Part, cells: &[Cell],
) -> Placed
{
    let first = cells[0];
    let text = &part.text[first.lo..cells[cells.len() - 1].hi];
    // the run column of each character, with its octet offset in `text`
    let chars: Vec<(usize, char, usize)> = cells
        .iter()
        .flat_map(|cell| {
            let col = cell.col - first.col;
            let lo = cell.lo - first.lo;
            part.text[cell.lo..cell.hi]
                .char_indices()
                .map(move |(at, ch)| (lo + at, ch, col))
        })
        .collect();
    let spaces: Vec<usize> = cells
        .iter()
        .filter(|cell| cell.space)
        .map(|cell| cell.col - first.col)
        .collect();
    let (starts, snaps) = units(&chars);
    let col_at = |at: usize| {
        let idx = chars.partition_point(|(lo, _, _)| *lo <= at).saturating_sub(1);
        chars[idx].2
    };

    let mut pen = 0.0_f64;
    let mut pen_end = 0.0_f64;
    let mut ink_end = 0.0_f64;
    let mut shift = 0.0_f64;
    let mut unit: Option<usize> = None;
    let mut out: Vec<(f64, f64)> = Vec::new();
    for drawn in shaper.shape(text, part.bold)
    {
        let col = col_at(drawn.at);
        let start_col = starts[col];
        if unit.is_none_or(|current| start_col > current)
        {
            let grid_x = start_col as f64 * grid.cell;
            let start = tern!(snaps[start_col], grid_x.max(pen_end), pen_end);
            shift = start - pen;
            unit = Some(start_col);
            out.push((f64::MAX, f64::MIN));
        }
        let left = pen + drawn.offset + shift;
        let advance =
            tern!(spaces.contains(&col), drawn.advance.max(grid.cell), drawn.advance);
        pen_end = pen_end.max(pen + advance + shift);
        ink_end = ink_end.max(left + advance);
        let last = out.last_mut().unwrap();
        *last = (last.0.min(left), last.1.max(left + advance));
        pen += drawn.advance;
    }
    let units = out
        .into_iter()
        .map(|(lo, hi)| (lo, tern!(hi <= lo, lo + grid.cell, hi)))
        .collect();
    Placed { part: first.part, start: 0.0, end: ink_end.max(0.0), units }
}

/// Mark where each unit of a Burmese run starts, and which units start a
/// word and so snap to the grid.
///
/// # Arguments
///
/// * `chars` - The octet offset, character and run column of each character.
///
/// # Returns
///
/// For each run column, the column where its unit starts; and for each
/// column that starts a unit, whether it snaps.
fn units(chars: &[(usize, char, usize)]) -> (Vec<usize>, Vec<bool>)
{
    let cols = chars.last().map_or(0, |last| last.2 + 1);
    let plain: Vec<char> = chars.iter().map(|each| each.1).collect();
    let mut starts = vec![0; cols];
    let mut snaps = vec![true; cols];
    let mut current = 0;
    let mut prev: Option<(char, usize)> = None;
    for (idx, &(_, ch, col)) in chars.iter().enumerate()
    {
        let new_cell = prev.is_none_or(|(_, prev_col)| prev_col != col);
        if new_cell && (prev.is_none() || starts_unit(&plain, idx))
        {
            current = col;
            snaps[col] = prev
                .is_none_or(|(prev_ch, _)| !(is_burmese(prev_ch) && is_burmese(ch)));
        }
        starts[col] = current;
        prev = Some((ch, col));
    }
    (starts, snaps)
}

/// Squeeze a row that ends past the right edge of the window, from its first
/// Burmese unit on, so that it ends at the edge; a squeeze takes at most a
/// fifth where the row allows it.
///
/// # Arguments
///
/// * `placed` - The runs of the row; their positions are changed in place.
/// * `grid` - The cell grid.
fn squeeze(placed: &mut [Placed], grid: Grid)
{
    let width = grid.cols as f64 * grid.cell;
    let Some(anchor) =
        placed.iter().find_map(|run| run.units.first().map(|unit| unit.0))
    else
    {
        return;
    };
    // the end of the row, and the width of what is drawn last
    let mut end = 0.0_f64;
    let mut last_width = grid.cell;
    for run in placed.iter()
    {
        let plain = run.units.is_empty();
        let stretches = tern!(plain, vec![(run.start, run.end)], run.units.clone());
        for (lo, hi) in stretches
        {
            do_if!(hi >= end, end = hi, last_width = tern!(plain, grid.cell, hi - lo));
        }
    }
    ret_if!(end <= width);
    let anchor = anchor.min(5.0 * width - 4.0 * end - last_width).max(0.0);
    let num = width - last_width - anchor;
    let den = end - last_width - anchor;
    ret_if!(num <= 0.0 || den <= 0.0);
    let moved = |px: f64| tern!(px <= anchor, px, anchor + (px - anchor) * num / den);
    for run in placed.iter_mut()
    {
        run.start = moved(run.start);
        run.end = moved(run.end);
        run.units.iter_mut().for_each(|unit| *unit = (moved(unit.0), moved(unit.1)));
    }
}

/// Where a part is drawn, in pixels.
///
/// # Arguments
///
/// * `placed` - The runs of the row.
/// * `cells` - The cells of the row.
/// * `grid` - The cell grid.
/// * `marked` - Index of the part.
///
/// # Returns
///
/// Its start and end; a part with no text starts where its first cell
/// would be drawn, and ends there too.
fn span_of(placed: &[Placed], cells: &[Cell], grid: Grid, marked: usize) -> (f64, f64)
{
    let mut runs = placed.iter().filter(|run| run.part == marked);
    if let Some(first) = runs.next()
    {
        let last = runs.last().unwrap_or(first);
        return (first.start, last.end);
    }
    let col: usize =
        cells.iter().filter(|cell| cell.part < marked).map(|cell| cell.cols).sum();
    let before = placed
        .iter()
        .filter(|run| run.part < marked)
        .map(|run| run.end)
        .fold(0.0, f64::max);
    let start = (col as f64 * grid.cell).max(before);
    (start, start)
}
