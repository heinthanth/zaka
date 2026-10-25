use leptos::prelude::window;
use zakac_i18n::SUPPORTED_LOCALES;
use zakac_rust_sugars::do_if;

/// Storage name of the chosen language; the website and the explorer share
/// it.
const LANG_STORE: &str = "zaka-lang";

/// Choice of the language picker that follows the browser.
pub const AUTO_LANG: &str = "auto";

/// Name of each bundled locale, written in that language, so readers find
/// theirs whatever the page shows.
const LANG_NAMES: &[(&str, &str)] = &[("en", "English")];

/// Name of a locale in its own language.
///
/// # Arguments
///
/// * `tag` - The locale tag.
///
/// # Returns
///
/// The name, or the tag when the table lacks it.
pub fn lang_name(tag: &str) -> &str
{
    LANG_NAMES.iter().find(|(t, _)| *t == tag).map_or(tag, |(_, name)| name)
}

/// The locale a choice of the language picker stands for.
///
/// # Arguments
///
/// * `choice` - The choice: a locale tag or [`AUTO_LANG`].
///
/// # Returns
///
/// The bundled locale, or `None` for [`AUTO_LANG`] and unknown tags.
pub fn chosen_locale(choice: &str) -> Option<&'static str>
{
    SUPPORTED_LOCALES.iter().copied().find(|tag| *tag == choice)
}

/// The stored choice of the language picker.
///
/// # Returns
///
/// A bundled locale tag, or [`AUTO_LANG`] when none is stored.
pub fn stored_lang() -> String
{
    let store = window().local_storage().ok().flatten();
    let stored = store.and_then(|s| s.get_item(LANG_STORE).ok().flatten());
    stored.filter(|c| chosen_locale(c).is_some()).unwrap_or_else(|| AUTO_LANG.into())
}

/// Switch to the stored language when one is chosen; else the detected
/// locale stands. Call before the page renders.
pub fn load_stored_lang()
{
    do_if!(let Some(tag) = chosen_locale(&stored_lang()), zakac_i18n::set_locale(tag));
}

/// Store a choice of the language picker.
///
/// # Arguments
///
/// * `choice` - A locale tag or [`AUTO_LANG`].
pub fn store_lang(choice: &str)
{
    let Some(store) = window().local_storage().ok().flatten()
    else
    {
        return;
    };
    let _ = store.set_item(LANG_STORE, choice);
}
