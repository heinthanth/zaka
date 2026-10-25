#![allow(dead_code)]

use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_with;
use zaka_vm::trap::Trap;
use zakac_codegen::artifact::run_phases;
use zakac_codegen::bc::Module;
use zakac_diagnostics::has_errors;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Compile a clean source text.
pub fn compile(text: &str) -> Module
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let options = SemcheckOptions::default();
    let emitted =
        run_phases(&file, &LexerOptions::default(), &options, &FoldOptions::default());
    let msgs: Vec<&str> =
        emitted.diagnostics().iter().map(|d| d.message.as_str()).collect();
    assert!(!has_errors(emitted.diagnostics()), "{text}: {msgs:?}");
    emitted.module.expect("emitted")
}

/// Run a module and collect its output.
pub fn run_module(module: &Module, options: &RunOptions) -> (String, Result<(), Trap>)
{
    let mut out = Vec::new();
    let result = run_with(module, &mut out, options);
    (String::from_utf8(out).unwrap(), result)
}

/// Compile and run a source text with limits.
pub fn run_source_with(text: &str, options: &RunOptions) -> (String, Result<(), Trap>)
{
    run_module(&compile(text), options)
}

/// Compile and run a source text with the default limits.
pub fn run_source(text: &str) -> (String, Result<(), Trap>)
{
    run_source_with(text, &RunOptions::default())
}

/// The output of a source text that must not trap.
pub fn output(text: &str) -> String
{
    let (out, result) = run_source(text);
    assert_eq!(result, Ok(()), "{text}\n{out}");
    out
}
