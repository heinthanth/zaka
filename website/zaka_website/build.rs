use std::path::PathBuf;

/// The pictures and fonts the pages name: the Rust name of each and its file,
/// from the package.
const ASSETS: [(&str, &str); 8] = [
    ("SABAE_FONT", "../zaka_web_ui_common/assets/fonts/MyanmarSabae.ttf"),
    (
        "SAUCE_FONT",
        "../zaka_web_ui_common/assets/fonts/SauceCodeProHTMMsCustom-Regular.ttf",
    ),
    ("FRANGIPANI_PHOTO", "assets/pictures/frangipani.jpg"),
    ("EXPLORER_PHOTO", "assets/pictures/explorer.jpg"),
    ("BLOSSOMS_PHOTO", "assets/pictures/blossoms.jpg"),
    ("LOGO_LIGHT", "../../artwork/logo/bloom-white-light.svg"),
    ("LOGO_DARK", "../../artwork/logo/bloom-white-dark.svg"),
    ("COVER_CARD", "assets/pictures/cover.png"),
];

/// Rebuild when a sheet or a locale table changes; the sheet and
/// `include_str!` macros read them. Then write the files in [`ASSETS`] to
/// `OUT_DIR/assets` under hashed names, with their addresses under `/assets/`
/// in `OUT_DIR/assets.rs`.
fn main()
{
    cargo_build::rerun_if_changed!("src");
    cargo_build::rerun_if_changed!("locales");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let names = zaka_web_builder::hash_files(&ASSETS, "/assets/", &[&out.join("assets")]);
    std::fs::write(out.join("assets.rs"), names).expect("OUT_DIR can be written");
}
