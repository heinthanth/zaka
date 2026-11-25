//! Build script: watches the locales, and writes the table of the stdlib
//! modules from the `[package.modules]` table of `stdlib/zakac.toml`, so the
//! compiler carries each file that table lists.

use std::path::Path;
use std::path::PathBuf;

fn main()
{
    cargo_build::rerun_if_changed!("locales");
    cargo_build::rerun_if_changed!("../../stdlib/zakac.toml");
    let stdlib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let text = std::fs::read_to_string(stdlib.join("zakac.toml"))
        .expect("stdlib/zakac.toml is readable");
    let table: toml::Table = text.parse().expect("stdlib/zakac.toml is TOML");
    let modules = table.get("package").and_then(|p| p.get("modules"));
    let modules = modules.and_then(toml::Value::as_table);
    let modules = modules.expect("stdlib/zakac.toml has a [package.modules] table");
    let rows: String =
        modules.iter().map(|(name, file)| row(&stdlib, name, file)).collect();
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    std::fs::write(out.join("std_modules.rs"), format!("&[\n{rows}]\n"))
        .expect("the table of the stdlib is written");
}

/// One row of the table: the import name, the file in the `stdlib` folder,
/// and the source of that file.
///
/// # Arguments
///
/// * `stdlib` - The `stdlib` folder.
/// * `name` - The name the table gives the module.
/// * `file` - The file it gives the module.
///
/// # Returns
///
/// The row, as Rust.
fn row(stdlib: &Path, name: &str, file: &toml::Value) -> String
{
    let file = file.as_str().expect("each module of the stdlib names a file");
    let source = stdlib.join(file).display().to_string();
    let import = format!("စံကား/{name}");
    format!("    ({import:?}, {file:?}, include_str!({source:?})),\n")
}
