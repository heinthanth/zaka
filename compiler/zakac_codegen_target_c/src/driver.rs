use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use etcetera::BaseStrategy;
use zakac_ast::target::Target;
use zakac_ast::target::TargetAbi;
use zakac_borrow_checker::artifact::Borrowchecked;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::has_errors;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_ir_builder::ir::FoldOptions;
use zakac_parser::load::Loaded;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::hash::SourceFileHash;
use zakac_span::hash::SourceFileHashAlgorithm;
use zakac_span::source_file::SourceFile;

use crate::emit::CFile;
use crate::emit::emit;
use crate::options::TargetCOptions;

/// The runtime files, written into the cache beside the generated ones.
pub const RUNTIME: &[(&str, &str)] = &[
    ("zaka_c_sugars.h", include_str!("../runtime/zaka_c_sugars.h")),
    ("zaka_target_c_runtime.h", include_str!("../runtime/zaka_target_c_runtime.h")),
    ("zaka_runtime.h", include_str!(concat!(env!("OUT_DIR"), "/zaka_runtime.h"))),
];

/// The runtime lib (`zaka_runtime`) built for the target of this compiler;
/// blank in a build for the browser.
const RUNTIME_LIB: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/libzaka_runtime.a"));

/// The OS libs the runtime lib needs at link time, one flag a line.
const NATIVE_LIBS: &str = include_str!(concat!(env!("OUT_DIR"), "/native-libs.txt"));

/// The runtime lib of the other Windows ABI, which a Windows compiler carries
/// when its build could make it; blank elsewhere.
const OTHER_LIB: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/zaka_runtime-other"));

/// The OS libs of the runtime lib of the other Windows ABI.
const OTHER_LIBS: &str =
    include_str!(concat!(env!("OUT_DIR"), "/native-libs-other.txt"));

/// The runtime lib of one ABI, as this compiler carries it.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeLib
{
    /// The octets of the lib.
    pub octets: &'static [u8],
    /// Its file name in the cache: `zaka_runtime.lib` for MSVC, else
    /// `libzaka_runtime.a`.
    pub name: &'static str,
    /// The OS libs it needs at link time, one flag a line, as rustc prints
    /// them.
    libs: &'static str,
}

impl RuntimeLib
{
    /// The OS libs the lib needs at link time: `-l` flags, or the `.lib`
    /// files and `/defaultlib:` flags of MSVC.
    ///
    /// # Returns
    ///
    /// The flags.
    pub fn native_libs(&self) -> impl Iterator<Item = &'static str>
    {
        // The C compiler links libc (libSystem on macOS) on its own, and the
        // link adds libm, so the list leaves them out.
        const LINKED: &[&str] = &["-lc", "-lSystem", "-lm"];
        self.libs.lines().filter(|l| !LINKED.contains(l))
    }
}

/// The ABI this compiler was built for, whose runtime lib each build of it
/// carries.
///
/// # Returns
///
/// The ABI.
pub fn host_abi() -> TargetAbi { Target::host().abi }

/// The runtime lib of an ABI.
///
/// # Arguments
///
/// * `abi` - The ABI.
///
/// # Returns
///
/// The lib, or `None` for an ABI this compiler carries no runtime of.
pub fn runtime_lib(abi: TargetAbi) -> Option<RuntimeLib>
{
    let name = tern!(abi == TargetAbi::Msvc, "zaka_runtime.lib", "libzaka_runtime.a");
    let other = match host_abi()
    {
        TargetAbi::Gnu if cfg!(windows) => TargetAbi::Msvc,
        TargetAbi::Msvc => TargetAbi::Gnu,
        _ => host_abi(),
    };
    let (octets, libs) = match abi
    {
        a if a == host_abi() => (RUNTIME_LIB, NATIVE_LIBS),
        a if a == other => (OTHER_LIB, OTHER_LIBS),
        _ => return None,
    };
    ret_if!(octets.is_empty() && !RUNTIME_LIB.is_empty(), None);
    Some(RuntimeLib { octets, name, libs })
}

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
/// * `target` - The choices of the C target; its overflow mode and platform
///   width come from `mem` and `options`.
///
/// # Returns
///
/// The output of each phase.
pub fn translate(
    loaded: Loaded, sources: &[Arc<SourceFile>], options: &SemcheckOptions,
    folding: &FoldOptions, mem: &BorrowCheckOptions, target: TargetCOptions,
    grants: &zaka_host::Grants,
) -> Translated
{
    let checked =
        zakac_borrow_checker::artifact::run_loaded(loaded, options, folding, mem);
    let clean = !has_errors(&checked.diagnostics);
    let platform_bits = options.platform_bits;
    let target = TargetCOptions { overflow: mem.overflow, platform_bits, ..target };
    let folded = checked.lowered.folded.as_ref().filter(|_| clean);
    let files = folded.map(|m| emit(m, sources, target, grants));
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

/// Write the generated files and the runtime of this compiler's own ABI into
/// the cache folder.
///
/// # Arguments
///
/// * `dir` - The cache folder; it is made when missing.
/// * `files` - The generated files.
///
/// # Returns
///
/// The paths of the C files to compile, then the runtime lib to link, or the
/// error of a write.
pub fn write_cache(
    dir: &Path, files: &[CFile],
) -> zakac_host_errors::Result<Vec<PathBuf>>
{
    write_cache_for(dir, files, host_abi())
}

/// Write the generated files and the runtime of an ABI into the cache folder.
///
/// # Arguments
///
/// * `dir` - The cache folder; it is made when missing.
/// * `files` - The generated files.
/// * `abi` - The ABI of the executable.
///
/// # Returns
///
/// The paths of the C files to compile, then the runtime lib to link, or the
/// error of a write or of an ABI this compiler carries no runtime of.
pub fn write_cache_for(
    dir: &Path, files: &[CFile], abi: TargetAbi,
) -> zakac_host_errors::Result<Vec<PathBuf>>
{
    let abi_name: &'static str = abi.into();
    let runtime =
        runtime_lib(abi).ok_or(Error::RuntimeNotCarried { abi: abi_name.into() })?;
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
    let lib = dir.join(runtime.name);
    let same = std::fs::read(&lib).is_ok_and(|old| old == runtime.octets);
    do_if!(!same, std::fs::write(&lib, runtime.octets).map_err(fail(&lib))?);
    sources.push(lib);
    Ok(sources)
}
