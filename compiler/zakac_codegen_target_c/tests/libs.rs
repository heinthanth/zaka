//! A C lib built from Zaka, `--emit lib`: a C program links the static lib
//! and the shared lib, calls the functions that code outside can call, gets
//! a trap back as a status, and finds the lib stopped after it.

use std::path::Path;
use std::process::Command;

use zakac_codegen_target_c::driver::runtime_lib;
use zakac_codegen_target_c::toolchain::Dialect;

use crate::common::cc;
use crate::common::root;
use crate::common::test_abi;

/// What the driver prints, for each form of the lib.
const EXPECTED: &str = "\
add 0 42
no result 0
မင်္ဂလာပါ Zaka
greet 0 4
trap before none
divide 1
trap the program stopped: integer division or remainder with zero as divisor (in စား, \
                        at mathlib.zaka:10:1)
after -1
";

/// Run a command and assert that it succeeded.
///
/// # Arguments
///
/// * `cmd` - The command.
/// * `what` - What it does, for the failure.
///
/// # Returns
///
/// What it printed.
fn ran(cmd: &mut Command, what: &str) -> String
{
    let out = cmd.output().unwrap();
    let errors = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{what} failed:\n{errors}");
    String::from_utf8(out.stdout).unwrap()
}

/// Build the lib of `inputs/libs/mathlib.zaka` into a folder, through the
/// command of the C target.
///
/// # Arguments
///
/// * `dir` - The folder.
fn build_lib(dir: &Path)
{
    let input = dir.join("mathlib.zaka");
    std::fs::copy(root().join("tests/inputs/libs/mathlib.zaka"), &input).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zakac_codegen_target_c"));
    // The trap names the file as the build was given it.
    cmd.current_dir(dir).env("ZAKAC_NO_CONFIG", "1").arg("mathlib.zaka");
    cmd.args(["--emit", "lib", "--cache"]);
    cmd.arg(dir.join("cache")).arg("-o").arg(dir.join("mathlib"));
    ran(&mut cmd, "the lib build");
}

#[test]
fn a_c_program_calls_a_zaka_lib()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    // shortcut: the driver builds with GNU flags; an MSVC host checks the lib
    // through a hand check until the driver has a form of `cl`.
    if Dialect::of(test_abi()) == Dialect::Msvc
    {
        eprintln!("the C lib test runs with a GNU toolchain alone");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    build_lib(dir);
    let driver = root().join("tests/inputs/libs/driver.c");
    let os_libs = runtime_lib(test_abi()).map(|r| r.native_libs().collect::<Vec<_>>());
    let mut static_cmd = cc.command();
    static_cmd.args(["-std=c11", "-I"]).arg(dir).arg(&driver);
    static_cmd
        .arg(dir.join("libmathlib.a"))
        .args(os_libs.unwrap_or_default())
        .arg("-lm");
    static_cmd.arg("-o").arg(dir.join("static_driver"));
    ran(&mut static_cmd, "the link of the static lib");
    let out = ran(&mut Command::new(dir.join("static_driver")), "the static driver");
    assert_eq!(out, EXPECTED);
    // A DLL is found beside the executable, which the test does not arrange.
    if cfg!(windows)
    {
        return;
    }
    let mut shared_cmd = cc.command();
    shared_cmd.args(["-std=c11", "-I"]).arg(dir).arg(&driver).arg("-L").arg(dir);
    shared_cmd.arg(format!("-Wl,-rpath,{}", dir.display())).arg("-lmathlib");
    shared_cmd.arg("-o").arg(dir.join("shared_driver"));
    ran(&mut shared_cmd, "the link of the shared lib");
    let out = ran(&mut Command::new(dir.join("shared_driver")), "the shared driver");
    assert_eq!(out, EXPECTED);
}
