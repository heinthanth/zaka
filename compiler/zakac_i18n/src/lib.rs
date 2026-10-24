#[macro_use]
pub extern crate rust_i18n;

pub use rust_i18n::locale;
pub use rust_i18n::set_locale;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

/// Locale tags with bundled translations.
pub const SUPPORTED_LOCALES: &[&str] = &["en"];

/// The fallback (default) locale.
/// When this is changed, don't forget to update the `fallback` argument of the
/// `i18n!` macro as well.
const FALLBACK_LOCALE: &str = "en";

/// Common locale environment variables.
const LOCALE_ENV_VARS: &[&str] = &["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"];

#[rustfmt::skip]
i18n!("locales",
    fallback = "en",
    minify_key = true,
    minify_key_prefix = "zkc_");

/// Normalize a locale tag into a BCP-47 language tag.
/// It can't handle all possible locale tags, but it's enough for most values
/// found in environment variables.
///
/// # Arguments
///
/// * `tag` - The locale tag to normalize.
///
/// # Returns
///
/// The normalized BCP-47 language tag or `None` if the locale tag is not valid.
fn normalize_locale(tag: &str) -> Option<String>
{
    tag.split(':')
        .next()
        .and_then(|part| part.split('.').next())
        .map(|part| part.replace('_', "-"))
        .filter(|tag| !tag.is_empty() && tag != "C")
}

/// Resolve a BCP-47 tag to the nearest supported locale.
///
/// Walks parent tags (`zh-Hans-CN` -> `zh-Hans` -> `zh`) and falls back to
/// the default locale.
///
/// # Arguments
///
/// * `tag` - A normalized BCP-47 language tag such as `en-US` or `zh-CN`.
/// * `default` - The default locale to use if no supported tag is found.
///
/// # Returns
///
/// The nearest supported BCP-47 language tag or default locale if no supported
/// tag is found.
fn resolve_supported_locale(tag: &str, supported: &[&str], default: &str) -> String
{
    let mut current = tag;
    loop
    {
        ret_if!(supported.contains(&current), current.to_string());
        current = tern!(let Some((p, _)) = current.rsplit_once('-'), p, default);
    }
}

/// Detect the locale from common environment
/// variables.
///
/// # Returns
///
/// The detected locale from common environment variables or
/// `None` if no supported locale is found.
fn detect_locale_from_env() -> Option<String>
{
    LOCALE_ENV_VARS
        .iter()
        .find_map(|var| std::env::var(var).ok())
        .and_then(|raw| normalize_locale(&raw))
        .map(|tag| resolve_supported_locale(&tag, SUPPORTED_LOCALES, FALLBACK_LOCALE))
}

/// Detect the locale using the platform API, environment variables, or falling
/// back to the fallback locale if no supported locale is found.
///
/// # Returns
///
/// The detected locale or the fallback locale if the locale is not detected.
pub fn detect_supported_locale() -> String
{
    detect_locale_from_env()
        .or_else(|| sys_locale::get_locale().and_then(|raw| normalize_locale(&raw)))
        .map(|tag| resolve_supported_locale(&tag, SUPPORTED_LOCALES, FALLBACK_LOCALE))
        .unwrap_or_else(|| FALLBACK_LOCALE.into())
}

/// Set the locale to the detected one, once per process. Later calls do
/// nothing, so a locale set on purpose after the first call holds for all
/// crates.
pub fn init_locale()
{
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| set_locale(&detect_supported_locale()));
}

/// Translate a string against this crate's locale table, with per-crate fallback
/// to the shared table when [`init!`] has been called in the consuming crate.
#[rustfmt::skip]
#[macro_export]
macro_rules! t { ($($all:tt)*) => {
$crate::rust_i18n::_tr!($($all)*,
    _minify_key = true,
    _minify_key_len = 24,
    _minify_key_prefix = "zk_",
    _minify_key_thresh = 127) };}

/// Initialize per-crate translations with fallback to the common locale table.
///
/// Call once at the root of a crate that uses [`t!`]. Platform locale detection
/// runs on the first translation lookup. Lookup order:
/// 1. This crate's `locales/` folder.
/// 2. [`zaka-common-locales`]'s shared translations
#[rustfmt::skip]
#[macro_export]
macro_rules! init { () => {
mod __zaka_locales
{
    $crate::rust_i18n::i18n!(
        "locales",
        fallback = "en", minify_key = true, minify_key_prefix = "zk_");
}

#[doc(hidden)]
pub fn _rust_i18n_try_translate<'r>(
    _locale: &str, k: impl AsRef<str>,
) -> Option<std::borrow::Cow<'r, str>>
{
    $crate::init_locale();
    let locale = $crate::locale();
    __zaka_locales::_rust_i18n_try_translate(&locale, k.as_ref())
        .or_else(|| $crate::_rust_i18n_try_translate(&locale, k.as_ref()))
} };}

#[cfg(test)]
mod tests
{
    use super::normalize_locale;
    use super::resolve_supported_locale;

    #[test]
    fn test_normalize_locale_tag()
    {
        // POSIX locale with codeset
        assert_eq!(normalize_locale("en_US.UTF-8"), Some("en-US".to_string()));
        // GNU LANGUAGE environment variable
        assert_eq!(normalize_locale("zh_CN:zh:en_US:en"), Some("zh-CN".to_string()));
        // BCP-47 locale tag
        assert_eq!(normalize_locale("zh-Hans"), Some("zh-Hans".to_string()));
        // POSIX "C" locale
        assert_eq!(normalize_locale("C"), None);
        assert_eq!(normalize_locale(""), None);
    }

    #[test]
    fn test_resolve_supported_locale()
    {
        assert_eq!(resolve_supported_locale("en-US", &["en"], "en"), "en");
        assert_eq!(resolve_supported_locale("zh-CN", &["en", "zh"], "en"), "zh");
        assert_eq!(resolve_supported_locale("zh-Hans-CN", &["en", "zh"], "en"), "zh");
    }
}
