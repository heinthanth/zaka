use std::path::Path;
use std::path::PathBuf;

use zakac_rust_sugars::tern;

/// All Sass sheets under `dir`.
fn sheets(dir: &Path) -> Vec<PathBuf>
{
    let items = std::fs::read_dir(dir).unwrap().map(|item| item.unwrap().path());
    let is_sass = |path: &Path| path.extension().is_some_and(|e| e == "sass");
    let walk = |path: PathBuf| {
        tern!(
            path.is_dir(),
            sheets(&path),
            is_sass(&path).then_some(path).into_iter().collect()
        )
    };
    items.flat_map(walk).collect()
}

#[test]
fn sheets_are_plain_ascii()
{
    // A char past ASCII makes the compiled sheet start with a BOM (U+FEFF),
    // and the browser then drops the first rule of the sheet.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let found = sheets(&root);
    assert!(!found.is_empty());
    for path in found
    {
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.is_ascii(), "{}", path.display());
    }
}
