use zaka_web_ui_common::font::HAS_CODE_FONT;
use zaka_web_ui_common::font::SCRAMBLED;
use zaka_web_ui_common::font::font_raw;
use zaka_web_ui_common::font::mask::SEED;
use zaka_web_ui_common::font::mask::xor_mask;
use zakac_rust_sugars::ret_if;

/// The TrueType file signature.
const TRUETYPE: [u8; 4] = [0, 1, 0, 0];

/// The sealed font in the repo.
const SEALED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/PDA10BurmeseMono-Regular.ttf.age"
);

/// Tags that would reveal a font file.
const FONT_TAGS: [&[u8]; 4] = [b"glyf", b"cmap", b"name", b"PDA10"];

/// Whether `raw` holds `tag`.
fn holds(raw: &[u8], tag: &[u8]) -> bool { raw.windows(tag.len()).any(|w| w == tag) }

#[test]
fn the_masked_font_and_the_build_agree()
{
    assert_eq!(SCRAMBLED.is_empty(), !HAS_CODE_FONT);
}

#[test]
fn the_masked_font_hides_its_file_format()
{
    ret_if!(!HAS_CODE_FONT);
    assert_ne!(SCRAMBLED[..4], TRUETYPE);
    for tag in FONT_TAGS
    {
        assert!(!holds(SCRAMBLED, tag), "{}", String::from_utf8_lossy(tag));
    }
}

#[test]
fn unmasking_gives_a_font_file_back()
{
    ret_if!(!HAS_CODE_FONT);
    let raw = font_raw();
    assert_eq!(raw[..4], TRUETYPE);
    assert!(holds(&raw, b"glyf"));
}

#[test]
fn the_font_in_the_repo_is_sealed()
{
    // A build with the font opened the file at build time; without the font,
    // the file can be missing.
    let sealed = std::fs::read(SEALED).ok();
    assert!(sealed.is_some() || !HAS_CODE_FONT, "the build opened {SEALED}");
    let Some(sealed) = sealed
    else
    {
        return;
    };
    assert!(sealed.starts_with(b"age-encryption.org/v1\n"));
    for tag in FONT_TAGS
    {
        assert!(!holds(&sealed, tag), "{}", String::from_utf8_lossy(tag));
    }
}

#[test]
fn the_mask_is_its_own_inverse()
{
    let text = b"abc \xe1\x80\x80 0123456789".to_vec();
    let mut masked = text.clone();
    xor_mask(&mut masked, SEED);
    assert_ne!(masked, text);
    xor_mask(&mut masked, SEED);
    assert_eq!(masked, text);
}
