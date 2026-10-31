#![allow(dead_code)]

use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_with;
use zaka_vm::trap::Trap;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_borrow_checker::options::MemCheck;
use zakac_codegen::artifact::run_phases;
use zakac_codegen::bc::Module;
use zakac_codegen::options::ResultMode;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::has_errors;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// The ownership check turned off, as for programs that go wrong on
/// purpose.
fn mem_off() -> BorrowCheckOptions
{
    BorrowCheckOptions::builder().check(MemCheck::Off).build()
}

/// Compile a clean source text with the ownership check off.
pub fn compile(text: &str) -> Module
{
    compile_with(text, &SemcheckOptions::default(), &mem_off(), ResultMode::default()).0
}

/// Compile a source text with the ownership check on, which must report
/// nothing at all, not even a warning.
pub fn compile_checked(text: &str) -> Module
{
    compile_results(text, ResultMode::default())
}

/// Compile a source text with the ownership check on, which must report
/// nothing at all, and a result mode.
pub fn compile_results(text: &str, results: ResultMode) -> Module
{
    let (module, diagnostics) = compile_with(
        text,
        &SemcheckOptions::default(),
        &BorrowCheckOptions::default(),
        results,
    );
    let msgs: Vec<&str> = diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(diagnostics.is_empty(), "{text}: {msgs:?}");
    module
}

/// Compile a clean source text for a platform width, with the ownership
/// check off.
pub fn compile_on(text: &str, platform_bits: u16) -> Module
{
    let options = SemcheckOptions::builder().platform_bits(platform_bits).build();
    compile_with(text, &options, &mem_off(), ResultMode::default()).0
}

/// Compile a source text that must have no error, with checker, ownership
/// and result options.
fn compile_with(
    text: &str, options: &SemcheckOptions, mem: &BorrowCheckOptions,
    results: ResultMode,
) -> (Module, Vec<Diagnostic>)
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let folding = FoldOptions::default();
    let lexer = LexerOptions::default();
    let emitted = run_phases(&file, &lexer, options, &folding, mem, results);
    let msgs: Vec<&str> =
        emitted.diagnostics().iter().map(|d| d.message.as_str()).collect();
    assert!(!has_errors(emitted.diagnostics()), "{text}: {msgs:?}");
    let diagnostics = emitted.diagnostics().to_vec();
    (emitted.module.expect("emitted"), diagnostics)
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

/// The output of a source text compiled for a platform width; it must not
/// trap.
pub fn output_on(text: &str, platform_bits: u16) -> String
{
    let (out, result) =
        run_module(&compile_on(text, platform_bits), &RunOptions::default());
    assert_eq!(result, Ok(()), "{text}\n{out}");
    out
}
