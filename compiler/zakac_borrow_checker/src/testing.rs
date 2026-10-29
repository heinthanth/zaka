use std::sync::Arc;

use bon::Builder;
use zakac_diagnostics::has_errors;
use zakac_ir_builder::ir::FoldOptions;
use zakac_ir_builder::ir::Module;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

/// A program lowered and folded, with the file its spans point into.
#[derive(Builder)]
pub(crate) struct Program
{
    /// The source file.
    pub(crate) file: Arc<SourceFile>,
    /// The folded module.
    pub(crate) module: Module,
}

impl Program
{
    /// The 1-based line a span starts on.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// The line.
    pub(crate) fn line(&self, span: Span) -> usize
    {
        self.file.lookup_file_pos(&self.file.relative_position(&span.lo)).0
    }
}

/// Lower and fold a program that has no error.
///
/// # Arguments
///
/// * `text` - The source.
///
/// # Returns
///
/// The program.
///
/// # Panics
///
/// When a phase reports an error.
pub(crate) fn program(text: &str) -> Program
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let lowered = zakac_ir_builder::artifact::run_phases(
        &file,
        &LexerOptions::default(),
        &SemcheckOptions::default(),
        &FoldOptions::default(),
    );
    let msgs: Vec<&str> =
        lowered.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(!has_errors(&lowered.diagnostics), "{msgs:?}");
    let module = lowered.folded.expect("a clean program lowers");
    Program::builder().file(file).module(module).build()
}

/// Lower and fold a program that has no error.
///
/// # Arguments
///
/// * `text` - The source.
///
/// # Returns
///
/// The folded module.
pub(crate) fn lower(text: &str) -> Module { program(text).module }
