use std::collections::HashMap;

use strum::IntoEnumIterator;
use zaka_web_ui_common::font::face::Face;
use zaka_web_ui_common::font::face::Source;
use zakac_explorer::monaco::VS_DIR;

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
fn web_fonts_sit_in_the_version_folders_trunk_toml_pins()
{
    let index = read("index.html");
    let pins = pinned();
    let folder = |dir: &str| format!("data-target-path=\"fonts/{dir}\"");
    for face in Face::iter()
    {
        match face.source()
        {
            Source::Sheets { dir, version, .. } =>
            {
                assert_eq!(pins[&format!("@fontsource/{dir}")], version, "{face:?}");
                assert!(
                    index.contains(&folder(&format!("{dir}/{version}"))),
                    "{face:?}"
                );
            }
            Source::Files(files) => files.iter().for_each(|(_, path)| {
                let (dir, _) = path.rsplit_once('/').unwrap();
                let (name, version) = dir.split_once('/').unwrap();
                assert_eq!(pins[&format!("@fontsource/{name}")], version, "{face:?}");
                assert!(index.contains(&folder(dir)), "{face:?}");
            }),
            Source::Embedded | Source::Native | Source::Shipped => (),
        }
    }
}

#[test]
fn monaco_sits_in_the_version_folder_trunk_toml_pins()
{
    let index = read("index.html");
    assert_eq!(VS_DIR, format!("monaco/{}/vs", pinned()["monaco-editor"]));
    assert!(index.contains(&format!("data-target-path=\"{VS_DIR}\"")));
    assert!(index.contains(&format!("src=\"{VS_DIR}/loader.js\"")));
}
