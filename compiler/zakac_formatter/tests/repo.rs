//! The formatter over the samples and the stdlib modules of the repo.

use std::path::Path;

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

/// Each sample and stdlib module formats, and a second pass changes nothing;
/// the samples written to show errors are left as they are.
#[test]
fn samples_and_stdlib_format_and_settle()
{
    let files =
        [zaka_files(&repo().join("samples")), zaka_files(&repo().join("stdlib"))];
    for path in files.iter().flatten()
    {
        let text = std::fs::read_to_string(path).unwrap();
        let shown = path.display();
        let broken = shown.to_string().ends_with("_errors.zaka");
        match format_at(path, &text)
        {
            Ok(once) => assert_eq!(format_at(path, &once).unwrap(), once, "{shown}"),
            Err(FmtError::Unparsed(_)) if broken =>
            {}
            Err(e) => panic!("{shown}: {e:?}"),
        }
    }
}
