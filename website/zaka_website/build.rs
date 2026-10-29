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

/// Each book of pages: its folder from the package, the variant of `Book`
/// its pages belong to, and the file its page table goes to in `OUT_DIR`.
const BOOKS: [(&str, &str, &str); 2] = [
    ("../../docs/learn", "Learn", "learn.rs"),
    ("../../docs/manual", "Manual", "manual.rs"),
];

/// The contents file of a book, in its folder.
const CONTENTS: &str = "CONTENTS.md";

/// Rebuild when a sheet, a locale table or a page of a book changes; the
/// sheet and `include_str!` macros read them. Then write the files in
/// [`ASSETS`] to `OUT_DIR/assets` under hashed names, with their addresses
/// under `/assets/` in `OUT_DIR/assets.rs`, and the page table of each book in
/// [`BOOKS`] to its own file in `OUT_DIR`.
fn main()
{
    cargo_build::rerun_if_changed!("src");
    cargo_build::rerun_if_changed!("locales");
    cargo_build::rerun_if_changed(BOOKS.map(|(folder, ..)| folder));
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let names =
        zaka_web_builder::hash_files(&ASSETS, "/assets/", &[&out.join("assets")]);
    std::fs::write(out.join("assets.rs"), names).expect("OUT_DIR can be written");
    BOOKS.iter().for_each(|(folder, book, file)| {
        std::fs::write(out.join(file), book_pages(folder, book))
            .expect("OUT_DIR can be written");
    });
}

/// The pages of a book, in the order of its contents file, as Rust code:
/// one `Page` each in `PAGES`, its text read in with `include_str!`.
///
/// # Arguments
///
/// * `folder` - The folder of the book, from the package.
/// * `book` - The variant of `Book` its pages belong to.
///
/// # Returns
///
/// The Rust code.
fn book_pages(folder: &str, book: &str) -> String
{
    let root = Path::new(folder)
        .canonicalize()
        .unwrap_or_else(|e| panic!("{folder}: the folder of a book is missing: {e}"));
    let contents = std::fs::read_to_string(root.join(CONTENTS))
        .unwrap_or_else(|e| panic!("{folder}/{CONTENTS}: {e}"));
    let mut appendix = false;
    let pages: Vec<String> = contents
        .lines()
        .filter_map(|line| {
            appendix |= line.trim() == "---";
            contents_line(line).map(|(depth, title, path)| {
                let place = Place { root: &root, book, depth, appendix };
                page_code(&place, title, path)
            })
        })
        .collect();
    format!(
        "/// The pages of the book, in order.\npub const PAGES: [Page; {}] = [\n{}];\n",
        pages.len(),
        pages.concat()
    )
}

/// Where a page sits: in which book, how deep, and on which side of the rule
/// of its contents.
struct Place<'a>
{
    /// The folder of the book.
    root: &'a Path,
    /// The variant of `Book`.
    book: &'a str,
    /// How deep the page sits in the contents; 0 for a chapter.
    depth: usize,
    /// Whether it comes after the rule of the contents.
    appendix: bool,
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
/// * `place` - Where the page sits.
/// * `title` - Its title in the contents.
/// * `path` - Its file, from the folder of its book.
///
/// # Returns
///
/// The Rust code.
fn page_code(place: &Place, title: &str, path: &str) -> String
{
    let file = place.root.join(path);
    assert!(file.is_file(), "{}: the contents name a missing page", file.display());
    let slug = slug_of(path);
    let Place { book, depth, appendix, .. } = place;
    format!(
        "    Page {{ book: Book::{book}, slug: {slug:?}, path: {path:?}, title: \
         {title:?}, depth: {depth}, appendix: {appendix}, text: include_str!({:?}) \
         }},\n",
        file.display().to_string()
    )
}

/// The address of a page in its book: its path with no `.md`, and a
/// chapter's `README.md` named after its folder.
///
/// # Arguments
///
/// * `path` - The file, from the folder of its book.
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
