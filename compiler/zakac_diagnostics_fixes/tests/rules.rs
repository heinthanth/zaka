use std::sync::Arc;

use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics_fixes::Recompiled;
use zakac_diagnostics_fixes::attach_fixes;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

/// A file of `text`, and a ZK0507 warning over its first `(...)`.
fn grouped(text: &str) -> (Arc<SourceFile>, Diagnostic)
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let lo = text.find('(').unwrap();
    let hi = text.find(')').unwrap() + 1;
    let span = Span::new(file.start_pos + lo, file.start_pos + hi);
    let d = Diagnostic::new(Code::RedundantGroup, "m", span).with_help("remove them");
    (file, d)
}

/// A recompile that finds `found` in whatever text it gets.
fn finds(found: Vec<Diagnostic>) -> impl FnMut(&str) -> Option<Recompiled>
{
    move |text: &str| {
        let mut sm = SourceMap::new();
        let root = sm.new_source_file(FileName::Custom("t.zaka".into()), text.into());
        Some(Recompiled::builder().diagnostics(found.clone()).root(root).build())
    }
}

#[test]
fn a_fix_that_loses_the_diagnostic_is_kept_and_takes_the_help()
{
    let (file, d) = grouped("(က) ကို ပြသမည်။\n");
    let out = attach_fixes(&file, vec![d], &mut finds(Vec::new()));
    assert_eq!(out[0].help, None);
    assert_eq!(out[0].fixes.len(), 1);
    assert_eq!(out[0].fixes[0].message, "remove them");
    assert_eq!(out[0].fixes[0].edits[0].text, "က");
}

#[test]
fn a_fix_that_keeps_the_diagnostic_goes_and_the_help_comes_back()
{
    let (file, d) = grouped("(က) ကို ပြသမည်။\n");
    let out = attach_fixes(&file, vec![d.clone()], &mut finds(vec![d]));
    assert!(out[0].fixes.is_empty());
    assert_eq!(out[0].help.as_deref(), Some("remove them"));
}

#[test]
fn a_fix_that_cannot_run_goes()
{
    let (file, d) = grouped("(က) ကို ပြသမည်။\n");
    let out = attach_fixes(&file, vec![d], &mut |_: &str| None);
    assert!(out[0].fixes.is_empty());
}
