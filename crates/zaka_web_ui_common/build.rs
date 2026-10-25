include!("src/font/mask.rs");

use std::str::FromStr;

/// The code font, sealed with age; the plain file is kept out of the repo.
const SEALED: &str = "assets/fonts/PDA10BurmeseMono-Regular.ttf.age";

/// Env var with the age secret that opens [`SEALED`].
const SECRET_VAR: &str = "ZAKA_PDA_FONT_SECRET";

/// Cfg set when the code font is part of the build.
const FONT_CFG: &str = "pda10";

/// Open the code font with the secret from the env and mask it into
/// `$OUT_DIR/code_font.bin`, so the wasm holds no plain font file. Without
/// the secret the file is blank and the pages build without the font.
fn main()
{
    cargo_build::rerun_if_changed!("{SEALED}");
    cargo_build::rerun_if_changed!("src/font/mask.rs");
    cargo_build::rerun_if_env_changed!("{SECRET_VAR}");
    // `scss_file!` reads the Sass sheets inside a macro, so cargo cannot see
    // them; a rerun here makes an edit to a sheet rebuild the crate.
    cargo_build::rerun_if_changed!("src");
    cargo_build::rerun_if_changed!("locales");
    cargo_build::rustc_check_cfg!(FONT_CFG);
    let secret = std::env::var(SECRET_VAR).ok().filter(|s| !s.trim().is_empty());
    let Some(secret) = secret
    else
    {
        cargo_build::warning!(
            "{SECRET_VAR} is not set; building without PDA10 Burmese Mono"
        );
        return write_font(Vec::new());
    };
    let mut raw = open_font(&secret);
    xor_mask(&mut raw, SEED);
    cargo_build::rustc_cfg!(FONT_CFG);
    write_font(raw);
}

/// Open the sealed code font.
///
/// # Arguments
///
/// * `secret` - The age secret, `AGE-SECRET-KEY-1...`.
///
/// # Returns
///
/// The font file; a wrong secret or a missing file stops the build, since
/// a set secret means the font is wanted.
fn open_font(secret: &str) -> Vec<u8>
{
    let opener = age::x25519::Identity::from_str(secret.trim())
        .unwrap_or_else(|err| panic!("{SECRET_VAR} is not an age secret: {err}"));
    let sealed = std::fs::read(SEALED)
        .unwrap_or_else(|err| panic!("cannot read {SEALED}: {err}"));
    age::decrypt(&opener, &sealed)
        .unwrap_or_else(|err| panic!("{SECRET_VAR} does not open {SEALED}: {err}"))
}

/// Write the masked code font to `$OUT_DIR/code_font.bin`.
///
/// # Arguments
///
/// * `raw` - The masked font; blank for a build without it.
fn write_font(raw: Vec<u8>)
{
    let out_dir = std::env::var("OUT_DIR").expect("cargo sets OUT_DIR");
    let out = std::path::Path::new(&out_dir).join("code_font.bin");
    std::fs::write(out, raw).expect("cannot write the masked code font");
}
