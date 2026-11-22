//! The formatter over the samples and the stdlib modules of the repo.

use std::path::Path;
use std::path::PathBuf;

use zakac_formatter::FmtError;
use zakac_formatter::FmtOptions;
use zakac_formatter::format;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

use crate::common::repo;
use crate::common::zaka_files;

/// Format text as the file at a path, so its imports are read beside it.
///
/// # Arguments
///
/// * `path` - The path.
/// * `text` - The text.
///
/// # Returns
///
/// The formatted text, or the reason it was left as it is.
fn format_at(path: &Path, text: &str) -> Result<String, FmtError>
{
    let name = FileName::real(path.to_path_buf());
    format(&mut SourceMap::new(), name, text, &FmtOptions::default())
}

/// Each sample and stdlib module is formatted already, as `mise run format`
/// leaves it; the samples written to show errors are left out, as the task
/// leaves them out.
#[test]
fn samples_and_stdlib_are_formatted()
{
    let files =
        [zaka_files(&repo().join("samples")), zaka_files(&repo().join("stdlib"))];
    let errors = |p: &&PathBuf| p.display().to_string().ends_with("_errors.zaka");
    for path in files.iter().flatten().filter(|p| !errors(p))
    {
        let text = std::fs::read_to_string(path).unwrap();
        let shown = path.display();
        let out = format_at(path, &text).unwrap_or_else(|e| panic!("{shown}: {e:?}"));
        assert_eq!(out, text, "{shown}");
    }
}
