//! A C lib built from Zaka with each backend, `zakac build --backend c|native
//! --emit lib`, called from a C program and from a Rust program through the
//! static lib.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use zakac_codegen_target_c::driver::host_abi;
use zakac_codegen_target_c::driver::runtime_lib;
use zakac_codegen_target_c::toolchain::Dialect;
use zakac_codegen_target_c::toolchain::find_cc_for;

/// What the C driver prints.
const FROM_C: &str = "\
add 0 42
no result 0
မင်္ဂလာပါ Zaka
greet 0 4
manhattan 0 7
trap before none
divide 1
trap the program stopped: integer division or remainder with zero as divisor (in စား, \
                      at mathlib.zaka:10:1)
after -1
";

/// What the Rust driver prints.
const FROM_RUST: &str = "\
add 0 42
မင်္ဂလာပါ Rust
greet 0 4
divide 1
trap the program stopped: integer division or remainder with zero as divisor (in စား, \
                         at mathlib.zaka:10:1)
after -1
";

/// The folder of the inputs of the C lib tests of the C target.
///
/// # Returns
///
/// The folder.
fn c_inputs() -> PathBuf
{
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../zakac_codegen_target_c/tests/inputs/libs")
}

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
    // The C of Windows writes a line end as `\r\n`; the runtime writes `\n`.
    String::from_utf8(out.stdout).unwrap().replace("\r\n", "\n")
}

/// Build the lib of `mathlib.zaka` with a backend into a folder of its own.
///
/// # Arguments
///
/// * `dir` - The folder.
/// * `backend` - The backend, `c` or `native`.
fn build_lib(dir: &Path, backend: &str)
{
    std::fs::copy(c_inputs().join("mathlib.zaka"), dir.join("mathlib.zaka")).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zakac"));
    // The trap names the file as the build was given it.
    cmd.current_dir(dir).env("ZAKAC_NO_CONFIG", "1");
    cmd.args(["build", "--backend", backend, "--emit", "lib", "--cache", "cache"]);
    cmd.args(["-o", "mathlib", "mathlib.zaka"]);
    ran(&mut cmd, &format!("the lib build of the {backend} backend"));
}

/// Link the C driver with the static lib in a folder, and run it.
///
/// # Arguments
///
/// * `dir` - The folder of the lib.
/// * `os_libs` - The OS libs of the runtime.
///
/// # Returns
///
/// What it printed.
fn run_c_driver(dir: &Path, os_libs: &[&str]) -> String
{
    let cc = find_cc_for(None, Dialect::Gnu).expect("a C compiler");
    let mut cmd = cc.command();
    cmd.args(["-std=c11", "-I"]).arg(dir).arg(c_inputs().join("driver.c"));
    cmd.arg(dir.join("libmathlib.a")).args(os_libs).arg("-lm");
    cmd.arg("-o").arg(dir.join(format!("c_driver{}", std::env::consts::EXE_SUFFIX)));
    ran(&mut cmd, "the link of the C driver");
    ran(
        &mut Command::new(
            dir.join(format!("c_driver{}", std::env::consts::EXE_SUFFIX)),
        ),
        "the C driver",
    )
}

/// Build the Rust driver with the static lib in a folder, and run it.
///
/// # Arguments
///
/// * `dir` - The folder of the lib.
/// * `os_libs` - The OS libs of the runtime.
///
/// # Returns
///
/// What it printed.
fn run_rust_driver(dir: &Path, os_libs: &[&str]) -> String
{
    let rustc = std::env::var_os("RUSTC").unwrap_or("rustc".into());
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/inputs/c_libs/driver.rs");
    let mut cmd = Command::new(rustc);
    // The static lib goes in a folder of its own: beside the shared lib, the
    // linker of macOS would take the shared one for `-l mathlib`.
    let alone = dir.join("static");
    std::fs::create_dir_all(&alone).unwrap();
    std::fs::copy(dir.join("libmathlib.a"), alone.join("libmathlib.a")).unwrap();
    cmd.args(["--edition", "2024"]).arg(source).arg("-L").arg(&alone);
    cmd.args(["-l", "static=mathlib"]);
    let flag = |f: &&str| match f.strip_prefix("-l")
    {
        Some(name) => ["-l".to_string(), name.to_string()],
        None => ["-C".to_string(), format!("link-arg={f}")],
    };
    cmd.args(os_libs.iter().flat_map(flag));
    cmd.arg("-o").arg(dir.join(format!("rust_driver{}", std::env::consts::EXE_SUFFIX)));
    ran(&mut cmd, "the build of the Rust driver");
    ran(
        &mut Command::new(
            dir.join(format!("rust_driver{}", std::env::consts::EXE_SUFFIX)),
        ),
        "the Rust driver",
    )
}

#[test]
fn c_and_rust_call_a_zaka_lib_of_each_backend()
{
    // shortcut: the drivers build with GNU flags; an MSVC host checks the lib
    // through a hand check until the drivers have a form of `cl`.
    let gnu = Dialect::of(host_abi()) == Dialect::Gnu;
    if !gnu || find_cc_for(None, Dialect::Gnu).is_err()
    {
        eprintln!("the C lib test needs a GNU C toolchain");
        return;
    }
    let os_libs = runtime_lib(host_abi()).map(|r| r.native_libs().collect::<Vec<_>>());
    let os_libs = os_libs.unwrap_or_default();
    for backend in ["c", "native"]
    {
        let dir = tempfile::tempdir().unwrap();
        build_lib(dir.path(), backend);
        assert_eq!(
            run_c_driver(dir.path(), &os_libs),
            FROM_C,
            "the {backend} lib from C"
        );
        let from_rust = run_rust_driver(dir.path(), &os_libs);
        assert_eq!(from_rust, FROM_RUST, "the {backend} lib from Rust");
    }
}
