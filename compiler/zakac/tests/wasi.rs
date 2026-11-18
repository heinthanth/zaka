//! The WASI builds of the C target: each repo sample, built with `zakac
//! build --backend c --target-os wasi` for preview 1 and preview 2, prints
//! under wasmtime what the VM prints for it with 32-bit platform kinds.
//!
//! The tests need `wasmtime` on the PATH and wasi-sdk in `WASI_SDK_PATH`, as
//! mise sets them; a host without them skips the tests, unless
//! `ZAKA_REQUIRE_WASI` is `1`, as on CI.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

use tempfile::TempDir;
use zakac_ast::target::ABI_FACT;
use zakac_ast::target::BITS_FACT;
use zakac_ast::target::OS_FACT;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

/// The two ABIs of WASI.
const ABIS: [&str; 2] = ["wasip1", "wasip2"];

/// Sample groups that hold programs with errors on purpose, and the group
/// of C functions, which WASI cannot call.
const SKIPPED_GROUPS: &[&str] = &["diagnostics", "mistakes", "ffi"];

/// The samples the WASI builds do not run, each with the reason.
const SKIPPED: &[(&str, &str)] =
    &[("programs/start", "it prints its own path, argument 0, which differs per host")];

/// The repo folder.
fn repo() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }

/// Whether the WASI tools are here: wasmtime runs, and `WASI_SDK_PATH` is
/// set.
///
/// # Returns
///
/// `true` with both; else `false`, after a note, or a failed assert under
/// `ZAKA_REQUIRE_WASI=1`.
fn tools_here() -> bool
{
    let wasmtime = Command::new("wasmtime").arg("--version").output();
    let here = wasmtime.is_ok_and(|o| o.status.success())
        && std::env::var_os("WASI_SDK_PATH").is_some();
    let required = std::env::var("ZAKA_REQUIRE_WASI").is_ok_and(|v| v == "1");
    assert!(
        here || !required,
        "ZAKA_REQUIRE_WASI=1, but wasmtime or wasi-sdk is missing"
    );
    do_if!(
        !here,
        eprintln!("no wasmtime or WASI_SDK_PATH: the WASI tests are skipped")
    );
    here
}

/// A fresh folder of a test, removed with its build cache when the test is
/// done with it.
///
/// # Arguments
///
/// * `name` - The name the folder starts with.
///
/// # Returns
///
/// The folder.
fn folder(name: &str) -> TempDir
{
    let prefix = format!("zakac-wasi-{}-", name.replace('/', "-"));
    tempfile::Builder::new().prefix(&prefix).tempdir().unwrap()
}

/// Build a program for a WASI ABI into a folder, with each right granted.
///
/// # Arguments
///
/// * `file` - The program.
/// * `abi` - The ABI.
/// * `dir` - The folder of the build.
///
/// # Returns
///
/// The output of `zakac`.
fn build(file: &Path, abi: &str, dir: &Path) -> Output
{
    #[rustfmt::skip]
    let args = ["build", "--backend", "c", "--target-os", "wasi", "--target-abi", abi, "-A"];
    Command::new(env!("CARGO_BIN_EXE_zakac"))
        .env("ZAKAC_NO_CONFIG", "1")
        .args(args)
        .arg("--cache")
        .arg(dir.join("cache"))
        .arg("-o")
        .arg(dir.join("program"))
        .arg(file)
        .output()
        .unwrap()
}

/// Run a WASI program under wasmtime, with its folder as the working folder
/// it sees, so a sample that writes a file writes it there.
///
/// # Arguments
///
/// * `dir` - The folder of the build, which holds `program.wasm`.
///
/// # Returns
///
/// The output of the run.
fn run(dir: &Path) -> Output
{
    let mapped = format!("{}::.", dir.display());
    Command::new("wasmtime")
        .args(["run", "--dir", &mapped])
        .arg(dir.join("program.wasm"))
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// The top-level sample files of each group the tests run, sorted, as
/// `<group>/<stem>` and path.
///
/// # Returns
///
/// The samples.
fn sample_files() -> Vec<(String, PathBuf)>
{
    let skipped = |name: &str| SKIPPED.iter().any(|(s, _)| *s == name);
    let mut found: Vec<(String, PathBuf)> = std::fs::read_dir(repo().join("samples"))
        .unwrap()
        .flatten()
        .map(|g| g.path())
        .filter(|g| g.is_dir() && !SKIPPED_GROUPS.iter().any(|s| g.ends_with(s)))
        .flat_map(|g| std::fs::read_dir(g).unwrap().flatten().map(|f| f.path()))
        .filter(|f| f.extension().is_some_and(|e| e == "zaka"))
        .map(|f| (sample_name(&f), f))
        .filter(|(name, _)| !skipped(name))
        .collect();
    found.sort();
    found
}

/// The name of a sample, `<group>/<stem>`.
///
/// # Arguments
///
/// * `file` - The sample.
///
/// # Returns
///
/// The name.
fn sample_name(file: &Path) -> String
{
    let group = file.parent().and_then(Path::file_name).unwrap().to_string_lossy();
    format!("{group}/{}", file.file_stem().unwrap().to_string_lossy())
}

/// Run a sample in the VM with 32-bit platform kinds, as WASI has them, in a
/// folder of its own, with each right granted.
///
/// # Arguments
///
/// * `file` - The sample.
/// * `dir` - The folder of the run.
///
/// # Returns
///
/// The output of `zakac run`.
fn run_in_vm(file: &Path, dir: &Path) -> Output
{
    std::fs::create_dir_all(dir).unwrap();
    Command::new(env!("CARGO_BIN_EXE_zakac"))
        .env("ZAKAC_NO_CONFIG", "1")
        .args(["run", "--target-bits", "32", "-A"])
        .arg(file)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

/// Build a sample for an ABI, run it, and compare how it ends and what it
/// prints with a run in the VM at 32 bits: a sample that does not compile
/// with 32-bit kinds builds for neither, and one that stops stops in both.
///
/// # Arguments
///
/// * `name` - The name of the sample.
/// * `file` - The sample.
/// * `abi` - The ABI.
///
/// # Returns
///
/// `None` when the two match, else what went wrong.
fn check_sample(name: &str, file: &Path, abi: &str) -> Option<String>
{
    let tmp = folder(&format!("{abi}-{name}"));
    let dir = tmp.path();
    let vm = run_in_vm(file, &dir.join("vm"));
    let text = |raw: &[u8]| String::from_utf8_lossy(raw).into_owned();
    let built = build(file, abi, dir);
    let refused_in_both =
        !built.status.success() && !vm.status.success() && vm.stdout.is_empty();
    ret_if!(refused_in_both, None);
    let failed = !built.status.success();
    ret_if!(
        failed,
        Some(format!("{name} ({abi}) did not build:\n{}", text(&built.stderr)))
    );
    let ran = run(dir);
    let wrong = ran.status.success() != vm.status.success() || ran.stdout != vm.stdout;
    let shown = || {
        let (got, want) = (text(&ran.stdout), text(&vm.stdout));
        format!(
            "{name} ({abi}) printed:\n{got}{}\nwhere the VM printed:\n{want}",
            text(&ran.stderr)
        )
    };
    wrong.then(shown)
}

/// Each repo sample prints under wasmtime, built for each ABI of WASI, the
/// output the VM prints for it. The samples build on each core at once.
#[test]
fn repo_samples_print_what_the_vm_prints_on_wasi()
{
    ret_if!(!tools_here());
    let jobs: Vec<(String, PathBuf, &str)> = sample_files()
        .into_iter()
        .flat_map(|(name, file)| ABIS.map(|abi| (name.clone(), file.clone(), abi)))
        .collect();
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let failed: Vec<String> = std::thread::scope(|s| {
        let jobs = &jobs;
        let work = move |i: usize| {
            let mine = jobs.iter().skip(i).step_by(threads);
            mine.filter_map(|(n, f, a)| check_sample(n, f, a)).collect::<Vec<_>>()
        };
        let workers: Vec<_> = (0..threads).map(|i| s.spawn(move || work(i))).collect();
        workers.into_iter().flat_map(|w| w.join().unwrap()).collect()
    });
    assert!(failed.is_empty(), "{}", failed.join("\n"));
    assert!(jobs.len() >= 140, "ran {} builds", jobs.len());
}

/// A program for WASI holds the facts of WASI: the OS `wasi`, the ABI it was
/// built for, and 32-bit platform kinds.
#[test]
fn a_wasi_program_holds_the_facts_of_wasi()
{
    ret_if!(!tools_here());
    let program = format!(
        "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{OS_FACT} ကို ပြသမည်။\n{ABI_FACT} ကို \
         ပြသမည်။\n{BITS_FACT} ကို ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n"
    );
    for abi in ABIS
    {
        let tmp = folder(&format!("facts-{abi}"));
        let dir = tmp.path();
        let file = dir.join("facts.zaka");
        std::fs::write(&file, &program).unwrap();
        let built = build(&file, abi, dir);
        assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
        let ran = run(dir);
        let printed = String::from_utf8_lossy(&ran.stdout);
        let lines: Vec<&str> = printed.lines().collect();
        assert_eq!(lines[..2], ["wasi", abi], "{printed}");
        assert!(["32", "၃၂"].contains(&lines[2]), "{printed}");
    }
}

/// A program that calls C functions does not build for WASI, which loads no
/// shared libs.
#[test]
fn c_functions_are_refused_on_wasi()
{
    let file = repo().join("samples/ffi/libc.zaka");
    let tmp = folder("ffi");
    let built = build(&file, "wasip1", tmp.path());
    assert!(!built.status.success());
    let stderr = String::from_utf8_lossy(&built.stderr);
    assert!(stderr.contains("cannot call C functions"), "{stderr}");
}
