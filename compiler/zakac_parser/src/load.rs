use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use bon::Builder;
use zakac_ast::stmt::ImportStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::Unit;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser as diag;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::ret_if;
use zakac_session::artifact::EmbeddedFile;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

use crate::parser::parse;
use crate::parser::parse_with;

/// A program read with its imports.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct Loaded
{
    /// The statements of each file, imported files first.
    pub program: Program,
    /// The imported files in reading order, which is the order
    /// the source map holds them, after the file that is run.
    #[builder(default)]
    pub imports: Vec<EmbeddedFile>,
    /// The lexer and parser diagnostics of each file, then the loader's.
    #[builder(default)]
    pub diagnostics: Vec<Diagnostic>,
}

impl Loaded
{
    /// Lex and parse one file that cannot import: each import it has is
    /// reported.
    ///
    /// # Arguments
    ///
    /// * `source` - The file.
    /// * `options` - The lexer options.
    ///
    /// # Returns
    ///
    /// The program of the file.
    pub fn single(source: &SourceFile, options: &LexerOptions) -> Self
    {
        let lexed = lex(source, options);
        let parsed = parse(source, &lexed);
        let mut diagnostics = lexed.diagnostics;
        diagnostics.extend(parsed.diagnostics);
        diagnostics.extend(refuse_imports(&parsed.program));
        #[rustfmt::skip]
        return Loaded::builder().program(parsed.program).diagnostics(diagnostics).build();
    }
}

/// Report each import of a program that was not read from a file.
///
/// # Arguments
///
/// * `program` - The program.
///
/// # Returns
///
/// One `ZK0232` diagnostic per import.
pub fn refuse_imports(program: &Program) -> Vec<Diagnostic>
{
    let imports = program.stmts.iter().filter_map(as_import);
    imports.map(|n| diag::import_needs_file(n.span, &n.path.lit.value)).collect()
}

/// Read a file, the files it imports, the files those import, and so on.
/// Each file is read once. A file that was not read from disk cannot
/// import, and its imports are reported.
///
/// # Arguments
///
/// * `root` - The file that is run.
/// * `source_map` - The source map; imported files are added to it.
/// * `options` - The lexer options.
///
/// # Returns
///
/// The program, or the error of a file that could not be read.
pub fn load(
    root: &Arc<SourceFile>, source_map: &mut SourceMap, options: &LexerOptions,
) -> zakac_host_errors::Result<Loaded>
{
    let FileName::Real(real) = &root.name
    else
    {
        return Ok(Loaded::single(root, options));
    };
    let path = normalize(&real.local_path);
    let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
    #[rustfmt::skip]
    let mut loader = Loader::builder().source_map(source_map).options(options)
        .folder(folder).build();
    loader.visit(root.clone(), path)?;
    let Loader { stmts, units, imports, diagnostics, .. } = loader;
    let span = zakac_span::span::Span::new(root.start_pos, root.end_pos());
    let program = Program::builder().stmts(stmts).units(units).span(span).build();
    #[rustfmt::skip]
    return Ok(Loaded::builder().program(program).imports(imports)
        .diagnostics(diagnostics).build());
}

/// The state of [`load`].
#[derive(Builder)]
struct Loader<'a>
{
    /// Where the files are read and registered.
    source_map: &'a mut SourceMap,
    /// The lexer options.
    options: &'a LexerOptions,
    /// The folder of the file that is run; file names in messages are
    /// relative to it.
    folder: PathBuf,
    /// The statements of the files read so far.
    #[builder(default)]
    stmts: Vec<Stmt>,
    /// The files read so far, in the order of their statements.
    #[builder(default)]
    units: Vec<Unit>,
    /// The struct names each unit exports.
    #[builder(default)]
    structs: Vec<HashSet<String>>,
    /// The unit of each file read so far, indexed on its path.
    #[builder(default)]
    done: HashMap<PathBuf, u32>,
    /// The files whose imports are being read, outermost first.
    #[builder(default)]
    chain: Vec<PathBuf>,
    /// The imported files in reading order.
    #[builder(default)]
    imports: Vec<EmbeddedFile>,
    /// The diagnostics so far.
    #[builder(default)]
    diagnostics: Vec<Diagnostic>,
    /// The number for the next expression node.
    #[builder(default)]
    next_id: u32,
}

impl Loader<'_>
{
    /// Read the imports of a file, then the file itself.
    ///
    /// # Arguments
    ///
    /// * `file` - The file.
    /// * `path` - Its normalized path.
    ///
    /// # Returns
    ///
    /// The unit of the file, or the error of an import that could not be
    /// read.
    fn visit(
        &mut self, file: Arc<SourceFile>, path: PathBuf,
    ) -> zakac_host_errors::Result<u32>
    {
        let lexed = lex(&file, self.options);
        // The imports do not hang on struct names, so a first parse finds
        // them; the second parse knows the struct names those imports bring.
        let first = parse(&file, &lexed);
        let imports: Vec<ImportStmt> =
            first.program.stmts.iter().filter_map(as_import).cloned().collect();
        self.chain.push(path.clone());
        let mut targets = Vec::new();
        for n in &imports
        {
            targets.push(self.import(&path, n)?);
        }
        self.chain.pop();
        let structs = imported_structs(&imports, &targets, &self.structs);
        let parsed = parse_with(&file, &lexed, self.next_id, &structs);
        self.next_id = parsed.next_id;
        self.diagnostics.extend(lexed.diagnostics);
        self.diagnostics.extend(parsed.diagnostics);
        let mut program = parsed.program;
        set_targets(&mut program, &imports, &targets);
        let unit = self.units.len() as u32;
        let name = self.shown(&path);
        let first_stmt = self.stmts.len() as u32;
        self.units.push(Unit::builder().name(name).first_stmt(first_stmt).build());
        self.structs.push(exported_structs(&program));
        self.stmts.extend(program.stmts);
        self.done.insert(path, unit);
        Ok(unit)
    }

    /// A file name as messages show it: relative to the folder of the file
    /// that is run, when it is inside that folder.
    ///
    /// # Arguments
    ///
    /// * `path` - The normalized path.
    ///
    /// # Returns
    ///
    /// The name.
    fn shown(&self, path: &Path) -> String
    {
        path.strip_prefix(&self.folder).unwrap_or(path).display().to_string()
    }

    /// Read the file an import names, unless it was read before or it is
    /// on the chain of files being read.
    ///
    /// # Arguments
    ///
    /// * `importer` - The path of the importing file.
    /// * `n` - The import.
    ///
    /// # Returns
    ///
    /// The unit of the imported file; `None` for a circular import, which
    /// is reported.
    fn import(
        &mut self, importer: &Path, n: &ImportStmt,
    ) -> zakac_host_errors::Result<Option<u32>>
    {
        let folder = importer.parent().unwrap_or(Path::new(""));
        let path = normalize(&folder.join(&n.path.lit.value));
        ret_if!(let Some(unit) = self.done.get(&path), Ok(Some(*unit)));
        if let Some(at) = self.chain.iter().position(|p| *p == path)
        {
            let names: Vec<String> =
                self.chain[at..].iter().map(|p| self.shown(p)).collect();
            let chain = format!("{} → {}", names.join(" → "), self.shown(&path));
            self.diagnostics.push(diag::circular_import(n.span, &chain));
            return Ok(None);
        }
        let file = self.source_map.load_file(&path).map_err(|e| {
            Error::from_io(e, IOErrorContext::new(path.clone(), IOErrorAction::Read))
        })?;
        self.imports.push(EmbeddedFile::of(&file));
        self.visit(file, path).map(Some)
    }
}

/// The import a statement is.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// The import, or `None` for another statement.
fn as_import(stmt: &Stmt) -> Option<&ImportStmt>
{
    match stmt
    {
        Stmt::Import(n) => Some(n),
        _ => None,
    }
}

/// The struct names a file imports.
///
/// # Arguments
///
/// * `imports` - The imports of the file.
/// * `targets` - The unit each import reads.
/// * `exports` - The struct names each unit exports.
///
/// # Returns
///
/// The imported names that are exported structs.
fn imported_structs(
    imports: &[ImportStmt], targets: &[Option<u32>], exports: &[HashSet<String>],
) -> HashSet<String>
{
    let pairs = imports.iter().zip(targets);
    let pairs = pairs.filter_map(|(n, t)| Some((n, &exports[(*t)? as usize])));
    let names = pairs.flat_map(|(n, set)| {
        n.names.iter().map(|i| &i.name).filter(|name| set.contains(*name))
    });
    names.cloned().collect()
}

/// The struct names a file exports.
///
/// # Arguments
///
/// * `program` - The program of the file.
///
/// # Returns
///
/// The names.
fn exported_structs(program: &Program) -> HashSet<String>
{
    let decls = program.stmts.iter().filter_map(|s| match s
    {
        Stmt::Struct(d) if d.exported => Some(d.name.name.clone()),
        _ => None,
    });
    decls.collect()
}

/// Record on each import of a parsed file the unit it reads. The imports
/// of the first parse and of the second come in one order.
///
/// # Arguments
///
/// * `program` - The program of the file.
/// * `imports` - The imports of the first parse.
/// * `targets` - The unit each of them reads.
fn set_targets(program: &mut Program, imports: &[ImportStmt], targets: &[Option<u32>])
{
    let parsed = program.stmts.iter_mut().filter_map(|s| match s
    {
        Stmt::Import(n) => Some(n),
        _ => None,
    });
    let pairs = parsed.zip(imports.iter().zip(targets));
    do_for!((n, (first, unit)) in pairs, n.unit = (*unit).filter(|_| n.span == first.span));
}

/// A path with `.` parts removed and `..` parts applied, without reading
/// the disk, so two spellings of one file meet.
///
/// # Arguments
///
/// * `path` - The path.
///
/// # Returns
///
/// The normalized path.
fn normalize(path: &Path) -> PathBuf
{
    let mut out = PathBuf::new();
    for part in path.components()
    {
        match part
        {
            Component::CurDir => (),
            Component::ParentDir if out.file_name().is_some() =>
            {
                out.pop();
            }
            _ => out.push(part),
        }
    }
    out
}
