use zakac_explorer::font::CODE_FONT;
use zakac_explorer::monaco::FONTS;

#[test]
fn the_editor_asks_for_the_code_font_first()
{
    assert!(FONTS.starts_with(&format!("\"{CODE_FONT}\"")));
}
