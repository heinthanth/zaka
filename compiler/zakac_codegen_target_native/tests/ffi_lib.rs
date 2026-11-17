//! A program that calls a C lib of its own, `inputs/ffi_lib.c` of the C
//! target, built for the test: it prints through the JIT what it prints in
//! the VM and as an executable of the C target.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crate::common::c_inputs;
use crate::common::run_jit;
use crate::common::stop_line;

include!("../../zakac_codegen_target_c/tests/inputs/ffi_programs.rs");

/// The C compiler that builds the test lib: `CC`, else `cc`; `None` skips
/// the test on a host with none, unless `ZAKA_REQUIRE_CC` asks for one.
///
/// # Returns
///
/// The compiler.
fn cc() -> Option<String>
{
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".into());
    let found = Command::new(&cc).arg("--version").output().is_ok();
    assert!(found || std::env::var_os("ZAKA_REQUIRE_CC").is_none(), "no C compiler");
    found.then_some(cc)
}

/// Build the test lib into a folder.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `dir` - The folder.
///
/// # Returns
///
/// The path of the lib.
fn build_lib(cc: &str, dir: &Path) -> PathBuf
{
    let name = format!(
        "{}zktest{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    let lib = dir.join(name);
    let mut cmd = Command::new(cc);
    cmd.args(["-shared", "-fPIC", "-o"]).arg(&lib).arg(c_inputs().join("ffi_lib.c"));
    let built = cmd.output().unwrap();
    let errors = String::from_utf8_lossy(&built.stderr);
    assert!(built.status.success(), "the test lib did not build:\n{errors}");
    lib
}

/// Write a program that names the lib.
///
/// # Arguments
///
/// * `dir` - The folder.
/// * `lib` - The lib.
/// * `text` - The program, with `{LIB}` for the path of the lib.
///
/// # Returns
///
/// The path of the program.
fn write_program(dir: &Path, lib: &Path, text: &str) -> PathBuf
{
    let path = dir.join("main.zaka");
    let text = text.replace("{LIB}", &lib.display().to_string().replace('\\', "/"));
    std::fs::write(&path, text).unwrap();
    path
}

/// The output of a program that calls the lib, through the JIT.
///
/// # Arguments
///
/// * `text` - The program, with `{LIB}` for the path of the lib.
///
/// # Returns
///
/// The run, or `None` with no C compiler.
fn run_with_lib(text: &str) -> Option<crate::common::Run>
{
    let cc = cc()?;
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(&cc, dir.path());
    let path = write_program(dir.path(), &lib, text);
    Some(run_jit(&path, &["--allow-all"]))
}

/// The output of a program that calls the lib, as an executable.
///
/// # Arguments
///
/// * `text` - The program, with `{LIB}` for the path of the lib.
///
/// # Returns
///
/// The run, or `None` with no C compiler.
fn run_with_lib_aot(text: &str) -> Option<crate::common::Run>
{
    let cc = cc()?;
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(&cc, dir.path());
    let path = write_program(dir.path(), &lib, text);
    let overflow = zakac_borrow_checker::options::OverflowMode::Stop;
    crate::common::run_aot(&path, overflow, &zaka_host::Grants::all())
}

/// Structs through `...`, text as a result, a callback and handles print
/// what the VM prints for them.
#[test]
fn a_program_calls_a_lib_of_its_own_through_the_jit()
{
    let Some(run) = run_with_lib(PROGRAM)
    else
    {
        return;
    };
    assert_eq!(run.stdout, WANT, "{}", run.stderr);
}

/// C calls functions of the program back with text, an optional of text, a
/// struct as a value, a handle and an optional of a handle, and a closure.
#[test]
fn callbacks_take_text_structs_and_handles_through_the_jit()
{
    let Some(run) = run_with_lib(&format!("{CALLED_BACK}{CALLS_BACK}"))
    else
    {
        return;
    };
    assert_eq!(run.stdout, WANT_CALLED_BACK, "{}", run.stderr);
}

/// NULL from C for a parameter of plain text stops the run.
#[test]
fn null_for_plain_text_stops_the_run_through_the_jit()
{
    let Some(run) = run_with_lib(&format!("{CALLED_BACK}{NULL_TEXT}"))
    else
    {
        return;
    };
    assert_eq!(run.status, Some(1), "{}", run.stderr);
    assert_eq!(run.stdout, "plain\n၇\n");
    assert!(
        stop_line(&run.stderr)
            .contains("C gave NULL where its declaration promised a value")
    );
}

/// A plain handle result for which C gives NULL stops the run.
#[test]
fn null_for_a_plain_handle_result_stops_the_run_through_the_jit()
{
    let Some(run) = run_with_lib(NULL_HANDLE)
    else
    {
        return;
    };
    assert_eq!(run.status, Some(1), "{}", run.stderr);
    assert_eq!(run.stdout, "before\n");
}

/// A read through a duplicate of a handle a C function closed stops the run.
#[test]
fn a_closed_handle_stops_the_run_through_the_jit()
{
    let Some(run) = run_with_lib(CLOSED)
    else
    {
        return;
    };
    assert_eq!(run.status, Some(1), "{}", run.stderr);
    assert_eq!(run.stdout, "");
}

/// NULL from C for a parameter of a plain handle stops the run.
#[test]
fn null_for_a_plain_handle_parameter_stops_the_run_through_the_jit()
{
    let Some(run) = run_with_lib(&format!("{CALLED_BACK}{NULL_HANDLE_ARG}"))
    else
    {
        return;
    };
    assert_eq!(run.status, Some(1), "{}", run.stderr);
    assert_eq!(run.stdout, "၀\n");
}

/// An executable calls the lib, and C calls its functions back, as the JIT
/// does.
#[test]
fn a_program_calls_a_lib_of_its_own_as_an_executable()
{
    let Some(run) = run_with_lib_aot(PROGRAM)
    else
    {
        return;
    };
    assert_eq!(run.stdout, WANT, "{}", run.stderr);
    let Some(run) = run_with_lib_aot(&format!("{CALLED_BACK}{CALLS_BACK}"))
    else
    {
        return;
    };
    assert_eq!(run.stdout, WANT_CALLED_BACK, "{}", run.stderr);
}
