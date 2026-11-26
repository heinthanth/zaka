use const_format::concatcp;
use zakac_rust_sugars::tern;

/// The path the site is served under, with no trailing slash: blank for the
/// latest release at the root, `/v0.1` for the snapshot of an old release. Set
/// at build time through `ZAKA_SITE_BASE`.
pub const SITE_BASE: &str = match option_env!("ZAKA_SITE_BASE")
{
    Some(base) => base,
    None => "",
};

/// Where the landing page of the website lives.
pub const HOME_URL: &str = concatcp!(SITE_BASE, "/");

/// Where the Learn pages live on the website.
pub const LEARN_URL: &str = concatcp!(SITE_BASE, "/learn/");

/// Where the manual of the language, the compiler and the machine lives on
/// the website.
pub const DOCS_URL: &str = concatcp!(SITE_BASE, "/docs/");

/// Where the explorer lives on the website.
pub const EXPLORER_URL: &str = concatcp!(SITE_BASE, "/explorer/");

/// A path from the site root, as a page served under a base names it:
/// `/explorer/` under `/v0.1` is `/v0.1/explorer/`. A path that does not
/// start at the root, an address on another host (`//`) among them, is left as
/// it is.
///
/// # Arguments
///
/// * `base` - The base, as [`SITE_BASE`] holds it.
/// * `path` - The path.
///
/// # Returns
///
/// The path under the base.
pub fn rooted_in(base: &str, path: &str) -> String
{
    let from_root = path.starts_with('/') && !path.starts_with("//");
    tern!(from_root, format!("{base}{path}"), path.to_string())
}

/// A path from the site root under the base of this build.
///
/// # Arguments
///
/// * `path` - The path.
///
/// # Returns
///
/// The path under [`SITE_BASE`].
pub fn rooted(path: &str) -> String { rooted_in(SITE_BASE, path) }

/// The window event the header of the explorer page sends to open the
/// settings of the explorer: the two are islands apart, with no signal in
/// common.
pub const EXPLORER_SETTINGS_EVENT: &str = "zaka-explorer-settings";

/// The window event the header sends right before it reloads the page for a
/// new language, so the explorer writes the link of its latest edits to the
/// URL first; the reload keeps it.
pub const EXPLORER_FLUSH_EVENT: &str = "zaka-explorer-flush";
