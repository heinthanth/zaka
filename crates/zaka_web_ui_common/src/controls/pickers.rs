use std::iter::once;

use leptos::prelude::*;
use strum::IntoEnumIterator;
use zakac_i18n::SUPPORTED_LOCALES;
use zakac_i18n::t;
use zakac_rust_sugars::ret_if;

use crate::controls::dropdown::Choice;
use crate::controls::dropdown::Dropdown;
use crate::controls::dropdown::dropdown;
use crate::controls::settings::picker;
use crate::icon::globe;
use crate::icon::swatches;
use crate::lang::AUTO_LANG;
use crate::lang::lang_name;
use crate::lang::store_lang;
use crate::lang::stored_lang;
use crate::theme::Theme;
use crate::theme::ThemeMode;

/// Picker of the color theme, grouped into plain and Catppuccin themes.
///
/// # Arguments
///
/// * `theme` - The theme of the page.
/// * `autofocus` - Whether it takes the focus when shown, as in a dialog.
///
/// # Returns
///
/// The view.
pub fn theme_picker(theme: ThemeMode, autofocus: bool) -> impl IntoView
{
    let mut spec = theme_spec(theme);
    spec.autofocus = autofocus;
    dropdown(spec)
}

/// The theme picker drawn as a book of swatches, for a bar.
///
/// # Arguments
///
/// * `theme` - The theme of the page.
///
/// # Returns
///
/// The view.
pub fn theme_icon_picker(theme: ThemeMode) -> impl IntoView
{
    let mut spec = theme_spec(theme);
    spec.icon = Some(|| swatches().into_any());
    dropdown(spec)
}

/// What the theme picker offers and where its pick goes.
///
/// # Arguments
///
/// * `theme` - The theme of the page.
///
/// # Returns
///
/// The dropdown.
fn theme_spec(theme: ThemeMode) -> Dropdown
{
    let choice = |t: Theme| {
        let value: &'static str = t.into();
        Choice::builder().value(value).label(t.label()).group(t.group()).build()
    };
    let current = Signal::derive(move || {
        <&'static str>::from(Theme::from_mode(&theme.mode.get())).to_string()
    });
    let on_pick = move |v: String| {
        let picked = v.parse::<Theme>().unwrap_or_default();
        theme.set_mode.set(picked.to_mode());
    };
    picker(
        t!("web_ui.settings.theme"),
        Theme::iter().map(choice).collect(),
        current,
        on_pick,
    )
}

/// Picker of the language of the pages and the compiler messages. A pick
/// reloads the page in the new language.
///
/// # Arguments
///
/// * `before_reload` - Called right before the reload, to keep the state of the
///   page.
///
/// # Returns
///
/// The view.
pub fn lang_picker(before_reload: impl Fn() + Send + Sync + 'static) -> impl IntoView
{
    dropdown(lang_spec(before_reload))
}

/// The language picker drawn as a globe alone, for a bar.
///
/// # Arguments
///
/// * `before_reload` - Called right before the reload, to keep the state of the
///   page.
///
/// # Returns
///
/// The view.
pub fn lang_icon_picker(
    before_reload: impl Fn() + Send + Sync + 'static,
) -> impl IntoView
{
    let mut spec = lang_spec(before_reload);
    spec.icon = Some(|| globe().into_any());
    dropdown(spec)
}

/// What the language picker offers and where its pick goes.
///
/// # Arguments
///
/// * `before_reload` - Called right before the reload.
///
/// # Returns
///
/// The dropdown.
fn lang_spec(before_reload: impl Fn() + Send + Sync + 'static) -> Dropdown
{
    let auto = Choice::builder().value(AUTO_LANG).label(t!("web_ui.lang.auto")).build();
    let named = SUPPORTED_LOCALES
        .iter()
        .map(|tag| Choice::builder().value(*tag).label(lang_name(tag)).build());
    let on_pick = move |v: String| {
        ret_if!(v == stored_lang());
        store_lang(&v);
        before_reload();
        let _ = window().location().reload();
    };
    let choices = once(auto).chain(named).collect();
    // Storage lives in the browser alone, so the stored choice is read there,
    // after the page is drawn; the server draws the Auto choice.
    let current = RwSignal::new(AUTO_LANG.to_string());
    Effect::new(move |_| current.set(stored_lang()));
    picker(t!("web_ui.lang.name"), choices, current.into(), on_pick)
}
