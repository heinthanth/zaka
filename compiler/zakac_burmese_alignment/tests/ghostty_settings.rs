use zakac_burmese_alignment::ghostty::settings::DEFAULT_POINTS;
use zakac_burmese_alignment::ghostty::settings::font_px;
use zakac_burmese_alignment::ghostty::settings::read_settings;

#[test]
fn a_flowing_build_lists_its_setting()
{
    let text = "font-family = \"SauceCodePro Nerd Font\"\nfont-family-bold = \
                X\nfont-size = 16\nmyanmar-prompt-width = wrap\n";
    let settings = read_settings(text);
    assert!(settings.flows);
    assert_eq!(settings.points, 16.0);
    assert_eq!(settings.font.as_deref(), Some("SauceCodePro Nerd Font"));
    // no grapheme-width-method: the default, unicode
    assert!(settings.clusters);
}

#[test]
fn the_legacy_width_method_counts_code_points()
{
    assert!(!read_settings("grapheme-width-method = legacy\n").clusters);
    assert!(read_settings("grapheme-width-method = unicode\n").clusters);
}

#[test]
fn other_builds_do_not_flow()
{
    let settings =
        read_settings("font-family = \nfont-size = 13\nwindow-padding-x = 2\n");
    assert!(!settings.flows);
    assert_eq!(settings.font, None);
    assert_eq!(read_settings("").points, DEFAULT_POINTS);
}

#[test]
fn the_font_size_fits_the_cell_width()
{
    // 16 points on a 2x screen: a 32 px font, cells of round(19.2)
    assert_eq!(font_px(19.0, 16.0, 0.6), 32.0);
    // 16 points on a 1x screen
    assert_eq!(font_px(10.0, 16.0, 0.6), 16.0);
    // 11 points at 96 dpi: a 14.67 px font, cells of round(8.8)
    assert!((font_px(9.0, 11.0, 0.6) - 44.0 / 3.0).abs() < 1e-9);
}
