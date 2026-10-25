use std::borrow::Cow;

use bon::Builder;
use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_scoped_style::scss_file;
use leptos_use_color_mode::ColorMode;
use leptos_use_color_mode::UseColorModeOptions;
use leptos_use_color_mode::UseColorModeReturn;
use leptos_use_color_mode::use_color_mode_with_options;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoStaticStr;
use zakac_i18n::t;

/// Group of the Zaka themes in the picker, drawn from the logo.
pub const ZAKA: &str = "Zaka";

/// Group of the Catppuccin themes in the picker.
pub const CATPPUCCIN: &str = "Catppuccin";

/// Color mode names of the Catppuccin themes.
pub const CUSTOM_MODES: [&str; 4] = ["latte", "frappe", "macchiato", "mocha"];

/// Local storage name of the color mode; the website and the explorer share
/// it, so a pick in one shows in the other.
pub const THEME_STORE: &str = "zaka-theme";

/// The color theme of the page, kept in local storage.
#[derive(Clone, Copy, Builder)]
pub struct ThemeMode
{
    /// The chosen color mode; `Auto` follows the OS.
    pub mode: Signal<ColorMode>,
    /// Choose the color mode.
    pub set_mode: WriteSignal<ColorMode>,
    /// The color mode on the page; never `Auto`.
    pub resolved: Signal<ColorMode>,
}

impl ThemeMode
{
    /// Whether a dark theme is on.
    ///
    /// # Returns
    ///
    /// `true` for Dark and the dark Catppuccin flavours.
    pub fn is_dark(&self) -> bool { Theme::is_dark(&self.resolved.get()) }
}

/// Read the color theme from local storage and keep `data-theme` on the
/// root element in step with it.
///
/// # Returns
///
/// The theme.
pub fn theme_mode() -> ThemeMode
{
    let options = UseColorModeOptions::default()
        .attribute("data-theme")
        .storage_key(THEME_STORE)
        .custom_modes(CUSTOM_MODES.map(String::from).to_vec())
        .emit_auto(true)
        // A page built ahead of time has no request to read the OS scheme
        // from; the page follows the OS through CSS until the browser takes over.
        .ssr_color_header_getter(|| None);
    let UseColorModeReturn { mode, set_mode, state: resolved, .. } =
        use_color_mode_with_options(options);
    ThemeMode::builder().mode(mode).set_mode(set_mode).resolved(resolved).build()
}

/// Document-wide rules: color tokens for each theme, the font variables, the
/// icon size, the scroll bars and the entrance frames.
static TOKENS: ComponentStyle =
    ComponentStyle::global_css("zaka-tokens", scss_file!("src/theme/tokens.sass"));

/// A color theme; its static name is the color mode name it stores.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    EnumIter,
    IntoStaticStr,
    EnumString
)]
#[strum(serialize_all = "lowercase")]
pub enum Theme
{
    /// Light or dark, as the OS prefers.
    #[default]
    Auto,
    /// Zaka Light: the logo on its slate tile.
    Light,
    /// Zaka Dark: the logo on its ink tile.
    Dark,
    /// Catppuccin Latte, the light flavour.
    Latte,
    /// Catppuccin Frappé.
    Frappe,
    /// Catppuccin Macchiato.
    Macchiato,
    /// Catppuccin Mocha.
    Mocha,
}

impl Theme
{
    /// Name in the picker.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn label(self) -> Cow<'static, str>
    {
        match self
        {
            Theme::Auto => t!("web_ui.theme.auto"),
            Theme::Light => "Zaka Light".into(),
            Theme::Dark => "Zaka Dark".into(),
            Theme::Latte => "Latte".into(),
            Theme::Frappe => "Frappé".into(),
            Theme::Macchiato => "Macchiato".into(),
            Theme::Mocha => "Mocha".into(),
        }
    }

    /// Group in the picker.
    ///
    /// # Returns
    ///
    /// [`ZAKA`] or [`CATPPUCCIN`].
    pub fn group(self) -> &'static str
    {
        match self
        {
            Theme::Auto | Theme::Light | Theme::Dark => ZAKA,
            _ => CATPPUCCIN,
        }
    }

    /// The color mode that shows this theme.
    ///
    /// # Returns
    ///
    /// The mode; a Catppuccin theme is a custom mode.
    pub fn to_mode(self) -> ColorMode
    {
        match self
        {
            Theme::Auto => ColorMode::Auto,
            Theme::Light => ColorMode::Light,
            Theme::Dark => ColorMode::Dark,
            flavour => ColorMode::Custom(<&'static str>::from(flavour).to_string()),
        }
    }

    /// The theme of a color mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - The mode.
    ///
    /// # Returns
    ///
    /// The theme; an unknown custom mode gives [`Theme::Auto`].
    pub fn from_mode(mode: &ColorMode) -> Theme
    {
        match mode
        {
            ColorMode::Auto => Theme::Auto,
            ColorMode::Light => Theme::Light,
            ColorMode::Dark => Theme::Dark,
            ColorMode::Custom(name) => name.parse().unwrap_or_default(),
        }
    }

    /// Whether the resolved color mode is a dark one.
    ///
    /// # Arguments
    ///
    /// * `resolved` - The mode on the page; never `Auto`.
    ///
    /// # Returns
    ///
    /// `true` for Dark, Frappé, Macchiato and Mocha.
    pub fn is_dark(resolved: &ColorMode) -> bool
    {
        let theme = Theme::from_mode(resolved);
        matches!(theme, Theme::Dark | Theme::Frappe | Theme::Macchiato | Theme::Mocha)
    }
}

/// Mount the shared token sheet.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn token_sheet() -> impl IntoView { crate::sheet(&TOKENS) }
