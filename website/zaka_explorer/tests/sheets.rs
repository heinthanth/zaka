use std::path::Path;
use std::path::PathBuf;

use zakac_rust_sugars::do_if;
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

/// The selectors of the rules in compiled CSS; those inside
/// `@keyframes` name the steps of an animation and are left out.
fn selectors(css: &str) -> Vec<String>
{
    let mut found = Vec::new();
    let mut frames = Vec::new();
    let mut start = 0;
    for (at, c) in css.char_indices()
    {
        match c
        {
            '{' =>
            {
                let prelude = css[start..at].trim();
                let in_frames = frames.last().copied().unwrap_or(false);
                do_if!(
                    !prelude.starts_with('@') && !in_frames,
                    found.push(prelude.to_string())
                );
                frames.push(prelude.starts_with("@keyframes") || in_frames);
                start = at + 1;
            }
            '}' =>
            {
                frames.pop();
                start = at + 1;
            }
            ';' => start = at + 1,
            _ => (),
        }
    }
    found
}

#[test]
fn component_rules_start_from_a_class()
{
    // The classes of a component hold a hash, so a rule that names one keeps
    // to the component; a rule of bare elements would reach the whole page.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let global = ["monaco.sass", "theme/mod.sass"];
    let options = grass::Options::default().style(grass::OutputStyle::Compressed);
    let mut seen = 0;
    for path in sheets(&root)
    {
        let name = path.to_string_lossy();
        do_if!(global.iter().any(|g| name.ends_with(g)), continue);
        let css = grass::from_path(&path, &options).unwrap();
        for list in selectors(&css)
        {
            seen += 1;
            let bare = list.split(',').find(|s| !s.contains('.'));
            assert!(bare.is_none(), "{name}: {list}");
        }
    }
    assert!(seen > 200, "{seen}");
}
