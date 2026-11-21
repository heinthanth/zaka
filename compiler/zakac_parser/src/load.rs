use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;

use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::IntLit;
use zakac_ast::expr::NodeId;
use zakac_ast::expr::StrExpr;
use zakac_ast::expr::StrPiece;
use zakac_ast::stmt::CORE;
use zakac_ast::stmt::FnDecl;
use zakac_ast::stmt::ImportStmt;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::ShareStmt;
use zakac_ast::stmt::Stmt;
use zakac_ast::stmt::StrArg;
use zakac_ast::stmt::Unit;
use zakac_ast::stmt::is_lib_path;
use zakac_ast::target::FACTS;
use zakac_ast::target::FactValue;
use zakac_ast::target::Target;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser as diag;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::reserved::PLURAL;
use zakac_lexer::spelling::canonical;
use zakac_lexer::token::DigitScript;
use zakac_lexer::token::Kw;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::StrLit;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_session::artifact::EmbeddedFile;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

use crate::alias::AliasRefs;
use crate::alias::expand_aliases;
use crate::comptime::CondPlace;
use crate::comptime::Resolved;
use crate::comptime::Spot;
use crate::comptime::settle;
use crate::hint::Hint;
use crate::hint::exported_hints;
use crate::parser::parse;
use crate::parser::parse_with;
use crate::rings::AUTO;
use crate::using::CallWords;
use crate::using::Functions;

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
    /// Each kind alias name, beside its declaration, for the editors.
    #[builder(default)]
    pub alias_refs: AliasRefs,
    /// Each condition of a compile-time conditional that calls a comptime
    /// function and waits for its value, with its place; its conditional
    /// remains in place, unsettled.
    #[builder(default)]
    pub open_conds: Vec<(Span, CondPlace)>,
}

/// The start of the import path of a module of the stdlib.
pub const STD_PREFIX: &str = "စံကား/";

/// A row of [`CORE_PARTS`]: the path in the `stdlib` folder of the source tree,
/// and the source of that file.
macro_rules! core_part {
    ($file:literal) => {
        (concat!("core/", $file), include_str!(concat!("../../../stdlib/core/", $file)))
    };
}

/// A row of [`STD_MODULES`]: the import name, the file name in the `stdlib`
/// folder of the source tree, and the source of that file.
macro_rules! std_module {
    ($name:expr, $file:literal) => {
        ($name, $file, include_str!(concat!("../../../stdlib/", $file)))
    };
}

/// The modules of the stdlib: the name a program imports, the file in the
/// source tree, and the source, which the compiler carries, so a program read
/// from storage imports them too. Each name is in normalized spelling, as an
/// import path is.
pub static STD_MODULES: &[(&str, &str, &str)] = &[
    std_module!("စံကား/အဝင်အထွက်", "io.zaka"),
    std_module!("စံကား/စာသား", "text.zaka"),
    std_module!("စံကား/လမ်းကြောင်း", "path.zaka"),
    std_module!("စံကား/စာရင်း", "list.zaka"),
    std_module!("စံကား/သင်္ချာ", "math.zaka"),
    std_module!("စံကား/အချိန်", "time.zaka"),
    // The core shares the parts in `CORE_PARTS`.
    std_module!(CORE, "core/mod.zaka"),
];

/// The parts of the core, each a file of the `core` folder of the stdlib
/// that `core/mod.zaka` shares: its path in the `stdlib` folder and its
/// source. A program cannot import a part on its own.
pub static CORE_PARTS: &[(&str, &str)] = &[
    core_part!("kinds.zaka"),
    core_part!("traits.zaka"),
    core_part!("verbs.zaka"),
    core_part!("heap.zaka"),
];

/// The parts of the core joined into one text, for the readers that take
/// the declarations of the whole core from its words.
pub const CORE_SOURCE: &str = concat!(
    include_str!("../../../stdlib/core/kinds.zaka"),
    "\n",
    include_str!("../../../stdlib/core/traits.zaka"),
    "\n",
    include_str!("../../../stdlib/core/verbs.zaka"),
    "\n",
    include_str!("../../../stdlib/core/heap.zaka"),
);

/// The file of the stdlib that a module or a part of the core is read
/// from, named as the source map holds it.
///
/// # Arguments
///
/// * `name` - The import name of a module, or the name of a part.
///
/// # Returns
///
/// The path in the `stdlib` folder and the source, or `None` for another name.
pub fn std_file(name: &str) -> Option<(&'static str, &'static str)>
{
    let module =
        STD_MODULES.iter().find(|(m, ..)| *m == name).map(|(_, f, s)| (*f, *s));
    let part = || {
        let file = name.strip_prefix(CORE)?.strip_prefix('/')?;
        CORE_PARTS.iter().find(|(f, _)| f.strip_prefix("core/") == Some(file)).copied()
    };
    module.or_else(part)
}

/// The folder of the replicas of the stdlib that a language server writes, so
/// an editor can open a declaration of the stdlib: `zakac/stdlib/<version>`
/// in a cache folder.
///
/// # Arguments
///
/// * `cache` - The cache folder of the user.
///
/// # Returns
///
/// The folder.
pub fn std_replicas(cache: &Path) -> PathBuf
{
    cache.join("zakac").join("stdlib").join(env!("CARGO_PKG_VERSION"))
}

/// The module of the stdlib a file holds when it is a file of the source tree
/// or a replica in [`std_replicas`], so a check of `stdlib/core/verbs.zaka`
/// in an editor reads it as `စံကား/အခြေခံအုတ်မြစ်`, compiler marks and all: a
/// file of the `core` folder reads as the core.
///
/// # Arguments
///
/// * `path` - The path of the file.
///
/// # Returns
///
/// The import name, or `None` for a file outside the `stdlib` folder this
/// compiler was built from and outside a folder of replicas.
fn std_module_at(path: &Path) -> Option<&'static str>
{
    let tree = normalize(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"));
    let path = normalize(&std::path::absolute(path).ok()?);
    let folder = path.parent()?;
    let replicas = std_replicas(Path::new(""));
    let in_tree = |f: &Path| f == tree || f.ends_with(&replicas);
    let core = folder.file_name().is_some_and(|f| f == "core");
    ret_if!(core && folder.parent().is_some_and(in_tree), Some(CORE));
    ret_if!(!in_tree(folder), None);
    let file = path.file_name()?;
    STD_MODULES.iter().find(|(_, f, _)| file == *f).map(|(module, ..)| *module)
}

impl Loaded
{
    /// Lex and parse one file that cannot read other files: each import it
    /// has is reported, except an import from the stdlib, which reads the
    /// module the compiler carries; a file that names a declaration of the
    /// core reads the core too.
    ///
    /// # Arguments
    ///
    /// * `source` - The file.
    /// * `options` - The lexer options.
    ///
    /// # Returns
    ///
    /// The program of the file, settled for the host.
    pub fn single(source: &SourceFile, options: &LexerOptions) -> Self
    {
        Loaded::single_for(source, options, &Target::host())
    }

    /// [`Loaded::single`] for a target: each compile-time conditional keeps
    /// the arm that holds for it, and the core's marked constants hold
    /// its facts.
    ///
    /// # Arguments
    ///
    /// * `source` - The file.
    /// * `options` - The lexer options.
    /// * `target` - The target.
    ///
    /// # Returns
    ///
    /// The program of the file.
    pub fn single_for(
        source: &SourceFile, options: &LexerOptions, target: &Target,
    ) -> Self
    {
        Loaded::single_resolved(source, options, target, &Resolved::new())
    }

    /// [`Loaded::single_for`] with the conditions that call comptime
    /// functions worked out so far.
    ///
    /// # Arguments
    ///
    /// * `source` - The file.
    /// * `options` - The lexer options.
    /// * `target` - The target.
    /// * `resolved` - The truth of each condition worked out so far.
    ///
    /// # Returns
    ///
    /// The program of the file.
    pub fn single_resolved(
        source: &SourceFile, options: &LexerOptions, target: &Target,
        resolved: &Resolved,
    ) -> Self
    {
        let lexed = lex(source, options);
        let parsed = parse(source, &lexed);
        let mut imports = parsed.program.stmts.iter().filter_map(as_import);
        let std = imports.any(is_std) || uses_core(source, &lexed);
        ret_if!(std, Loaded::with_std(source, options, target, resolved));
        let mut diagnostics = lexed.diagnostics;
        diagnostics.extend(parsed.diagnostics);
        let mut program = parsed.program;
        let text_of = |span: Span| snippet(source, span);
        #[rustfmt::skip]
        let spot = Spot { file: source.name.to_string(), base: source.start_pos.as_u32(),
            resolved };
        let (stmts, settled, met, open) =
            settle(program.stmts, target, &text_of, &spot);
        program.stmts = stmts;
        diagnostics.extend(settled);
        diagnostics.extend(refuse_imports(&program));
        let fixed = Fixed { target, next_id: parsed.next_id, used: met };
        Loaded { open_conds: open, ..loaded(program, Vec::new(), diagnostics, &fixed) }
    }

    /// Read one file with the core and the stdlib modules it imports, in a
    /// source map of its own that starts with the file; each other import is
    /// reported.
    ///
    /// # Arguments
    ///
    /// * `source` - The file.
    /// * `options` - The lexer options.
    ///
    /// # Returns
    ///
    /// The program, imported modules first.
    fn with_std(
        source: &SourceFile, options: &LexerOptions, target: &Target,
        resolved: &Resolved,
    ) -> Self
    {
        let root = Arc::new(source.clone());
        let mut source_map = SourceMap::new();
        source_map.add_file(root.clone());
        #[rustfmt::skip]
        let mut loader = Loader::builder().source_map(&mut source_map).options(options)
            .folder(PathBuf::new()).disk(false).target(*target).resolved(resolved.clone())
            .build();
        let path = PathBuf::from(source.name.to_string());
        let read = loader.visit(root, path);
        read.expect("a file read from storage reads no file from disk");
        let fixed = Fixed { target, next_id: loader.next_id, used: loader.target_used };
        let Loader { stmts, units, imports, diagnostics, open, .. } = loader;
        let span = zakac_span::span::Span::new(source.start_pos, source.end_pos());
        let program = Program::builder().stmts(stmts).units(units).span(span).build();
        Loaded { open_conds: open, ..loaded(program, imports, diagnostics, &fixed) }
    }
}

/// What a program read for a target is settled with.
struct Fixed<'t>
{
    /// The target.
    target: &'t Target,
    /// The first node number no node of the program holds.
    next_id: u32,
    /// Whether a compile-time conditional or a fact of the target is met.
    used: bool,
}

/// A program read with its imports, each kind alias in it expanded and each
/// marked constant of the core given the value of the target.
///
/// # Arguments
///
/// * `program` - The program.
/// * `imports` - The imported files.
/// * `diagnostics` - The diagnostics so far; the alias diagnostics follow.
/// * `fixed` - The target, and what the program met of it.
///
/// # Returns
///
/// The loaded program.
fn loaded(
    mut program: Program, imports: Vec<EmbeddedFile>, mut diagnostics: Vec<Diagnostic>,
    fixed: &Fixed<'_>,
) -> Loaded
{
    fill_facts(&mut program, fixed.target, fixed.next_id);
    program.target = fixed.used.then_some(*fixed.target);
    let (expanded, alias_refs) = expand_aliases(&mut program);
    diagnostics.extend(expanded);
    #[rustfmt::skip]
    return Loaded::builder().program(program).imports(imports).diagnostics(diagnostics)
        .alias_refs(alias_refs).build();
}

/// Whether an import reads a module of the stdlib.
///
/// # Arguments
///
/// * `n` - The import.
///
/// # Returns
///
/// `true` when its path starts with [`STD_PREFIX`].
fn is_std(n: &ImportStmt) -> bool
{
    canonical(&n.path.lit.value).starts_with(STD_PREFIX)
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
/// The program, settled for the host, or the error of a file that could not
/// be read.
pub fn load(
    root: &Arc<SourceFile>, source_map: &mut SourceMap, options: &LexerOptions,
) -> zakac_host_errors::Result<Loaded>
{
    load_for(root, source_map, options, &Target::host())
}

/// [`load`] for a target: each compile-time conditional keeps the arm that
/// holds for it, and the core's marked constants hold its facts.
///
/// # Arguments
///
/// * `root` - The file that is run.
/// * `source_map` - The source map; imported files are added to it.
/// * `options` - The lexer options.
/// * `target` - The target.
///
/// # Returns
///
/// The program, or the error of a file that could not be read.
pub fn load_for(
    root: &Arc<SourceFile>, source_map: &mut SourceMap, options: &LexerOptions,
    target: &Target,
) -> zakac_host_errors::Result<Loaded>
{
    load_resolved(root, source_map, options, target, &Resolved::new())
}

/// [`load_for`] with the conditions that call comptime functions worked out
/// so far.
///
/// # Arguments
///
/// * `root` - The file that is run.
/// * `source_map` - The source map, which gets each file read.
/// * `options` - The lexer options.
/// * `target` - The target.
/// * `resolved` - The truth of each condition worked out so far.
///
/// # Returns
///
/// The program, or the error of a file that could not be read.
pub fn load_resolved(
    root: &Arc<SourceFile>, source_map: &mut SourceMap, options: &LexerOptions,
    target: &Target, resolved: &Resolved,
) -> zakac_host_errors::Result<Loaded>
{
    let FileName::Real(real) = &root.name
    else
    {
        return Ok(Loaded::single_resolved(root, options, target, resolved));
    };
    let path = normalize(&real.local_path);
    let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let path = std_module_at(&path).map_or(path, PathBuf::from);
    #[rustfmt::skip]
    let mut loader = Loader::builder().source_map(source_map).options(options)
        .folder(folder).target(*target).resolved(resolved.clone()).build();
    loader.visit(root.clone(), path)?;
    let fixed = Fixed { target, next_id: loader.next_id, used: loader.target_used };
    let Loader { stmts, units, imports, diagnostics, open, .. } = loader;
    let span = zakac_span::span::Span::new(root.start_pos, root.end_pos());
    let program = Program::builder()
        .stmts(stmts)
        .units(units)
        .span(span)
        .maybe_folder(real.local_path.parent().map(Path::to_path_buf))
        .build();
    Ok(Loaded { open_conds: open, ..loaded(program, imports, diagnostics, &fixed) })
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
    /// The functions each unit exports that declare prepositions, with the
    /// number of prepositions of each.
    #[builder(default)]
    prepositions: Vec<Prepositions>,
    /// The written-as hints of the kinds and traits each unit exports.
    #[builder(default)]
    hints: Vec<Vec<Hint>>,
    /// The names each unit exports, in order, each with the unit that
    /// declares it: the unit itself, or the file a share passes it on from.
    #[builder(default)]
    exports: Vec<Vec<(String, u32)>>,
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
    /// Whether imports can read files from disk; a file read from storage
    /// imports from the stdlib alone.
    #[builder(default = true)]
    disk: bool,
    /// Whether a file read so far names a declaration of the core.
    #[builder(default)]
    wants_core: bool,
    /// Whether the core was read, or is being read.
    #[builder(default)]
    core_read: bool,
    /// The target the files are settled for.
    #[builder(default)]
    target: Target,
    /// Whether a file read so far holds a compile-time conditional or names a
    /// fact of the target.
    #[builder(default)]
    target_used: bool,
    /// The truth of each condition that calls a comptime function, worked
    /// out in a read before.
    #[builder(default)]
    resolved: Resolved,
    /// Each condition still open, with its place.
    #[builder(default)]
    open: Vec<(Span, CondPlace)>,
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
        // A part of the core opened on its own reads as the core, so the
        // whole core, read too, would declare its names twice.
        self.core_read |= path == Path::new(CORE);
        self.wants_core |= uses_core(&file, &lexed);
        // The imports do not hang on struct names, so a first parse finds
        // them; the second parse knows the struct names those imports bring.
        let first = parse(&file, &lexed);
        // A compile-time conditional can hold imports: the arm it keeps gives
        // them, in both parses alike.
        let text_of = |span: Span| snippet(&file, span);
        let resolved = self.resolved.clone();
        #[rustfmt::skip]
        let spot = Spot { file: self.shown(&path), base: file.start_pos.as_u32(),
            resolved: &resolved };
        let (first_stmts, ..) =
            settle(first.program.stmts, &self.target, &text_of, &spot);
        let imports: Vec<ImportStmt> =
            first_stmts.iter().filter_map(as_import).cloned().collect();
        let shares: Vec<ShareStmt> =
            first_stmts.iter().filter_map(as_share).cloned().collect();
        self.chain.push(path.clone());
        let mut targets = Vec::new();
        for n in &imports
        {
            targets.push(self.import(&path, n)?);
        }
        self.chain.pop();
        // The core is read once a file names one of its declarations, as
        // a file of its own ahead of that file.
        do_if!(self.wants_core, self.read_core());
        let own: HashSet<String> = top_names(&first_stmts, false).into_iter().collect();
        let brought = self.bring(&imports, &targets, &own);
        let seen = imports.iter().zip(&brought.names).map(|(n, b)| {
            let names = b.iter().map(|(i, _)| i.clone()).collect();
            ImportStmt { names, ..n.clone() }
        });
        let seen: Vec<ImportStmt> = seen.collect();
        let structs = imported_structs(&seen, &targets, &self.structs);
        let prepositions = imported_prepositions(&seen, &targets, &self.prepositions);
        let hints = imported_hints(&seen, &targets, &self.hints);
        #[rustfmt::skip]
        let parsed = parse_with(&file, &lexed, self.next_id, &structs, &prepositions, &hints);
        self.next_id = parsed.next_id;
        self.diagnostics.extend(lexed.diagnostics.iter().cloned());
        self.diagnostics.extend(parsed.diagnostics);
        let mut program = parsed.program;
        let (stmts, settled, met, open) =
            settle(program.stmts, &self.target, &text_of, &spot);
        program.stmts = stmts;
        self.diagnostics.extend(settled);
        self.open.extend(open);
        split_imports(&mut program, &imports, &targets, &brought.names);
        self.report_clashes(&file, &lexed, &brought.clashes);
        let shared = self.shared(&shares, &imports, &targets, &brought);
        let facts = named_facts(&file, &lexed);
        self.target_used |= met || !facts.is_empty();
        self.import_facts(&mut program, &path, facts);
        do_if!(self.disk, lib_paths_from(&mut program, &path));
        let unit = self.units.len() as u32;
        let name = self.shown(&path);
        let first_stmt = self.stmts.len() as u32;
        self.units.push(Unit::builder().name(name).first_stmt(first_stmt).build());
        self.structs.push(exported_structs(&program));
        self.prepositions.push(exported_prepositions(&program));
        self.hints.push(exported_hints(&program));
        let own = top_names(&program.stmts, true).into_iter().map(|n| (n, unit));
        self.exports.push(own.collect());
        do_for!((name, origin, from) in shared, self.pass_on(&name, origin, from));
        self.stmts.extend(program.stmts);
        self.done.insert(path, unit);
        Ok(unit)
    }

    /// A file name as messages show it: relative to the folder of the file
    /// that is run, when it is inside that folder, with `/` between its
    /// parts on each host, as the imports of a program write them.
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
        let name =
            path.strip_prefix(&self.folder).unwrap_or(path).display().to_string();
        name.replace(std::path::MAIN_SEPARATOR, "/")
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
        ret_if!(is_std(n), Ok(self.import_std(n)));
        ret_if!(importer == Path::new(CORE), Ok(self.import_core_part(n)));
        if !self.disk
        {
            self.diagnostics.push(diag::import_needs_file(n.span, &n.path.lit.value));
            return Ok(None);
        }
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

/// What the imports of a file bring.
struct Brought
{
    /// The names each import brings, each with the unit that declares it; a
    /// share of a whole file brings none.
    names: Vec<Vec<(Ident, u32)>>,
    /// Each name two whole imports bring from two files, with the paths of
    /// the two imports.
    clashes: HashMap<String, (String, String)>,
}

impl Loader<'_>
{
    /// The names each import of a file brings. A named import brings the
    /// names it lists; a whole import, each name its file exports, but for a
    /// name the file declares, one a named import brings, and one another
    /// whole import brings from a different file, which clashes.
    ///
    /// # Arguments
    ///
    /// * `imports` - The imports of the file.
    /// * `targets` - The unit each reads.
    /// * `own` - The top-level names the file declares.
    ///
    /// # Returns
    ///
    /// The names.
    fn bring(
        &self, imports: &[ImportStmt], targets: &[Option<u32>], own: &HashSet<String>,
    ) -> Brought
    {
        let named = |(n, t): (&ImportStmt, &Option<u32>)| {
            let t = t.filter(|_| !n.whole && !n.share);
            let found = t.map(move |t| {
                n.names.iter().map(move |i| (i.clone(), self.origin(t, i)))
            });
            found.into_iter().flatten().collect::<Vec<_>>()
        };
        let mut names: Vec<Vec<(Ident, u32)>> =
            imports.iter().zip(targets).map(named).collect();
        let taken: HashSet<String> =
            names.iter().flatten().map(|(i, _)| i.name.to_string()).collect();
        let mut whole =
            Whole { own, taken, first: HashMap::new(), clashes: HashMap::new() };
        let wholes = imports.iter().zip(targets).enumerate();
        let wholes = wholes.filter_map(|(k, (n, t))| Some((k, t.filter(|_| n.whole)?)));
        for (k, t) in wholes
        {
            let exports = &self.exports[t as usize];
            do_for!((name, origin) in exports, whole.add(&mut names, imports, k, name, *origin));
        }
        let clashes = whole.clashes;
        let clear = |list: &mut Vec<(Ident, u32)>| {
            list.retain(|(i, _)| !clashes.contains_key(i.name.as_str()));
        };
        names.iter_mut().for_each(clear);
        Brought { names, clashes }
    }

    /// The unit that declares a name a unit exports: the unit itself, or the
    /// file a share passes it on from; the unit itself for a name it does not
    /// export, which the check reports.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit.
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The declaring unit.
    fn origin(&self, unit: u32, name: &Ident) -> u32
    {
        let exports = &self.exports[unit as usize];
        let found = exports.iter().find(|(n, _)| n.as_str() == name.name.as_str());
        found.map_or(unit, |(_, origin)| *origin)
    }

    /// Report the first use of each name two whole imports bring from two
    /// files.
    ///
    /// # Arguments
    ///
    /// * `file` - The file.
    /// * `lexed` - Its tokens.
    /// * `clashes` - Each such name, with the two imports.
    fn report_clashes(
        &mut self, file: &SourceFile, lexed: &LexOutput,
        clashes: &HashMap<String, (String, String)>,
    )
    {
        ret_if!(clashes.is_empty());
        let mut seen = HashSet::new();
        let used = undeclared_names(file, lexed, |t| clashes.contains_key(t));
        let first = used.into_iter().filter(|i| seen.insert(i.name));
        for name in first
        {
            let (a, b) = &clashes[name.name.as_str()];
            self.diagnostics.push(diag::import_clash(name.span, &name.name, a, b));
        }
    }

    /// The names a file shares again: each name a share lists, which the
    /// file must import, and each name a file it shares whole exports.
    ///
    /// # Arguments
    ///
    /// * `shares` - The shares of the file.
    /// * `imports` - Its imports.
    /// * `targets` - The unit each import reads.
    /// * `brought` - What the imports bring.
    ///
    /// # Returns
    ///
    /// Each name, with the unit that declares it and the unit it comes from.
    fn shared(
        &mut self, shares: &[ShareStmt], imports: &[ImportStmt],
        targets: &[Option<u32>], brought: &Brought,
    ) -> Vec<(String, u32, u32)>
    {
        let mut found = Vec::new();
        for name in shares.iter().flat_map(|s| &s.names)
        {
            let clash = brought.clashes.contains_key(name.name.as_str());
            let item = bringing_import(name, targets, &brought.names);
            let missing = item.is_none() && !clash;
            found.extend(item);
            do_if!(
                missing,
                self.diagnostics.push(diag::share_not_imported(name.span, &name.name))
            );
        }
        let wholes = imports.iter().zip(targets).filter(|(n, _)| n.share);
        let wholes = wholes.filter_map(|(_, t)| *t);
        for t in wholes
        {
            let exports = self.exports[t as usize].iter();
            found.extend(exports.map(|(name, origin)| (name.clone(), *origin, t)));
        }
        found
    }

    /// Export from the unit read last a name it shares again, with what the
    /// unit it comes from exports of that name: its struct names, its
    /// functions and its hints. A name the unit exports before keeps its first
    /// export.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `origin` - The unit that declares it.
    /// * `from` - The unit the share takes it from.
    fn pass_on(&mut self, name: &str, origin: u32, from: u32)
    {
        let (from, at) = (from as usize, self.exports.len() - 1);
        ret_if!(self.exports[at].iter().any(|(n, _)| n == name));
        self.exports[at].push((name.to_string(), origin));
        let stem = name.strip_suffix(PLURAL);
        let structs = self.structs[from].iter();
        let structs: Vec<String> = structs
            .filter(|s| s.as_str() == name || Some(s.as_str()) == stem)
            .cloned()
            .collect();
        self.structs[at].extend(structs);
        let fns = self.prepositions[from].get(name).cloned();
        do_if!(let Some(fns) = fns, _ = self.prepositions[at].entry(name.to_string()).or_insert(fns));
        let hints: Vec<Hint> =
            self.hints[from].iter().filter(|h| h.name == name).cloned().collect();
        self.hints[at].extend(hints);
    }
}

/// The names the whole imports of a file bring, in the order of the imports.
struct Whole<'a>
{
    /// The top-level names the file declares.
    own: &'a HashSet<String>,
    /// The names its named imports bring.
    taken: HashSet<String>,
    /// Each name a whole import brought, with its declaring unit and the
    /// import that brought it.
    first: HashMap<String, (u32, usize)>,
    /// Each name two whole imports bring from two files, with the paths of
    /// the two imports.
    clashes: HashMap<String, (String, String)>,
}

impl Whole<'_>
{
    /// Take one name a whole import brings: a name the file declares or a
    /// named import brings is left out, a name brought before from the same
    /// declaration is brought once, and one brought from another clashes.
    ///
    /// # Arguments
    ///
    /// * `names` - The names each import brings so far.
    /// * `imports` - The imports.
    /// * `k` - The whole import.
    /// * `name` - The name.
    /// * `origin` - The unit that declares it.
    fn add(
        &mut self, names: &mut [Vec<(Ident, u32)>], imports: &[ImportStmt], k: usize,
        name: &str, origin: u32,
    )
    {
        ret_if!(self.own.contains(name) || self.taken.contains(name));
        ret_if!(self.clashes.contains_key(name));
        let path = |k: usize| imports[k].path.lit.value.clone();
        let Some((was, held)) = self.first.get(name).copied()
        else
        {
            self.first.insert(name.to_string(), (origin, k));
            let span = imports[k].path.span;
            names[k].push((Ident::builder().name(name).span(span).build(), origin));
            return;
        };
        ret_if!(was == origin);
        self.clashes.insert(name.to_string(), (path(held), path(k)));
    }
}

/// The import that brings a name, as a name a file shares again.
///
/// # Arguments
///
/// * `name` - The name.
/// * `targets` - The unit each import reads.
/// * `names` - The names each import brings.
///
/// # Returns
///
/// The name, its declaring unit and the unit its import reads; `None` when no
/// import brings it.
fn bringing_import(
    name: &Ident, targets: &[Option<u32>], names: &[Vec<(Ident, u32)>],
) -> Option<(String, u32, u32)>
{
    let held = |(list, t): (&Vec<(Ident, u32)>, &Option<u32>)| {
        let (_, origin) = list.iter().find(|(i, _)| i.name == name.name)?;
        Some((name.name.to_string(), *origin, (*t)?))
    };
    names.iter().zip(targets).find_map(held)
}

/// The top-level names a file declares: its functions, structs,
/// enumerations, kind aliases, variables, constants and traits.
///
/// # Arguments
///
/// * `stmts` - The statements of the file.
/// * `exported` - Whether to keep the exported names alone.
///
/// # Returns
///
/// The names, in order.
fn top_names(stmts: &[Stmt], exported: bool) -> Vec<String>
{
    let named = stmts.iter().filter_map(|s| match s
    {
        Stmt::Fn(d) => Some((&d.name, d.exported)),
        Stmt::Struct(d) => Some((&d.name, d.exported)),
        Stmt::Enum(d) => Some((&d.name, d.exported)),
        Stmt::Alias(d) => Some((&d.name, d.exported)),
        Stmt::Decl(d) => Some((&d.name, d.exported)),
        Stmt::Trait(d) => Some((&d.name, d.exported)),
        _ => None,
    });
    let kept = named.filter(|(_, e)| *e || !exported).map(|(n, _)| n.name.to_string());
    let mut seen = HashSet::new();
    kept.filter(|n| seen.insert(n.clone())).collect()
}

impl Loader<'_>
{
    /// Give a file the facts of the target it names, as imports of them from
    /// the parts of the core ahead of its statements: the core's constants are
    /// seen in each file with no import written.
    ///
    /// # Arguments
    ///
    /// * `program` - The program of the file.
    /// * `path` - Its path.
    /// * `facts` - The facts it names, each at its first use.
    fn import_facts(&mut self, program: &mut Program, path: &Path, facts: Vec<Ident>)
    {
        let core = self.done.get(Path::new(CORE)).copied();
        let Some(unit) = core.filter(|_| path != Path::new(CORE) && !facts.is_empty())
        else
        {
            return;
        };
        let span = facts[0].span;
        let lit = StrLit { parts: Vec::new(), value: CORE.to_string() };
        let path = StrArg::builder().lit(lit).span(span).build();
        let brought: Vec<(Ident, u32)> =
            facts.iter().map(|f| (f.clone(), self.origin(unit, f))).collect();
        #[rustfmt::skip]
        let import = ImportStmt::builder().path(path).names(facts).unit(unit).span(span)
            .build();
        // Each fact is imported from the part of the core that declares it.
        let imports = split_import(import, unit, &brought);
        program.stmts.splice(0..0, imports);
    }

    /// Read the core, the module of the stdlib each file sees with no
    /// import, as a file of the program, once.
    fn read_core(&mut self)
    {
        ret_if!(self.core_read);
        self.core_read = true;
        let path = PathBuf::from(CORE);
        let (_, _, src) = STD_MODULES
            .iter()
            .find(|(module, ..)| *module == CORE)
            .expect("the stdlib carries the core");
        let file = self
            .source_map
            .new_source_file(FileName::Custom(CORE.to_string()), src.to_string());
        self.imports.push(EmbeddedFile::of(&file));
        let read = self.visit(file, path);
        read.expect("the core reads no file from disk");
    }

    /// Read the part of the core a share of `core/mod.zaka` names, from the
    /// source the compiler carries, unless it was read before.
    ///
    /// # Arguments
    ///
    /// * `n` - The share.
    ///
    /// # Returns
    ///
    /// The unit of the part; `None` for a file the core lacks, which is
    /// reported.
    fn import_core_part(&mut self, n: &ImportStmt) -> Option<u32>
    {
        let path = Path::new(CORE).join(&n.path.lit.value);
        ret_if!(let Some(unit) = self.done.get(&path), Some(*unit));
        let file = format!("core/{}", n.path.lit.value);
        let Some((_, src)) = CORE_PARTS.iter().find(|(f, _)| *f == file)
        else
        {
            self.diagnostics.push(diag::import_needs_file(n.span, &n.path.lit.value));
            return None;
        };
        let name = FileName::Custom(self.shown(&path));
        let file = self.source_map.new_source_file(name, src.to_string());
        self.imports.push(EmbeddedFile::of(&file));
        let read = self.visit(file, path);
        Some(read.expect("a part of the core reads no file from disk"))
    }

    /// Read the module of the stdlib an import names, from the source the
    /// compiler carries, unless it was read before. A name the stdlib does
    /// not have is reported.
    ///
    /// # Arguments
    ///
    /// * `n` - The import.
    ///
    /// # Returns
    ///
    /// The unit of the module; `None` for a name the stdlib lacks.
    fn import_std(&mut self, n: &ImportStmt) -> Option<u32>
    {
        let name = canonical(&n.path.lit.value).into_owned();
        let path = PathBuf::from(&name);
        ret_if!(let Some(unit) = self.done.get(&path), Some(*unit));
        let Some((_, _, src)) = STD_MODULES.iter().find(|(module, ..)| *module == name)
        else
        {
            let list: Vec<&str> =
                STD_MODULES.iter().map(|(module, ..)| *module).collect();
            let diag = diag::unknown_std_module(n.path.span, &name, &list.join(", "));
            self.diagnostics.push(diag);
            return None;
        };
        let file =
            self.source_map.new_source_file(FileName::Custom(name), src.to_string());
        self.imports.push(EmbeddedFile::of(&file));
        let read = self.visit(file, path);
        Some(read.expect("a module of the stdlib reads no file from disk"))
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

/// The share of imported names a statement is.
///
/// # Arguments
///
/// * `stmt` - The statement.
///
/// # Returns
///
/// The share, or `None` for another statement.
fn as_share(stmt: &Stmt) -> Option<&ShareStmt>
{
    match stmt
    {
        Stmt::Share(n) => Some(n),
        _ => None,
    }
}

/// The struct and enumeration names a file imports.
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
    // an enumeration brings its stem along: `အရောင်` with `အရောင်များ`
    let with_stem = |name: &str, set: &HashSet<String>| {
        let stem = name.strip_suffix(PLURAL).filter(|s| set.contains(*s));
        std::iter::once(name.to_string()).chain(stem.map(String::from))
    };
    let names = pairs.flat_map(|(n, set)| {
        let found = n.names.iter().filter(|i| set.contains(i.name.as_str()));
        found.flat_map(|i| with_stem(i.name.as_str(), set)).collect::<Vec<_>>()
    });
    names.collect()
}

/// The functions of a file, each name with the prepositions of each function of
/// that name.
type Prepositions = Functions;

/// The imported names that are exported functions, with their prepositions. A
/// name imported from several files brings the functions of each.
///
/// # Arguments
///
/// * `imports` - The imports of the file.
/// * `targets` - The unit each import reads.
/// * `exports` - The functions each unit exports.
///
/// # Returns
///
/// The imported functions.
fn imported_prepositions(
    imports: &[ImportStmt], targets: &[Option<u32>], exports: &[Prepositions],
) -> Prepositions
{
    let pairs = imports.iter().zip(targets);
    let pairs = pairs.filter_map(|(n, t)| Some((n, &exports[(*t)? as usize])));
    let found = pairs.flat_map(|(n, set)| {
        let named = n.names.iter().filter_map(|i| set.get_key_value(i.name.as_str()));
        named.map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()
    });
    let mut merged = Prepositions::new();
    found.for_each(|(name, lists)| merged.entry(name).or_default().extend(lists));
    merged
}

/// The imported names that are kinds or traits with written-as hints, with
/// the hint of each.
///
/// # Arguments
///
/// * `imports` - The imports of the file.
/// * `targets` - The unit each import reads.
/// * `exports` - The hints each unit exports.
///
/// # Returns
///
/// The hints.
fn imported_hints(
    imports: &[ImportStmt], targets: &[Option<u32>], exports: &[Vec<Hint>],
) -> Vec<Hint>
{
    let pairs = imports.iter().zip(targets);
    let pairs = pairs.filter_map(|(n, t)| Some((n, &exports[(*t)? as usize])));
    let found = pairs.flat_map(|(n, hints)| {
        let named = |h: &&Hint| n.names.iter().any(|i| i.name.as_str() == h.name);
        hints.iter().filter(named).cloned().collect::<Vec<_>>()
    });
    found.collect()
}

/// The exported functions of a file, with the prepositions of each.
///
/// # Arguments
///
/// * `program` - The program of the file.
///
/// # Returns
///
/// The functions.
fn exported_prepositions(program: &Program) -> Prepositions
{
    let mut found = Prepositions::new();
    let decls = program.stmts.iter().filter_map(|s| match s
    {
        Stmt::Fn(d) if d.exported => Some(d),
        _ => None,
    });
    for d in decls
    {
        found.entry(d.name.name.to_string()).or_default().push(CallWords::of_decl(d));
    }
    found
}

/// The struct, enumeration and kind alias names a file exports.
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
        Stmt::Struct(d) if d.exported => Some(vec![d.name.name.to_string()]),
        Stmt::Enum(d) if d.exported => Some(enum_names(&d.name.name)),
        Stmt::Alias(d) if d.exported => Some(enum_names(&d.name.name)),
        _ => None,
    });
    decls.flatten().collect()
}

/// The names an enumeration starts a kind with: its name, and its stem when
/// the name ends in `များ`.
///
/// # Arguments
///
/// * `name` - The enumeration name.
///
/// # Returns
///
/// The names.
fn enum_names(name: &str) -> Vec<String>
{
    let stem = name.strip_suffix(PLURAL).map(String::from);
    std::iter::once(name.to_string()).chain(stem).collect()
}

/// Record on each import of a parsed file the unit it reads, and the names it
/// brings: an import whose names come from several files, through shares,
/// becomes one import of each such file. The imports of the first parse and
/// of the second come in one order.
///
/// # Arguments
///
/// * `program` - The program of the file.
/// * `imports` - The imports of the first parse.
/// * `targets` - The unit each of them reads.
/// * `brought` - The names each brings, with the unit that declares each.
fn split_imports(
    program: &mut Program, imports: &[ImportStmt], targets: &[Option<u32>],
    brought: &[Vec<(Ident, u32)>],
)
{
    let mut k = 0;
    let mut split = |stmt: Stmt| {
        let Stmt::Import(n) = stmt
        else
        {
            return vec![stmt];
        };
        k += 1;
        let same = imports.get(k - 1).is_some_and(|first| first.span == n.span);
        let unit = targets.get(k - 1).copied().flatten().filter(|_| same);
        let Some(unit) = unit
        else
        {
            return vec![Stmt::Import(ImportStmt { unit: None, ..n })];
        };
        split_import(n, unit, &brought[k - 1])
    };
    let stmts = std::mem::take(&mut program.stmts);
    program.stmts = stmts.into_iter().flat_map(&mut split).collect();
}

/// One import as an import of each file that declares a name it brings; an
/// import that brings no name remains one import of the file it reads.
///
/// # Arguments
///
/// * `n` - The import.
/// * `unit` - The unit it reads.
/// * `brought` - The names it brings, with the unit that declares each.
///
/// # Returns
///
/// The imports.
fn split_import(n: ImportStmt, unit: u32, brought: &[(Ident, u32)]) -> Vec<Stmt>
{
    ret_if!(
        brought.is_empty(),
        vec![Stmt::Import(ImportStmt { unit: Some(unit), names: Vec::new(), ..n })]
    );
    let mut origins: Vec<u32> = Vec::new();
    do_for!((_, o) in brought, do_if!(!origins.contains(o), origins.push(*o)));
    let of = |o: u32| {
        let names = brought.iter().filter(|(_, x)| *x == o).map(|(i, _)| i.clone());
        Stmt::Import(ImportStmt { unit: Some(o), names: names.collect(), ..n.clone() })
    };
    origins.into_iter().map(of).collect()
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

/// Make the lib path of each C function of a file absolute: a relative one
/// starts at the folder of the file, as an import does, so the program finds
/// its lib wherever it runs from. A lib name is left as it is.
///
/// # Arguments
///
/// * `program` - The statements of the file.
/// * `file` - The path of the file.
fn lib_paths_from(program: &mut Program, file: &Path)
{
    let folder = file.parent().unwrap_or(Path::new(""));
    let libs = program.stmts.iter_mut().filter_map(|s| match s
    {
        Stmt::Fn(f) => f.lib.as_mut(),
        _ => None,
    });
    for lib in libs.filter(|l| is_lib_path(&l.lit.value))
    {
        let joined = normalize(&folder.join(&lib.lit.value));
        let full = std::path::absolute(&joined).unwrap_or(joined);
        lib.lit.value = full.to_string_lossy().into_owned();
    }
}

/// Whether the words of a file name a declaration of the core: one of
/// its traits, and no name the file declares itself, which
/// hides the core's.
///
/// # Arguments
///
/// * `file` - The file.
/// * `lexed` - Its tokens.
///
/// # Returns
///
/// `true` when one of its names is a name the core declares.
fn uses_core(file: &SourceFile, lexed: &LexOutput) -> bool
{
    let names = core_names();
    let named = !undeclared_names(file, lexed, |t| names.contains(t)).is_empty();
    // The stats of the collector are a struct of the core that no word names.
    let auto = |t: &Token| t.kind == TokenKind::Ident && snippet(file, t.span) == AUTO;
    let stats = lexed
        .tokens
        .windows(2)
        .any(|w| auto(&w[0]) && w[1].kind == TokenKind::Kw(Kw::Mem));
    named || stats
}

/// The facts of the target a file names and does not declare itself, each
/// at its first use.
///
/// # Arguments
///
/// * `file` - The file.
/// * `lexed` - Its tokens.
///
/// # Returns
///
/// The names.
fn named_facts(file: &SourceFile, lexed: &LexOutput) -> Vec<Ident>
{
    let mut found = undeclared_names(file, lexed, |t| FACTS.contains(&t));
    let mut seen = HashSet::new();
    found.retain(|n| seen.insert(n.name));
    found
}

/// The names of a file that a test picks and that the file does not
/// declare: a name after `လုပ်ဆောင်ချက်`, `ကိန်းရှင်` or `ကိန်းသေ`, or before
/// `ဟူသော`, is declared there.
///
/// # Arguments
///
/// * `file` - The file.
/// * `lexed` - Its tokens.
/// * `pick` - The test of a name.
///
/// # Returns
///
/// Each use of such a name, in source order.
fn undeclared_names(
    file: &SourceFile, lexed: &LexOutput, pick: impl Fn(&str) -> bool,
) -> Vec<Ident>
{
    let text = |t: &Token| snippet(file, t.span);
    let tokens = &lexed.tokens;
    let opens = |t: &Token| matches!(t.kind, TokenKind::Kw(Kw::Var | Kw::Const));
    let called = |t: &Token| t.kind == TokenKind::Kw(Kw::Called);
    let declared: HashSet<String> = tokens
        .windows(2)
        .filter_map(|w| match (opens(&w[0]), called(&w[1]))
        {
            (true, _) => Some(text(&w[1])),
            (_, true) => Some(text(&w[0])),
            _ => None,
        })
        .collect();
    let idents = tokens.iter().filter(|t| t.kind == TokenKind::Ident);
    let named = idents.map(|t| (text(t), t.span));
    let named = named.filter(|(n, _)| pick(n) && !declared.contains(n));
    named
        .map(|(n, span)| Ident::builder().name(n.as_str()).span(span).build())
        .collect()
}

/// The source text of a region of a file.
///
/// # Arguments
///
/// * `file` - The file.
/// * `span` - The region.
///
/// # Returns
///
/// The text; `""` for a region outside the file.
fn snippet(file: &SourceFile, span: Span) -> String
{
    let lo = file.relative_position(&span.lo);
    let hi = file.relative_position(&span.hi);
    file.snippet(&lo, &hi).unwrap_or("").to_string()
}

/// Give each marked constant of the core the value of the target: the
/// declaration names no value, and the later stages see a plain constant.
///
/// # Arguments
///
/// * `program` - The program.
/// * `target` - The target.
/// * `next_id` - The first node number no node holds.
fn fill_facts(program: &mut Program, target: &Target, mut next_id: u32)
{
    let marked = program.stmts.iter_mut().filter_map(|s| match s
    {
        Stmt::Decl(d) if d.builtin && d.init.is_none() => Some(d),
        _ => None,
    });
    for decl in marked
    {
        let Some(value) = target.fact(&decl.name.name)
        else
        {
            continue;
        };
        let (id, span) = (NodeId(next_id), decl.name.span);
        next_id += 1;
        decl.init = Some(match value
        {
            FactValue::Text(text, _) =>
            {
                let pieces = vec![StrPiece::Text(text.to_string())];
                Expr::Str(StrExpr::builder().id(id).pieces(pieces).span(span).build())
            }
            #[rustfmt::skip]
            FactValue::Bits(bits) => Expr::Int(IntLit::builder().id(id).value(u128::from(bits))
                .base(10).script(DigitScript::Burmese).span(span).build()),
        });
    }
}

/// The names of the traits the core declares.
///
/// # Returns
///
/// The names, read from the core once.
fn core_names() -> &'static HashSet<String>
{
    static NAMES: OnceLock<HashSet<String>> = OnceLock::new();
    NAMES.get_or_init(|| {
        // The traits, and the constants the compiler gives: the facts of the
        // target.
        let names = core_stmts().iter().filter_map(|s| match s
        {
            Stmt::Trait(d) => Some(d.name.name.to_string()),
            Stmt::Decl(d) if d.builtin => Some(d.name.name.to_string()),
            _ => None,
        });
        names.collect()
    })
}

/// The functions the core declares with the compiler mark, which a
/// program sees whether it reads the core or not.
///
/// # Returns
///
/// The declarations, read from the core once.
pub fn core_builtin_fns() -> &'static [FnDecl]
{
    static FNS: OnceLock<Vec<FnDecl>> = OnceLock::new();
    FNS.get_or_init(|| {
        let marked = core_stmts().iter().filter_map(|s| match s
        {
            Stmt::Fn(d) if d.builtin => Some(d.clone()),
            _ => None,
        });
        marked.collect()
    })
}

/// The statements of the core, parsed once on their own.
///
/// # Returns
///
/// The statements.
fn core_stmts() -> &'static [Stmt]
{
    static STMTS: OnceLock<Vec<Stmt>> = OnceLock::new();
    STMTS.get_or_init(|| {
        let src = CORE_SOURCE;
        let mut sm = SourceMap::new();
        let file =
            sm.new_source_file(FileName::Custom(CORE.to_string()), src.to_string());
        parse(&file, &lex(&file, &LexerOptions::default())).program.stmts
    })
}
