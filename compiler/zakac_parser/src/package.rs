//! Packages: the deps a `zakac.toml` lists under `[dependencies]`, and the
//! `[package.modules]` table through which a package gives Burmese import
//! names to its files.

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_lexer::spelling::canonical;
use zakac_rust_sugars::ret_if;
use zakac_span::file_loader::FileLoader;
use zakac_span::hash::SourceFileHash;
use zakac_span::hash::SourceFileHashAlgorithm;

use crate::load::STD_PREFIX;

/// The parts of a `zakac.toml` that packages read; the flags it sets are the
/// config's.
#[derive(Debug, Clone, Default, Deserialize, Builder)]
pub struct Manifest
{
    /// The folder of the file: the root of the package.
    #[serde(skip)]
    #[builder(default)]
    pub root: PathBuf,
    /// Each dep, under the name its imports start with, in normalized
    /// spelling.
    #[serde(default, rename = "dependencies")]
    #[builder(default)]
    pub deps: BTreeMap<String, Dep>,
    /// What the file tells of its own package.
    #[serde(default)]
    #[builder(default)]
    pub package: PackageInfo,
}

/// The `[package]` table: what a `zakac.toml` tells of its own package.
#[derive(Debug, Clone, Default, Deserialize, Builder)]
pub struct PackageInfo
{
    /// The file of each import name, relative to the root, each name in
    /// normalized spelling.
    #[serde(default)]
    #[builder(default)]
    pub modules: BTreeMap<String, String>,
}

/// One dep of a package: a folder, or a commit of a git repo.
#[derive(Debug, Clone, Deserialize, Builder)]
#[serde(deny_unknown_fields)]
pub struct Dep
{
    /// Its root, relative to the root of the package that lists it.
    #[builder(into)]
    pub path: Option<PathBuf>,
    /// The URL of its git repo.
    #[builder(into)]
    pub git: Option<String>,
    /// The commit, tag or branch of the repo, which `zakac fetch` pins to a
    /// commit in the lock file.
    #[builder(into)]
    pub rev: Option<String>,
}

/// Where a dep comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source<'a>
{
    /// A folder, relative to the root of the package that lists the dep.
    Path(&'a Path),
    /// The URL of a git repo, and the rev in it.
    Git(&'a str, &'a str),
}

impl Dep
{
    /// Where the dep comes from: `path` alone, or `git` with `rev`.
    ///
    /// # Returns
    ///
    /// The source, or `None` for a dep that gives neither or both.
    pub fn source(&self) -> Option<Source<'_>>
    {
        match (&self.path, &self.git, &self.rev)
        {
            (Some(path), None, None) => Some(Source::Path(path)),
            (None, Some(url), Some(rev)) => Some(Source::Git(url, rev)),
            _ => None,
        }
    }
}

/// The file beside a `zakac.toml` that pins each git dep to a commit.
pub const LOCK_FILE: &str = "zakac.lock";

/// The first lines of a lock file.
const LOCK_HEAD: &str = "# zakac.lock: the commit of each git dep, as `zakac fetch` \
                         pinned it.\n# Keep it with the sources, so each build reads \
                         the same commits.\n\n";

/// The lock file: the commit of each git dep of a package and of its deps.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Builder)]
pub struct Lock
{
    /// Each checkout, in the order of its URL and rev.
    #[serde(default, rename = "git")]
    #[builder(default)]
    pub checkouts: Vec<Checkout>,
}

/// The commit a rev of a git repo stands at.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Builder)]
pub struct Checkout
{
    /// The URL of the repo.
    #[builder(into)]
    pub url: String,
    /// The rev as the manifest writes it.
    #[builder(into)]
    pub rev: String,
    /// The commit the rev stood at when it was fetched.
    #[builder(into)]
    pub commit: String,
}

impl Lock
{
    /// The commit a rev of a repo is pinned to.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the repo.
    /// * `rev` - The rev.
    ///
    /// # Returns
    ///
    /// The commit, or `None` when the lock has no such checkout.
    pub fn commit_of(&self, url: &str, rev: &str) -> Option<&str>
    {
        let found = self.checkouts.iter().find(|c| c.url == url && c.rev == rev);
        found.map(|c| c.commit.as_str())
    }

    /// The text of the lock file, its checkouts in order and each once.
    ///
    /// # Returns
    ///
    /// The text.
    pub fn text(&self) -> String
    {
        let mut checkouts = self.checkouts.clone();
        checkouts.sort();
        checkouts.dedup();
        let sorted = Lock::builder().checkouts(checkouts).build();
        let rows = toml::to_string(&sorted).unwrap_or_default();
        format!("{LOCK_HEAD}{rows}")
    }
}

/// Read a lock file.
///
/// # Arguments
///
/// * `loader` - What reads the file.
/// * `file` - The file.
///
/// # Returns
///
/// The lock, one with no checkouts when the file is not there, or the error of
/// the read or the TOML.
pub fn read_lock(
    loader: &dyn FileLoader, file: &Path,
) -> zakac_host_errors::Result<Lock>
{
    ret_if!(!loader.file_exists(file), Ok(Lock::default()));
    let ctx = IOErrorContext::new(file.to_path_buf(), IOErrorAction::Read);
    let text = loader.read_file(file).map_err(|e| Error::from_io(e, ctx))?;
    let shown = file.display().to_string().replace(std::path::MAIN_SEPARATOR, "/");
    #[rustfmt::skip]
    let not_toml = |e: toml::de::Error| Error::ConfigNotToml {
        path: shown.clone(), msg: e.message().to_string() };
    toml::from_str(&text).map_err(not_toml)
}

/// The folder of the checkout of a commit of a git repo, in the cache:
/// `zakac/git/<repo>-<hash of its URL>/<commit>`.
///
/// # Arguments
///
/// * `cache` - The cache folder.
/// * `url` - The URL of the repo.
/// * `commit` - The commit.
///
/// # Returns
///
/// The folder.
pub fn checkout_dir(cache: &Path, url: &str, commit: &str) -> PathBuf
{
    let hash = SourceFileHash::compute(SourceFileHashAlgorithm::Sha256, url.as_bytes());
    let short: String = hash.hash[..8].iter().map(|b| format!("{b:02x}")).collect();
    let last = url.trim_end_matches('/').rsplit(['/', ':']).next().unwrap_or_default();
    let last = last.trim_end_matches(".git");
    let keep = |c: &char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.');
    let name: String = last.chars().filter(keep).collect();
    cache.join("zakac").join("git").join(format!("{name}-{short}")).join(commit)
}

/// Read the manifest of a package.
///
/// # Arguments
///
/// * `loader` - What reads the file.
/// * `file` - Its `zakac.toml`.
///
/// # Returns
///
/// The manifest, or the error of the read, the TOML or a dep named as the
/// stdlib.
pub fn read_manifest(
    loader: &dyn FileLoader, file: &Path,
) -> zakac_host_errors::Result<Manifest>
{
    let ctx = IOErrorContext::new(file.to_path_buf(), IOErrorAction::Read);
    let text = loader.read_file(file).map_err(|e| Error::from_io(e, ctx))?;
    let shown = file.display().to_string().replace(std::path::MAIN_SEPARATOR, "/");
    let root = file.parent().map(Path::to_path_buf).unwrap_or_default();
    manifest_of(&text, root, &shown)
}

/// Parse the text of a `zakac.toml` as a manifest.
///
/// # Arguments
///
/// * `text` - The text.
/// * `root` - The folder of the file.
/// * `shown` - The file, as an error shows it.
///
/// # Returns
///
/// The manifest, with its names in normalized spelling, or the error of the
/// TOML or a dep named as the stdlib.
pub fn manifest_of(
    text: &str, root: PathBuf, shown: &str,
) -> zakac_host_errors::Result<Manifest>
{
    #[rustfmt::skip]
    let not_toml = |e: toml::de::Error| Error::ConfigNotToml {
        path: shown.into(), msg: e.message().to_string() };
    let read: Manifest = toml::from_str(text).map_err(not_toml)?;
    let deps: BTreeMap<String, Dep> =
        read.deps.into_iter().map(|(k, v)| (canonical(&k).into_owned(), v)).collect();
    let std_name = STD_PREFIX.trim_end_matches('/');
    ret_if!(
        deps.contains_key(std_name),
        Err(Error::DepNamedStd { path: shown.into() })
    );
    let bad = deps.iter().find(|(_, dep)| dep.source().is_none());
    #[rustfmt::skip]
    ret_if!(let Some((name, _)) = bad, Err(Error::BadDep { path: shown.into(),
        name: name.clone() }));
    let modules = read.package.modules.into_iter();
    let modules = modules.map(|(k, v)| (canonical(&k).into_owned(), v)).collect();
    let package = PackageInfo::builder().modules(modules).build();
    Ok(Manifest::builder().root(root).deps(deps).package(package).build())
}

/// Split an import path into its first part and the rest.
///
/// # Arguments
///
/// * `written` - The path in normalized spelling.
///
/// # Returns
///
/// The first part and the rest, or `None` for a path with one part.
pub fn dep_and_module(written: &str) -> Option<(&str, &str)>
{
    written.split_once('/').filter(|(head, rest)| !head.is_empty() && !rest.is_empty())
}

impl Manifest
{
    /// The file an import name of this package reads: the file
    /// `[package.modules]` gives it, else `<name>.zaka` under the root.
    ///
    /// # Arguments
    ///
    /// * `name` - The name, after the dep part, in normalized spelling.
    ///
    /// # Returns
    ///
    /// The path, and whether `[package.modules]` gave it.
    pub fn module_file(&self, name: &str) -> (PathBuf, bool)
    {
        let listed = self.package.modules.get(name);
        let file = listed.map_or_else(
            || self.root.join(format!("{name}.zaka")),
            |f| self.root.join(f),
        );
        (file, listed.is_some())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn deps_and_modules_are_read_in_normalized_spelling()
    {
        let text = "backend = \"vm\"\n[dependencies]\n\"ပုံသဏ္ဌာန်\" = { path = \
                    \"../shapes\" }\n[package.modules]\n\"စက်ဝိုင်း\" = \"circle.zaka\"\n";
        let read = manifest_of(text, PathBuf::from("app"), "zakac.toml").unwrap();
        let dep = read.deps.get(canonical("ပုံသဏ္ဌာန်").as_ref()).unwrap();
        assert_eq!(dep.source(), Some(Source::Path(Path::new("../shapes"))));
        let (file, listed) = read.module_file(&canonical("စက်ဝိုင်း"));
        assert!(listed);
        assert_eq!(file, Path::new("app").join("circle.zaka"));
        let (file, listed) = read.module_file("geo/rect");
        assert!(!listed);
        assert_eq!(file, Path::new("app").join("geo/rect.zaka"));
    }

    #[test]
    fn a_dep_gives_a_path_or_a_git_url_with_a_rev()
    {
        let read = |text: &str| manifest_of(text, PathBuf::new(), "zakac.toml");
        let git = read(
            "[dependencies]\nj = { git = \"https://x.org/j.git\", rev = \"v1\" }\n",
        );
        let dep = &git.unwrap().deps["j"];
        assert_eq!(dep.source(), Some(Source::Git("https://x.org/j.git", "v1")));
        for wrong in
            ["j = { git = \"u\" }", "j = { path = \"p\", git = \"u\", rev = \"r\" }"]
        {
            let read = read(&format!("[dependencies]\n{wrong}\n"));
            assert!(matches!(read, Err(Error::BadDep { .. })), "{wrong}");
        }
        let unknown = read("[dependencies]\nj = { path = \"p\", tag = \"t\" }\n");
        assert!(matches!(unknown, Err(Error::ConfigNotToml { .. })));
    }

    #[test]
    fn a_lock_reads_back_its_own_text_in_order()
    {
        let one = |url: &str, rev: &str| {
            Checkout::builder().url(url).rev(rev).commit(format!("{url}{rev}")).build()
        };
        let lock = Lock::builder()
            .checkouts(vec![one("b", "2"), one("a", "1"), one("b", "2")])
            .build();
        let back: Lock = toml::from_str(&lock.text()).unwrap();
        assert_eq!(back.checkouts, [one("a", "1"), one("b", "2")]);
        assert_eq!(back.commit_of("b", "2"), Some("b2"));
        assert_eq!(back.commit_of("b", "3"), None);
    }

    #[test]
    fn a_checkout_folder_names_its_repo_and_its_commit()
    {
        let dir = checkout_dir(Path::new("c"), "https://x.org/zaka-json.git", "abc");
        let parts: Vec<String> = dir
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        assert_eq!(parts[..3], ["c", "zakac", "git"]);
        assert!(
            parts[3].starts_with("zaka-json-")
                && parts[3].len() == "zaka-json-".len() + 16
        );
        assert_eq!(parts[4], "abc");
    }

    #[test]
    fn a_dep_named_as_the_stdlib_is_refused()
    {
        let text = "[dependencies]\n\"စံကား\" = { path = \"x\" }\n";
        let read = manifest_of(text, PathBuf::new(), "zakac.toml");
        assert!(matches!(read, Err(Error::DepNamedStd { .. })));
    }

    #[test]
    fn an_import_splits_at_its_first_slash()
    {
        assert_eq!(dep_and_module("a/b/c"), Some(("a", "b/c")));
        assert_eq!(dep_and_module("a.zaka"), None);
        assert_eq!(dep_and_module("/a"), None);
    }
}
