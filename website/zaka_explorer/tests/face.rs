use std::collections::HashMap;

use zaka_explorer::monaco::VS_DIR;

/// The text of a file of the explorer.
fn read(name: &str) -> String
{
    std::fs::read_to_string(format!("{}/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

/// The npm packages `Trunk.toml` pins, under their names.
fn pinned() -> HashMap<String, String>
{
    let trunk: toml::Table = read("Trunk.toml").parse().unwrap();
    let packages = trunk["node_packages"].as_array().unwrap();
    let pin = |p: &toml::Value| {
        (
            p["name"].as_str().unwrap().to_string(),
            p["version"].as_str().unwrap().to_string(),
        )
    };
    packages.iter().map(pin).collect()
}

#[test]
fn monaco_sits_in_the_version_folder_trunk_toml_pins()
{
    let index = read("index.html");
    assert_eq!(VS_DIR, format!("monaco/{}/vs", pinned()["monaco-editor"]));
    assert!(index.contains(&format!("data-target-path=\"{VS_DIR}\"")));
    assert!(index.contains(&format!("src=\"{VS_DIR}/loader.js\"")));
}
