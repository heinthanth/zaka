#![allow(dead_code)]

use zakac_ast::s_expr::ToSexpr;
use zakac_diagnostics::Code;
use zakac_ir_builder::artifact::Lowered;
use zakac_ir_builder::artifact::run_phases;
use zakac_ir_builder::ir::FoldOptions;
use zakac_ir_builder::ir::Module;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Run all phases on `text`.
pub fn run(text: &str, folding: &FoldOptions) -> Lowered
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let options = SemcheckOptions::default();
    run_phases(&file, &LexerOptions::default(), &options, folding)
}

/// The statements of the main list, one form each.
pub fn main_forms(module: &Module) -> Vec<String>
{
    module.main.iter().map(ToSexpr::to_sexpr).collect()
}

/// The main statements of `text` as lowered, before folding.
pub fn plain(text: &str) -> Vec<String>
{
    let lowered = run(text, &FoldOptions::default());
    let msgs: Vec<&str> =
        lowered.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(lowered.plain.is_some(), "{text}: {msgs:?}");
    main_forms(lowered.plain.as_ref().expect("lowered"))
}

/// The main statements of `text` after folding with `folding`.
pub fn folded_with(text: &str, folding: &FoldOptions) -> Vec<String>
{
    let lowered = run(text, folding);
    let msgs: Vec<&str> =
        lowered.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(lowered.diagnostics.is_empty(), "{text}: {msgs:?}");
    main_forms(lowered.folded.as_ref().expect("folded"))
}

/// The main statements of `text` after all foldings.
pub fn folded(text: &str) -> Vec<String> { folded_with(text, &FoldOptions::default()) }

/// The diagnostic codes of `text` with `folding`.
pub fn codes_with(text: &str, folding: &FoldOptions) -> Vec<Code>
{
    run(text, folding).diagnostics.iter().map(|d| d.code).collect()
}

/// The diagnostic codes of `text` with all foldings.
pub fn codes(text: &str) -> Vec<Code> { codes_with(text, &FoldOptions::default()) }
