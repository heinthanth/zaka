use bon::Builder;
use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_use_color_mode::ColorMode;
use leptos_use_css_var::use_css_var;
use leptos_use_storage::UseStorageOptions;
use leptos_use_storage::use_local_storage;
use leptos_use_storage::use_local_storage_with_options;
use zakac_rust_sugars::tern;

use crate::font::face::Face;
use crate::font::load_face;
use crate::theme::Theme;
use crate::theme::ThemeMode;
use crate::theme::theme_mode;

/// Local storage name of the interface font.
pub const UI_FONT_STORE: &str = "zaka-ui-font";

/// Local storage name of the code font.
pub const CODE_FONT_STORE: &str = "zaka-code-font";

/// Local storage name of the PDA10-for-Burmese switch.
pub const PDA_STORE: &str = "zaka-pda-burmese";

/// Local storage name of the italic comments switch.
pub const ITALIC_STORE: &str = "zaka-italic-comments";

/// Local storage name of the code font size.
pub const SIZE_STORE: &str = "zaka-code-size";

/// Code font sizes in the picker, in CSS px.
pub const FONT_SIZES: [u32; 9] = [12, 13, 14, 15, 16, 18, 20, 22, 24];

/// Code font size when none is stored.
pub const DEFAULT_FONT_SIZE: u32 = 15;

/// Size in px the interface font is loaded at.
const UI_SIZE: u32 = 14;

/// How the page looks: the theme, the fonts and the code size, kept in local
/// storage.
#[derive(Clone, Copy, Builder)]
pub struct Look
{
    /// The chosen color mode; `Auto` follows the OS.
    pub mode: Signal<ColorMode>,
    /// Choose the color mode.
    pub set_mode: WriteSignal<ColorMode>,
    /// The color mode on the page; never `Auto`.
    pub resolved: Signal<ColorMode>,
    /// The font of interface text.
    pub ui_font: Signal<Face>,
    /// Choose the font of interface text.
    pub set_ui_font: WriteSignal<Face>,
    /// The font of code text.
    pub code_font: Signal<Face>,
    /// Choose the font of code text.
    pub set_code_font: WriteSignal<Face>,
    /// Draw Burmese text in PDA10 Burmese Mono whatever the code font.
    pub pda_burmese: Signal<bool>,
    /// Switch PDA10 for Burmese text on or off.
    pub set_pda_burmese: WriteSignal<bool>,
    /// Draw comments in italics.
    pub italic_comments: Signal<bool>,
    /// Switch italic comments on or off.
    pub set_italic_comments: WriteSignal<bool>,
    /// The size of code text in px.
    pub font_size: Signal<u32>,
    /// Choose the size of code text.
    pub set_font_size: WriteSignal<u32>,
}

impl Look
{
    /// Whether a dark theme is on.
    ///
    /// # Returns
    ///
    /// `true` for Dark and the dark Catppuccin flavours.
    pub fn is_dark(&self) -> bool { Theme::is_dark(&self.resolved.get()) }

    /// The color theme alone.
    ///
    /// # Returns
    ///
    /// The theme.
    pub fn theme(&self) -> ThemeMode
    {
        ThemeMode::builder()
            .mode(self.mode)
            .set_mode(self.set_mode)
            .resolved(self.resolved)
            .build()
    }

    /// The CSS font stack of code text.
    ///
    /// # Returns
    ///
    /// The stack.
    pub fn code_stack(&self) -> String
    {
        self.code_font.get().code_stack(self.pda_burmese.get())
    }

    /// The CSS font stack and size of code text, without tracking.
    ///
    /// # Returns
    ///
    /// The stack and the size in px.
    pub fn font_untracked(&self) -> (String, u32)
    {
        let stack =
            self.code_font.get_untracked().code_stack(self.pda_burmese.get_untracked());
        (stack, self.font_size.get_untracked())
    }
}

/// Read the look from local storage, provide it as context and keep the CSS
/// variables of the page and the loaded fonts in step with it.
///
/// # Returns
///
/// The look.
pub fn provide_look() -> Look
{
    let ThemeMode { mode, set_mode, resolved } = theme_mode();
    let (stored_ui, set_ui_font, _) =
        use_local_storage_with_options::<Face, FromToStringCodec>(
            UI_FONT_STORE,
            UseStorageOptions::default().initial_value(Face::default_ui()),
        );
    let ui_font = Signal::derive(move || Face::ui_or_default(stored_ui.get()));
    let (stored_code, set_code_font, _) =
        use_local_storage::<Face, FromToStringCodec>(CODE_FONT_STORE);
    let code_font = Signal::derive(move || Face::code_or_default(stored_code.get()));
    let (pda_burmese, set_pda_burmese, _) =
        use_local_storage::<bool, FromToStringCodec>(PDA_STORE);
    let (italic_comments, set_italic_comments, _) =
        use_local_storage::<bool, FromToStringCodec>(ITALIC_STORE);
    let (stored_size, set_font_size, _) =
        use_local_storage::<u32, FromToStringCodec>(SIZE_STORE);
    let font_size = Signal::derive(move || font_size_or_default(stored_size.get()));
    let look = Look::builder()
        .mode(mode)
        .set_mode(set_mode)
        .resolved(resolved)
        .ui_font(ui_font)
        .set_ui_font(set_ui_font)
        .code_font(code_font)
        .set_code_font(set_code_font)
        .pda_burmese(pda_burmese)
        .set_pda_burmese(set_pda_burmese)
        .italic_comments(italic_comments)
        .set_italic_comments(set_italic_comments)
        .font_size(font_size)
        .set_font_size(set_font_size)
        .build();
    provide_context(look);
    wire_fonts(look);
    look
}

/// The look of the page.
///
/// # Returns
///
/// The look from context.
pub fn use_look() -> Look { expect_context::<Look>() }

/// Keep the CSS variables of interface and code text in step with the chosen
/// fonts and size, and load each font when it is picked.
///
/// # Arguments
///
/// * `look` - The look.
fn wire_fonts(look: Look)
{
    let (_, set_stack) = use_css_var("--mono-font");
    let (_, set_size) = use_css_var("--code-size");
    let (_, set_ui_stack) = use_css_var("--ui-font");
    let (_, set_comment) = use_css_var("--comment-style");
    Effect::new(move |_| {
        set_comment.set(tern!(look.italic_comments.get(), "italic", "normal").into());
    });
    Effect::new(move |_| {
        let face = look.ui_font.get();
        set_ui_stack.set(face.ui_stack());
        leptos::task::spawn_local(load_face(face, UI_SIZE));
    });
    Effect::new(move |_| {
        let (face, size) = (look.code_font.get(), look.font_size.get());
        set_stack.set(look.code_stack());
        set_size.set(format!("{size}px"));
        leptos::task::spawn_local(load_face(face, size));
    });
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

/// Line height of code text for a font size; Burmese marks stack high.
///
/// # Arguments
///
/// * `size` - The font size in px.
///
/// # Returns
///
/// The line height in px.
pub fn line_height(size: u32) -> u32 { size * 2 }
