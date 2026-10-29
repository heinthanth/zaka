#![allow(dead_code)]

use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_codegen::artifact::Emitted;
use zakac_codegen::artifact::run_phases;
use zakac_codegen::bc::Func;
use zakac_codegen::bc::Module;
use zakac_diagnostics::has_errors;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Run each phase on a source text.
pub fn run(text: &str) -> Emitted
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let options = SemcheckOptions::default();
    let folding = FoldOptions::default();
    let mem = BorrowCheckOptions::default();
    run_phases(&file, &LexerOptions::default(), &options, &folding, &mem)
}

/// The ops of a clean source text.
pub fn emit_text(text: &str) -> Module
{
    let emitted = run(text);
    let msgs: Vec<&str> =
        emitted.diagnostics().iter().map(|d| d.message.as_str()).collect();
    assert!(!has_errors(emitted.diagnostics()), "{text}: {msgs:?}");
    emitted.module.expect("emitted")
}

/// The ops of a function as text.
pub fn ops(func: &Func) -> Vec<String>
{
    func.code.iter().map(|op| op.to_string()).collect()
}

/// The main function.
pub fn main_func(module: &Module) -> &Func { &module.funcs[module.main as usize] }

/// The ops of the main function of a clean source text, as text.
pub fn main_ops(text: &str) -> Vec<String> { ops(main_func(&emit_text(text))) }
