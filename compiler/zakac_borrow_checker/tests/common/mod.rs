#![allow(dead_code)]

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use zakac_ast::s_expr::ToSexpr;
use zakac_borrow_checker::artifact::Borrowchecked;
use zakac_borrow_checker::artifact::run_loaded;
use zakac_borrow_checker::artifact::run_phases;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_borrow_checker::options::MemCheck;
use zakac_diagnostics::Code;
use zakac_ir_builder::ir::FoldOptions;
use zakac_ir_builder::ir::Module;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::load;
use zakac_rust_sugars::tern;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

/// The output of all phases on a text, with the file it came from.
pub struct Run
{
    /// The source file.
    pub file: Arc<SourceFile>,
    /// The output.
    pub out: Borrowchecked,
}

/// The default options: errors on, strict off.
pub fn strict() -> BorrowCheckOptions
{
    BorrowCheckOptions::builder().strict(true).build()
}

/// Findings reported as warnings.
pub fn warn() -> BorrowCheckOptions
{
    BorrowCheckOptions::builder().check(MemCheck::Warn).build()
}

/// The check off.
pub fn off() -> BorrowCheckOptions
{
    BorrowCheckOptions::builder().check(MemCheck::Off).build()
}

/// Run all phases on `text` with `options`.
pub fn run_with(text: &str, options: &BorrowCheckOptions) -> Run
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let out = run_phases(
        &file,
        &LexerOptions::default(),
        &SemcheckOptions::default(),
        &FoldOptions::default(),
        options,
    );
    Run { file, out }
}

/// Run all phases on `text` with the default options.
pub fn run(text: &str) -> Run { run_with(text, &BorrowCheckOptions::default()) }

/// The diagnostic codes of `text` with `options`.
pub fn codes_with(text: &str, options: &BorrowCheckOptions) -> Vec<Code>
{
    run_with(text, options).out.diagnostics.iter().map(|d| d.code).collect()
}

/// The diagnostic codes of a source file and the files it imports, with the
/// default options.
pub fn file_codes(path: &Path) -> Vec<Code>
{
    let mut sm = SourceMap::new();
    let file = sm.load_file(path).unwrap();
    let loaded = load(&file, &mut sm, &LexerOptions::default()).unwrap();
    let options = SemcheckOptions::default();
    let mem = BorrowCheckOptions::default();
    let out = run_loaded(loaded, &options, &FoldOptions::default(), &mem);
    out.diagnostics.iter().map(|d| d.code).collect()
}

/// The diagnostic codes of `text` with the default options.
pub fn codes(text: &str) -> Vec<Code>
{
    codes_with(text, &BorrowCheckOptions::default())
}

/// Each diagnostic code of `text` with the 1-based line of its span.
pub fn lines(text: &str) -> Vec<(Code, usize)>
{
    let run = run(text);
    let line = |pos| run.file.lookup_file_pos(&run.file.relative_position(pos)).0;
    run.out.diagnostics.iter().map(|d| (d.code, line(&d.span.lo))).collect()
}

/// Assert that `text` has no diagnostic at all.
pub fn ok(text: &str)
{
    let run = run(text);
    let msgs: Vec<&str> =
        run.out.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(run.out.diagnostics.is_empty(), "{text}: {msgs:?}");
}

/// The statements of the main list, one form each.
pub fn main_forms(module: &Module) -> Vec<String>
{
    module.main.iter().map(ToSexpr::to_sexpr).collect()
}

/// The repo's root dir.
pub fn repo_root() -> PathBuf
{
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The `.zaka` files under a dir, at each depth, sorted.
pub fn zaka_files(dir: &Path) -> Vec<PathBuf>
{
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .flat_map(|p| tern!(p.is_dir(), zaka_files(&p), vec![p]))
        .filter(|p| p.extension().is_some_and(|e| e == "zaka"))
        .collect();
    files.sort();
    files
}
