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

include!("../zaka_runtime/stamp.rs");

/// The start of the attribute that names an export of the runtime.
const STAMPED: &str = "#[unsafe(export_name = stamped!(\"";

/// The profile of the runtime lib: release, with a panic that aborts.
const PROFILE: &str = "zaka-runtime";
/// The lowest macOS the lib targets: Rust's own floor, so the linker
/// never meets an object built for a newer macOS than the program.
const MACOS_FLOOR: &str = "11.0";

fn main()
{
    cargo_build::rerun_if_changed!("locales");
    println!("cargo:rerun-if-env-changed=ZAKA_RUNTIME_SALT");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let target = std::env::var("TARGET").expect("cargo sets TARGET");
    let lib = out.join("libzaka_runtime.a");
    let libs = out.join("native-libs.txt");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    // The root manifest holds the profile the lib is built with.
    let watched =
        ["compiler/zaka_runtime", "compiler/zaka_host/src", "Cargo.lock", "Cargo.toml"];
    for watched in watched
    {
        println!("cargo:rerun-if-changed={}", root.join(watched).display());
    }
    write_header(&root.join("compiler/zaka_runtime"), &out);
    // Beside the lib of the target itself, a Windows host carries the lib of
    // the other ABI, so `--target-abi` picks either, and each host carries
    // the libs of WASI.
    let carried = [
        ("other", other_windows(&target)),
        ("wasip1", Some("wasm32-wasip1")),
        ("wasip2", Some("wasm32-wasip2")),
    ];
    // The explorer builds this crate for the browser, which links no program.
    let blank = || {
        [&lib, &libs].iter().for_each(|f| write(f, b""));
        carried.iter().for_each(|(name, _)| build_other(&root, &out, name, None));
    };
    ret_if!(target.starts_with("wasm32"), blank());
    let printed = build_runtime(&root, &out.join("zaka_runtime-target"), &target, &lib);
    let printed =
        printed.unwrap_or_else(|e| panic!("the runtime lib did not build:\n{e}"));
    write(&libs, native_libs(&printed).as_bytes());
    // The nested builds of the carried libs each take a folder of their own,
    // so they run at once.
    let (root, out) = (&root, &out);
    std::thread::scope(|s| {
        do_for!((name, triple) in carried, _ = s.spawn(move || build_other(root, out, name, triple)));
    });
}

/// Build the runtime lib of another target and write it, with its link list,
/// to `zaka_runtime-<name>` and `native-libs-<name>.txt`; both are blank for
/// no target, or when the host cannot build for it.
///
/// # Arguments
///
/// * `root` - The workspace folder.
/// * `out` - The folder of the build script.
/// * `name` - The name of the lib among those carried.
/// * `triple` - The target triple, or `None` for a blank lib.
fn build_other(root: &Path, out: &Path, name: &str, triple: Option<&str>)
{
    let lib = out.join(format!("zaka_runtime-{name}"));
    let libs = out.join(format!("native-libs-{name}.txt"));
    let blank = || [&lib, &libs].iter().for_each(|f| write(f, b""));
    let Some(triple) = triple
    else
    {
        return blank();
    };
    let dir = out.join(format!("zaka_runtime-{name}-target"));
    let Ok(printed) = build_runtime(root, &dir, triple, &lib)
    else
    {
        println!(
            "cargo:warning=no runtime lib for {triple}: `rustup target add {triple}`"
        );
        return blank();
    };
    write(&libs, native_libs(&printed).as_bytes());
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

/// Write the header of the runtime, `zaka_runtime.h`, from its sources: the
/// declarations under their plain names, and a `#define` before them that
/// maps each plain name to the name it links under.
///
/// # Arguments
///
/// * `runtime` - The folder of the runtime crate.
/// * `out` - `OUT_DIR`, where the header goes.
fn write_header(runtime: &Path, out: &Path)
{
    // cbindgen leaves out an export whose name a macro gives, so it reads a
    // duplicate of the sources with a plain `no_mangle` in its place.
    let plain = out.join("runtime_src");
    let names = plain_sources(&runtime.join("src"), &plain);
    let config = cbindgen::Config::from_file(runtime.join("cbindgen.toml"))
        .expect("cbindgen.toml of the runtime reads");
    let made = cbindgen::Builder::new()
        .with_config(config)
        .with_src(plain.join("lib.rs"))
        .generate()
        .expect("the runtime header generates");
    let mut text = Vec::new();
    made.write(&mut text);
    let text = String::from_utf8(text).expect("cbindgen writes UTF-8");
    let prefix = runtime_prefix(&runtime.join(".."));
    println!("cargo:rustc-env=ZK_NAME_PREFIX={prefix}");
    let defines: String =
        names.iter().map(|n| format!("#define zk_{n} {prefix}{n}\n")).collect();
    let head = "/* Each name of the runtime links with the version and the hash of \
                the runtime\n   that zakac carries, so libs from other builds of \
                zakac link beside it. */\n";
    let header = format!("{head}{defines}\n{}", first_paragraphs(&text));
    write(&out.join("zaka_runtime.h"), header.as_bytes());
}

/// Copy the sources of the runtime with each `export_name` of a macro read as
/// a plain `no_mangle`.
///
/// # Arguments
///
/// * `src` - The source folder of the runtime.
/// * `to` - The folder of the duplicates.
///
/// # Returns
///
/// The name of each export, without its `zk_`.
fn plain_sources(src: &Path, to: &Path) -> Vec<String>
{
    std::fs::create_dir_all(to).expect("OUT_DIR is writable");
    let files = std::fs::read_dir(src).expect("the runtime sources are readable");
    let mut names = Vec::new();
    for path in files.flatten().map(|e| e.path())
    {
        let text = std::fs::read_to_string(&path).expect("a source file is readable");
        let lines = text.lines().map(|l| plain_line(l, &mut names));
        let plain: String = lines.map(|l| format!("{l}\n")).collect();
        write(&to.join(path.file_name().expect("a file name")), plain.as_bytes());
    }
    names
}

/// A line of a source with an `export_name` of a macro read as `no_mangle`.
///
/// # Arguments
///
/// * `line` - The line.
/// * `names` - Where the name of the export goes.
///
/// # Returns
///
/// The line.
fn plain_line(line: &str, names: &mut Vec<String>) -> String
{
    let rest = line.trim_start().strip_prefix(STAMPED);
    let Some(name) = rest.and_then(|r| r.split('"').next())
    else
    {
        return line.to_string();
    };
    names.push(name.to_string());
    line.replace(line.trim_start(), "#[unsafe(no_mangle)]")
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
    // Rust derives the link names of each crate from its metadata, not its
    // sources: two builds of the runtime from other sources would clash in one
    // program, so the prefix of the build, a hash of its sources, joins the
    // metadata of each crate of the lib.
    let prefix = runtime_prefix(&root.join("compiler"));
    cmd.env("RUSTFLAGS", format!("-C metadata={prefix}"));
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
    // `strip` of the host reads no WebAssembly.
    let gnu = !target.ends_with("-msvc") && !target.starts_with("wasm32");
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
