//! What the tests of the native target share: the samples, and a run of a
//! program through the JIT in a process of its own.

use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;

/// The folder of the crate.
///
/// # Returns
///
/// The folder.
pub fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// The folder of the repo samples.
///
/// # Returns
///
/// The folder.
pub fn samples() -> PathBuf { root().join("../../samples") }

/// What a run printed and how it ended.
pub struct Run
{
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// The exit status.
    pub status: Option<i32>,
}

/// Run a program through the JIT in a fresh folder, with nothing on standard
/// input.
///
/// # Arguments
///
/// * `file` - The program.
/// * `flags` - The flags of `zakac run` before the program.
///
/// # Returns
///
/// What the run printed.
pub fn run_jit(file: &Path, flags: &[&str]) -> Run
{
    let dir = tempfile::tempdir().unwrap();
    let exe = env!("CARGO_BIN_EXE_zakac_codegen_target_native");
    let mut cmd = std::process::Command::new(exe);
    cmd.env("ZAKAC_NO_CONFIG", "1").arg("--jit").args(flags).arg(file);
    // A sample that writes a file writes it in the temp folder.
    let ran = cmd.current_dir(dir.path()).stdin(Stdio::null()).output().unwrap();
    let text = |raw: &[u8]| String::from_utf8_lossy(raw).into_owned();
    Run {
        stdout: text(&ran.stdout),
        stderr: text(&ran.stderr),
        status: ran.status.code(),
    }
}

/// The inputs of the tests of the C target, which the native target runs too.
///
/// # Returns
///
/// The folder.
pub fn c_inputs() -> PathBuf { root().join("../zakac_codegen_target_c/tests/inputs") }

/// The stop line of a run, from `the program stopped:` to the end of its
/// reason: the name of the program before it and the place of the stop after
/// it, ` (in ...)`, left out.
///
/// # Arguments
///
/// * `stderr` - What the run wrote to standard error.
///
/// # Returns
///
/// The line, with a line break, or a blank when the run did not stop.
pub fn stop_line(stderr: &str) -> String
{
    let marker = "the program stopped: ";
    let Some(line) = stderr.lines().find(|l| l.contains(marker))
    else
    {
        return String::new();
    };
    let reason = &line[line.find(marker).unwrap_or(0)..];
    let reason = reason.split(" (in ").next().unwrap_or(reason);
    format!("{reason}\n")
}

/// Build a program with the native target, AOT, link it with the runtime
/// lib, and run it in a fresh folder with nothing on standard input.
///
/// # Arguments
///
/// * `path` - The program.
/// * `overflow` - The overflow mode.
/// * `grants` - The rights the build writes into the executable.
///
/// # Returns
///
/// What the run printed, or `None` when no C compiler links.
pub fn run_aot(
    path: &Path, overflow: zakac_borrow_checker::options::OverflowMode,
    grants: &zaka_host::Grants,
) -> Option<Run>
{
    use zakac_codegen_target_c::driver::write_cache_for;
    use zakac_codegen_target_c::options::TargetCOptions;
    use zakac_codegen_target_c::toolchain;
    use zakac_codegen_target_c::toolchain::BuildPlan;
    use zakac_codegen_target_c::toolchain::Dialect;

    let dialect = Dialect::of(test_abi());
    let cc = toolchain::find_cc_for(None, dialect).ok()?;
    let (sm, program) = checked(path, overflow);
    let program = &program;
    let target = native_target();
    let target_options = TargetCOptions::builder().overflow(overflow).build();
    let made = zakac_codegen_target_native::aot::object(
        program,
        sm.files(),
        target_options,
        grants,
        &target,
        false,
        None,
    );
    let made = made.unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let dir = tempfile::tempdir().unwrap();
    let glue: Vec<zakac_codegen_target_c::emit::CFile> = made
        .threads
        .then(|| {
            let text = zakac_codegen_target_c::threads::native_threads_glue();
            zakac_codegen_target_c::emit::CFile::builder()
                .name("zaka_threads.c")
                .text(text)
                .build()
        })
        .into_iter()
        .collect();
    let lib = write_cache_for(dir.path(), &glue, test_abi()).unwrap();
    let obj = dir.path().join(format!("main.{}", dialect.object()));
    std::fs::write(&obj, &made.octets).unwrap();
    let sources: Vec<PathBuf> = std::iter::once(obj).chain(lib).collect();
    let exe = dir.path().join(format!("program{}", std::env::consts::EXE_SUFFIX));
    let libs = zakac_codegen_target_c::ffi::link_libs_for(&program.foreign, dialect);
    #[rustfmt::skip]
    let plan = BuildPlan::builder().cc(&cc).sources(&sources).out(&exe)
        .checks(target_options.checks).sanitizers(false).libs(&libs).abi(test_abi()).build();
    let built = toolchain::build(&plan).unwrap();
    let errors: Vec<_> =
        built.outputs.iter().map(|o| String::from_utf8_lossy(&o.stderr)).collect();
    assert!(built.failed.is_none(), "the link failed:\n{}", errors.concat());
    // Windows looks for a lib in the folder of the executable and on the PATH,
    // and the libs of a program sit beside it.
    let folder = path.parent().map(Path::to_path_buf);
    let paths = std::env::var_os("PATH").unwrap_or_default();
    let search = folder.into_iter().chain(std::env::split_paths(&paths));
    let ran = std::process::Command::new(&exe)
        .current_dir(dir.path())
        .env("PATH", std::env::join_paths(search).unwrap())
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let text = |raw: &[u8]| String::from_utf8_lossy(raw).into_owned();
    Some(Run {
        stdout: text(&ran.stdout),
        stderr: text(&ran.stderr),
        status: ran.status.code(),
    })
}

/// The target of a native build for the host.
///
/// # Returns
///
/// The target.
pub fn native_target() -> zakac_ast::target::Target
{
    let host = zakac_ast::target::Target::host();
    let output = zakac_ast::target::BuildTarget::Native;
    zakac_ast::target::Target { output, abi: test_abi(), ..host }
}

/// The ABI the executables of the tests are for: `ZAKA_TEST_ABI` (`gnu` or
/// `msvc`), else the compiler's own, so a Windows host runs the suite once for
/// each ABI. The JIT runs in the process, with the ABI of the compiler.
///
/// # Returns
///
/// The ABI.
pub fn test_abi() -> zakac_ast::target::TargetAbi
{
    let given = std::env::var("ZAKA_TEST_ABI").ok().and_then(|v| v.parse().ok());
    given.unwrap_or_else(zakac_codegen_target_c::driver::host_abi)
}

/// Load a program for the native target and run the checks: the source map
/// and the IR, which has no errors.
///
/// # Arguments
///
/// * `path` - The program.
/// * `overflow` - The overflow mode.
///
/// # Returns
///
/// The source map and the IR.
pub fn checked(
    path: &Path, overflow: zakac_borrow_checker::options::OverflowMode,
) -> (zakac_span::source_map::SourceMap, zakac_ir_builder::ir::Module)
{
    zaka_vm::comptime::install();
    let mut sm = zakac_span::source_map::SourceMap::new();
    let file = sm.load_file(path).unwrap();
    let options = zakac_semcheck::semcheck::SemcheckOptions::default();
    let lexer = zakac_lexer::lexer::LexerOptions::default();
    let loaded = zakac_semcheck::open_chains::load_settled(
        &file,
        &mut sm,
        &lexer,
        &native_target(),
        &options,
    );
    let mem = zakac_borrow_checker::options::BorrowCheckOptions::builder()
        .overflow(overflow)
        .build();
    let fold = zakac_ir_builder::ir::FoldOptions::default();
    let checked = zakac_borrow_checker::artifact::run_loaded(
        loaded.unwrap(),
        &options,
        &fold,
        &mem,
    );
    let program = checked.lowered.folded.expect("no errors");
    (sm, program)
}
