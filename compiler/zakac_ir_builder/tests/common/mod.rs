#![allow(dead_code)]

use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::START_NAME;
use zakac_diagnostics::Code;
use zakac_ir_builder::artifact::Lowered;
use zakac_ir_builder::artifact::run_phases;
use zakac_ir_builder::ir::FoldOptions;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Stmt;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_semcheck::unsafe_code::in_unsafe_code;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Run all phases on `text`.
pub fn run(text: &str, folding: &FoldOptions) -> Lowered
{
    let mut sm = SourceMap::new();
    let file =
        sm.new_source_file(FileName::Custom("t.zaka".into()), in_unsafe_code(text));
    let options = SemcheckOptions::default();
    run_phases(&file, &LexerOptions::default(), &options, folding)
}

/// The statements of the main list, one form each, with the statements of
/// the start block in place of the call of it.
pub fn main_forms(module: &Module) -> Vec<String>
{
    let start = module.funcs.iter().position(|f| f.name == START_NAME);
    let block = start.map(|i| &module.funcs[i].block);
    let block = block.map(|b| match b.last()
    {
        Some(Stmt::Return { value: None, .. }) => &b[..b.len() - 1],
        _ => &b[..],
    });
    let is_call = |s: &Stmt| matches!(s, Stmt::Call { func, .. } if Some(*func as usize) == start);
    let forms = |s: &Stmt| match (is_call(s), block)
    {
        (true, Some(b)) => b.iter().map(ToSexpr::to_sexpr).collect(),
        _ => vec![s.to_sexpr()],
    };
    module.main.iter().flat_map(forms).collect()
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
