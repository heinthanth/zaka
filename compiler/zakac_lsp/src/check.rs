use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use gen_lsp_types::Code;
use gen_lsp_types::DiagnosticRelatedInformation;
use gen_lsp_types::DiagnosticSeverity;
use gen_lsp_types::Location;
use gen_lsp_types::Message;
use gen_lsp_types::Uri;
use zakac_ast::stmt::Program;
use zakac_borrow_checker::artifact::Borrowchecked;
use zakac_borrow_checker::artifact::run_loaded;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Level;
use zakac_diagnostics::has_errors;
use zakac_i18n::t;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::token::Kw;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::TokenKind;
use zakac_parser::load::load;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_semcheck::semcheck::semcheck;
use zakac_span::file_loader::FileLoader;
use zakac_span::file_loader::RealFileLoader;
use zakac_span::filename::FileName;
use zakac_span::pos::Pos;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;
use zakac_span::text_range::Range;
use zakac_span::text_range::to_original;

use crate::lines::LineIndex;

/// Reads the files the editor holds open from their unsaved text, and each
/// other file from the disk, so a check sees what the user sees.
#[derive(Debug, Clone, Default)]
pub struct Buffers
{
    /// The text of each open file, on its path.
    pub open: HashMap<PathBuf, String>,
}

impl FileLoader for Buffers
{
    fn file_exists(&self, path: &Path) -> bool
    {
        self.open.contains_key(path) || RealFileLoader.file_exists(path)
    }

    fn read_raw_file(&self, path: &Path) -> io::Result<Vec<u8>>
    {
        let open = self.open.get(path).map(|text| text.clone().into_bytes());
        open.map_or_else(|| RealFileLoader.read_raw_file(path), Ok)
    }
}

/// What a check of one open file found.
pub struct FileCheck
{
    /// The address of the open file.
    pub uri: Uri,
    /// The files of the program: the open file first, then its imports.
    pub sm: SourceMap,
    /// The open file.
    pub root: Arc<SourceFile>,
    /// Its text as the editor holds it.
    pub text: String,
    /// Positions in that text.
    pub lines: LineIndex,
    /// Its tokens and comments.
    pub lexed: LexOutput,
    /// The output of each phase up to the ownership check; `None` when an
    /// imported file could not be read.
    pub checked: Option<Borrowchecked>,
    /// What to show in the editor, on the open file.
    pub diagnostics: Vec<gen_lsp_types::Diagnostic>,
    /// Each resolved name with its declaration, as places in the editor.
    pub links: Vec<(Location, Location)>,
    /// The address, text and lines of each imported file met so far, on its
    /// start position; `None` for a file that cannot be read.
    imported: RefCell<HashMap<Pos, Option<Imported>>>,
}

/// An imported file as the editor sees it.
struct Imported
{
    /// Its address.
    uri: Uri,
    /// Its text before normalization.
    original: String,
    /// Positions in that text.
    lines: LineIndex,
}

/// Check an open file and the files it imports, up to the ownership check.
///
/// # Arguments
///
/// * `uri` - The address of the open file.
/// * `path` - Its path.
/// * `text` - Its text as the editor holds it.
/// * `buffers` - The text of each open file.
///
/// # Returns
///
/// What the check found.
pub fn check_file(uri: &Uri, path: &Path, text: &str, buffers: Buffers) -> FileCheck
{
    let mut sm = SourceMap::with_loader(Arc::new(buffers));
    let root = sm.new_source_file(FileName::real(path.to_path_buf()), text.to_string());
    let lexer = LexerOptions::default();
    let lexed = lex(&root, &lexer);
    let lines = LineIndex::new(text);
    let run = || {
        load(&root, &mut sm, &lexer).map(|loaded| {
            let folding = FoldOptions::builder().build();
            let options =
                SemcheckOptions::builder().as_module(is_module(&lexed)).build();
            let parsed =
                has_errors(&loaded.diagnostics).then(|| loaded.program.clone());
            let mut checked =
                run_loaded(loaded, &options, &folding, &BorrowCheckOptions::default());
            do_if!(let Some(p) = parsed, check_recovered(&mut checked, p, &options));
            checked
        })
    };
    // A compiler panic on half-written text must not end the server: it becomes
    // one diagnostic, and the colors from the lexer remain.
    let (checked, stopped) = match catch_unwind(AssertUnwindSafe(run))
    {
        Ok(Ok(checked)) => (Some(checked), None),
        Ok(Err(unread)) => (None, Some(at_start(unread.to_string()))),
        Err(panic) => (None, Some(at_start(panicked(panic.as_ref())))),
    };
    let mut file = FileCheck {
        uri: uri.clone(),
        sm,
        root,
        text: text.to_string(),
        lines,
        lexed,
        checked,
        diagnostics: stopped.into_iter().collect(),
        links: Vec::new(),
        imported: RefCell::default(),
    };
    let found = file.checked.as_ref().map(|c| c.diagnostics.clone());
    let shown: Vec<_> = found.iter().flatten().map(|d| file.convert(d)).collect();
    file.diagnostics.extend(shown);
    let refs = file.checked.as_ref().map(|c| c.lowered.checked.refs.clone());
    let link = |(at, decl): &(Span, Span)| Some((file.place(at)?, file.place(decl)?));
    file.links = refs.iter().flatten().filter_map(link).collect();
    file
}

/// Check a program the parser recovered from a line it could not read: the
/// compiler stops before the check, but the editor still wants the kinds
/// and the names of the rest while a line is being written. The diagnostics
/// of this check are left out, as the parse error comes first.
///
/// # Arguments
///
/// * `checked` - The output of the phases, which holds no check.
/// * `program` - The recovered program, as parsed.
/// * `options` - The check options.
fn check_recovered(
    checked: &mut Borrowchecked, mut program: Program, options: &SemcheckOptions,
)
{
    let out = semcheck(&mut program, options);
    let found = &mut checked.lowered.checked;
    found.kinds = out.kinds;
    found.structs = out.structs;
    found.refs.extend(out.refs);
}

/// The address of a file on the disk.
///
/// # Arguments
///
/// * `path` - The absolute path.
///
/// # Returns
///
/// The address, or `None` for a path no address can hold.
pub fn file_uri(path: &Path) -> Option<Uri>
{
    #[cfg(not(target_arch = "wasm32"))]
    return Uri::from_file_path(path).ok();
    // The browser has no file paths of its own; the path stands as written.
    #[cfg(target_arch = "wasm32")]
    return Uri::parse(&format!("file://{}", path.display())).ok();
}

/// Whether an open file is a module for other files: it exports a name and
/// has no start block. A module is checked with no start block required.
///
/// # Arguments
///
/// * `lexed` - The tokens of the file.
///
/// # Returns
///
/// `true` for a module.
fn is_module(lexed: &LexOutput) -> bool
{
    let has = |kw| lexed.tokens.iter().any(|t| t.kind == TokenKind::Kw(kw));
    has(Kw::Export) && !has(Kw::Program)
}

/// The message of a compiler panic.
///
/// # Arguments
///
/// * `panic` - What the panic carried.
///
/// # Returns
///
/// The message for the editor.
fn panicked(panic: &(dyn Any + Send)) -> String
{
    let text = panic.downcast_ref::<&str>().map(|s| s.to_string());
    let text = text.or_else(|| panic.downcast_ref::<String>().cloned());
    t!("lsp.panicked", reason = text.unwrap_or_default()).to_string()
}

/// A diagnostic at the start of the open file, for what stopped the check:
/// an imported file that could not be read, or a compiler panic.
///
/// # Arguments
///
/// * `reason` - What stopped the check.
///
/// # Returns
///
/// The diagnostic.
fn at_start(reason: String) -> gen_lsp_types::Diagnostic
{
    gen_lsp_types::Diagnostic {
        severity: Some(DiagnosticSeverity::Error),
        source: Some(SOURCE.to_string()),
        message: Message::String(reason),
        ..Default::default()
    }
}

/// The name that marks the diagnostics of the compiler in the editor.
const SOURCE: &str = "zakac";

impl FileCheck
{
    /// Whether a span lies in the open file. A placeholder counts as the
    /// start of the open file.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// `true` when it does.
    pub fn in_root(&self, span: &Span) -> bool
    {
        let (lo, hi) = (self.root.start_pos, self.root.end_pos());
        span.is_placeholder() || (lo <= span.lo && span.lo <= hi)
    }

    /// The range of a span of the open file in its text.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// The range, in UTF-8 offsets of the text the editor holds.
    pub fn root_range(&self, span: &Span) -> Range
    {
        to_original(&self.root, &self.text, span)
    }

    /// The file and LSP range of a span of an imported file.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// The place, or `None` when the file has no path or cannot be read.
    fn import_place(&self, span: &Span) -> Option<Location>
    {
        let file = self.sm.lookup_source_file(&span.lo);
        let mut imported = self.imported.borrow_mut();
        let seen = imported.entry(file.start_pos).or_insert_with(|| self.read(&file));
        let seen = seen.as_ref()?;
        let range = seen.lines.range(to_original(&file, &seen.original, span));
        Some(Location { uri: seen.uri.clone(), range })
    }

    /// Read an imported file again, for its text before normalization.
    ///
    /// # Arguments
    ///
    /// * `file` - The file.
    ///
    /// # Returns
    ///
    /// The file, or `None` when it has no path or cannot be read.
    fn read(&self, file: &SourceFile) -> Option<Imported>
    {
        let FileName::Real(real) = &file.name
        else
        {
            return None;
        };
        let original = self.sm.file_loader().read_file(&real.local_path).ok()?;
        let uri = file_uri(&real.local_path)?;
        Some(Imported { uri, lines: LineIndex::new(&original), original })
    }

    /// The place of a span: in the open file, or in an imported one.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// The place, or `None` for an imported file that cannot be read.
    pub fn place(&self, span: &Span) -> Option<Location>
    {
        let range = self.lines.range(self.root_range(span));
        let here = || Location { uri: self.uri.clone(), range };
        if self.in_root(span)
        {
            Some(here())
        }
        else
        {
            self.import_place(span)
        }
    }

    /// Turn a diagnostic of the compiler into one the editor shows on the
    /// open file. A diagnostic in an imported file shows at the start of the
    /// open file, named after that file and linked to its place.
    ///
    /// # Arguments
    ///
    /// * `d` - The diagnostic.
    ///
    /// # Returns
    ///
    /// The diagnostic for the editor.
    fn convert(&self, d: &Diagnostic) -> gen_lsp_types::Diagnostic
    {
        let help = d.help.as_ref().map(|h| format!("\n{h}")).unwrap_or_default();
        let message = format!("{}{help}", d.message);
        let label = |span: &Span, text: &str| {
            let location = self.place(span)?;
            Some(DiagnosticRelatedInformation { location, message: text.to_string() })
        };
        let mut related: Vec<_> =
            d.labels.iter().filter_map(|l| label(&l.span, &l.text)).collect();
        let home = self.in_root(&d.span);
        let message = tern!(home, message, self.imported(d, message, &mut related));
        let range = home.then(|| self.lines.range(self.root_range(&d.span)));
        gen_lsp_types::Diagnostic {
            range: range.unwrap_or_default(),
            severity: Some(lsp_level(d.level)),
            code: Some(Code::String(d.code.to_string())),
            source: Some(SOURCE.to_string()),
            message: Message::String(message),
            related_information: (!related.is_empty()).then_some(related),
            ..Default::default()
        }
    }

    /// The message of a diagnostic in an imported file, named after that
    /// file; its place joins the related places.
    ///
    /// # Arguments
    ///
    /// * `d` - The diagnostic.
    /// * `message` - Its message.
    /// * `related` - The related places.
    ///
    /// # Returns
    ///
    /// The message.
    fn imported(
        &self, d: &Diagnostic, message: String,
        related: &mut Vec<DiagnosticRelatedInformation>,
    ) -> String
    {
        let info = self.place(&d.span).map(|location| DiagnosticRelatedInformation {
            location,
            message: d.message.clone(),
        });
        related.extend(info);
        let file = self.sm.lookup_source_file(&d.span.lo).name.virtual_path_string();
        t!("lsp.imported", file = file, message = message).to_string()
    }
}

/// The LSP level of a diagnostic level.
///
/// # Arguments
///
/// * `level` - The level.
///
/// # Returns
///
/// The LSP level.
fn lsp_level(level: Level) -> DiagnosticSeverity
{
    match level
    {
        Level::Error => DiagnosticSeverity::Error,
        Level::Warning => DiagnosticSeverity::Warning,
        Level::Note => DiagnosticSeverity::Information,
    }
}
