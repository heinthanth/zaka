use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

use bon::Builder;
use zakac_ast::target::TargetAbi;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::driver::host_abi;
use crate::driver::runtime_lib;
use crate::options::RunCheck;
use crate::options::RunChecks;

/// The flag form a C toolchain takes: the GNU form of gcc, Clang and `zig cc`,
/// or the form of MSVC's `cl` and `clang-cl`. The ABI of the target picks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dialect
{
    /// `-std=c11 -c x.c -o x.o`, `-lX`.
    #[default]
    Gnu,
    /// `/std:c11 /c x.c /Fox.obj`, `X.lib`, link options after `/link`.
    Msvc,
}

/// The flags of a dialect, as data.
struct Flags
{
    /// The compilers tried, in order, when neither `--cc` nor `CC` names one.
    /// Each is a program and the words that come before the C flags.
    searched: &'static [&'static [&'static str]],
    /// The flag the generated C is written for: C11. A later standard flag
    /// from the user replaces it.
    std: &'static str,
    /// The flags the C target adds unless `--no-default-flags` leaves them
    /// out. GNU: the optimizer, no warnings about the generated code, and no
    /// fused mul-add (one rounding for a product and a sum), since the VM
    /// rounds after each float operation; GCC skips the `#pragma STDC
    /// FP_CONTRACT OFF` the runtime holds, so the flag remains. MSVC: the same,
    /// with `/fp:precise` (no contraction), the source read as UTF-8, and the
    /// C runtime of a DLL, as the runtime lib of Rust links it. The user flags
    /// come after them, so a later `-O3` replaces `-O2`.
    defaults: &'static [&'static str],
    /// What each link needs: libm for `fmod` and kin, which the UCRT of MSVC
    /// holds.
    link_libs: &'static [&'static str],
    /// The flags of the mem checks: the address and undefined-behavior
    /// sanitizers, the address one alone for MSVC. Both the compile and the
    /// link need them.
    sanitize: &'static [&'static str],
    /// The flag that compiles a file to an object and stops.
    to_object: &'static str,
    /// The extension of an object.
    object: &'static str,
}

/// The flags of GCC, Clang and `zig cc`.
const GNU: Flags = Flags {
    searched: &[&["cc"], &["gcc"], &["clang"], &["zig", "cc"]],
    std: "-std=c11",
    defaults: &["-O2", "-w", "-ffp-contract=off"],
    link_libs: &["-lm"],
    sanitize: &["-g", "-fsanitize=address,undefined"],
    to_object: "-c",
    object: "o",
};

/// The flags of MSVC's `cl` and of `clang-cl`.
const MSVC: Flags = Flags {
    searched: &[&["cl"], &["clang-cl"]],
    std: "/std:c11",
    defaults: &["/nologo", "/O2", "/w", "/fp:precise", "/utf-8", "/MD"],
    link_libs: &[],
    sanitize: &["/fsanitize=address"],
    to_object: "/c",
    object: "obj",
};

impl Dialect
{
    /// The dialect of the toolchain of an ABI.
    ///
    /// # Arguments
    ///
    /// * `abi` - The ABI of the target.
    ///
    /// # Returns
    ///
    /// MSVC for the ABI of MSVC, else GNU.
    pub fn of(abi: TargetAbi) -> Dialect
    {
        tern!(abi == TargetAbi::Msvc, Dialect::Msvc, Dialect::Gnu)
    }

    /// The flags of the dialect.
    ///
    /// # Returns
    ///
    /// The flags.
    fn flags(self) -> &'static Flags { tern!(self == Dialect::Msvc, &MSVC, &GNU) }

    /// The extension of an object, `o` or `obj`.
    ///
    /// # Returns
    ///
    /// The extension.
    pub fn object(self) -> &'static str { self.flags().object }

    /// The words that write a file: `-o <path>`, or the one word of MSVC,
    /// `/Fo<path>` for an object and `/Fe<path>` for an executable.
    ///
    /// # Arguments
    ///
    /// * `exe` - Whether the file is an executable.
    /// * `path` - The file.
    ///
    /// # Returns
    ///
    /// The words.
    fn out(self, exe: bool, path: &Path) -> Vec<OsString>
    {
        ret_if!(self == Dialect::Gnu, vec!["-o".into(), path.into()]);
        let mut word = OsString::from(tern!(exe, "/Fe", "/Fo"));
        word.push(path);
        vec![word]
    }

    /// The link flag of a lib a program names: `-lX`, or `X.lib` for MSVC.
    ///
    /// # Arguments
    ///
    /// * `stem` - The name of the lib with no `lib` in front.
    ///
    /// # Returns
    ///
    /// The flag.
    pub fn lib(self, stem: &str) -> String
    {
        tern!(self == Dialect::Msvc, format!("{stem}.lib"), format!("-l{stem}"))
    }
}

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
    /// The variables the tool runs with, such as the `INCLUDE`, `LIB` and
    /// `PATH` of an MSVC found outside a Developer Prompt.
    #[builder(default)]
    pub env: Vec<(OsString, OsString)>,
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
    pub fn command(&self) -> Command
    {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.lead).envs(self.env.iter().map(|(k, v)| (k, v)));
        cmd
    }
}

/// Find the C compiler of this compiler's own ABI: `--cc`, then `CC`, then
/// the compilers of its dialect, as [`find_cc_for`] finds them.
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
    // A `BuildPlan` that names no ABI takes this one too.
    find_cc_for(chosen, Dialect::of(host_abi()))
}

/// Find the C compiler of a dialect: `--cc`, then `CC`, then the compilers
/// of the dialect on the PATH; for MSVC, then the `cl` that Visual Studio
/// installed, with the variables of its Developer Prompt.
///
/// # Arguments
///
/// * `chosen` - The compiler `--cc` names, words split on white space.
/// * `dialect` - The dialect.
///
/// # Returns
///
/// The compiler, or the error when the one named is missing or none is
/// found.
pub fn find_cc_for(
    chosen: Option<&str>, dialect: Dialect,
) -> zakac_host_errors::Result<Tool>
{
    let env = std::env::var("CC").ok().filter(|v| !v.trim().is_empty());
    ret_if!(let Some(words) = chosen.map(str::to_owned).or(env), named(&words));
    let searched = dialect.flags().searched.iter();
    let on_path = searched.clone().find_map(|words| from_words(words));
    let installed = || tern!(dialect == Dialect::Msvc, installed_cl(), None);
    on_path.or_else(installed).ok_or(Error::NoCCompiler)
}

/// The `cl` of the newest Visual Studio with the C++ tools, with the
/// variables its Developer Prompt sets.
///
/// # Returns
///
/// The tool, or `None` with no such Visual Studio.
#[cfg(windows)]
fn installed_cl() -> Option<Tool>
{
    let arch = tern!(cfg!(target_arch = "aarch64"), "aarch64", "x86_64");
    let found =
        find_msvc_tools::find_tool(&format!("{arch}-pc-windows-msvc"), "cl.exe")?;
    let env = found.env().into_iter().cloned().collect();
    Some(Tool::builder().program(found.path().to_path_buf()).env(env).build())
}

/// The `cl` of Visual Studio: none off Windows.
///
/// # Returns
///
/// `None`.
#[cfg(not(windows))]
fn installed_cl() -> Option<Tool> { None }

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
/// * `dialect` - The flag form of the compiler.
///
/// # Returns
///
/// `true` when the build succeeds.
pub fn has_sanitizers(cc: &Tool, cache: &Path, dialect: Dialect) -> bool
{
    let src = cache.join("zk_probe.c");
    ret_if!(std::fs::write(&src, "int main(void) { return 0; }\n").is_err(), false);
    let exe = cache.join(format!("zk_probe{}", std::env::consts::EXE_SUFFIX));
    let mut cmd = cc.command();
    cmd.args(dialect.flags().sanitize).arg(&src).args(dialect.out(true, &exe));
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
    /// The ABI of the executable, which picks the flag form and the OS libs of
    /// the runtime lib; this compiler's own when left out.
    #[builder(default = host_abi())]
    pub abi: TargetAbi,
}

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
/// each to an object beside it, then link the objects and the runtime lib. The
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
    let is_c = |s: &&PathBuf| s.extension().is_some_and(|e| e == "c");
    let sources: Vec<&PathBuf> = plan.sources.iter().filter(is_c).collect();
    // A lib among the sources, such as the runtime, links as it is.
    let libs = plan.sources.iter().filter(|s| !is_c(s)).cloned();
    // shortcut: each compile starts at once, with no cap; cap them at the core
    // count if programs reach hundreds of files.
    let object = Dialect::of(plan.abi).object();
    let started = sources
        .iter()
        .map(|s| start(plan.cc, compile_command(plan, s, &s.with_extension(object))));
    let started = started.collect::<zakac_host_errors::Result<Vec<Child>>>()?;
    let finished = started.into_iter().map(|child| finish(plan.cc, child));
    let mut outputs = finished.collect::<zakac_host_errors::Result<Vec<Output>>>()?;
    ret_if!(outputs.iter().any(|o| !o.status.success()), Ok(failed(outputs, plan.cc)));
    let objects = sources.iter().map(|s| s.with_extension(object)).chain(libs);
    let objects: Vec<PathBuf> = objects.collect();
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
    let dialect = Dialect::of(plan.abi);
    let mut cmd = plan.cc.command();
    cmd.args(compile_flags(plan)).arg(dialect.flags().to_object).arg(source);
    cmd.args(dialect.out(false, object));
    cmd
}

/// The flags that compile one C file, before its paths.
///
/// # Arguments
///
/// * `plan` - What to build.
///
/// # Returns
///
/// The flags.
fn compile_flags(plan: &BuildPlan<'_>) -> Vec<String>
{
    let table = Dialect::of(plan.abi).flags();
    let mut flags = vec![table.std.to_owned()];
    let words = |list: &[&str]| list.iter().map(|f| f.to_string()).collect::<Vec<_>>();
    do_if!(plan.defaults, flags.extend(words(table.defaults)));
    let sanitized = plan.checks.has(RunCheck::Sanitizers) && plan.sanitizers;
    do_if!(sanitized, flags.extend(words(table.sanitize)));
    flags.extend(plan.cflags.iter().cloned());
    flags
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
    let dialect = Dialect::of(plan.abi);
    let table = dialect.flags();
    let mut cmd = linker.command();
    let sanitized = plan.checks.has(RunCheck::Sanitizers) && plan.sanitizers;
    do_if!(sanitized, _ = cmd.args(table.sanitize));
    let os_libs: Vec<&str> =
        runtime_lib(plan.abi).map(|r| r.native_libs().collect()).unwrap_or_default();
    ret_if!(dialect == Dialect::Msvc, msvc_link(plan, cmd, objects, &os_libs));
    cmd.args(objects)
        .args(dialect.out(true, plan.out))
        .args(table.link_libs)
        .args(os_libs)
        .args(plan.libs)
        .args(plan.ldflags);
    cmd
}

/// The rest of a link of MSVC: the objects and the libs, the executable, then
/// the options of the linker after `/link` (the `/defaultlib:` rustc asks for
/// and the user's link flags). A `link.exe` or `lld-link` given as the linker
/// takes each word itself, and `/OUT:` for the executable.
///
/// # Arguments
///
/// * `plan` - What to build.
/// * `cmd` - The command so far.
/// * `objects` - The objects and the runtime lib.
/// * `os_libs` - The OS libs of the runtime lib.
///
/// # Returns
///
/// The command.
fn msvc_link(
    plan: &BuildPlan<'_>, mut cmd: Command, objects: &[PathBuf], os_libs: &[&str],
) -> Command
{
    let is_option = |w: &&&str| w.starts_with('/');
    let (options, libs): (Vec<&&str>, Vec<&&str>) = os_libs.iter().partition(is_option);
    let stem = |t: &Tool| t.program.file_stem().map(|s| s.to_ascii_lowercase());
    let linker = plan.linker.and_then(stem);
    let link_exe = linker.is_some_and(|s| s == "link" || s == "lld-link");
    cmd.args(objects).args(libs).args(plan.libs);
    let mut out = OsString::from("/OUT:");
    out.push(plan.out);
    do_if!(link_exe, _ = cmd.arg("/NOLOGO").arg(out));
    let cl_out = Dialect::Msvc.out(true, plan.out);
    do_if!(!link_exe, _ = cmd.arg("/nologo").args(cl_out).args(["/link", "/NOLOGO"]));
    cmd.args(options).args(plan.ldflags);
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
