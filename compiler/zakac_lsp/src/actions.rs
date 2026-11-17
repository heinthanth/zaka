//! The quick fixes of the editor: each fix a diagnostic holds becomes an
//! action the reader picks; the server never applies one on its own.

use std::collections::HashMap;

use gen_lsp_types::CodeAction;
use gen_lsp_types::CodeActionKind;
use gen_lsp_types::Range;
use gen_lsp_types::TextEdit;
use gen_lsp_types::Uri;
use gen_lsp_types::WorkspaceEdit;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Fix;

use crate::check::FileCheck;

/// The quick fixes of the diagnostics that touch a range of the open file:
/// one action for each fix, in the order the diagnostic holds them.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `range` - The range the editor asks about.
///
/// # Returns
///
/// The actions; none when the check stopped before the end.
pub fn quick_fixes(file: &FileCheck, range: Range) -> Vec<CodeAction>
{
    let Some(checked) = &file.checked
    else
    {
        return Vec::new();
    };
    let shown = checked.diagnostics.iter().map(|d| (d, file.convert(d)));
    let touching = shown.filter(|(_, shown)| overlaps(shown.range, range));
    touching.flat_map(|(d, shown)| actions_of(file, d, &shown)).collect()
}

/// Whether two ranges share a position; a range touching the end of the
/// other counts, so a cursor at the end of a word finds its fix.
///
/// # Arguments
///
/// * `a` - One range.
/// * `b` - The other.
///
/// # Returns
///
/// `true` for ranges that touch.
fn overlaps(a: Range, b: Range) -> bool { a.start <= b.end && b.start <= a.end }

/// The actions of one diagnostic. The first rule fix is the preferred one,
/// which an editor applies on its quick fix shortcut; a fix of the model never
/// is.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `d` - The diagnostic.
/// * `shown` - The diagnostic as the editor shows it.
///
/// # Returns
///
/// The actions; a fix with an edit in a file that cannot be read is left out.
fn actions_of(
    file: &FileCheck, d: &Diagnostic, shown: &gen_lsp_types::Diagnostic,
) -> Vec<CodeAction>
{
    let action = |(i, fix): (usize, &Fix)| {
        let edit = workspace_edit(file, fix)?;
        Some(CodeAction {
            title: fix.message.clone(),
            kind: Some(CodeActionKind::QuickFix),
            diagnostics: Some(vec![shown.clone()]),
            is_preferred: Some(i == 0),
            edit: Some(edit),
            ..Default::default()
        })
    };
    d.fixes.iter().enumerate().filter_map(action).collect()
}

/// The edits of a fix as changes to the files the edits touch.
///
/// # Arguments
///
/// * `file` - The check of the open file.
/// * `fix` - The fix.
///
/// # Returns
///
/// The change, or `None` when an edit lies in a file that cannot be read.
fn workspace_edit(file: &FileCheck, fix: &Fix) -> Option<WorkspaceEdit>
{
    let mut changes: HashMap<Uri, Vec<TextEdit>> = HashMap::new();
    for edit in &fix.edits
    {
        let place = file.place(&edit.span)?;
        let text = TextEdit { range: place.range, new_text: edit.text.clone() };
        changes.entry(place.uri).or_default().push(text);
    }
    Some(WorkspaceEdit { changes: Some(changes), ..Default::default() })
}
