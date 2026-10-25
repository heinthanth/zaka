use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Prefix of each locale id of the shared crate; an id has an area and a
/// name after it.
const PREFIX: &str = "web_ui.";

/// Suffix a settings row adds to its id for the note under it.
const NOTE: &str = "_note";

/// Ids of the locale table; each must have an English text.
///
/// # Returns
///
/// The ids.
fn table_ids() -> BTreeSet<String>
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let table = fs::read_to_string(root.join("locales/web_ui.toml")).unwrap();
    let mut ids = BTreeSet::new();
    let mut open: Option<String> = None;
    for line in table.lines().map(str::trim)
    {
        if let Some(id) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']'))
        {
            assert!(open.is_none(), "{} has no English text", open.unwrap());
            open = Some(id.to_string());
            continue;
        }
        if line.starts_with("en = ")
        {
            ids.extend(open.take());
        }
    }
    assert!(open.is_none(), "{} has no English text", open.unwrap());
    ids
}

/// Ids written in the source, as string literals that start with
/// [`PREFIX`] and hold two dots or more.
///
/// # Arguments
///
/// * `dir` - The folder to scan.
/// * `found` - Where the ids go.
fn source_ids(dir: &Path, found: &mut BTreeSet<String>)
{
    for item in fs::read_dir(dir).unwrap().map(Result::unwrap)
    {
        let path = item.path();
        if path.is_dir()
        {
            source_ids(&path, found);
            continue;
        }
        if path.extension().is_none_or(|e| e != "rs")
        {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let quoted = text.split('"').skip(1).step_by(2);
        let ids =
            quoted.filter(|s| s.starts_with(PREFIX) && s.matches('.').count() > 1);
        found.extend(ids.map(str::to_string));
    }
}

#[test]
fn each_id_in_the_source_has_an_english_text()
{
    let table = table_ids();
    let mut used = BTreeSet::new();
    source_ids(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut used);
    let missing: Vec<&String> = used.iter().filter(|k| !table.contains(*k)).collect();
    assert!(missing.is_empty(), "ids without a text: {missing:?}");
    let unused: Vec<&String> = table
        .iter()
        .filter(|k| !used.contains(*k))
        .filter(|k| k.strip_suffix(NOTE).is_none_or(|row| !used.contains(row)))
        .collect();
    assert!(unused.is_empty(), "ids the source never reads: {unused:?}");
}
