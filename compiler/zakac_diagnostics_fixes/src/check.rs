//! The check of a fix: patch the text of the root file, compile it again and
//! compare what the compile finds.

use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Edit;
use zakac_diagnostics::Fix;
use zakac_diagnostics::FixOrigin;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_span::source_file::SourceFile;
use zakac_span::span::Span;

use crate::Recompiled;

/// The most recompiles one run makes.
///
/// shortcut: one budget for the whole run; give each diagnostic its own share
/// when programs report dozens of fixable diagnostics.
const MAX_RECOMPILES: usize = 20;

/// Checks candidate fixes against recompiles of the patched text.
pub(crate) struct Checker<'a>
{
    /// The file the program starts from.
    pub root: &'a SourceFile,
    /// The diagnostics of the first compile.
    before: Vec<Diagnostic>,
    /// Compiles a patched text.
    recompile: &'a mut dyn FnMut(&str) -> Option<Recompiled>,
    /// The recompiles left.
    left: usize,
}

/// A text patched with a fix, and the line of its first edit.
struct Patched
{
    /// The text.
    text: String,
    /// The 0-based line of the first edit.
    line: usize,
}

impl<'a> Checker<'a>
{
    /// A checker with the full budget.
    ///
    /// # Arguments
    ///
    /// * `root` - The file the program starts from.
    /// * `before` - The diagnostics of the first compile.
    /// * `recompile` - Compiles a patched text.
    ///
    /// # Returns
    ///
    /// The checker.
    pub fn new(
        root: &'a SourceFile, before: &[Diagnostic],
        recompile: &'a mut dyn FnMut(&str) -> Option<Recompiled>,
    ) -> Self
    {
        #[rustfmt::skip]
        return Checker { root, before: before.to_vec(), recompile, left: MAX_RECOMPILES };
    }

    /// Whether a fix holds: the patched text has fewer diagnostics of the code
    /// and no more errors on the line of the fix, less the one fixed. Errors
    /// on other lines can show up, as a fixed parse lets the later stages run.
    ///
    /// # Arguments
    ///
    /// * `d` - The diagnostic the fix is for.
    /// * `fix` - The fix.
    ///
    /// # Returns
    ///
    /// `true` when it holds; `false` too when the budget is spent or an edit
    /// lies outside the root file.
    pub fn passes(&mut self, d: &Diagnostic, fix: &Fix) -> bool
    {
        ret_false!(self.left == 0);
        let Some(patched) = patch(self.root, fix)
        else
        {
            return false;
        };
        self.left -= 1;
        let Some(after) = (self.recompile)(&patched.text)
        else
        {
            return false;
        };
        let same =
            |list: &[Diagnostic]| list.iter().filter(|x| x.code == d.code).count();
        ret_false!(same(&after.diagnostics) >= same(&self.before));
        let here = d.is_error() && line_of(self.root, &d.span) == Some(patched.line);
        let had = errors_on(self.root, &self.before, patched.line);
        let has = errors_on(&after.root, &after.diagnostics, patched.line);
        has + usize::from(here) <= had
    }
}

/// The text of the root file with the edits of a fix made.
///
/// # Arguments
///
/// * `root` - The file.
/// * `fix` - The fix.
///
/// # Returns
///
/// The patched text, or `None` when an edit lies outside the file, cuts a
/// character or overlaps another.
fn patch(root: &SourceFile, fix: &Fix) -> Option<Patched>
{
    let spots: Option<Vec<(usize, usize, &str)>> = fix
        .edits
        .iter()
        .map(|e| offsets_in(root, &e.span).map(|(lo, hi)| (lo, hi, e.text.as_str())))
        .collect();
    let mut spots = spots?;
    // The last edit goes first, so the offsets of the others hold.
    spots.sort_by_key(|(lo, ..)| std::cmp::Reverse(*lo));
    ret_if!(spots.windows(2).any(|w| w[1].1 > w[0].0), None);
    let first = spots.last()?.0;
    let mut text = root.src.clone();
    for (lo, hi, new) in spots
    {
        ret_if!(!text.is_char_boundary(lo) || !text.is_char_boundary(hi), None);
        text.replace_range(lo..hi, new);
    }
    let line = text[..first].matches('\n').count();
    Some(Patched { text, line })
}

/// The errors of a list on one line of a file.
///
/// # Arguments
///
/// * `file` - The file.
/// * `list` - The diagnostics.
/// * `line` - The 0-based line.
///
/// # Returns
///
/// The count.
fn errors_on(file: &SourceFile, list: &[Diagnostic], line: usize) -> usize
{
    let on_line =
        |d: &&Diagnostic| d.is_error() && line_of(file, &d.span) == Some(line);
    list.iter().filter(on_line).count()
}

/// The 0-based line of the start of a span.
///
/// # Arguments
///
/// * `file` - The file.
/// * `span` - The span.
///
/// # Returns
///
/// The line, or `None` for a span outside the file.
fn line_of(file: &SourceFile, span: &Span) -> Option<usize>
{
    offsets_in(file, span).map(|(lo, _)| file.src[..lo].matches('\n').count())
}

/// The offsets of a span in the text of a file.
///
/// # Arguments
///
/// * `file` - The file.
/// * `span` - The span.
///
/// # Returns
///
/// The start and end offsets, or `None` for a placeholder or a span outside
/// the file.
pub(crate) fn offsets_in(file: &SourceFile, span: &Span) -> Option<(usize, usize)>
{
    ret_if!(span.is_placeholder(), None);
    let inside = file.start_pos <= span.lo && span.hi <= file.end_pos();
    ret_if!(!inside || span.hi < span.lo, None);
    let start = file.start_pos.as_u32();
    let lo = (span.lo.as_u32() - start) as usize;
    let hi = (span.hi.as_u32() - start) as usize;
    Some((lo, hi))
}

/// A rule fix of one edit.
///
/// # Arguments
///
/// * `file` - The file the offsets are in.
/// * `message` - What the fix does.
/// * `lo` - The start offset of the region to replace.
/// * `hi` - Its end offset.
/// * `text` - The new text.
///
/// # Returns
///
/// The fix.
pub(crate) fn rule_fix(
    file: &SourceFile, message: impl Into<String>, lo: usize, hi: usize,
    text: impl Into<String>,
) -> Fix
{
    let span = Span::new(file.start_pos + lo, file.start_pos + hi);
    let edit = Edit::builder().span(span).text(text).build();
    Fix::builder().message(message).edits(vec![edit]).origin(FixOrigin::Rule).build()
}
