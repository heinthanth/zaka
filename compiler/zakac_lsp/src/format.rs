//! Format a file for the editor: one edit that replaces the whole text with
//! the text the formatter writes.

use gen_lsp_types::Position;
use gen_lsp_types::Range;
use gen_lsp_types::TextEdit;
use zakac_formatter::FmtOptions;
use zakac_formatter::format;
use zakac_rust_sugars::ret_if;
use zakac_span::source_map::SourceMap;

use crate::check::FileCheck;

/// The edits that format a file.
///
/// # Arguments
///
/// * `file` - The file as the editor holds it.
/// * `options` - The settings.
///
/// # Returns
///
/// One edit for the whole file, none for a file already formatted, or `None`
/// for a file the formatter leaves as it is, such as one that does not parse.
pub fn formatting(file: &FileCheck, options: &FmtOptions) -> Option<Vec<TextEdit>>
{
    let name = file.root.name.clone();
    let out = format(&mut SourceMap::new(), name, &file.text, options).ok()?;
    ret_if!(out == file.text, Some(Vec::new()));
    let end = file.lines.position(file.text.len() as u32);
    let range = Range { start: Position { line: 0, character: 0 }, end };
    Some(vec![TextEdit { range, new_text: out }])
}
