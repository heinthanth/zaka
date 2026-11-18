//! A program that calls a C lib of its own, `inputs/ffi_lib.c`, built for
//! the test: a struct through `...`, text as a result, and functions C calls
//! back with numbers, text, structs and handles. It prints the same in the
//! VM, which opens the lib through libffi, and as an executable linked to it.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

use zaka_vm::machine::HostData;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_and_keep;
use zaka_vm::trap::Trap;
use zaka_vm::trap::TrapKind;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::driver::write_cache_for;
use zakac_codegen_target_c::options::Checks;
use zakac_codegen_target_c::toolchain;
use zakac_codegen_target_c::toolchain::BuildPlan;
use zakac_codegen_target_c::toolchain::Tool;
use zakac_codegen_target_vm::artifact::run_loaded;
use zakac_codegen_target_vm::options::EmitOptions;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::load;
use zakac_rust_sugars::tern;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::source_map::SourceMap;

use crate::common::c_files;
use crate::common::cc;
use crate::common::root;
use crate::common::stop_line;

include!("inputs/ffi_programs.rs");

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
fn build_lib(dir: &Path) -> PathBuf
{
    // The lib is plain C for each ABI, so the GNU compiler builds it, with the
    // import lib MSVC links it through.
    let gnu = toolchain::find_cc_for(None, toolchain::Dialect::Gnu);
    let cc = gnu.expect("a GNU C compiler builds the test lib");
    let name = format!(
        "{}zktest{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    let lib = dir.join(name);
    let source = root().join("tests/inputs/ffi_lib.c");
    let mut cmd = Command::new(&cc.program);
    cmd.args(&cc.lead).args(["-shared", "-fPIC", "-o"]).arg(&lib).arg(source);
    if cfg!(windows)
    {
        cmd.arg(format!("-Wl,--out-implib,{}", lib.with_extension("lib").display()));
    }
    // macOS finds a linked lib through the name the lib gives itself.
    if cfg!(target_os = "macos")
    {
        cmd.arg(format!("-Wl,-install_name,{}", lib.display()));
    }
    let built = cmd.output().unwrap();
    let errors = String::from_utf8_lossy(&built.stderr);
    assert!(built.status.success(), "the test lib did not build:\n{errors}");
    lib
}

/// Run the program in the VM.
///
/// # Arguments
///
/// * `path` - The program.
///
/// # Returns
///
/// What it printed.
fn run_in_vm(path: &Path) -> String
{
    let (out, ran) = vm_run(path);
    assert_eq!(ran, Ok(()), "the VM trapped after:\n{out}");
    out
}

/// Run the program in the VM, which can stop it.
///
/// # Arguments
///
/// * `path` - The program.
///
/// # Returns
///
/// What it printed, and how the run ended.
fn vm_run(path: &Path) -> (String, Result<(), Trap>)
{
    let mut sm = SourceMap::new();
    let file = sm.load_file(path).unwrap();
    let loaded = load(&file, &mut sm, &LexerOptions::default()).unwrap();
    let mem = BorrowCheckOptions::default();
    let options = SemcheckOptions::default();
    let emitted = run_loaded(
        loaded,
        &options,
        &FoldOptions::default(),
        &mem,
        EmitOptions::default(),
    );
    let msgs: Vec<&str> =
        emitted.diagnostics().iter().map(|d| d.message.as_str()).collect();
    assert!(emitted.diagnostics().is_empty(), "{msgs:?}");
    let module = emitted.module.expect("emitted");
    let mut out = Vec::new();
    let host = HostData::program("main.zaka");
    let ran = run_and_keep(&module, &mut out, &RunOptions::default(), &host).1;
    (String::from_utf8(out).unwrap(), ran.map_err(|t| *t))
}

/// Build the program as an executable linked to the lib, and run it.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `path` - The program.
/// * `lib` - The lib.
///
/// # Returns
///
/// What it printed.
fn run_native(cc: &Tool, path: &Path, lib: &Path) -> String
{
    let ran = native_run(cc, path, lib);
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert!(ran.status.success(), "the executable stopped:\n{stderr}");
    String::from_utf8(ran.stdout).unwrap()
}

/// Build the program as an executable linked to the lib, and run it; the
/// run can stop.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `path` - The program.
/// * `lib` - The lib.
///
/// # Returns
///
/// How the run ended, and what it wrote.
fn native_run(cc: &Tool, path: &Path, lib: &Path) -> Output
{
    let files = c_files(path, OverflowMode::Stop, Checks::Default).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let abi = crate::common::test_abi();
    let sources = write_cache_for(dir.path(), &files, abi).unwrap();
    let folder = lib.parent().unwrap();
    // Windows finds a lib in the folder of the executable or on the PATH, so
    // the executable goes beside the lib.
    let exe = folder.join(format!("program{}", std::env::consts::EXE_SUFFIX));
    let folder = folder.display();
    let msvc = toolchain::Dialect::of(abi) == toolchain::Dialect::Msvc;
    let linked = tern!(msvc, lib.with_extension("lib"), lib.to_path_buf());
    let mut libs = vec![linked.display().to_string()];
    if !cfg!(windows)
    {
        libs.push(format!("-Wl,-rpath,{folder}"));
    }
    #[rustfmt::skip]
    let plan = BuildPlan::builder().cc(cc).sources(&sources).out(&exe)
        .checks(Checks::Default).sanitizers(false).libs(&libs).abi(abi).build();
    let built = toolchain::build(&plan).unwrap();
    let errors: Vec<_> =
        built.outputs.iter().map(|o| String::from_utf8_lossy(&o.stderr)).collect();
    assert!(built.failed.is_none(), "the C build failed:\n{}", errors.concat());
    Command::new(&exe).stdin(Stdio::null()).output().unwrap()
}

/// The program prints the same in the VM and as an executable.
#[test]
fn a_program_calls_a_lib_of_its_own_on_both_targets()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(dir.path());
    let path = dir.path().join("main.zaka");
    let text = PROGRAM.replace("{LIB}", &lib.display().to_string().replace('\\', "/"));
    std::fs::write(&path, text).unwrap();
    assert_eq!(run_in_vm(&path), WANT, "in the VM");
    assert_eq!(run_native(&cc, &path, &lib), WANT, "as an executable");
}

/// A C function that takes a handle over closes it for each value that
/// shares it, so a later read stops the run.
#[test]
fn a_closed_handle_stops_the_run()
{
    let Some(_cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(dir.path());
    let path = dir.path().join("main.zaka");
    let text = CLOSED.replace("{LIB}", &lib.display().to_string().replace('\\', "/"));
    std::fs::write(&path, text).unwrap();
    let (out, ran) = vm_run(&path);
    let trap = ran.expect_err("the read stops the run");
    assert_eq!(trap.kind, TrapKind::ClosedHandle, "{trap}");
    assert_eq!(out, "");
}

/// A plain handle result for which C gives NULL stops the run with the trap
/// of a promised value, in the VM and as an executable.
#[test]
fn null_for_a_plain_handle_result_stops_the_run_on_both_targets()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(dir.path());
    let path = write_program(dir.path(), &lib, NULL_HANDLE);
    let (out, ran) = vm_run(&path);
    let trap = ran.expect_err("NULL for a plain handle stops the run");
    assert_eq!(trap.kind, TrapKind::NullFromC, "{trap}");
    assert_eq!(out, "before\n");
    let ran = native_run(&cc, &path, &lib);
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert_eq!(ran.status.code(), Some(1), "{stderr}");
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "before\n");
    assert!(
        stderr.contains("C gave NULL where its declaration promised a value"),
        "{stderr}"
    );
}

/// Write a program that calls the lib into a folder, with the path of the
/// lib in it.
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

/// C calls functions of the program back with text, an optional of text, a
/// struct as a value, a handle and an optional of a handle, and a closure
/// with text; each prints the same in the VM and as an executable.
#[test]
fn callbacks_take_text_structs_and_handles_on_both_targets()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(dir.path());
    let path = write_program(dir.path(), &lib, &format!("{CALLED_BACK}{CALLS_BACK}"));
    assert_eq!(run_in_vm(&path), WANT_CALLED_BACK, "in the VM");
    assert_eq!(run_native(&cc, &path, &lib), WANT_CALLED_BACK, "as an executable");
}

/// NULL from C for a parameter of plain text stops the run with the same
/// trap in the VM and as an executable.
#[test]
fn null_for_plain_text_stops_the_run_on_both_targets()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(dir.path());
    let path = write_program(dir.path(), &lib, &format!("{CALLED_BACK}{NULL_TEXT}"));
    let (out, ran) = vm_run(&path);
    let trap = ran.expect_err("NULL for plain text stops the run");
    assert_eq!(trap.kind, TrapKind::NullFromC, "{trap}");
    assert_eq!(out, "plain\n၇\n");
    let ran = native_run(&cc, &path, &lib);
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert_eq!(ran.status.code(), Some(1), "{stderr}");
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "plain\n၇\n");
    let line = "the program stopped: C gave NULL where its declaration promised a \
                value; declare the result or the parameter as an optional, \"T \
                ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်\", when C can give NULL\n";
    assert_eq!(stop_line(&stderr), line, "stderr: {stderr}");
}

/// NULL from C for a parameter of a plain handle stops the run, as for plain
/// text: the parameter promised a handle, and an optional takes NULL.
#[test]
fn null_for_a_plain_handle_parameter_stops_the_run_on_both_targets()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(dir.path());
    let text = format!("{CALLED_BACK}{NULL_HANDLE_ARG}");
    let path = write_program(dir.path(), &lib, &text);
    let (out, ran) = vm_run(&path);
    let trap = ran.expect_err("NULL for a plain handle stops the run");
    assert_eq!(trap.kind, TrapKind::NullFromC, "{trap}");
    assert_eq!(out, "၀\n");
    let ran = native_run(&cc, &path, &lib);
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert_eq!(ran.status.code(), Some(1), "{stderr}");
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "၀\n");
    assert!(
        stderr.contains("C gave NULL where its declaration promised a value"),
        "{stderr}"
    );
}
