use std::fs;
use std::path::Path;

use serde_json::Value;
use serde_json::json;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::extract;

/// The glue wasm-pack writes for the crate: the file VS Code loads.
const MAIN: &str = "./pkg/zaka_vscode.js";

/// The logos of the repo and the file icons of the language, copied from them.
const ICONS: [(&str, &str); 2] = [
    ("../../artwork/logo/bloom-white-light.svg", "icons/zaka-light.svg"),
    ("../../artwork/logo/bloom-white-dark.svg", "icons/zaka-dark.svg"),
];

/// The TOML sources and the JSON files VS Code reads, written from them.
const FILES: [(&str, &str); 2] = [
    ("language-configuration.toml", ".cache/language-configuration.json"),
    ("grammars/zaka.tmLanguage.toml", ".cache/zaka.tmLanguage.json"),
];

/// Write the JSON files of the extension from their TOML sources: the
/// manifest gets the name, version, description, license and repo link of the
/// crate, and its main file is the wasm-pack glue.
fn main()
{
    cargo_build::rerun_if_changed(["extension.toml"]);
    let mut manifest = read("extension.toml");
    let crate_fields = json!({
        "name": "zaka",
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "license": env!("CARGO_PKG_LICENSE"),
        "repository": env!("CARGO_PKG_REPOSITORY"),
        "main": MAIN,
    });
    extract!(Value::Object(fields) = crate_fields);
    manifest.as_object_mut().expect("extension.toml is a table").extend(fields);
    write("package.json", &manifest);
    // vsce packs the license of the folder the manifest sits in.
    cargo_build::rerun_if_changed(["../../LICENSE"]);
    fs::copy("../../LICENSE", "LICENSE").expect("the repo license is readable");
    fs::create_dir_all("icons").expect("the crate folder is writable");
    do_for!((logo, icon) in ICONS, {
        cargo_build::rerun_if_changed([logo]);
        fs::copy(logo, icon).expect("the repo logo is readable");
    });
    fs::create_dir_all(".cache").expect("the crate folder is writable");
    do_for!((source, json) in FILES, {
        cargo_build::rerun_if_changed([source]);
        write(json, &read(source));
    });
}

/// Read a TOML file of the crate as JSON.
///
/// # Arguments
///
/// * `source` - Its path in the crate.
///
/// # Returns
///
/// The JSON value.
fn read(source: &str) -> Value
{
    let text = fs::read_to_string(source).expect("the TOML source is readable");
    toml::from_str(&text).expect("the TOML source is valid")
}

/// Write a JSON file of the crate, when its content changed.
///
/// # Arguments
///
/// * `path` - Its path in the crate.
/// * `value` - The content.
fn write(path: &str, value: &Value)
{
    let text =
        serde_json::to_string_pretty(value).expect("TOML values print as JSON") + "\n";
    let same = fs::read_to_string(path).is_ok_and(|old| old == text);
    do_if!(
        !same,
        fs::write(Path::new(path), text).expect("the crate folder is writable")
    );
}
