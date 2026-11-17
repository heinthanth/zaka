//! Fixes for the diagnostics of a program: the fixes the stages attach and the
//! rules of this crate. Each fix is kept when the program, patched with it,
//! compiles without the diagnostic. The compiler never applies a fix; the
//! reader does.

mod check;
mod names;
mod particles;

use std::sync::Arc;

use bon::Builder;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Fix;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_span::source_file::SourceFile;

use crate::check::Checker;

zakac_i18n::init!();

/// What a compile of a patched text found.
#[derive(Debug, Clone, Builder)]
pub struct Recompiled
{
    /// Its diagnostics.
    pub diagnostics: Vec<Diagnostic>,
    /// The file of the patched text, which their spans point into.
    pub root: Arc<SourceFile>,
}

/// The most candidates checked for one diagnostic.
const MAX_CANDIDATES: usize = 5;

/// Attach the checked fixes to the diagnostics of a program. The candidates
/// of a diagnostic are the fixes it holds, then the rules of this crate; a
/// candidate is kept when a recompile of the patched text loses the diagnostic
/// and adds no error on the line of the fix. A diagnostic that keeps a fix
/// drops its help text, which the fix stands for.
///
/// # Arguments
///
/// * `root` - The file the program starts from; its text is patched.
/// * `diags` - The diagnostics of the program.
/// * `recompile` - Compiles a patched text with the options of the first
///   compile; `None` when the compile could not run.
///
/// # Returns
///
/// The diagnostics, each with its checked fixes.
pub fn attach_fixes(
    root: &SourceFile, diags: Vec<Diagnostic>,
    recompile: &mut dyn FnMut(&str) -> Option<Recompiled>,
) -> Vec<Diagnostic>
{
    let mut checker = Checker::new(root, &diags, recompile);
    let mut out = Vec::with_capacity(diags.len());
    do_for!(d in diags, out.push(fixed(&mut checker, d)));
    out
}

/// One diagnostic with its checked fixes.
///
/// # Arguments
///
/// * `checker` - The checker of the program.
/// * `d` - The diagnostic.
///
/// # Returns
///
/// The diagnostic; with no fix that holds, a help text that became a fix
/// comes back.
fn fixed(checker: &mut Checker<'_>, mut d: Diagnostic) -> Diagnostic
{
    let mut candidates = std::mem::take(&mut d.fixes);
    candidates.extend(particles::fixes(checker.root, &d));
    candidates.extend(names::fixes(checker.root, &d));
    candidates.truncate(MAX_CANDIDATES);
    let mut kept: Vec<Fix> = Vec::new();
    for fix in &candidates
    {
        // Two fixes with one message read as one; the first that holds is kept.
        let told = kept.iter().any(|k| k.message == fix.message);
        do_if!(!told && checker.passes(&d, fix), kept.push(fix.clone()));
    }
    let first = candidates.into_iter().next().map(|f| f.message);
    if kept.is_empty()
    {
        d.help = d.help.or(first);
        return d;
    }
    d.help = None;
    d.fixes = kept;
    d
}
