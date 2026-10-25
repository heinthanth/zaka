use leptos_use_color_mode::ColorMode;
use strum::IntoEnumIterator;
use zaka_web_ui_common::look::DEFAULT_FONT_SIZE;
use zaka_web_ui_common::look::FONT_SIZES;
use zaka_web_ui_common::look::font_size_or_default;
use zaka_web_ui_common::look::line_height;
use zaka_web_ui_common::theme::CUSTOM_MODES;
use zaka_web_ui_common::theme::Theme;

#[test]
fn themes_round_trip_through_modes_and_names()
{
    for theme in Theme::iter()
    {
        assert_eq!(Theme::from_mode(&theme.to_mode()), theme, "{theme:?}");
        let name: &'static str = theme.into();
        assert_eq!(name.parse::<Theme>().unwrap(), theme);
    }
    assert_eq!(Theme::Auto.to_mode(), ColorMode::Auto);
    assert_eq!(Theme::from_mode(&ColorMode::Custom("nope".into())), Theme::Auto);
}

#[test]
fn custom_modes_are_the_catppuccin_themes()
{
    let flavours: Vec<String> = Theme::iter()
        .filter(|t| t.group() == "Catppuccin")
        .map(|t| <&'static str>::from(t).to_string())
        .collect();
    assert_eq!(flavours, CUSTOM_MODES);
}

#[test]
fn dark_themes()
{
    let dark: Vec<Theme> = Theme::iter()
        .filter(|t| t != &Theme::Auto && Theme::is_dark(&t.to_mode()))
        .collect();
    assert_eq!(dark, [Theme::Dark, Theme::Frappe, Theme::Macchiato, Theme::Mocha]);
    assert!(!Theme::is_dark(&ColorMode::Light));
}

#[test]
fn font_sizes()
{
    assert!(FONT_SIZES.contains(&DEFAULT_FONT_SIZE));
    assert_eq!(font_size_or_default(18), 18);
    assert_eq!(font_size_or_default(17), DEFAULT_FONT_SIZE);
    assert_eq!(line_height(15), 30);
}

#[test]
fn labels_come_from_the_table()
{
    assert_eq!(Theme::Auto.label(), "Auto (OS)");
    assert_eq!(Theme::Mocha.label(), "Mocha");
}
