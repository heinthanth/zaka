//! Build the runtime lib (`zaka_runtime`) for the target and embed it, so
//! `zakac` links an executable with nothing beside it: a nested `cargo` builds
//! the lib into a folder of its own under `OUT_DIR`, so it never waits on the
//! lock of the outer build.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

/// The profile of the runtime lib: release, with a panic that aborts.
const PROFILE: &str = "zaka-runtime";
/// The lowest macOS the lib targets: Rust's own floor, so the linker
/// never meets an object built for a newer macOS than the program.
const MACOS_FLOOR: &str = "11.0";

fn main()
{
    cargo_build::rerun_if_changed!("locales");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let target = std::env::var("TARGET").expect("cargo sets TARGET");
    let lib = out.join("libzaka_runtime.a");
    let libs = out.join("native-libs.txt");
    let (other_lib, other_libs) =
        (out.join("zaka_runtime-other"), out.join("native-libs-other.txt"));
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for watched in ["compiler/zaka_runtime", "compiler/zaka_host/src", "Cargo.lock"]
    {
        println!("cargo:rerun-if-changed={}", root.join(watched).display());
    }
    write_header(&root.join("compiler/zaka_runtime"), &out.join("zaka_runtime.h"));
    // The explorer builds this crate for the browser, which links no program.
    let blank = [&lib, &libs, &other_lib, &other_libs];
    ret_if!(target.starts_with("wasm32"), blank.iter().for_each(|f| write(f, b"")));
    let printed = build_runtime(&root, &out.join("zaka_runtime-target"), &target, &lib);
    let printed =
        printed.unwrap_or_else(|e| panic!("the runtime lib did not build:\n{e}"));
    write(&libs, native_libs(&printed).as_bytes());
    // A Windows host carries the runtime of the other ABI too, so
    // `--target-abi` picks either; a host that cannot build it carries
    // none.
    let other =
        other_windows(&target).map(|t| (t, out.join("zaka_runtime-other-target")));
    let built = other.map(|(t, dir)| (t, build_runtime(&root, &dir, t, &other_lib)));
    match built
    {
        Some((_, Ok(printed))) => write(&other_libs, native_libs(&printed).as_bytes()),
        Some((t, Err(_))) =>
        {
            println!("cargo:warning=no runtime lib for {t}: `rustup target add {t}`");
            [&other_lib, &other_libs].iter().for_each(|f| write(f, b""));
        }
        None => [&other_lib, &other_libs].iter().for_each(|f| write(f, b"")),
    }
}

/// The Windows target of the other ABI, for a Windows target.
///
/// # Arguments
///
/// * `target` - The target triple.
///
/// # Returns
///
/// The triple with `gnu` and `msvc` swapped, or `None` off Windows.
fn other_windows(target: &str) -> Option<&'static str>
{
    match target
    {
        "x86_64-pc-windows-gnu" => Some("x86_64-pc-windows-msvc"),
        "x86_64-pc-windows-msvc" => Some("x86_64-pc-windows-gnu"),
        _ => None,
    }
}

/// The file name a staticlib of the runtime takes for a target: MSVC names it
/// `zaka_runtime.lib`, the others `libzaka_runtime.a`.
///
/// # Arguments
///
/// * `target` - The target triple.
///
/// # Returns
///
/// The file name.
fn lib_file(target: &str) -> &'static str
{
    tern!(target.ends_with("-msvc"), "zaka_runtime.lib", "libzaka_runtime.a")
}

/// Write the C header of the runtime from its Rust: the functions, the
/// structs the generated code and the runtime share, the ids of the traps,
/// the rights and the kinds, as `cbindgen.toml` of the crate sets them out.
/// A browser build writes it too, as the explorer shows it.
///
/// # Arguments
///
/// * `runtime` - The folder of the crate `zaka_runtime`.
/// * `header` - Where the header goes.
fn write_header(runtime: &Path, header: &Path)
{
    let config = cbindgen::Config::from_file(runtime.join("cbindgen.toml"))
        .expect("cbindgen.toml of the runtime reads");
    let made = cbindgen::Builder::new()
        .with_config(config)
        .with_src(runtime.join("src/lib.rs"))
        .generate()
        .expect("the runtime header generates");
    let mut text = Vec::new();
    made.write(&mut text);
    let text = String::from_utf8(text).expect("cbindgen writes UTF-8");
    write(header, first_paragraphs(&text).as_bytes());
}

/// The header with each doc comment cut to its first paragraph: the gist
/// of the item, where the Rust doc goes on with its arguments and results.
///
/// # Arguments
///
/// * `text` - The header as cbindgen wrote it.
///
/// # Returns
///
/// The header.
fn first_paragraphs(text: &str) -> String
{
    let mut skip = false;
    // A blank line inside a doc ends its first paragraph, and `*/` ends the
    // doc.
    let kept = |line: &&str| {
        let mark = line.trim_start();
        skip = (skip && mark != "*/") || mark == "*";
        !skip
    };
    text.lines().filter(kept).map(|l| format!("{l}\n")).collect()
}

/// Build the lib with a nested `cargo` and put it, stripped of debug info,
/// to `lib`.
///
/// # Arguments
///
/// * `root` - The workspace folder.
/// * `target_dir` - The target folder of the nested build.
/// * `target` - The target triple.
/// * `lib` - Where the lib goes.
///
/// # Returns
///
/// What the nested build wrote to standard error, which holds the link list,
/// or all of it when the build failed.
fn build_runtime(
    root: &Path, target_dir: &Path, target: &str, lib: &Path,
) -> Result<String, String>
{
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.current_dir(root)
        .args(["rustc", "-p", "zaka_runtime", "--lib", "--crate-type", "staticlib"])
        .args(["--profile", PROFILE, "--target", target, "--target-dir"])
        .arg(target_dir)
        .args(["--", "--print", "native-static-libs"]);
    // The flags of the outer build (coverage, wrappers, its target folder)
    // are not the lib's.
    let outer = |name: &String| {
        let flags = ["RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTC"];
        (name.starts_with("CARGO_") && name != "CARGO_HOME")
            || flags.contains(&name.as_str())
    };
    let names = std::env::vars().map(|(name, _)| name).filter(outer);
    do_for!(name in names, cmd.env_remove(&name));
    let floor = target.contains("apple")
        && std::env::var_os("MACOSX_DEPLOYMENT_TARGET").is_none();
    do_if!(floor, _ = cmd.env("MACOSX_DEPLOYMENT_TARGET", MACOS_FLOOR));
    let output = cmd.output().expect("cargo runs");
    let printed = String::from_utf8_lossy(&output.stderr).into_owned();
    ret_if!(!output.status.success(), Err(printed));
    let built = target_dir.join(target).join(PROFILE).join(lib_file(target));
    std::fs::copy(&built, lib).expect("the runtime lib is readable");
    // Debug info is most of the lib; a host with no `strip` keeps it, and the
    // `strip` of GNU leaves an MSVC lib alone.
    let gnu = !target.ends_with("-msvc");
    do_if!(gnu, _ = Command::new("strip").arg("-S").arg(lib).status());
    Ok(printed)
}

/// The link list that rustc printed for the lib, one flag a line.
///
/// # Arguments
///
/// * `printed` - The standard error of the nested build.
///
/// # Returns
///
/// The flags.
fn native_libs(printed: &str) -> String
{
    let marker = "native-static-libs:";
    let line = printed.lines().find_map(|l| l.split_once(marker).map(|(_, rest)| rest));
    let flags = line.unwrap_or_default().split_whitespace();
    flags.map(|f| format!("{f}\n")).collect()
}

/// Write a file of the build, when its content changes.
///
/// # Arguments
///
/// * `path` - The file.
/// * `content` - Its content.
fn write(path: &Path, content: &[u8])
{
    let same = std::fs::read(path).is_ok_and(|old| old == content);
    do_if!(!same, std::fs::write(path, content).expect("OUT_DIR is writable"));
}
