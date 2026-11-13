use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use bon::Builder;
use xxhash_rust::xxh3::Xxh3;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::callbacks::CB_SOURCE;
use crate::callbacks::ffi_flags;
use crate::driver::RUNTIME;
use crate::emit::CHECKS_HEADER;
use crate::options::RunCheck;
use crate::options::RunChecks;

/// The compilers tried, in order, when neither `--cc` nor `CC` names one.
/// Each is a program and the words that come before the C flags.
const SEARCHED: &[&[&str]] = &[&["cc"], &["gcc"], &["clang"], &["zig", "cc"]];

/// The flag the generated C is written for: C11. A later `-std` from the
/// user replaces it.
const STD_FLAG: &str = "-std=c11";

/// The flags the C target adds unless `--no-default-flags` leaves them out:
/// the optimizer, no warnings about the generated code, and no fused mul-add
/// (one rounding for a product and a sum), since the VM rounds after each
/// float operation; GCC skips the `#pragma STDC FP_CONTRACT OFF` the runtime
/// holds, so the flag remains. The user flags come after them, so a later `-O3`
/// or `-ffp-contract=fast` replaces them; `-w` holds wherever it stands, so
/// warnings need `--no-default-flags`.
const DEFAULT_FLAGS: &[&str] = &["-O2", "-w", "-ffp-contract=off"];

/// What each link needs: libm, for `fmod` and kin.
const LINK_LIBS: &[&str] = &["-lm"];

/// The flags of the mem checks: the address and undefined-behavior
/// sanitizers. Both the compile and the link need them.
const SANITIZE_FLAGS: &[&str] = &["-g", "-fsanitize=address,undefined"];

/// The variable of the user's C flags.
pub const CFLAGS: &str = "CFLAGS";
/// The variable of the user's link flags.
pub const LDFLAGS: &str = "LDFLAGS";

/// A program of the toolchain, a C compiler or the program that links: the
/// program to run and the words before its flags, such as `zig` and `cc`.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Tool
{
    /// The program.
    pub program: PathBuf,
    /// The words that come before the flags.
    #[builder(default)]
    pub lead: Vec<String>,
}

impl Tool
{
    /// The tool as a reader sees it, such as `zig cc`.
    ///
    /// # Returns
    ///
    /// The text.
    pub fn shown(&self) -> String
    {
        let mut words = vec![self.program.display().to_string().replace('\\', "/")];
        words.extend(self.lead.iter().cloned());
        words.join(" ")
    }

    /// A command for this tool with its leading words.
    ///
    /// # Returns
    ///
    /// The command.
    fn command(&self) -> Command
    {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.lead);
        cmd
    }
}

/// Find the C compiler: `--cc`, then `CC`, then the first of `cc`, `gcc`,
/// `clang` and `zig cc` on the PATH.
///
/// # Arguments
///
/// * `chosen` - The compiler `--cc` names, words split on white space.
///
/// # Returns
///
/// The compiler, or the error when the one named is missing or none is on
/// the PATH.
pub fn find_cc(chosen: Option<&str>) -> zakac_host_errors::Result<Tool>
{
    let env = std::env::var("CC").ok().filter(|v| !v.trim().is_empty());
    ret_if!(let Some(words) = chosen.map(str::to_owned).or(env), named(&words));
    SEARCHED.iter().find_map(|words| from_words(words)).ok_or(Error::NoCCompiler)
}

/// A tool from the words of an option or a variable.
///
/// # Arguments
///
/// * `words` - The program, then the words before the flags, split on white
///   space.
///
/// # Returns
///
/// The tool, or the error when its program is not found.
pub fn named(words: &str) -> zakac_host_errors::Result<Tool>
{
    let split: Vec<&str> = words.split_whitespace().collect();
    from_words(&split).ok_or_else(|| Error::ToolNotFound { tool: words.trim().into() })
}

/// A tool from its words, when its program is on the PATH or names a
/// file.
///
/// # Arguments
///
/// * `words` - The program, then the words before the flags.
///
/// # Returns
///
/// The tool, or `None`.
fn from_words(words: &[&str]) -> Option<Tool>
{
    let (first, rest) = words.split_first()?;
    let program = locate(first)?;
    let lead = rest.iter().map(|w| w.to_string()).collect();
    Some(Tool::builder().program(program).lead(lead).build())
}

/// The path of a program: a path that names a file, or the first match on
/// the PATH, with `.exe` on Windows.
///
/// # Arguments
///
/// * `name` - The program name or path.
///
/// # Returns
///
/// The path, or `None`.
fn locate(name: &str) -> Option<PathBuf>
{
    let given = PathBuf::from(name);
    ret_if!(given.components().count() > 1, given.is_file().then_some(given));
    let file = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).map(|dir| dir.join(&file)).find(|p| p.is_file())
}

/// The flags in a variable, split on white space as `make` splits them;
/// none when the variable is not set.
///
/// # Arguments
///
/// * `name` - The variable, such as `CFLAGS`.
///
/// # Returns
///
/// The flags.
pub fn env_flags(name: &str) -> Vec<String>
{
    let value = std::env::var(name).unwrap_or_default();
    value.split_whitespace().map(str::to_owned).collect()
}

/// Whether the compiler builds with the sanitizers: a one-line program,
/// built in the cache.
///
/// # Arguments
///
/// * `cc` - The compiler.
/// * `cache` - The cache folder.
///
/// # Returns
///
/// `true` when the build succeeds.
pub fn has_sanitizers(cc: &Tool, cache: &Path) -> bool
{
    let src = cache.join("zk_probe.c");
    ret_if!(std::fs::write(&src, "int main(void) { return 0; }\n").is_err(), false);
    let exe = cache.join(format!("zk_probe{}", std::env::consts::EXE_SUFFIX));
    let mut cmd = cc.command();
    cmd.args(SANITIZE_FLAGS).arg(&src).arg("-o").arg(&exe);
    cmd.output().is_ok_and(|o| o.status.success())
}

/// What a build of the generated C needs.
#[derive(Debug, Clone, Builder)]
pub struct BuildPlan<'a>
{
    /// The compiler.
    pub cc: &'a Tool,
    /// The program that links, when it is not the compiler.
    pub linker: Option<&'a Tool>,
    /// The C files to compile.
    pub sources: &'a [PathBuf],
    /// The executable to write.
    pub out: &'a Path,
    /// The run-time checks.
    #[builder(into)]
    pub checks: RunChecks,
    /// Whether the compiler has the sanitizers.
    pub sanitizers: bool,
    /// Whether the default flags go in.
    #[builder(default = true)]
    pub defaults: bool,
    /// The user's C flags: `CFLAGS`, then each `--cc-flag`.
    #[builder(default)]
    pub cflags: &'a [String],
    /// The user's link flags: `LDFLAGS`, then each `--ld-flag`.
    #[builder(default)]
    pub ldflags: &'a [String],
    /// The link flag of each lib the C functions of the program come from.
    #[builder(default)]
    pub libs: &'a [String],
    /// The folder that keeps the objects of the runtime from one build to the
    /// next, each named after a hash of the compiler, the flags, the runtime
    /// and its checks header; `None` compiles the runtime in each build.
    pub runtime_objects: Option<&'a Path>,
}

/// A C file of a build: where its object is written, and where the object
/// is kept for later builds when it is a runtime file.
#[derive(Builder)]
struct Unit
{
    /// The C file.
    source: PathBuf,
    /// The object the compiler writes.
    out: PathBuf,
    /// The kept object, for a runtime file with a store.
    kept: Option<PathBuf>,
}

impl Unit
{
    /// Whether an earlier build left the object, so the file needs no
    /// compile.
    ///
    /// # Returns
    ///
    /// `true` when the kept object exists.
    fn is_kept(&self) -> bool { self.kept.as_ref().is_some_and(|k| k.exists()) }
}

/// The number of objects this process has started to write into a store, so
/// two builds at once never write the same temp file.
static WRITES: AtomicUsize = AtomicUsize::new(0);

/// What a build ran: the output of each step, and the tool of the step that
/// failed.
#[derive(Debug, Builder)]
pub struct Built
{
    /// The output of each step that ran, in order.
    pub outputs: Vec<Output>,
    /// The tool of the step that failed, as a reader sees it; `None` when
    /// each step succeeded.
    pub failed: Option<String>,
}

/// Build the generated C into an executable: compile the C files at once,
/// each to an object beside it, then link the objects. A runtime file whose
/// object the store keeps from an earlier build is not compiled again. The
/// user flags come after the flags of the C target, so a flag the user gives
/// wins where the compiler takes the last one.
///
/// # Arguments
///
/// * `plan` - What to build.
///
/// # Returns
///
/// What ran, or the error when a tool cannot start.
pub fn build(plan: &BuildPlan<'_>) -> zakac_host_errors::Result<Built>
{
    let units: Vec<Unit> = plan.sources.iter().map(|s| unit_of(plan, s)).collect();
    // shortcut: each compile starts at once, with no cap; cap them at the core
    // count if programs reach hundreds of files.
    let started = units.iter().filter(|u| !u.is_kept()).map(|u| {
        let cmd = compile_command(plan, &u.source, &u.out);
        start(plan.cc, cmd)
    });
    let started = started.collect::<zakac_host_errors::Result<Vec<Child>>>()?;
    let finished = started.into_iter().map(|child| finish(plan.cc, child));
    let mut outputs = finished.collect::<zakac_host_errors::Result<Vec<Output>>>()?;
    if outputs.iter().any(|o| !o.status.success())
    {
        // A temp object in the store is never kept after a failed build.
        let temps = units.iter().filter(|u| u.kept.is_some() && !u.is_kept());
        temps.for_each(|u| _ = std::fs::remove_file(&u.out));
        return Ok(failed(outputs, plan.cc));
    }
    let objects: Vec<PathBuf> = units.iter().map(settle).collect();
    let linker = plan.linker.unwrap_or(plan.cc);
    let output = run(linker, link_command(plan, linker, &objects))?;
    let ok = output.status.success();
    outputs.push(output);
    ret_if!(!ok, Ok(failed(outputs, linker)));
    Ok(Built::builder().outputs(outputs).build())
}

/// The command that compiles one C file to an object.
///
/// # Arguments
///
/// * `plan` - What to build.
/// * `source` - The C file.
/// * `object` - The object to write.
///
/// # Returns
///
/// The command.
fn compile_command(plan: &BuildPlan<'_>, source: &Path, object: &Path) -> Command
{
    let mut cmd = plan.cc.command();
    cmd.args(compile_flags(plan, source)).arg("-c").arg(source).arg("-o").arg(object);
    cmd
}

/// The flags that compile one C file, before its paths.
///
/// # Arguments
///
/// * `plan` - What to build.
/// * `source` - The C file.
///
/// # Returns
///
/// The flags.
fn compile_flags(plan: &BuildPlan<'_>, source: &Path) -> Vec<String>
{
    let mut flags = vec![STD_FLAG.to_owned()];
    let words = |list: &[&str]| list.iter().map(|f| f.to_string()).collect::<Vec<_>>();
    do_if!(plan.defaults, flags.extend(words(DEFAULT_FLAGS)));
    let sanitized = plan.checks.has(RunCheck::Sanitizers) && plan.sanitizers;
    do_if!(sanitized, flags.extend(words(SANITIZE_FLAGS)));
    let callbacks = source.file_name() == Some(CB_SOURCE.as_ref());
    do_if!(callbacks, flags.extend(ffi_flags().0.iter().cloned()));
    flags.extend(plan.cflags.iter().cloned());
    flags
}

/// A C file of the build, with the object it compiles to.
///
/// # Arguments
///
/// * `plan` - What to build.
/// * `source` - The C file.
///
/// # Returns
///
/// The unit: a runtime file with a store compiles to a temp file in the
/// store, and each other file to an object beside it.
fn unit_of(plan: &BuildPlan<'_>, source: &Path) -> Unit
{
    let unit = Unit::builder().source(source.to_path_buf());
    let Some(kept) = kept_object(plan, source)
    else
    {
        return unit.out(source.with_extension("o")).build();
    };
    let n = WRITES.fetch_add(1, Ordering::Relaxed);
    let temp = format!("{}.{}-{n}.part", kept.display(), std::process::id());
    unit.out(PathBuf::from(temp)).kept(kept).build()
}

/// Where the store keeps the object of a runtime file: named after a hash of
/// each input of the compile, so a change to the compiler, the flags, the
/// runtime or the checks gives a new name.
///
/// # Arguments
///
/// * `plan` - What to build.
/// * `source` - The C file.
///
/// # Returns
///
/// The path, or `None` for a file that is not a runtime file, for a plan
/// with no store, or when the store or the checks header cannot be reached.
fn kept_object(plan: &BuildPlan<'_>, source: &Path) -> Option<PathBuf>
{
    let store = plan.runtime_objects?;
    let name = source.file_name()?.to_str()?;
    let stem = name.strip_suffix(".c")?;
    ret_if!(!RUNTIME.iter().any(|(n, _)| *n == name), None);
    let checks = std::fs::read(source.with_file_name(CHECKS_HEADER)).ok()?;
    std::fs::create_dir_all(store).ok()?;
    let mut hash = Xxh3::new();
    for word in std::iter::once(plan.cc.shown()).chain(compile_flags(plan, source))
    {
        hash.update(word.as_bytes());
        hash.update(&[0]);
    }
    RUNTIME.iter().for_each(|(_, text)| hash.update(text.as_bytes()));
    hash.update(&checks);
    Some(store.join(format!("{stem}-{:032x}.o", hash.digest128())))
}

/// The object a unit links with: a fresh runtime object moves into the store
/// for later builds.
///
/// # Arguments
///
/// * `unit` - The unit, compiled or kept.
///
/// # Returns
///
/// The object.
fn settle(unit: &Unit) -> PathBuf
{
    let Some(kept) = &unit.kept
    else
    {
        return unit.out.clone();
    };
    ret_if!(!unit.out.exists(), kept.clone());
    match std::fs::rename(&unit.out, kept)
    {
        Ok(()) => kept.clone(),
        // Another build stored the same object first; Windows will not
        // rename over it.
        Err(_) if kept.exists() =>
        {
            let _ = std::fs::remove_file(&unit.out);
            kept.clone()
        }
        Err(_) => unit.out.clone(),
    }
}

/// Start a compile, with its output piped.
///
/// # Arguments
///
/// * `tool` - The tool, for the error when it cannot start.
/// * `cmd` - The command.
///
/// # Returns
///
/// The running compile, or the error when it cannot start.
fn start(tool: &Tool, mut cmd: Command) -> zakac_host_errors::Result<Child>
{
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    cmd.spawn().map_err(|e| start_error(tool, e))
}

/// Wait for a compile to end.
///
/// # Arguments
///
/// * `tool` - The tool, for the error when the wait fails.
/// * `child` - The running compile.
///
/// # Returns
///
/// Its output, or the error of the wait.
fn finish(tool: &Tool, child: Child) -> zakac_host_errors::Result<Output>
{
    child.wait_with_output().map_err(|e| start_error(tool, e))
}

/// The command that links the objects into the executable.
///
/// # Arguments
///
/// * `plan` - What to build.
/// * `linker` - The program that links.
/// * `objects` - The objects.
///
/// # Returns
///
/// The command.
fn link_command(plan: &BuildPlan<'_>, linker: &Tool, objects: &[PathBuf]) -> Command
{
    let mut cmd = linker.command();
    if plan.checks.has(RunCheck::Sanitizers) && plan.sanitizers
    {
        cmd.args(SANITIZE_FLAGS);
    }
    // libffi links when `zaka_cb.c` is built, with the flags `pkg-config`
    // gives.
    let callbacks =
        plan.sources.iter().any(|s| s.file_name() == Some(CB_SOURCE.as_ref()));
    let ffi: &[String] = tern!(callbacks, &ffi_flags().1, &[]);
    cmd.args(objects)
        .arg("-o")
        .arg(plan.out)
        .args(LINK_LIBS)
        .args(plan.libs)
        .args(ffi)
        .args(plan.ldflags);
    cmd
}

/// Run a step of the build.
///
/// # Arguments
///
/// * `tool` - The tool, for the error when it cannot start.
/// * `cmd` - The command.
///
/// # Returns
///
/// Its output, or the error when it cannot start.
fn run(tool: &Tool, mut cmd: Command) -> zakac_host_errors::Result<Output>
{
    cmd.output().map_err(|e| start_error(tool, e))
}

/// The error of a tool that cannot start or be waited for.
///
/// # Arguments
///
/// * `tool` - The tool.
/// * `e` - The error of the host.
///
/// # Returns
///
/// The error.
fn start_error(tool: &Tool, e: std::io::Error) -> Error
{
    Error::from_io(e, IOErrorContext::new(tool.program.clone(), IOErrorAction::Read))
}

/// A build that stopped at a step.
///
/// # Arguments
///
/// * `outputs` - The output of each step that ran.
/// * `tool` - The tool of the step that failed.
///
/// # Returns
///
/// The build.
fn failed(outputs: Vec<Output>, tool: &Tool) -> Built
{
    Built::builder().outputs(outputs).failed(tool.shown()).build()
}
