use leptos::prelude::*;
use leptos_scoped_style::ComponentStyle;

use crate::sheet;

pub mod dropdown;
pub mod modal;
pub mod pickers;
pub mod settings;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/controls/mod.sass");

/// Rules of the shared controls: line icons, icon buttons, dropdowns, modals
/// and the settings menu. Mounted whole, so a page rendered on the server
/// arrives with them.
static CONTROLS: ComponentStyle =
    ComponentStyle::global_css("zaka-controls", STYLE_SHEET);

/// Mount the sheet of the shared controls.
///
/// # Returns
///
/// The view; it renders no element of its own.
pub fn controls_sheet() -> impl IntoView { sheet(&CONTROLS) }
