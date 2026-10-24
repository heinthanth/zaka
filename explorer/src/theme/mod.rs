use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;
use leptos_scoped_style::GlobalStyles;
use leptos_scoped_style::component::GlobalStylesProps;
use leptos_scoped_style::scss_file;

/// Document-wide rules: color tokens for light and dark, the base font and
/// the box of the scope hosts.
static THEME: ComponentStyle =
    ComponentStyle::global_css("zaka-theme", scss_file!("src/theme/mod.sass"));

/// Mount the global theme sheet.
///
/// # Returns
///
/// The view; it renders no element.
pub fn theme_sheet() -> impl IntoView
{
    GlobalStyles(GlobalStylesProps::builder().style(&THEME).build())
}
