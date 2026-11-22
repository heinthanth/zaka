//! Helpers of the tests.

use std::path::Path;
use std::path::PathBuf;

use zakac_formatter::FmtOptions;
use zakac_formatter::format_text;

/// The root of the repo.
///
/// # Returns
///
/// The path.
pub fn repo() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../..") }

/// Format text with the default settings, failing the test on an error.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The formatted text.
pub fn fmt(text: &str) -> String { fmt_with(text, &FmtOptions::default()) }

/// Format text, failing the test on an error.
///
/// # Arguments
///
/// * `text` - The text.
/// * `options` - The settings.
///
/// # Returns
///
/// The formatted text.
pub fn fmt_with(text: &str, options: &FmtOptions) -> String
{
    format_text(text, options).unwrap_or_else(|e| panic!("not formatted: {e:?}"))
}

/// The `.zaka` files of a folder and its subfolders.
///
/// # Arguments
///
/// * `dir` - The folder.
///
/// # Returns
///
/// The paths, sorted.
pub fn zaka_files(dir: &Path) -> Vec<PathBuf>
{
    let entries = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path());
    let mut found: Vec<PathBuf> = entries
        .flat_map(|p| if p.is_dir() { zaka_files(&p) } else { vec![p] })
        .filter(|p| p.extension().is_some_and(|e| e == "zaka"))
        .collect();
    found.sort();
    found
}
