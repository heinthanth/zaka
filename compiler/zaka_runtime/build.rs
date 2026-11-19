//! Write the table of each function the runtime exports, so a JIT finds each
//! one under its C name: the linker keeps a function the table names.

use std::path::Path;

use zakac_rust_sugars::ret_if;

include!("stamp.rs");

fn main()
{
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=../zaka_host/src");
    println!("cargo:rerun-if-changed=stamp.rs");
    println!("cargo:rerun-if-env-changed=ZAKA_RUNTIME_SALT");
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets it");
    let prefix = runtime_prefix(&Path::new(&manifest).join(".."));
    println!("cargo:rustc-env=ZK_NAME_PREFIX={prefix}");
    let files = std::fs::read_dir("src").expect("src is readable").flatten();
    let mut paths: Vec<_> = files.map(|e| e.path()).collect();
    paths.sort();
    let rows: String = paths.iter().filter_map(|p| export_rows(p, &prefix)).collect();
    let out = std::env::var("OUT_DIR").expect("cargo sets OUT_DIR");
    let dest = Path::new(&out).join("exports.rs");
    std::fs::write(dest, format!("&[\n{rows}]")).expect("OUT_DIR is writable");
}

/// The rows of the table for each `pub extern "C" fn` of a source file.
///
/// # Arguments
///
/// * `path` - The source file.
/// * `prefix` - The prefix of the exported names, in place of `zk_`.
///
/// # Returns
///
/// The rows, or `None` for a file that is no module of exports.
fn export_rows(path: &Path, prefix: &str) -> Option<String>
{
    let module = path.file_stem()?.to_str()?;
    ret_if!(module == "lib" || module == "exports", None);
    // WASI builds leave out the modules that call through libffi.
    let wasi = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "wasi");
    ret_if!(wasi && (module == "ffi" || module == "callbacks"), None);
    let text = std::fs::read_to_string(path).expect("a source file is readable");
    let heads = text
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix("pub extern \"C\" fn "));
    let names = heads.map(|rest| {
        rest.chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
    });
    let row = |name: String| {
        let linked = name.strip_prefix("zk_").unwrap_or(&name);
        format!("    (\"{prefix}{linked}\", crate::{module}::{name} as *const u8),\n")
    };
    Some(names.map(row).collect())
}
