use std::fmt::Write;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

/// The sample folder, relative to this package.
const SAMPLES_DIR: &str = "../../samples";

/// The files the explorer names under hashed names: the Rust name of each and
/// its file, from the package.
const ASSETS: [(&str, &str); 6] = [
    ("LOGO_LIGHT", "../../artwork/logo/bloom-white-light.svg"),
    ("LOGO_DARK", "../../artwork/logo/bloom-white-dark.svg"),
    (
        "SAUCE_FONT",
        "../zaka_web_ui_common/assets/fonts/SauceCodeProHTMMsCustom-Regular.ttf",
    ),
    ("SABAE_FONT", "../zaka_web_ui_common/assets/fonts/MyanmarSabae.ttf"),
    ("COVER_CARD", "../zaka_website/assets/pictures/cover.png"),
    ("BLOSSOMS_PHOTO", "../zaka_website/assets/pictures/blossoms.jpg"),
];

/// Rebuild when a sheet or a locale table changes; the sheet macros read the
/// sheets, so cargo cannot see them on its own. Then write the files in
/// [`ASSETS`] under hashed names to `OUT_DIR/assets`, and to `target/assets`,
/// where the post-build hook of `Trunk.toml` takes them into the build; their
/// addresses, under `assets/` beside the page, go to `OUT_DIR/assets.rs`.
/// Generate the sample picker from the sample folder at the repo root.
fn main()
{
    cargo_build::rerun_if_changed!("src");
    cargo_build::rerun_if_changed!("locales");
    cargo_build::rerun_if_changed([SAMPLES_DIR]);
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let manifest =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets it"));
    let dirs = [out.join("assets"), manifest.join("target").join("assets")];
    let names = zaka_web_builder::hash_files(&ASSETS, "assets/", &[&dirs[0], &dirs[1]]);
    fs::write(out.join("assets.rs"), names).expect("OUT_DIR can be written");
    let root = manifest.join(SAMPLES_DIR);
    let samples =
        sample_table(&root).unwrap_or_else(|e| panic!("{}: {e}", root.display()));
    fs::write(out.join("samples.rs"), samples).expect("OUT_DIR can be written");
}

/// Generate picker groups and embed the source of each standalone sample.
///
/// # Arguments
/// * `root` - The sample folder at the repo root.
/// # Returns
/// The Rust definitions of the groups and samples, in picker order.
fn sample_table(root: &Path) -> io::Result<String>
{
    let mut groups = fs::read_dir(root)?
        .filter_map(|item| item.and_then(read_group).transpose())
        .collect::<io::Result<Vec<_>>>()?;
    assert!(!groups.is_empty(), "no explorer samples in {}", root.display());
    // Put the basics group first and the error examples last.
    let order = |group: &str| match group
    {
        "basics" => 0,
        "mistakes" => 2,
        "diagnostics" => 3,
        _ => 1,
    };
    groups.sort_by(|(a, _), (b, _)| (order(a), a).cmp(&(order(b), b)));

    let mut rust = String::from(
        "/// The groups, in picker order.\npub const GROUPS: &[&str] = &[\n",
    );
    do_for!((group, _) in &groups, writeln!(rust, "    {group:?},").unwrap());
    rust.push_str(
        "];\n\n/// The samples, in picker order.\npub const SAMPLES: &[Sample] = &[\n",
    );
    let samples = groups
        .iter()
        .flat_map(|(group, names)| names.iter().map(move |name| (group, name)));
    for (group, name) in samples
    {
        #[rustfmt::skip]
        writeln!(
            rust,
            "    Sample {{ group: {group:?}, name: {name:?}, text: \
             include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/\", \
             {SAMPLES_DIR:?}, \"/\", {group:?}, \"/\", {name:?}, \".zaka\")) }},").unwrap();
    }
    rust.push_str("];\n");
    Ok(rust)
}

/// Read the names of the sample programs in one group.
///
/// # Arguments
/// * `folder` - The group folder.
/// * `group` - The group name.
/// # Returns
/// The names of the standalone samples, sorted.
fn sample_names(folder: &Path, group: &str) -> io::Result<Vec<String>>
{
    let mut names = Vec::new();
    for file in fs::read_dir(folder)?
    {
        let f = file?;
        let path = f.path();
        let is_sample =
            f.file_type()?.is_file() && path.extension().is_some_and(|e| e == "zaka");
        do_if!(!is_sample, continue);
        let name =
            path.file_stem().and_then(|s| s.to_str()).expect("sample name is UTF-8");
        // This example imports generics/lib/pair.zaka.
        do_if!(group == "generics" && name == "hints", continue);
        names.push(name.to_owned());
    }
    names.sort();
    Ok(names)
}

/// Read a group folder with at least one sample the explorer can open.
///
/// # Arguments
/// * `item` - An item in the sample folder.
/// # Returns
/// The group name and its sorted sample names, if present.
fn read_group(item: fs::DirEntry) -> io::Result<Option<(String, Vec<String>)>>
{
    ret_if!(!item.file_type()?.is_dir(), Ok(None));
    let g = item.file_name().into_string().expect("sample group is UTF-8");
    // The browser cannot call native C functions or load local modules.
    ret_if!(g.starts_with('.') || matches!(g.as_str(), "ffi" | "modules"), Ok(None));
    let names = sample_names(&item.path(), &g)?;
    Ok((!names.is_empty()).then_some((g, names)))
}
