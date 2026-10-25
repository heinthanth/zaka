use std::path::PathBuf;

/// The files the explorer names under hashed names: the Rust name of each and
/// its file, from the package.
const ASSETS: [(&str, &str); 4] = [
    ("LOGO_LIGHT", "../../artwork/logo/bloom-white-light.svg"),
    ("LOGO_DARK", "../../artwork/logo/bloom-white-dark.svg"),
    ("SAUCE_FONT", "assets/fonts/SauceCodeProHTMMsCustom-Regular.ttf"),
    ("SABAE_FONT", "../zaka_website/assets/fonts/MyanmarSabae.ttf"),
];

/// Rebuild when a sheet or a locale table changes; the sheet macros read the
/// sheets, so cargo cannot see them on its own. Then write the files in
/// [`ASSETS`] under hashed names to `OUT_DIR/assets`, and to `target/assets`,
/// where the post-build hook of `Trunk.toml` takes them into the build; their
/// addresses, under `assets/` beside the page, go to `OUT_DIR/assets.rs`.
fn main()
{
    cargo_build::rerun_if_changed!("src");
    cargo_build::rerun_if_changed!("locales");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let manifest =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets it"));
    let dirs = [out.join("assets"), manifest.join("target").join("assets")];
    let names = zaka_web_build::hash_files(&ASSETS, "assets/", &[&dirs[0], &dirs[1]]);
    std::fs::write(out.join("assets.rs"), names).expect("OUT_DIR can be written");
}
