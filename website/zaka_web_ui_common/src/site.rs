use bon::Builder;
use const_format::concatcp;
use serde::Deserialize;
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

/// The list of the releases the site holds, at the root of the site: each
/// snapshot reads it from there, so an old one learns of the releases after
/// it.
pub const VERSIONS_FILE: &str = "/versions.toml";

/// One release in [`VERSIONS_FILE`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Builder)]
pub struct Release
{
    /// Its version, as `0.1`.
    #[builder(into)]
    pub version: String,
    /// The home page of its site, as `/v0.1/`; `/` for the latest.
    #[builder(into)]
    pub path: String,
}

/// The shape of [`VERSIONS_FILE`]: a `[[release]]` table for each release,
/// the newest first, so the first is the latest.
#[derive(Deserialize)]
struct Releases
{
    /// The releases.
    #[serde(default)]
    release: Vec<Release>,
}

/// The releases a [`VERSIONS_FILE`] lists.
///
/// # Arguments
///
/// * `text` - The TOML text of the file.
///
/// # Returns
///
/// The releases, the newest first; none when the text does not parse.
pub fn read_releases(text: &str) -> Vec<Release>
{
    toml::from_str::<Releases>(text).map(|r| r.release).unwrap_or_default()
}

/// The same page in the site of another release.
///
/// # Arguments
///
/// * `pathname` - The path of the page.
/// * `home` - The home page of its site, as [`HOME_URL`] holds it.
/// * `to` - The home page of the other site.
///
/// # Returns
///
/// The path in the other site; its home page for a path outside `home`.
pub fn moved_path(pathname: &str, home: &str, to: &str) -> String
{
    format!("{to}{}", pathname.strip_prefix(home).unwrap_or_default())
}

/// The window event the header of the explorer page sends to open the
/// settings of the explorer: the two are islands apart, with no signal in
/// common.
pub const EXPLORER_SETTINGS_EVENT: &str = "zaka-explorer-settings";

/// The window event the header sends right before it reloads the page for a
/// new language, so the explorer writes the link of its latest edits to the
/// URL first; the reload keeps it.
pub const EXPLORER_FLUSH_EVENT: &str = "zaka-explorer-flush";
