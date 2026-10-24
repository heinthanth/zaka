use std::borrow::Cow;

use leptos_use_color_mode::ColorMode;
use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoStaticStr;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

/// Group of the Zaka themes in the picker, drawn from the logo.
pub const ZAKA: &str = "Zaka";

/// Group of the Catppuccin themes in the picker.
pub const CATPPUCCIN: &str = "Catppuccin";

/// Color mode names of the Catppuccin themes.
pub const CUSTOM_MODES: [&str; 4] = ["latte", "frappe", "macchiato", "mocha"];

/// Code font sizes in the picker, in CSS px.
pub const FONT_SIZES: [u32; 9] = [12, 13, 14, 15, 16, 18, 20, 22, 24];

/// Code font size when none is stored.
pub const DEFAULT_FONT_SIZE: u32 = 15;

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
            Theme::Auto => t!("explorer.theme.auto"),
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

/// How the AST view shows the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum AstMode
{
    /// The tree, with nodes that open and shut.
    #[default]
    Tree,
    /// The S-expression form, indented as Lisp code is.
    Sexpr,
    /// The tree drawn as boxes and lines.
    Graph,
}

impl AstMode
{
    /// Label of the switch.
    ///
    /// # Returns
    ///
    /// The label.
    pub fn label(self) -> Cow<'static, str>
    {
        match self
        {
            AstMode::Tree => t!("explorer.ast.tree"),
            AstMode::Sexpr => t!("explorer.ast.sexpr"),
            AstMode::Graph => t!("explorer.ast.graph"),
        }
    }
}

/// Which IR the lowering view shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum LowerMode
{
    /// The IR as lowered, before folding.
    Plain,
    /// The IR after folding.
    #[default]
    Folded,
}

impl LowerMode
{
    /// Label of the switch.
    ///
    /// # Returns
    ///
    /// The label.
    pub fn label(self) -> Cow<'static, str>
    {
        match self
        {
            LowerMode::Plain => t!("explorer.lower.plain"),
            LowerMode::Folded => t!("explorer.lower.folded"),
        }
    }
}

/// Clamp a stored font size to one the picker offers.
///
/// # Arguments
///
/// * `size` - The stored size.
///
/// # Returns
///
/// The size, or [`DEFAULT_FONT_SIZE`] when the picker lacks it.
pub fn font_size_or_default(size: u32) -> u32
{
    tern!(FONT_SIZES.contains(&size), size, DEFAULT_FONT_SIZE)
}

/// Line height of the editor for a font size; Burmese marks stack high.
///
/// # Arguments
///
/// * `size` - The font size in px.
///
/// # Returns
///
/// The line height in px.
pub fn line_height(size: u32) -> u32 { size * 2 }
