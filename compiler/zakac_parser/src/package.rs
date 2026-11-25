//! Packages: the deps a `zakac.toml` lists under `[dependencies]`, and the
//! `[package.modules]` table through which a package gives Burmese import
//! names to its files.

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use bon::Builder;
use serde::Deserialize;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_lexer::spelling::canonical;
use zakac_rust_sugars::ret_if;
use zakac_span::file_loader::FileLoader;

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

/// One dep of a package.
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct Dep
{
    /// Its root, relative to the root of the package that lists it.
    pub path: PathBuf,
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
        assert_eq!(dep.path, PathBuf::from("../shapes"));
        let (file, listed) = read.module_file(&canonical("စက်ဝိုင်း"));
        assert!(listed);
        assert_eq!(file, Path::new("app").join("circle.zaka"));
        let (file, listed) = read.module_file("geo/rect");
        assert!(!listed);
        assert_eq!(file, Path::new("app").join("geo/rect.zaka"));
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
