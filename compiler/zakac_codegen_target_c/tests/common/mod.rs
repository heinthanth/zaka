#![allow(dead_code)]

use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;

use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::driver::translate;
use zakac_codegen_target_c::driver::write_cache;
use zakac_codegen_target_c::emit::CFile;
use zakac_codegen_target_c::options::Checks;
use zakac_codegen_target_c::toolchain;
use zakac_codegen_target_c::toolchain::BuildPlan;
use zakac_codegen_target_c::toolchain::Tool;
use zakac_diagnostics::has_errors;
use zakac_host_errors::Error;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::load;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::source_map::SourceMap;

/// The crate folder.
pub fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// The repo samples folder.
pub fn samples() -> PathBuf { root().join("../../samples") }

/// The C compiler on this host, or `None`; a test with no compiler notes it
/// and passes, so a host without one keeps passing.
///
/// # Returns
///
/// The compiler.
pub fn cc() -> Option<Tool>
{
    let found = toolchain::find_cc(None).ok();
    if found.is_none()
    {
        eprintln!("no C compiler on the PATH: the native build tests are skipped");
    }
    found
}

/// The C files of a source file and the files it imports, which must
/// compile with no error.
///
/// # Arguments
///
/// * `path` - The source file.
/// * `overflow` - The overflow mode.
/// * `checks` - The run-time checks.
///
/// # Returns
///
/// The files, or the construct the C target cannot build.
pub fn c_files(
    path: &Path, overflow: OverflowMode, checks: Checks,
) -> Result<Vec<CFile>, Error>
{
    let mut sm = SourceMap::new();
    let file = sm.load_file(path).unwrap();
    let loaded = load(&file, &mut sm, &LexerOptions::default()).unwrap();
    let mem = BorrowCheckOptions::builder().overflow(overflow).build();
    let options = SemcheckOptions::default();
    let out =
        translate(loaded, sm.files(), &options, &FoldOptions::default(), &mem, checks);
    let msgs: Vec<&str> =
        out.diagnostics().iter().map(|d| d.message.as_str()).collect();
    assert!(!has_errors(out.diagnostics()), "{}: {msgs:?}", path.display());
    out.files.expect("no errors, so C")
}

/// What a native run printed and how it ended.
#[derive(bon::Builder)]
pub struct Run
{
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// The exit status.
    pub status: Option<i32>,
}

/// Write C files to a fresh folder, compile them and run the executable
/// with nothing on standard input.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `files` - The generated files.
/// * `checks` - The run-time checks.
///
/// # Returns
///
/// What the run printed.
pub fn compile_and_run(cc: &Tool, files: &[CFile], checks: Checks) -> Run
{
    let dir = tempfile::tempdir().unwrap();
    let sources = write_cache(dir.path(), files).unwrap();
    let exe = dir.path().join(format!("program{}", std::env::consts::EXE_SUFFIX));
    #[rustfmt::skip]
    let plan = BuildPlan::builder().cc(cc).sources(&sources).out(&exe).checks(checks)
        .sanitizers(false).build();
    let built = toolchain::build(&plan).unwrap();
    let errors: Vec<_> =
        built.outputs.iter().map(|o| String::from_utf8_lossy(&o.stderr)).collect();
    assert!(built.failed.is_none(), "the C build failed:\n{}", errors.concat());
    // A sample that writes a file, such as `io/files`, writes it in the temp
    // folder, never in the source tree.
    let mut cmd = std::process::Command::new(&exe);
    let ran = cmd.current_dir(dir.path()).stdin(Stdio::null()).output().unwrap();
    let text = |raw: &[u8]| String::from_utf8_lossy(raw).into_owned();
    #[rustfmt::skip]
    return Run::builder().stdout(text(&ran.stdout)).stderr(text(&ran.stderr))
        .maybe_status(ran.status.code()).build();
}
