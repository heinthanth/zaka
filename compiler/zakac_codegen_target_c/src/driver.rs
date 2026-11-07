use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use etcetera::BaseStrategy;
use zakac_borrow_checker::artifact::Borrowchecked;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::has_errors;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_ir_builder::ir::FoldOptions;
use zakac_parser::load::Loaded;
use zakac_rust_sugars::tern;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::hash::SourceFileHash;
use zakac_span::hash::SourceFileHashAlgorithm;
use zakac_span::source_file::SourceFile;

use crate::emit::CFile;
use crate::emit::NotBuilt;
use crate::emit::emit;
use crate::options::Checks;
use crate::options::TargetCOptions;

/// The runtime files, written into the cache beside the generated ones.
pub const RUNTIME: &[(&str, &str)] = &[
    ("zaka_c_sugars.h", include_str!("../runtime/zaka_c_sugars.h")),
    ("zaka_rt.h", include_str!("../runtime/zaka_rt.h")),
    ("zaka_rt.c", include_str!("../runtime/zaka_rt.c")),
    ("zaka_rt_data.c", include_str!("../runtime/zaka_rt_data.c")),
    ("zaka_rt_host.c", include_str!("../runtime/zaka_rt_host.c")),
];

/// The output of the phases up to the C text.
#[derive(bon::Builder)]
pub struct Translated
{
    /// The output of the phases up to the ownership check.
    pub borrowchecked: Borrowchecked,
    /// The C files, or the construct the C target cannot build; `None` when
    /// an earlier phase reported an error.
    pub files: Option<Result<Vec<CFile>, Error>>,
}

impl Translated
{
    /// The diagnostics of each phase.
    ///
    /// # Returns
    ///
    /// The diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] { &self.borrowchecked.diagnostics }
}

/// Run each phase from the check to the ownership check on a program read
/// with its imports, then write it as C when no phase reported an error.
///
/// # Arguments
///
/// * `loaded` - The program and the lexer and parser diagnostics.
/// * `sources` - The source files of the program, in source map order.
/// * `options` - The check options.
/// * `folding` - The foldings to run.
/// * `mem` - How the ownership check reports, and the overflow mode.
/// * `checks` - The run-time checks of the executable.
///
/// # Returns
///
/// The output of each phase.
pub fn translate(
    loaded: Loaded, sources: &[Arc<SourceFile>], options: &SemcheckOptions,
    folding: &FoldOptions, mem: &BorrowCheckOptions, checks: Checks,
) -> Translated
{
    let checked =
        zakac_borrow_checker::artifact::run_loaded(loaded, options, folding, mem);
    let clean = !has_errors(&checked.diagnostics);
    #[rustfmt::skip]
    let target = TargetCOptions::builder().checks(checks).overflow(mem.overflow)
        .platform_bits(options.platform_bits).build();
    let folded = checked.lowered.folded.as_ref().filter(|_| clean);
    let built = folded.map(|m| emit(m, sources, target));
    let mut checked = checked;
    let files = match built
    {
        Some(Err(NotBuilt::Kinds(diags))) =>
        {
            checked.diagnostics.extend(diags);
            None
        }
        Some(Err(NotBuilt::Missing(error))) => Some(Err(error)),
        Some(Ok(files)) => Some(Ok(files)),
        None => None,
    };
    Translated::builder().borrowchecked(checked).maybe_files(files).build()
}

/// The cache folder beside the input, when the user's cache folder is
/// missing or cannot be made; hidden, as the caches of Zig and of Python tools
/// are.
const LOCAL_CACHE: &str = ".zakac_c_cache";

/// The cache folder of an input file when `--cache` is absent:
/// `zakac/<folder>/<input>-<hash>` in the user's cache folder, which is
/// `XDG_CACHE_HOME` or `~/.cache` (each host but Windows) or `%LOCALAPPDATA%`
/// (Windows). `<folder>` is the folder that holds the input, `<input>` its
/// name without the extension, and `<hash>` the start of the SHA-256 of its
/// full path, so two inputs of one name never share a cache. The folder is
/// made here; when the user has no cache folder, or it cannot be made (no
/// home, no permission), the cache is `.zakac_c_cache` beside the input.
///
/// # Arguments
///
/// * `input` - The input file.
///
/// # Returns
///
/// The cache folder.
pub fn default_cache(input: &Path) -> PathBuf
{
    let full = input.canonicalize().or_else(|_| std::path::absolute(input));
    let full = full.unwrap_or_else(|_| input.to_path_buf());
    let local = full.parent().unwrap_or(Path::new("")).join(LOCAL_CACHE);
    let Some(dir) = user_cache(&full)
    else
    {
        return local;
    };
    tern!(std::fs::create_dir_all(&dir).is_ok(), dir, local)
}

/// The folder for an input in the user's cache folder.
///
/// # Arguments
///
/// * `full` - The full path of the input.
///
/// # Returns
///
/// `zakac/<folder>/<input>-<hash>` there, or `None` when the user has no
/// cache folder.
fn user_cache(full: &Path) -> Option<PathBuf>
{
    let base = etcetera::choose_base_strategy().ok()?.cache_dir();
    let raw = full.as_os_str().as_encoded_bytes();
    let hash = SourceFileHash::compute(SourceFileHashAlgorithm::Sha256, raw);
    let short: String = hash.hash[..8].iter().map(|b| format!("{b:02x}")).collect();
    let name = |p: Option<&std::ffi::OsStr>| {
        p.unwrap_or_default().to_string_lossy().into_owned()
    };
    let folder = name(full.parent().and_then(Path::file_name));
    let stem = name(full.file_stem());
    Some(base.join("zakac").join(folder).join(format!("{stem}-{short}")))
}

/// Write the generated files and the runtime into the cache folder.
///
/// # Arguments
///
/// * `dir` - The cache folder; it is made when missing.
/// * `files` - The generated files.
///
/// # Returns
///
/// The paths of the C files to compile, or the error of a write.
pub fn write_cache(
    dir: &Path, files: &[CFile],
) -> zakac_host_errors::Result<Vec<PathBuf>>
{
    let fail = |path: &Path| {
        let path = path.to_path_buf();
        move |e| Error::from_io(e, IOErrorContext::new(path, IOErrorAction::Write))
    };
    std::fs::create_dir_all(dir).map_err(fail(dir))?;
    let generated = files.iter().map(|f| (f.name.as_str(), f.text.as_str()));
    let mut sources = Vec::new();
    for (name, text) in RUNTIME.iter().copied().chain(generated)
    {
        let path = dir.join(name);
        std::fs::write(&path, text).map_err(fail(&path))?;
        if name.ends_with(".c")
        {
            sources.push(path);
        }
    }
    Ok(sources)
}
