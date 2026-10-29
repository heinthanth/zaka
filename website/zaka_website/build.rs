use std::path::Path;
use std::path::PathBuf;

/// The pictures and fonts the pages name: the Rust name of each and its file,
/// from the package.
const ASSETS: [(&str, &str); 8] = [
    ("SABAE_FONT", "../zaka_web_ui_common/assets/fonts/MyanmarSabae.ttf"),
    (
        "SAUCE_FONT",
        "../zaka_web_ui_common/assets/fonts/SauceCodeProHTMMsCustom-Regular.ttf",
    ),
    ("FRANGIPANI_PHOTO", "assets/pictures/frangipani.jpg"),
    ("EXPLORER_PHOTO", "assets/pictures/explorer.jpg"),
    ("BLOSSOMS_PHOTO", "assets/pictures/blossoms.jpg"),
    ("LOGO_LIGHT", "../../artwork/logo/bloom-white-light.svg"),
    ("LOGO_DARK", "../../artwork/logo/bloom-white-dark.svg"),
    ("COVER_CARD", "assets/pictures/cover.png"),
];

/// The Learn folder, from the package.
const LEARN: &str = "../../docs/learn";

/// The contents file of the Learn pages, in their folder.
const CONTENTS: &str = "CONTENTS.md";

/// Rebuild when a sheet, a locale table or a Learn page changes; the sheet and
/// `include_str!` macros read them. Then write the files in [`ASSETS`] to
/// `OUT_DIR/assets` under hashed names, with their addresses under `/assets/`
/// in `OUT_DIR/assets.rs`, and the Learn pages to
/// `OUT_DIR/learn.rs`.
fn main()
{
    cargo_build::rerun_if_changed!("src");
    cargo_build::rerun_if_changed!("locales");
    cargo_build::rerun_if_changed([LEARN]);
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let names =
        zaka_web_builder::hash_files(&ASSETS, "/assets/", &[&out.join("assets")]);
    std::fs::write(out.join("assets.rs"), names).expect("OUT_DIR can be written");
    std::fs::write(out.join("learn.rs"), learn_pages())
        .expect("OUT_DIR can be written");
}

/// The Learn pages, in the order of their contents file, as Rust code:
/// one `Page` each in `PAGES`, its text read in with `include_str!`.
///
/// # Returns
///
/// The Rust code.
fn learn_pages() -> String
{
    let root = Path::new(LEARN).canonicalize().expect("the Learn folder is there");
    let contents = std::fs::read_to_string(root.join(CONTENTS))
        .unwrap_or_else(|e| panic!("{CONTENTS}: {e}"));
    let mut appendix = false;
    let pages: Vec<String> = contents
        .lines()
        .filter_map(|line| {
            appendix |= line.trim() == "---";
            contents_line(line).map(|(depth, title, path)| {
                page_code(&root, depth, title, path, appendix)
            })
        })
        .collect();
    format!(
        "/// The Learn pages, in order.\npub const PAGES: [Page; {}] = [\n{}];\n",
        pages.len(),
        pages.concat()
    )
}

/// Read one line of the contents file: `- [Title](path.md)`, indented two
/// spaces a level, or the bare link of the title page.
///
/// # Arguments
///
/// * `line` - The line.
///
/// # Returns
///
/// The depth, the title and the path, or `None` for a line with no link.
fn contents_line(line: &str) -> Option<(usize, &str, &str)>
{
    let indent = line.len() - line.trim_start().len();
    let rest = line.trim_start().trim_start_matches("- ");
    let (title, path) = rest.strip_prefix('[')?.split_once("](")?;
    Some((indent / 2, title, path.strip_suffix(')')?))
}

/// The Rust code of one page.
///
/// # Arguments
///
/// * `root` - The Learn folder.
/// * `depth` - How deep the page sits in the contents; 0 for a chapter.
/// * `title` - Its title in the contents.
/// * `path` - Its file, from the Learn folder.
/// * `appendix` - Whether it comes after the rule of the contents.
///
/// # Returns
///
/// The Rust code.
fn page_code(
    root: &Path, depth: usize, title: &str, path: &str, appendix: bool,
) -> String
{
    let file = root.join(path);
    assert!(file.is_file(), "{}: the contents name a missing page", file.display());
    let slug = slug_of(path);
    format!(
        "    Page {{ slug: {slug:?}, path: {path:?}, title: {title:?}, depth: \
         {depth}, appendix: {appendix}, text: include_str!({:?}) }},\n",
        file.display().to_string()
    )
}

/// The address of a Learn page: its path with no `.md`, and a
/// chapter's `README.md` named after its folder.
///
/// # Arguments
///
/// * `path` - The file, from the Learn folder.
///
/// # Returns
///
/// The slug; blank for the title page.
fn slug_of(path: &str) -> String
{
    let stem = path.strip_suffix(".md").unwrap_or(path);
    let stem = stem.strip_suffix("README").unwrap_or(stem);
    stem.trim_end_matches('/').to_string()
}
