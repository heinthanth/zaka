//! The check of the fixes of a compile: each fix is tried on a patched text
//! of the root file, compiled again with the options of the first compile, up
//! to the checker.

use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::sync::Arc;

use bon::Builder;
use zakac_ast::target::Target;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics_fixes::Recompiled;
use zakac_diagnostics_fixes::attach_fixes;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::open_chains::load_settled;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::file_loader::FileLoader;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

use crate::artifact::run_loaded;
use crate::options::BorrowCheckOptions;

/// The options of the first compile, which each recompile takes.
#[derive(Clone, Builder)]
pub struct Recompile
{
    /// The file the program starts from.
    pub root: Arc<SourceFile>,
    /// What reads the imported files; `None` reads them from the disk.
    pub loader: Option<Arc<dyn FileLoader>>,
    /// The target.
    pub target: Target,
    /// The check options.
    pub options: SemcheckOptions,
    /// The foldings.
    pub folding: FoldOptions,
    /// The options of the checker.
    pub mem: BorrowCheckOptions,
}

impl Recompile
{
    /// Attach the checked fixes to the diagnostics of the first compile.
    ///
    /// # Arguments
    ///
    /// * `diags` - The diagnostics.
    ///
    /// # Returns
    ///
    /// The diagnostics, each with the fixes that hold.
    pub fn attach(&self, diags: Vec<Diagnostic>) -> Vec<Diagnostic>
    {
        let mut run = |text: &str| self.compile(text);
        attach_fixes(&self.root, diags, &mut run)
    }

    /// Compile a patched text of the root file.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    ///
    /// # Returns
    ///
    /// What the compile found, or `None` when an import cannot be read or
    /// the compiler fails on the text.
    fn compile(&self, text: &str) -> Option<Recompiled>
    {
        let mut sm =
            self.loader.clone().map_or_else(SourceMap::new, SourceMap::with_loader);
        let file = sm.new_source_file(self.root.name.clone(), text.to_string());
        let lexer = LexerOptions::default();
        let run = || {
            let loaded =
                load_settled(&file, &mut sm, &lexer, &self.target, &self.options);
            let checked =
                run_loaded(loaded.ok()?, &self.options, &self.folding, &self.mem);
            Some(checked.diagnostics)
        };
        // A patched text can reach a compiler failure the first text did not;
        // the fix then does not hold.
        let diagnostics = catch_unwind(AssertUnwindSafe(run)).ok().flatten()?;
        Some(Recompiled::builder().diagnostics(diagnostics).root(file).build())
    }
}
