use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

use bon::Builder;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::callbacks::CB_SOURCE;
use crate::callbacks::ffi_flags;
use crate::options::Checks;

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
    pub checks: Checks,
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

/// Build the generated C into an executable: compile each C file to an
/// object beside it, then link the objects. The user flags come after the
/// flags of the C target, so a flag the user gives wins where the compiler
/// takes the last one.
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
    let mut outputs = Vec::new();
    let mut objects = Vec::new();
    for source in plan.sources
    {
        let object = source.with_extension("o");
        let output = run(plan.cc, compile_command(plan, source, &object))?;
        let ok = output.status.success();
        outputs.push(output);
        ret_if!(!ok, Ok(failed(outputs, plan.cc)));
        objects.push(object);
    }
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
    cmd.arg(STD_FLAG);
    if plan.defaults
    {
        cmd.args(DEFAULT_FLAGS);
    }
    if plan.checks == Checks::Mem && plan.sanitizers
    {
        cmd.args(SANITIZE_FLAGS);
    }
    let callbacks = source.file_name() == Some(CB_SOURCE.as_ref());
    do_if!(callbacks, _ = cmd.args(&ffi_flags().0));
    cmd.args(plan.cflags).arg("-c").arg(source).arg("-o").arg(object);
    cmd
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
    if plan.checks == Checks::Mem && plan.sanitizers
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
    cmd.output().map_err(|e| {
        Error::from_io(
            e,
            IOErrorContext::new(tool.program.clone(), IOErrorAction::Read),
        )
    })
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
