//! A C lib built from Zaka, `--emit lib`: a C program links the static lib
//! and the shared lib, calls the functions that code outside can call, gets
//! a trap back as a status, and finds the lib stopped after it.

use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use zakac_codegen_target_c::driver::runtime_lib;
use zakac_codegen_target_c::toolchain::Dialect;
use zakac_codegen_target_c::toolchain::Tool;
use zakac_rust_sugars::tern;

use crate::common::cc;
use crate::common::root;
use crate::common::test_abi;

/// What the driver prints, for each form of the lib.
const EXPECTED: &str = "\
add 0 42
no result 0
မင်္ဂလာပါ Zaka
greet 0 4
manhattan 0 7
mirror 0 4 3
echo 0 hi!
echo with no out 0
squares 0 2 7 49
older 0 Ann Jr 31
trap before none
divide 1
trap the program stopped: integer division or remainder with zero as divisor (in စား, \
                        at mathlib.zaka:10:1)
trap code 1 1 zero_divisor mathlib
trap at စား mathlib.zaka:10:1, 1 calls, the latest 1
after -1
trap after reset none
reset 0 3
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
    // The C of Windows writes a line end as `\r\n`; the runtime writes `\n`.
    String::from_utf8(out.stdout).unwrap().replace("\r\n", "\n")
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
    // The lib is for the ABI the test links with.
    let abi: &str = test_abi().into();
    cmd.args(["--target-abi", abi, "--emit", "lib", "--cache"]);
    cmd.arg(dir.join("cache")).arg("-o").arg(dir.join("mathlib"));
    ran(&mut cmd, "the lib build");
}

/// The names a shared lib gives, sorted, with no `_` of Mach-O before them.
///
/// # Arguments
///
/// * `path` - The shared lib.
///
/// # Returns
///
/// The names.
fn exported_names(path: &Path) -> Vec<String>
{
    use object::Object;

    let octets = std::fs::read(path).unwrap();
    let file = object::File::parse(&*octets).unwrap();
    let macho = file.format() == object::BinaryFormat::MachO;
    let name = |raw: &[u8]| {
        let text = String::from_utf8_lossy(raw).into_owned();
        let bare = text.strip_prefix('_').filter(|_| macho).map(str::to_string);
        bare.unwrap_or(text)
    };
    let named = |e: object::Export<'_>| match e.name()
    {
        object::NameOrOrdinal::Name(raw) => Some(name(raw)),
        _ => None,
    };
    let exports = file.exports().unwrap().map(Result::unwrap);
    let mut names: Vec<String> = exports.filter_map(named).collect();
    names.sort();
    names
}

/// The command that builds the driver with the libs given, in the form of the
/// toolchain: GNU flags, or those of MSVC's `cl`, with the options of the
/// linker after `/link`. It runs in the folder, where `cl` leaves its objects.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `dialect` - Its flag form.
/// * `dir` - The folder of the lib and the header.
/// * `driver` - The C file of the driver.
/// * `libs` - The libs and the flags of the link.
/// * `exe` - The executable to write.
///
/// # Returns
///
/// The command.
fn driver_cmd(
    cc: &Tool, dialect: Dialect, dir: &Path, driver: &Path, libs: &[OsString],
    exe: &Path,
) -> Command
{
    let mut cmd = cc.command();
    cmd.current_dir(dir);
    if dialect == Dialect::Gnu
    {
        cmd.args(["-std=c11", "-I"]).arg(dir).arg(driver).args(libs).arg("-o").arg(exe);
        return cmd;
    }
    let option = |l: &&OsString| l.to_string_lossy().starts_with('/');
    let (options, files): (Vec<&OsString>, Vec<&OsString>) =
        libs.iter().partition(option);
    let mut named = OsString::from("/Fe");
    named.push(exe);
    cmd.args(["/nologo", "/std:c11", "/I"]).arg(dir).arg(driver).args(files).arg(named);
    cmd.arg("/link").args(options);
    cmd
}

/// Build the driver with the libs given, run it, and check what it prints.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `dir` - The folder of the lib.
/// * `libs` - The libs and the flags of the link.
/// * `name` - The name of the executable.
fn check_driver(cc: &Tool, dir: &Path, libs: &[OsString], name: &str)
{
    let output = run_driver(cc, dir, "driver.c", libs, name);
    assert_eq!(output, EXPECTED, "{name}");
}

/// Build a driver of `inputs/libs` with the libs given and run it.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `dir` - The folder of the lib.
/// * `file` - The C file of the driver.
/// * `libs` - The libs and the flags of the link.
/// * `name` - The name of the executable.
///
/// # Returns
///
/// What it printed.
fn run_driver(
    cc: &Tool, dir: &Path, file: &str, libs: &[OsString], name: &str,
) -> String
{
    let exe = dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    let driver = root().join("tests/inputs/libs").join(file);
    let dialect = Dialect::of(test_abi());
    let mut link = driver_cmd(cc, dialect, dir, &driver, libs, &exe);
    ran(&mut link, &format!("the link of {name}"));
    ran(&mut Command::new(&exe), name)
}

/// The static lib of the test and what a program that links it needs.
///
/// # Arguments
///
/// * `dir` - The folder of the lib.
///
/// # Returns
///
/// The lib, then the libs and the flags of the link.
fn static_libs(dir: &Path) -> Vec<OsString>
{
    let msvc = Dialect::of(test_abi()) == Dialect::Msvc;
    let os_libs = runtime_lib(test_abi()).map(|r| r.native_libs().collect::<Vec<_>>());
    let os_libs = os_libs.unwrap_or_default().into_iter().map(OsString::from);
    let archive = dir.join(tern!(msvc, "mathlib.lib", "libmathlib.a"));
    // pthreads, which the threads driver uses, need the flag on each OS but
    // Windows; a program of one thread links the same with it.
    let gnu = (!msvc).then(|| ["-lm", "-pthread"].map(OsString::from));
    let gnu = gnu.into_iter().flatten().filter(|f| !cfg!(windows) || f != "-pthread");
    std::iter::once(archive.into()).chain(os_libs).chain(gnu).collect()
}

#[test]
fn a_c_program_calls_a_zaka_lib()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    build_lib(dir);
    let shared = format!(
        "{}mathlib{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    let names = exported_names(&dir.join(&shared));
    let api = [
        "add_numbers",
        "digit_count",
        "divide",
        "echo",
        "greet",
        "manhattan",
        "mirror",
        "older",
        "squares",
        "zk_last_trap_from_mathlib",
        "zk_reset_mathlib",
    ];
    let runtime = ["zk_free", "zk_last_trap"].map(zakac_codegen_target_c::linked_name);
    let mut api: Vec<String> =
        api.iter().map(|n| n.to_string()).chain(runtime).collect();
    api.sort();
    assert_eq!(names, api);
    let msvc = Dialect::of(test_abi()) == Dialect::Msvc;
    check_driver(&cc, dir, &static_libs(dir), "static_driver");
    // Windows finds a DLL beside the executable; GNU links the DLL itself, MSVC
    // its import lib.
    let shared_libs: Vec<OsString> = match (cfg!(windows), msvc)
    {
        (true, true) => vec![dir.join("mathlib.dll.lib").into()],
        (true, false) => vec![dir.join(&shared).into()],
        _ =>
        {
            let rpath = format!("-Wl,-rpath,{}", dir.display());
            vec!["-L".into(), dir.into(), rpath.into(), "-lmathlib".into()]
        }
    };
    check_driver(&cc, dir, &shared_libs, "shared_driver");
}

#[test]
fn threads_call_a_zaka_lib_one_at_a_time()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    build_lib(dir);
    let output = run_driver(&cc, dir, "threads.c", &static_libs(dir), "threads");
    // Eight threads count the digits of 0 to 1999, three octets each.
    assert_eq!(output, "165360\n");
}

/// The `zk_` names the members of a static lib define, with no `_` of Mach-O
/// before them.
///
/// # Arguments
///
/// * `path` - The static lib.
///
/// # Returns
///
/// The names.
fn defined_zk_names(path: &Path) -> Vec<String>
{
    use object::Object;
    use object::ObjectSymbol;

    let octets = std::fs::read(path).unwrap();
    let archive = object::read::archive::ArchiveFile::parse(&*octets).unwrap();
    let members = archive.members().map(Result::unwrap);
    let files =
        members.filter_map(|m| object::File::parse(m.data(&*octets).ok()?).ok());
    let names = files.flat_map(|f| {
        let defined = f.symbols().filter(|s| s.is_global() && s.is_definition());
        defined.filter_map(|s| s.name().ok().map(str::to_string)).collect::<Vec<_>>()
    });
    let bare = names.map(|n| n.strip_prefix('_').map(str::to_string).unwrap_or(n));
    bare.filter(|n| n.starts_with("zk_")).collect()
}

#[test]
fn each_name_a_lib_links_under_is_its_own_or_the_runtime_s()
{
    let Some(_) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    build_lib(dir);
    let msvc = Dialect::of(test_abi()) == Dialect::Msvc;
    let lib = tern!(msvc, "mathlib.lib", "libmathlib.a");
    let names = defined_zk_names(&dir.join(lib));
    let api = ["zk_reset_mathlib", "zk_last_trap_from_mathlib"];
    let runtime = zakac_codegen_target_c::RUNTIME_PREFIX;
    let foreign: Vec<&String> = names
        .iter()
        .filter(|n| !api.contains(&n.as_str()))
        .filter(|n| !n.starts_with("zk_mathlib_") && !n.starts_with(runtime))
        .collect();
    assert!(foreign.is_empty(), "names of no lib: {foreign:?}");
    assert!(names.iter().any(|n| n.starts_with("zk_mathlib_")), "{names:?}");
}
