pub mod controls;
pub mod credits;
pub mod font;
pub mod icon;
pub mod lang;
pub mod look;
pub mod paint;
pub mod site;
pub mod theme;

use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_scoped_style::GlobalStyles;
use leptos_scoped_style::component::GlobalStylesProps;

zakac_i18n::init!();

/// Mount a sheet in the head of the page, once however often it is asked for.
///
/// # Arguments
///
/// * `sheet` - The sheet.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn sheet(sheet: &'static ComponentStyle) -> impl IntoView
{
    GlobalStyles(GlobalStylesProps::builder().style(sheet).build())
}

/// Mount the shared sheets: the tokens of each theme and the controls.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn shared_sheets() -> impl IntoView
{
    (theme::token_sheet(), controls::controls_sheet())
}
