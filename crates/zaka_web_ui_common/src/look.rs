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
use crate::font::face::Fallback;
use crate::font::face::Pair;
use crate::font::load_pair;
use crate::theme::Theme;
use crate::theme::ThemeMode;
use crate::theme::theme_mode;

/// Local storage name of the Burmese font of interface text.
pub const UI_BURMESE_STORE: &str = "zaka-ui-burmese";

/// Local storage name of the fallback font of interface text.
pub const UI_FALLBACK_STORE: &str = "zaka-ui-fallback";

/// Local storage name of the Burmese font of code text.
pub const CODE_BURMESE_STORE: &str = "zaka-code-burmese";

/// Local storage name of the fallback font of code text.
pub const CODE_FALLBACK_STORE: &str = "zaka-code-fallback";

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
    /// The font of Burmese interface text.
    pub ui_burmese: Signal<Face>,
    /// Choose the font of Burmese interface text.
    pub set_ui_burmese: WriteSignal<Face>,
    /// The font of the rest of the interface text.
    pub ui_fallback: Signal<Fallback>,
    /// Choose the font of the rest of the interface text.
    pub set_ui_fallback: WriteSignal<Fallback>,
    /// The font of Burmese code text.
    pub code_burmese: Signal<Face>,
    /// Choose the font of Burmese code text.
    pub set_code_burmese: WriteSignal<Face>,
    /// The font of the rest of the code text.
    pub code_fallback: Signal<Fallback>,
    /// Choose the font of the rest of the code text.
    pub set_code_fallback: WriteSignal<Fallback>,
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

    /// The fonts of interface text.
    ///
    /// # Returns
    ///
    /// The pair.
    pub fn ui_pair(&self) -> Pair
    {
        Pair::builder()
            .burmese(self.ui_burmese.get())
            .fallback(self.ui_fallback.get())
            .build()
    }

    /// The fonts of code text.
    ///
    /// # Returns
    ///
    /// The pair.
    pub fn code_pair(&self) -> Pair
    {
        Pair::builder()
            .burmese(self.code_burmese.get())
            .fallback(self.code_fallback.get())
            .build()
    }

    /// The CSS font stack of code text.
    ///
    /// # Returns
    ///
    /// The stack.
    pub fn code_stack(&self) -> String { self.code_pair().code_stack() }

    /// The CSS font stack and size of code text, without tracking.
    ///
    /// # Returns
    ///
    /// The stack and the size in px.
    pub fn font_untracked(&self) -> (String, u32)
    {
        let stack = untrack(|| self.code_stack());
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
    let (ui, code) = (Pair::default_ui(), Pair::default_code());
    let (ui_burmese, set_ui_burmese) = stored_burmese(UI_BURMESE_STORE, ui.burmese);
    let (ui_fallback, set_ui_fallback) =
        stored_fallback(UI_FALLBACK_STORE, ui.fallback, Face::ui_faces);
    let (code_burmese, set_code_burmese) =
        stored_burmese(CODE_BURMESE_STORE, code.burmese);
    let (code_fallback, set_code_fallback) =
        stored_fallback(CODE_FALLBACK_STORE, code.fallback, Face::code_faces);
    let (italic_comments, set_italic_comments, _) =
        use_local_storage::<bool, FromToStringCodec>(ITALIC_STORE);
    let (stored_size, set_font_size, _) =
        use_local_storage::<u32, FromToStringCodec>(SIZE_STORE);
    let font_size = Signal::derive(move || font_size_or_default(stored_size.get()));
    let look = Look::builder()
        .mode(mode)
        .set_mode(set_mode)
        .resolved(resolved)
        .ui_burmese(ui_burmese)
        .set_ui_burmese(set_ui_burmese)
        .ui_fallback(ui_fallback)
        .set_ui_fallback(set_ui_fallback)
        .code_burmese(code_burmese)
        .set_code_burmese(set_code_burmese)
        .code_fallback(code_fallback)
        .set_code_fallback(set_code_fallback)
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

/// A Burmese font kept in local storage; a stored face the picker lacks
/// reads as the default.
///
/// # Arguments
///
/// * `store` - The local storage name.
/// * `default` - The face before one is kept.
///
/// # Returns
///
/// The face and its setter.
fn stored_burmese(
    store: &'static str, default: Face,
) -> (Signal<Face>, WriteSignal<Face>)
{
    let options = UseStorageOptions::default().initial_value(default);
    let (stored, set, _) =
        use_local_storage_with_options::<Face, FromToStringCodec>(store, options);
    (Signal::derive(move || Face::burmese_or(stored.get(), default)), set)
}

/// A fallback font kept in local storage; a stored face the picker lacks
/// reads as the default.
///
/// # Arguments
///
/// * `store` - The local storage name.
/// * `default` - The fallback before one is kept.
/// * `faces` - The faces of the picker.
///
/// # Returns
///
/// The fallback and its setter.
fn stored_fallback<I: Iterator<Item = Face> + 'static>(
    store: &'static str, default: Fallback, faces: fn() -> I,
) -> (Signal<Fallback>, WriteSignal<Fallback>)
{
    let options = UseStorageOptions::default().initial_value(default);
    let (stored, set, _) =
        use_local_storage_with_options::<Fallback, FromToStringCodec>(store, options);
    (Signal::derive(move || stored.get().or(faces(), default)), set)
}

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
        let pair = look.ui_pair();
        set_ui_stack.set(pair.ui_stack());
        leptos::task::spawn_local(load_pair(pair, UI_SIZE));
    });
    Effect::new(move |_| {
        let (pair, size) = (look.code_pair(), look.font_size.get());
        set_stack.set(pair.code_stack());
        set_size.set(format!("{size}px"));
        leptos::task::spawn_local(load_pair(pair, size));
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
