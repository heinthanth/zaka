//! The lines of a declaration: a struct with a field on each line, an
//! enumeration with a member on each line, and a function signature with its
//! name on a line, a parameter on each line and its opener on a line of its
//! own. A separator (`၊`, `နှင့်`, `လည်းကောင်း`) ends its line.
//!
//! With [`Lines::Fit`], a declaration that fits in the widest line goes on
//! one line, and one that does not is broken; with [`Lines::Split`] it is
//! broken, short or long. Breaking a line between two words that the file
//! keeps apart changes nothing the parser reads, as the parser reads no line
//! breaks; a word joined to the one before it is never moved to a line of its
//! own.

use std::collections::HashMap;

use zakac_ast::stmt::BlockStmt;
use zakac_ast::stmt::EnumDecl;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::StructDecl;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_block;
use zakac_ast::visit::walk_fn;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::pos::Pos;
use zakac_span::span::Span;

use crate::options::FmtOptions;
use crate::options::Lines;
use crate::wrap::wrap_line;

/// What the space before a piece becomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gap
{
    /// One space, or none before a joined word: the two share a line.
    Join,
    /// A line break, with no blank line.
    Break,
}

/// A declaration, or a block written as one statement, with its positions:
/// where it starts and ends, and where each of its lines starts after the
/// first.
#[derive(Debug, Clone)]
pub struct Shape
{
    lo: Pos,
    hi: Pos,
    /// The start of each line after the first: a break goes before the first
    /// piece at or after each.
    cuts: Vec<Pos>,
    /// The pieces a second break goes after, when a line is still too wide:
    /// the opener of a function that gives a result.
    soft_after: Vec<Pos>,
    lines: Lines,
    /// A block written as one statement: the formatter decides the gap
    /// before the statement alone, and leaves the other lines as written.
    inline: bool,
}

impl Shape
{
    /// A shape whose gaps the formatter decides.
    ///
    /// # Arguments
    ///
    /// * `span` - Where it starts and ends.
    /// * `cuts` - The start of each line after the first.
    /// * `lines` - Its lines.
    ///
    /// # Returns
    ///
    /// The shape.
    fn owned(span: Span, cuts: Vec<Pos>, lines: Lines) -> Self
    {
        Shape {
            lo: span.lo,
            hi: span.hi,
            cuts,
            soft_after: Vec::new(),
            lines,
            inline: false,
        }
    }
}

/// A token or a comment as the plan of breaks reads it.
#[derive(Debug, Clone, Copy)]
pub struct Word<'a>
{
    /// Its absolute position.
    pub at: Pos,
    /// Its text as it will be printed.
    pub text: &'a str,
    /// No space stands before it.
    pub joined: bool,
    pub comment: bool,
    /// Its kind, for a token.
    pub kind: Option<TokenKind>,
    /// A line break stands before it as written.
    pub broken: bool,
}

/// The declarations and the blocks written as one statement, each with the
/// lines the settings give its kind.
///
/// # Arguments
///
/// * `stmts` - The statements of the file.
/// * `options` - The settings.
///
/// # Returns
///
/// The shapes.
pub fn shapes(stmts: &[Stmt], options: &FmtOptions) -> Vec<Shape>
{
    let mut found = Shapes { options, shapes: Vec::new() };
    do_for!(s in stmts, found.visit_stmt(s));
    found.shapes
}

/// The walk that finds the shapes.
struct Shapes<'a>
{
    options: &'a FmtOptions,
    shapes: Vec<Shape>,
}

impl Visitor for Shapes<'_>
{
    fn visit_struct(&mut self, n: &StructDecl)
    {
        let spans: Vec<Span> = n.fields.iter().map(|f| f.span).collect();
        self.list(n.span, &spans, self.options.struct_lines);
    }

    fn visit_enum(&mut self, n: &EnumDecl)
    {
        let spans: Vec<Span> = n.members.iter().map(|m| m.span).collect();
        self.list(n.span, &spans, self.options.enum_lines);
    }

    fn visit_fn(&mut self, n: &FnDecl)
    {
        // The head ends at the stop after the opener; the block is not part
        // of it.
        let mut cuts: Vec<Pos> = n.params.iter().map(|p| p.span.lo).collect();
        cuts.push(n.block.span.lo);
        let head = Span::new(n.span.lo, n.block.span.lo);
        let mut shape = Shape::owned(head, cuts, self.options.signature_lines);
        shape.soft_after.push(n.block.span.lo);
        self.shapes.push(shape);
        walk_fn(self, n);
    }

    fn visit_block(&mut self, n: &BlockStmt)
    {
        let lines = self.options.inline_block_lines;
        let inline =
            Shape { inline: true, ..Shape::owned(n.span, vec![n.span.lo], lines) };
        do_if!(n.single, self.shapes.push(inline));
        walk_block(self, n);
    }
}

impl Shapes<'_>
{
    /// A declaration of a list: its head on a line, an item on each line,
    /// then its end on a line of its own.
    ///
    /// # Arguments
    ///
    /// * `whole` - The span of the declaration.
    /// * `items` - The span of each item.
    /// * `lines` - The lines of its kind.
    fn list(&mut self, whole: Span, items: &[Span], lines: Lines)
    {
        ret_if!(items.is_empty());
        let mut cuts: Vec<Pos> = items.iter().map(|s| s.lo).collect();
        cuts.extend(items.last().map(|s| s.hi));
        self.shapes.push(Shape::owned(whole, cuts, lines));
    }
}

/// What the plan measures with: the pieces of the file, the space at the
/// start of a line and the widest line.
pub struct Ruler<'a>
{
    pub words: &'a [Word<'a>],
    /// The width of the space before a piece that starts a statement line.
    pub lead: &'a dyn Fn(usize) -> usize,
    /// The width of the space before a piece that starts a line the
    /// formatter makes.
    pub made: &'a dyn Fn(usize) -> usize,
    pub max_width: usize,
}

/// The gaps the shapes decide, keyed on the index of the piece after each.
///
/// # Arguments
///
/// * `shapes` - The shapes.
/// * `ruler` - The pieces and the widths.
///
/// # Returns
///
/// The gaps, where a gap the map leaves out keeps the line breaks as
/// written, and the pieces of each declaration, as the index of the first
/// and the last.
pub fn plan(
    shapes: &[Shape], ruler: &Ruler<'_>,
) -> (HashMap<usize, Gap>, Vec<(usize, usize)>)
{
    let mut gaps = HashMap::new();
    let mut owned = Vec::new();
    for shape in shapes
    {
        let Some((first, last)) = range(shape, ruler.words)
        else
        {
            continue;
        };
        // The span of a declaration can start after its name; its statement
        // starts after the stop or the comment before it.
        let first = tern!(shape.inline, first, statement_start(ruler.words, first));
        do_if!(!shape.inline, owned.push((first, last)));
        if shape.lines == Lines::Preserve
        {
            continue;
        }
        // A comment ends its line, so the lines around it remain as written.
        if ruler.words[first..=last].iter().any(|w| w.comment)
        {
            continue;
        }
        match shape.inline
        {
            true => gaps.extend(ruler.inline_gap(shape, first, last)),
            false => gaps.extend(ruler.owned_gaps(shape, first, last)),
        }
    }
    (gaps, owned)
}

impl Ruler<'_>
{
    /// The gaps of a declaration: a join at each gap when it fits, else a
    /// break before each of its lines, a second break after its opener on a
    /// line still too wide, then before each `သို့မဟုတ်` of a line still too
    /// wide, and last the wrap of a line still too wide.
    ///
    /// # Arguments
    ///
    /// * `shape` - The declaration.
    /// * `first` - The index of its first piece.
    /// * `last` - The index of its last piece.
    ///
    /// # Returns
    ///
    /// The gaps.
    fn owned_gaps(&self, shape: &Shape, first: usize, last: usize)
    -> Vec<(usize, Gap)>
    {
        let width = (self.lead)(first) + line_width(&self.words[first..=last]);
        let fits = shape.lines == Lines::Fit && width <= self.max_width;
        let breaks = match fits
        {
            true => Vec::new(),
            false => self.refine(shape, first, last),
        };
        let gap = |i: usize| tern!(breaks.contains(&i), Gap::Break, Gap::Join);
        (first + 1..=last).map(|i| (i, gap(i))).collect()
    }

    /// The breaks of a declaration that does not fit.
    ///
    /// # Arguments
    ///
    /// * `shape` - The declaration.
    /// * `first` - The index of its first piece.
    /// * `last` - The index of its last piece.
    ///
    /// # Returns
    ///
    /// The indices of the pieces a break goes before, in order.
    fn refine(&self, shape: &Shape, first: usize, last: usize) -> Vec<usize>
    {
        let mut breaks = cut_indices(shape, self.words, first, last);
        let soft = soft_indices(shape, self.words, first, last);
        let wide: Vec<usize> = soft
            .into_iter()
            .filter(|&i| self.too_wide(&breaks, first, last, i))
            .collect();
        breaks.extend(wide);
        breaks.sort_unstable();
        // A kind too wide breaks before its `သို့မဟုတ်` first.
        let or = |i: &usize| self.words[*i].kind == Some(TokenKind::Kw(Kw::Or));
        let ors: Vec<usize> =
            (first + 1..=last).filter(or).filter(|&i| !self.words[i].joined).collect();
        let wide: Vec<usize> = ors
            .into_iter()
            .filter(|&i| self.too_wide(&breaks, first, last, i))
            .collect();
        breaks.extend(wide);
        breaks.sort_unstable();
        let lines = self.lines(&breaks, first, last);
        let fill = lines.iter().flat_map(|&(s, e)| wrap_line(self, s, e, s != first));
        let mut all: Vec<usize> = breaks.iter().copied().chain(fill).collect();
        all.sort_unstable();
        all.dedup();
        all
    }

    /// Whether the line that holds a piece is wider than the widest line.
    ///
    /// # Arguments
    ///
    /// * `breaks` - The breaks so far.
    /// * `first` - The index of the first piece of the shape.
    /// * `last` - The index of its last piece.
    /// * `at` - The piece.
    ///
    /// # Returns
    ///
    /// `true` for a line too wide.
    fn too_wide(&self, breaks: &[usize], first: usize, last: usize, at: usize) -> bool
    {
        let lines = self.lines(breaks, first, last);
        let line = lines.iter().find(|&&(s, e)| s <= at && at < e);
        line.is_some_and(|&(s, e)| self.width(s, e, first) > self.max_width)
    }

    /// The lines of a shape between its breaks, each as the index of its
    /// first piece and the index past its last.
    ///
    /// # Arguments
    ///
    /// * `breaks` - The breaks, in order.
    /// * `first` - The index of the first piece.
    /// * `last` - The index of the last piece.
    ///
    /// # Returns
    ///
    /// The lines.
    fn lines(&self, breaks: &[usize], first: usize, last: usize)
    -> Vec<(usize, usize)>
    {
        let starts = std::iter::once(first).chain(breaks.iter().copied());
        let ends = breaks.iter().copied().chain(std::iter::once(last + 1));
        starts.zip(ends).collect()
    }

    /// The width of a line of pieces, with the space at its start.
    ///
    /// # Arguments
    ///
    /// * `start` - The index of its first piece.
    /// * `end` - The index past its last piece.
    /// * `first` - The index of the first piece of the shape.
    ///
    /// # Returns
    ///
    /// The width.
    fn width(&self, start: usize, end: usize, first: usize) -> usize
    {
        let lead = tern!(start == first, (self.lead)(start), (self.made)(start));
        lead + line_width(&self.words[start..end])
    }

    /// The gap before a statement that stands for a block: a break when the
    /// line that holds it, joined, is wider than the widest line, or when the
    /// lines are `split`; a join otherwise.
    ///
    /// # Arguments
    ///
    /// * `shape` - The block.
    /// * `first` - The index of the first piece of the statement.
    /// * `last` - The index of its last piece.
    ///
    /// # Returns
    ///
    /// The gap, or none for a statement joined to the word before it.
    fn inline_gap(
        &self, shape: &Shape, first: usize, last: usize,
    ) -> Option<(usize, Gap)>
    {
        let words = self.words;
        ret_if!(first == 0 || words[first].joined, None);
        // The line as written that ends right before the statement, and the
        // first line of the statement.
        let start = (0..first).rev().find(|&j| words[j].broken || j == 0).unwrap_or(0);
        let end = (first + 1..=last).find(|&j| words[j].broken).unwrap_or(last + 1);
        ret_if!(words[start..end].iter().any(|w| w.comment), None);
        let wide = (self.lead)(start) + line_width(&words[start..end]) > self.max_width;
        let split = shape.lines == Lines::Split;
        Some((first, tern!(wide || split, Gap::Break, Gap::Join)))
    }
}

/// The pieces a declaration takes: from its first piece to the stop that ends
/// it.
///
/// # Arguments
///
/// * `shape` - The declaration.
/// * `words` - The pieces of the file.
///
/// # Returns
///
/// The index of the first and the last piece, or `None` for a declaration
/// that holds no piece.
fn range(shape: &Shape, words: &[Word<'_>]) -> Option<(usize, usize)>
{
    let first = words.partition_point(|w| w.at < shape.lo);
    let end = words.partition_point(|w| w.at < shape.hi);
    // The span of a declaration can end before its stop or take it in.
    let from = end.saturating_sub(1).max(first);
    let stop = |w: &Word<'_>| w.kind == Some(TokenKind::Punct(Punct::FullStop));
    let last = words[from..].iter().position(stop).map(|n| from + n)?;
    (first < words.len() && first < last).then_some((first, last))
}

/// The first piece of the statement that holds a piece: the piece after the
/// stop or the comment before it.
///
/// # Arguments
///
/// * `words` - The pieces of the file.
/// * `at` - The index of the piece.
///
/// # Returns
///
/// The index of the first piece of its statement.
fn statement_start(words: &[Word<'_>], at: usize) -> usize
{
    let stop = TokenKind::Punct(Punct::FullStop);
    let ends = |w: &Word<'_>| w.comment || w.kind == Some(stop);
    words[..at].iter().rposition(ends).map_or(0, |i| i + 1)
}

/// The pieces a break goes before: the first piece at or after each cut,
/// back over the mark of the initial member before a member, and never a
/// piece joined to the one before it.
///
/// # Arguments
///
/// * `shape` - The declaration.
/// * `words` - The pieces of the file.
/// * `first` - The index of its first piece.
/// * `last` - The index of its last piece.
///
/// # Returns
///
/// The indices.
fn cut_indices(
    shape: &Shape, words: &[Word<'_>], first: usize, last: usize,
) -> Vec<usize>
{
    let initial = |i: usize| words[i].kind == Some(TokenKind::Kw(Kw::InitialMark));
    let back = |i: usize| match i > first + 1 && initial(i - 1)
    {
        true => i - 1,
        false => i,
    };
    let at = |pos: &Pos| back(words.partition_point(|w| w.at < *pos));
    let each = shape.cuts.iter().map(at);
    each.filter(|&i| first < i && i <= last && !words[i].joined).collect()
}

/// The pieces a second break goes before: the piece after each opener of a
/// function that gives a result.
///
/// # Arguments
///
/// * `shape` - The declaration.
/// * `words` - The pieces of the file.
/// * `first` - The index of its first piece.
/// * `last` - The index of its last piece.
///
/// # Returns
///
/// The indices.
fn soft_indices(
    shape: &Shape, words: &[Word<'_>], first: usize, last: usize,
) -> Vec<usize>
{
    let opener = |i: &usize| words[*i].kind == Some(TokenKind::Kw(Kw::FnOpenRet));
    let at = |pos: &Pos| words.partition_point(|w| w.at < *pos);
    let each = shape.soft_after.iter().map(at).filter(opener).map(|i| i + 1);
    each.filter(|&i| first < i && i <= last && !words[i].joined).collect()
}

/// The width of pieces on one line, in code points, as an editor counts the
/// column: each character of each text, a mark too, and one for each space
/// between two pieces that are not joined.
///
/// # Arguments
///
/// * `words` - The pieces.
///
/// # Returns
///
/// The width.
pub fn line_width(words: &[Word<'_>]) -> usize
{
    let texts: usize = words.iter().map(|w| w.text.chars().count()).sum();
    let spaces = words.iter().skip(1).filter(|w| !w.joined).count();
    texts + spaces
}
