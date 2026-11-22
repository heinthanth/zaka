//! The shape of a file: each token and comment printed again, with the space
//! between them set by the rules of [`FmtOptions`].

use std::borrow::Cow;
use std::collections::HashMap;

use zakac_ast::stmt::BlockStmt;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_block;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::lint::LINT_CODES;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_parser::load::parse_in_place;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::filename::FileName;
use zakac_span::pos::RelativePos;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

use crate::breaks::Gap;
use crate::breaks::Ruler;
use crate::breaks::Shape;
use crate::breaks::Word;
use crate::breaks::plan;
use crate::breaks::shapes;
use crate::check::same_words;
use crate::options::ContinuationIndent;
use crate::options::FmtOptions;
use crate::options::ImportOrder;
use crate::rewrite::Swap;
use crate::rewrite::sort_imports;
use crate::rewrite::swapped;
use crate::rewrite::swaps;
use crate::wrap::wrap;

/// The reason a file was left as it is.
#[derive(Debug)]
pub enum FmtError
{
    /// The file does not lex or parse; the errors, with spans in the source
    /// map the file was read into.
    Unparsed(Vec<Diagnostic>),
    /// The text the formatter wrote reads as other words than the file; a
    /// fault of the formatter, so nothing is written.
    Changed,
    /// A file the file imports could not be read.
    Unread(zakac_host_errors::Error),
}

/// Format a file.
///
/// # Arguments
///
/// * `source_map` - The map the file is read into; the spans of the errors
///   point into it.
/// * `name` - The name of the file.
/// * `text` - The text of the file.
/// * `options` - The settings.
///
/// # Returns
///
/// The formatted text, or the reason the file is left as it is.
pub fn format(
    source_map: &mut SourceMap, name: FileName, text: &str, options: &FmtOptions,
) -> Result<String, FmtError>
{
    let file = source_map.new_source_file(name.clone(), text.to_string());
    let lexer = LexerOptions::default();
    let lexed = lex(&file, &lexer);
    let parsed = parse_in_place(&file, source_map, &lexer).map_err(FmtError::Unread)?;
    // The errors of the files it imports leave this file as it is not.
    let own =
        |d: &&Diagnostic| file.start_pos <= d.span.lo && d.span.lo <= file.end_pos();
    let found = parsed.diagnostics.iter().filter(own);
    let refused: Vec<Diagnostic> = found.filter(|d| refuses(d)).cloned().collect();
    ret_if!(!refused.is_empty(), Err(FmtError::Unparsed(refused)));
    // Sorted imports move whole lines: the moved text is formatted
    // as a file of its own.
    let sorted = (options.import_order == ImportOrder::Sorted)
        .then(|| sort_imports(&file, text, &lexed, &parsed.stmts))
        .flatten();
    if let Some(sorted) = sorted
    {
        let rest =
            FmtOptions { import_order: ImportOrder::Preserve, ..options.clone() };
        return format(source_map, name, &sorted, &rest);
    }
    let swaps = swaps(&file, &lexed, &parsed, options);
    let mut blocks = Blocks::default();
    do_for!(s in &parsed.stmts, blocks.visit_stmt(s));
    let printer = Printer::builder()
        .file(&file)
        .original(text)
        .lexed(&lexed)
        .blocks(lines_of_blocks(&file, &blocks.spans))
        .swaps(swaps.clone())
        .shapes(shapes(&parsed.stmts, options))
        .options(options)
        .build();
    let out = printer.print()?;
    ret_if!(!same_words(&file, &lexed, &swaps, &out), Err(FmtError::Changed));
    Ok(out)
}

/// Format text that comes from no file, in a source map of its own.
///
/// # Arguments
///
/// * `text` - The text.
/// * `options` - The settings.
///
/// # Returns
///
/// The formatted text, or the reason the text is left as it is.
pub fn format_text(text: &str, options: &FmtOptions) -> Result<String, FmtError>
{
    let name = FileName::Custom("fmt".to_string());
    format(&mut SourceMap::new(), name, text, options)
}

/// Whether a diagnostic keeps a file from being formatted: an error the lint
/// cannot turn down, so a file in spoken words is still formatted.
///
/// # Arguments
///
/// * `diag` - The diagnostic.
///
/// # Returns
///
/// `true` for such an error.
pub fn refuses(diag: &Diagnostic) -> bool
{
    diag.is_error() && !LINT_CODES.contains(&diag.code)
}

/// The spans of the blocks written with their opener and closer.
#[derive(Default)]
struct Blocks
{
    spans: Vec<Span>,
}

impl Visitor for Blocks
{
    fn visit_block(&mut self, n: &BlockStmt)
    {
        do_if!(!n.single, self.spans.push(n.span));
        walk_block(self, n);
    }
}

/// The line of the opener and the line of the closer of each block.
///
/// # Arguments
///
/// * `file` - The file.
/// * `spans` - The blocks.
///
/// # Returns
///
/// The pairs of lines.
fn lines_of_blocks(file: &SourceFile, spans: &[Span]) -> Vec<(usize, usize)>
{
    let line = |pos| line_at(file, relative(file, pos));
    spans.iter().map(|s| (line(s.lo), line(s.hi))).collect()
}

/// The line that holds an offset of a file.
///
/// # Arguments
///
/// * `file` - The file.
/// * `at` - The offset.
///
/// # Returns
///
/// The line, from 0.
pub(crate) fn line_at(file: &SourceFile, at: usize) -> usize
{
    file.lookup_line(&RelativePos::new(at as u32)).unwrap_or(0)
}

/// The offset of a position in a file.
///
/// # Arguments
///
/// * `file` - The file.
/// * `pos` - The position.
///
/// # Returns
///
/// The offset.
pub(crate) fn relative(file: &SourceFile, pos: zakac_span::pos::Pos) -> usize
{
    file.relative_position(&pos).as_u32() as usize
}

/// One token or comment of the file, with its offsets in the normalized text.
#[derive(Debug, Clone, Copy)]
struct Piece
{
    lo: usize,
    hi: usize,
    /// No space stands between the piece and the one before it.
    joined: bool,
    /// The piece is `။`, which ends a statement.
    stop: bool,
    comment: bool,
    /// Its kind, for a token.
    kind: Option<TokenKind>,
}

/// The tokens and comments of a file, in the order of the text.
///
/// # Arguments
///
/// * `file` - The file.
/// * `lexed` - Its tokens and comments.
///
/// # Returns
///
/// The pieces.
fn pieces(file: &SourceFile, lexed: &LexOutput) -> Vec<Piece>
{
    let at = |span: Span| (relative(file, span.lo), relative(file, span.hi));
    let stop = TokenKind::Punct(Punct::FullStop);
    let words = lexed.tokens.iter().filter(|t| t.kind != TokenKind::Eof).map(|t| {
        let (lo, hi) = at(t.span);
        let kind = Some(t.kind);
        Piece { lo, hi, joined: t.joined, stop: t.kind == stop, comment: false, kind }
    });
    let notes = lexed.comments.iter().map(|c| {
        let (lo, hi) = at(c.span);
        // A comment runs to the end of its line, spaces there too.
        let hi = lo + file.src[lo..hi].trim_end().len();
        Piece { lo, hi, joined: false, stop: false, comment: true, kind: None }
    });
    let mut all: Vec<Piece> = words.chain(notes).collect();
    all.sort_by_key(|p| p.lo);
    all
}

/// The state of printing one file.
#[derive(bon::Builder)]
struct Printer<'a>
{
    file: &'a SourceFile,
    /// The text as read, before it was normalized.
    original: &'a str,
    lexed: &'a LexOutput,
    /// The line of the opener and the closer of each block.
    blocks: Vec<(usize, usize)>,
    /// The words the rewrites swap in, in the order of the text.
    swaps: Vec<Swap>,
    /// The declarations whose lines the formatter lays out.
    shapes: Vec<Shape>,
    options: &'a FmtOptions,
}

impl Printer<'_>
{
    /// Print the file.
    ///
    /// # Returns
    ///
    /// The text, or [`FmtError::Changed`] when the text between two pieces
    /// holds more than space.
    fn print(&self) -> Result<String, FmtError>
    {
        let pieces = pieces(self.file, self.lexed);
        let texts: Vec<Cow<'_, str>> = pieces.iter().map(|p| self.swapped(p)).collect();
        let gaps = self.gaps(&pieces, &texts);
        let mut out = String::with_capacity(self.file.src.len());
        // Whether the next token starts a statement.
        let mut starts = true;
        for (i, piece) in pieces.iter().enumerate()
        {
            let prev = i.checked_sub(1).map(|j| pieces[j]);
            let gap = prev.map_or("", |p| &self.file.src[p.hi..piece.lo]);
            ret_if!(!gap.trim().is_empty(), Err(FmtError::Changed));
            let forced = gaps.get(&i).copied();
            let breaks = gap.matches('\n').count();
            let split = prev.is_some_and(|p| self.splits(p, piece, breaks));
            // A declaration laid out by the formatter sets its own gaps.
            let (breaks, split) = match forced
            {
                Some(Gap::Join) => (0, false),
                Some(Gap::Break) => (1, false),
                None => (breaks, split),
            };
            match prev
            {
                Some(_) if breaks == 0 && !split =>
                {
                    do_if!(!piece.joined, out.push(' '));
                }
                Some(_) =>
                {
                    let blank =
                        breaks.saturating_sub(1).min(self.options.blank_lines_max);
                    out.extend(std::iter::repeat_n('\n', blank + 1));
                    let made = forced == Some(Gap::Break);
                    let lead = tern!(
                        made,
                        self.made_lead(piece),
                        self.lead(&pieces[i..], starts)
                    );
                    out.push_str(&lead);
                }
                None => out.push_str(&self.lead(&pieces[i..], starts)),
            }
            out.push_str(&texts[i]);
            do_if!(!piece.comment, starts = piece.stop);
        }
        do_if!(!out.is_empty(), out.push('\n'));
        Ok(out)
    }

    /// The gaps the declarations decide, keyed on the index of the piece after
    /// each.
    ///
    /// # Arguments
    ///
    /// * `pieces` - The pieces of the file.
    /// * `texts` - The text each piece is printed with.
    ///
    /// # Returns
    ///
    /// The gaps.
    fn gaps(&self, pieces: &[Piece], texts: &[Cow<'_, str>]) -> HashMap<usize, Gap>
    {
        let base = self.file.start_pos;
        let broken = |i: usize| {
            let from = i.checked_sub(1).map_or(0, |j| pieces[j].hi);
            i > 0 && self.file.src[from..pieces[i].lo].contains('\n')
        };
        let words: Vec<Word<'_>> = pieces
            .iter()
            .zip(texts)
            .enumerate()
            .map(|(i, (p, text))| Word {
                at: base + p.lo,
                text,
                joined: p.joined,
                comment: p.comment,
                kind: p.kind,
                broken: broken(i),
            })
            .collect();
        let lead = |i: usize| {
            self.depth(line_at(self.file, pieces[i].lo)) * self.options.indent
        };
        let made = |i: usize| self.made_lead(&pieces[i]).len();
        let ruler = Ruler {
            words: &words,
            lead: &lead,
            made: &made,
            max_width: self.options.max_width,
        };
        let (mut gaps, owned) = plan(&self.shapes, &ruler);
        ret_if!(!self.options.wrap_long_lines, gaps);
        // Whether a piece starts a line with the gaps so far: a break the
        // plan decides, else a line break as written or between two
        // statements that share a line.
        let starts_line = |i: usize, gaps: &HashMap<usize, Gap>| match gaps.get(&i)
        {
            Some(gap) => *gap == Gap::Break,
            None => words[i].broken || self.splits(pieces[i - 1], &pieces[i], 0),
        };
        wrap(&ruler, &mut gaps, &owned, &starts_line);
        gaps
    }

    /// The space at the start of a line the formatter breaks inside a
    /// declaration: the depth of the line, and the count of
    /// `continuation-indent` when it gives one.
    ///
    /// # Arguments
    ///
    /// * `piece` - The first piece of the line.
    ///
    /// # Returns
    ///
    /// The space.
    fn made_lead(&self, piece: &Piece) -> String
    {
        let depth = self.depth(line_at(self.file, piece.lo)) * self.options.indent;
        let more = match self.options.continuation_indent
        {
            ContinuationIndent::Spaces(n) => n,
            ContinuationIndent::Preserve => 0,
        };
        " ".repeat(depth + more)
    }

    /// Whether a line break goes between two pieces that share a line: the
    /// first ends a statement and the second starts the next one.
    ///
    /// # Arguments
    ///
    /// * `prev` - The first piece.
    /// * `next` - The second piece.
    /// * `breaks` - The line breaks between them as written.
    ///
    /// # Returns
    ///
    /// `true` for a break.
    fn splits(&self, prev: Piece, next: &Piece, breaks: usize) -> bool
    {
        let shared = breaks == 0 && prev.stop && !next.comment && !next.joined;
        self.options.one_statement_per_line && shared
    }

    /// The space at the start of a line.
    ///
    /// # Arguments
    ///
    /// * `rest` - The pieces from the first piece of the line on.
    /// * `starts` - Whether the line starts a statement.
    ///
    /// # Returns
    ///
    /// The space.
    fn lead(&self, rest: &[Piece], starts: bool) -> String
    {
        let first = rest[0];
        // A comment on a line of its own between statements takes the depth
        // of the statement after it.
        let shown = rest.iter().find(|p| !p.comment || !starts).unwrap_or(&first);
        let depth = self.depth(line_at(self.file, shown.lo)) * self.options.indent;
        ret_if!(starts, " ".repeat(depth));
        match self.options.continuation_indent
        {
            ContinuationIndent::Preserve => self.written_lead(first.lo).to_string(),
            ContinuationIndent::Spaces(n) => " ".repeat(depth + n),
        }
    }

    /// The count of blocks a line stands inside: below the line of a block's
    /// opener and above the line of its closer.
    ///
    /// # Arguments
    ///
    /// * `line` - The line.
    ///
    /// # Returns
    ///
    /// The count.
    fn depth(&self, line: usize) -> usize
    {
        self.blocks.iter().filter(|(open, close)| *open < line && line < *close).count()
    }

    /// The space the author put at the start of the line of an offset.
    ///
    /// # Arguments
    ///
    /// * `at` - The offset.
    ///
    /// # Returns
    ///
    /// The space.
    fn written_lead(&self, at: usize) -> &str
    {
        let start = self.file.src[..at].rfind('\n').map_or(0, |n| n + 1);
        &self.file.src[start..at]
    }

    /// The text of a piece, with the words the rewrites swap in.
    ///
    /// # Arguments
    ///
    /// * `piece` - The piece.
    ///
    /// # Returns
    ///
    /// The text.
    fn swapped(&self, piece: &Piece) -> Cow<'_, str>
    {
        let inside = |s: &Swap| piece.lo <= s.lo && s.hi <= piece.hi;
        ret_if!(!self.swaps.iter().any(inside), Cow::Borrowed(self.text_of(piece)));
        // A piece with a swap is written from the normalized text, as the word
        // swapped in is.
        swapped(&self.file.src, piece.lo, piece.hi, &self.swaps)
    }

    /// The text of a piece: from the normalized text, or as written when the
    /// marks keep their order.
    ///
    /// # Arguments
    ///
    /// * `piece` - The piece.
    ///
    /// # Returns
    ///
    /// The text.
    fn text_of(&self, piece: &Piece) -> &str
    {
        ret_if!(self.options.mark_order, &self.file.src[piece.lo..piece.hi]);
        let at = |n: usize| {
            let pos = self.file.original_relative_pos(&RelativePos::new(n as u32));
            pos.as_u32() as usize
        };
        let (lo, hi) = (at(piece.lo), at(piece.hi));
        tern!(hi <= self.original.len(), &self.original[lo..hi], "")
    }
}
