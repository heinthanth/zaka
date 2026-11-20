//! A C lib built from Zaka with each backend, `zakac build --backend c|native
//! --emit lib`, called from a C program and from a Rust program through the
//! static lib.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use zakac_codegen_target_c::driver::host_abi;
use zakac_codegen_target_c::driver::prefix_in;
use zakac_codegen_target_c::driver::runtime_lib;
use zakac_codegen_target_c::toolchain::Dialect;
use zakac_codegen_target_c::toolchain::Tool;
use zakac_codegen_target_c::toolchain::find_cc_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

/// What the C driver prints.
const FROM_C: &str = "\
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
names 0 2 Dr Ann, Dr Bo
count_names 0 2
same_people 0 2 Ann 30 Bo 4
trap before none
divide 1
trap the program stopped: integer division or remainder with zero as divisor (in စား, \
                      at mathlib.zaka:10:1)
trap code 1 1 zero_divisor mathlib
trap at စား mathlib.zaka:10:1, 1 calls, the latest 1
after 0
trap after reset none
reset 0 3
";

/// What the Rust driver prints.
const FROM_RUST: &str = "\
add 0 42
မင်္ဂလာပါ Rust
greet 0 4
divide 1
trap the program stopped: integer division or remainder with zero as divisor (in စား, \
                         at mathlib.zaka:10:1)
trap kind zero_divisor mathlib
after 0
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

/// What the C program that links two libs prints.
const TWO_LIBS: &str = "\
add 0 42
twice 0 42
half 1 twicelib
မင်္ဂလာပါ Zaka
greet 0 4
divide 1
traps math none
latest mathlib
after 0 0 10
reset 0 3
";

/// Build the lib of `mathlib.zaka` with a backend into a folder of its own.
///
/// # Arguments
///
/// * `dir` - The folder.
/// * `backend` - The backend, `c` or `native`.
fn build_lib(dir: &Path, backend: &str) { build_named(dir, backend, "mathlib", &[]) }

/// Build the lib of `<name>.zaka` of the inputs with a backend into a folder.
///
/// # Arguments
///
/// * `dir` - The folder.
/// * `backend` - The backend, `c` or `native`.
/// * `name` - The name of the lib and of its file.
/// * `flags` - More flags of the build.
fn build_named(dir: &Path, backend: &str, name: &str, flags: &[&str])
{
    let file = format!("{name}.zaka");
    do_if!(!dir.join(&file).exists(), {
        std::fs::copy(c_inputs().join(&file), dir.join(&file)).unwrap();
    });
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zakac"));
    // The trap names the file as the build was given it.
    cmd.current_dir(dir).env("ZAKAC_NO_CONFIG", "1");
    cmd.args(["build", "--backend", backend, "--emit", "lib", "--cache"]);
    cmd.arg(format!("cache_{name}")).args(flags).args(["-o", name, &file]);
    ran(&mut cmd, &format!("the build of {name} with the {backend} backend"));
}

/// The file of a static lib of a folder, as the toolchain of the host names
/// it: `lib<name>.a`, or `<name>.lib` for MSVC.
///
/// # Arguments
///
/// * `name` - The name of the lib.
///
/// # Returns
///
/// The file name.
fn static_file(name: &str) -> String
{
    match Dialect::of(host_abi())
    {
        Dialect::Msvc => format!("{name}.lib"),
        Dialect::Gnu => format!("lib{name}.a"),
    }
}

/// The C compiler of the host, or `None` when the host has none.
///
/// # Returns
///
/// The compiler.
fn host_cc() -> Option<Tool>
{
    let cc = find_cc_for(None, Dialect::of(host_abi())).ok();
    do_if!(cc.is_none(), eprintln!("the C lib tests need a C compiler"));
    cc
}

/// The OS libs the runtime of the host needs.
///
/// # Returns
///
/// The flags.
fn os_libs() -> Vec<&'static str>
{
    let libs = runtime_lib(host_abi()).map(|r| r.native_libs().collect::<Vec<_>>());
    libs.unwrap_or_default()
}

/// Link a C program of the inputs with static libs of a folder, in the form of
/// the toolchain of the host, and run it: GNU flags, or those of MSVC's `cl`,
/// with the options of the linker after `/link`.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `dir` - The folder of the libs and their headers.
/// * `source` - The C file of the inputs.
/// * `libs` - The names of the libs.
///
/// # Returns
///
/// What the program printed.
fn run_c_program(cc: &Tool, dir: &Path, source: &str, libs: &[&str]) -> String
{
    let stem = source.trim_end_matches(".c");
    let exe = dir.join(format!("{stem}{}", std::env::consts::EXE_SUFFIX));
    let archives = libs.iter().map(|l| dir.join(static_file(l)));
    let mut cmd = cc.command();
    cmd.current_dir(dir);
    if Dialect::of(host_abi()) == Dialect::Gnu
    {
        cmd.args(["-std=c11", "-I"]).arg(dir).arg(c_inputs().join(source));
        cmd.args(archives).args(os_libs()).arg("-lm").arg("-o").arg(&exe);
        ran(&mut cmd, &format!("the link of {source}"));
        return ran(&mut Command::new(&exe), source);
    }
    let os = os_libs();
    let (options, files): (Vec<&str>, Vec<&str>) =
        os.iter().partition(|l| l.starts_with('/'));
    let mut named = std::ffi::OsString::from("/Fe");
    named.push(&exe);
    cmd.args(["/nologo", "/std:c11", "/I"]).arg(dir).arg(c_inputs().join(source));
    cmd.args(archives).args(files).arg(named).arg("/link").args(options);
    ran(&mut cmd, &format!("the link of {source}"));
    ran(&mut Command::new(&exe), source)
}

/// Build the Rust driver with the static lib in a folder, and run it.
///
/// # Arguments
///
/// * `dir` - The folder of the lib.
///
/// # Returns
///
/// What it printed.
fn run_rust_driver(dir: &Path) -> String
{
    let rustc = std::env::var_os("RUSTC").unwrap_or("rustc".into());
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/inputs/c_libs/driver.rs");
    let mut cmd = Command::new(rustc);
    // The static lib goes in a folder of its own: beside the shared lib, the
    // linker of macOS would take the shared one for `-l mathlib`.
    let alone = dir.join("static");
    std::fs::create_dir_all(&alone).unwrap();
    let file = static_file("mathlib");
    std::fs::copy(dir.join(&file), alone.join(&file)).unwrap();
    cmd.args(["--edition", "2024"]).arg(source).arg("-L").arg(&alone);
    cmd.args(["-l", "static=mathlib"]);
    let flag = |f: &&str| match f.strip_prefix("-l")
    {
        Some(name) => ["-l".to_string(), name.to_string()],
        None => ["-C".to_string(), format!("link-arg={f}")],
    };
    cmd.args(os_libs().iter().flat_map(flag));
    let exe = dir.join(format!("rust_driver{}", std::env::consts::EXE_SUFFIX));
    cmd.arg("-o").arg(&exe);
    ran(&mut cmd, "the build of the Rust driver");
    ran(&mut Command::new(&exe), "the Rust driver")
}

#[test]
fn c_and_rust_call_a_zaka_lib_of_each_backend()
{
    let Some(cc) = host_cc()
    else
    {
        return;
    };
    for backend in ["c", "native"]
    {
        let dir = tempfile::tempdir().unwrap();
        build_lib(dir.path(), backend);
        let from_c = run_c_program(&cc, dir.path(), "driver.c", &["mathlib"]);
        assert_eq!(from_c, FROM_C, "the {backend} lib from C");
        let from_rust = run_rust_driver(dir.path());
        assert_eq!(from_rust, FROM_RUST, "the {backend} lib from Rust");
    }
}

#[test]
fn two_zaka_libs_link_into_one_c_program()
{
    let Some(cc) = host_cc()
    else
    {
        return;
    };
    for backend in ["c", "native"]
    {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        build_named(dir, backend, "mathlib", &[]);
        build_named(dir, backend, "twicelib", &[]);
        let printed = run_c_program(&cc, dir, "two_libs.c", &["mathlib", "twicelib"]);
        assert_eq!(printed, TWO_LIBS, "two libs of the {backend} backend");
    }
}

/// What the C program that links two libs with a heap cap each prints.
const HEAP_LIBS: &str = "\
a 0 65536
b 0 65536
a again 1 out_of_mem
";

#[test]
fn each_lib_counts_its_heap_under_its_own_cap()
{
    let Some(cc) = host_cc()
    else
    {
        return;
    };
    let template = std::fs::read_to_string(c_inputs().join("heaplib.zaka")).unwrap();
    for backend in ["c", "native"]
    {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        for name in ["heapa", "heapb"]
        {
            let source = template.replace("fill_NAME", &format!("fill_{}", &name[4..]));
            std::fs::write(dir.join(format!("{name}.zaka")), source).unwrap();
            build_named(dir, backend, name, &["--max-heap", "800000"]);
        }
        let printed = run_c_program(&cc, dir, "heap_libs.c", &["heapa", "heapb"]);
        assert_eq!(printed, HEAP_LIBS, "two capped libs of the {backend} backend");
    }
}

/// The triple of the host, as rustc names it.
///
/// # Returns
///
/// The triple.
fn host_triple() -> String
{
    let rustc = std::env::var_os("RUSTC").unwrap_or("rustc".into());
    let told = ran(Command::new(rustc).arg("-vV"), "rustc -vV");
    let host = told.lines().find_map(|l| l.strip_prefix("host: "));
    host.expect("rustc names its host").trim().to_string()
}

/// A runtime lib built from the sources of the one `zakac` carries with a salt
/// in its hash, so its names differ, as those of a lib from another build of
/// `zakac` do: built once into `target/zaka-salted-runtime`, then reused, and
/// put with its `native-libs.txt` into a folder.
///
/// # Arguments
///
/// * `dir` - The folder.
///
/// # Returns
///
/// The lib.
fn salted_runtime(dir: &Path) -> PathBuf
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (triple, target) = (host_triple(), root.join("target/zaka-salted-runtime"));
    let mut cmd = Command::new(std::env::var_os("CARGO").unwrap_or("cargo".into()));
    cmd.current_dir(&root).env("ZAKA_RUNTIME_SALT", "a second build");
    // Rust derives the link names of each crate from its metadata, as the
    // build of `zakac` sets it for its own runtime.
    cmd.env("RUSTFLAGS", "-C metadata=zk-rt-a-second-build");
    cmd.args(["rustc", "-p", "zaka_runtime", "--lib", "--crate-type", "staticlib"]);
    cmd.args(["--profile", "zaka-runtime", "--target", &triple, "--target-dir"]);
    cmd.arg(&target).args(["--", "--print", "native-static-libs"]);
    // The flags of the outer build are not the lib's.
    let outer = |name: &String| name.starts_with("CARGO_") && name != "CARGO_HOME";
    let names = std::env::vars().map(|(name, _)| name).filter(outer);
    names.for_each(|name| _ = cmd.env_remove(name));
    do_if!(triple.contains("apple"), _ = cmd.env("MACOSX_DEPLOYMENT_TARGET", "11.0"));
    let out = cmd.output().unwrap();
    let told = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(out.status.success(), "the salted runtime did not build:\n{told}");
    let line = told.lines().find_map(|l| l.split("native-static-libs: ").nth(1));
    let libs: String =
        line.unwrap_or("").split_whitespace().map(|f| format!("{f}\n")).collect();
    let salted = dir.join("salted");
    std::fs::create_dir_all(&salted).unwrap();
    std::fs::write(salted.join("native-libs.txt"), libs).unwrap();
    let file =
        tern!(triple.ends_with("-msvc"), "zaka_runtime.lib", "libzaka_runtime.a");
    let lib = salted.join(file);
    std::fs::copy(target.join(&triple).join("zaka-runtime").join(file), &lib).unwrap();
    lib
}

/// The prefix of the runtime names a static lib of a folder links with.
///
/// # Arguments
///
/// * `dir` - The folder.
/// * `name` - The name of the lib.
///
/// # Returns
///
/// The prefix.
fn runtime_of(dir: &Path, name: &str) -> String
{
    let octets = std::fs::read(dir.join(static_file(name))).unwrap();
    prefix_in(&octets).expect("the lib holds the runtime")
}

#[test]
fn libs_of_two_runtime_builds_link_into_one_program()
{
    let Some(cc) = host_cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let salted = salted_runtime(dir);
    let salted = salted.to_string_lossy().into_owned();
    for backend in ["c", "native"]
    {
        build_named(dir, backend, "mathlib", &[]);
        build_named(dir, backend, "twicelib", &["--runtime-lib", &salted]);
        let (math, twice) = (runtime_of(dir, "mathlib"), runtime_of(dir, "twicelib"));
        assert_ne!(math, twice, "two builds of the runtime name their functions apart");
        let printed = run_c_program(&cc, dir, "two_libs.c", &["mathlib", "twicelib"]);
        assert_eq!(printed, TWO_LIBS, "two runtimes with the {backend} backend");
    }
}

/// What the C program that calls threadlib.zaka prints.
#[cfg(unix)]
const THREAD_CALLS: &str = "\
thread 0 1
thread 0 20
fail 0 2
trap zero_divisor in ပျက်
after -1
reset 0 1
";

/// Run a program, and fail when it runs past a deadline: a lib that keeps
/// its lock while it waits in C would hang.
///
/// # Arguments
///
/// * `exe` - The program.
///
/// # Returns
///
/// What it printed.
#[cfg(unix)]
fn ran_in_time(exe: &Path) -> String
{
    use std::time::Duration;
    use std::time::Instant;

    let mut child =
        Command::new(exe).stdout(std::process::Stdio::piped()).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    while child.try_wait().unwrap().is_none()
    {
        do_if!(Instant::now() > deadline, {
            _ = child.kill();
            panic!("{} ran past its deadline", exe.display());
        });
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{} failed", exe.display());
    String::from_utf8(out.stdout).unwrap()
}

#[cfg(unix)]
#[test]
fn a_call_that_waits_in_c_lets_another_thread_into_the_lib()
{
    let Some(cc) = host_cc()
    else
    {
        return;
    };
    for backend in ["c", "native"]
    {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        build_named(dir, backend, "threadlib", &["--allow-ffi"]);
        let exe = dir.join("thread_calls");
        let mut cmd = cc.command();
        cmd.args(["-std=c11", "-I"]).arg(dir).arg(c_inputs().join("thread_calls.c"));
        cmd.arg(dir.join(static_file("threadlib")))
            .args(os_libs())
            .args(["-lm", "-pthread"]);
        ran(cmd.arg("-o").arg(&exe), "the link of thread_calls.c");
        assert_eq!(
            ran_in_time(&exe),
            THREAD_CALLS,
            "threads with the {backend} backend"
        );
    }
}

/// What the C program that calls freelib.zaka prints.
#[cfg(unix)]
const FREE_CALLS: &str = "\
wrong 0
fail trap zero_divisor
after 0 2
count 0 401
";

#[cfg(unix)]
#[test]
fn exports_that_reach_no_global_run_on_threads_at_once()
{
    let Some(cc) = host_cc()
    else
    {
        return;
    };
    for backend in ["c", "native"]
    {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        build_named(dir, backend, "freelib", &[]);
        let exe = dir.join("free_calls");
        let mut cmd = cc.command();
        cmd.args(["-std=c11", "-I"]).arg(dir).arg(c_inputs().join("free_calls.c"));
        cmd.arg(dir.join(static_file("freelib")))
            .args(os_libs())
            .args(["-lm", "-pthread"]);
        ran(cmd.arg("-o").arg(&exe), "the link of free_calls.c");
        let printed = ran_in_time(&exe);
        assert_eq!(printed, FREE_CALLS, "free calls with the {backend} backend");
    }
}

#[cfg(unix)]
/// The folder of the programs of these tests that call C.
///
/// # Returns
///
/// The folder.
fn program_inputs() -> PathBuf
{
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/inputs/c_libs")
}

#[cfg(unix)]
/// Build a program of [`program_inputs`] into an executable, run it, and
/// give back its status, its output and its errors.
///
/// # Arguments
///
/// * `dir` - The folder to build in.
/// * `backend` - The backend.
/// * `name` - The name of the program, with no `.zaka`.
///
/// # Returns
///
/// Whether it ended well, what it printed, and its errors.
fn built_and_ran(dir: &Path, backend: &str, name: &str) -> (bool, String, String)
{
    let file = format!("{name}.zaka");
    std::fs::copy(program_inputs().join(&file), dir.join(&file)).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zakac"));
    cmd.current_dir(dir).env("ZAKAC_NO_CONFIG", "1");
    cmd.args(["build", "--backend", backend, "--allow-ffi", "-o", name, &file]);
    ran(&mut cmd, "the build of a program");
    let out = Command::new(dir.join(name)).output().unwrap();
    let text = |o: &[u8]| String::from_utf8_lossy(o).replace("\r\n", "\n");
    (out.status.success(), text(&out.stdout), text(&out.stderr))
}

#[cfg(unix)]
#[test]
fn c_calls_a_function_that_uses_no_global_from_a_thread_of_its_own()
{
    for backend in ["c", "native"]
    {
        let dir = tempfile::tempdir().unwrap();
        let (ok, printed, _) = built_and_ran(dir.path(), backend, "c_thread");
        assert!(ok, "the program ends well with the {backend} backend");
        assert_eq!(printed, "၁၀\nပြီး\n", "C threads with the {backend} backend");
    }
}

#[cfg(unix)]
#[test]
fn a_call_from_a_c_thread_of_a_function_that_uses_a_global_ends_the_run()
{
    for backend in ["c", "native"]
    {
        let dir = tempfile::tempdir().unwrap();
        let (ok, _, errors) = built_and_ran(dir.path(), backend, "c_thread_global");
        assert!(!ok, "the run ends with the {backend} backend");
        assert!(errors.contains("uses a mutable global"), "{backend}: {errors}");
    }
}

#[cfg(unix)]
#[test]
fn c_calls_a_closure_of_copied_captures_from_a_thread_of_its_own()
{
    for backend in ["c", "native"]
    {
        let dir = tempfile::tempdir().unwrap();
        let (ok, printed, _) = built_and_ran(dir.path(), backend, "c_thread_closure");
        assert!(ok, "the program ends well with the {backend} backend");
        assert_eq!(printed, "၄၂\nzaka\nပြီး\n", "a closure with the {backend} backend");
    }
}

#[cfg(unix)]
#[test]
fn a_call_from_a_c_thread_of_a_closure_whose_capture_is_written_ends_the_run()
{
    for backend in ["c", "native"]
    {
        let dir = tempfile::tempdir().unwrap();
        let name = "c_thread_closure_written";
        let (ok, _, errors) = built_and_ran(dir.path(), backend, name);
        assert!(!ok, "the run ends with the {backend} backend");
        assert!(
            errors.contains("a variable a statement writes"),
            "{backend}: {errors}"
        );
    }
}
