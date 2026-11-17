//! Write the table of the page that teaches each code: the See column of the
//! error page of the manual, each link turned into its address on the site.

use std::path::Path;

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

/// The error page, from the folder of this crate.
const ERRORS_PAGE: &str = "../../docs/manual/reference/errors.md";

/// The folder of the error page, under `docs/`.
const PAGE_DIR: [&str; 2] = ["manual", "reference"];

fn main()
{
    cargo_build::rerun_if_changed!("locales");
    cargo_build::rerun_if_changed!("{}", ERRORS_PAGE);
    let text =
        std::fs::read_to_string(ERRORS_PAGE).expect("the error page is readable");
    let rows: String = text.lines().filter_map(page_row).collect();
    let out = std::env::var("OUT_DIR").expect("cargo sets OUT_DIR");
    let dest = Path::new(&out).join("pages.rs");
    std::fs::write(dest, format!("&[\n{rows}]")).expect("OUT_DIR is writable");
}

/// The table row of one line of the error page.
///
/// # Arguments
///
/// * `line` - The line.
///
/// # Returns
///
/// The row, or `None` for a line that is no code row or links to no page.
fn page_row(line: &str) -> Option<String>
{
    let cells: Vec<&str> = line.strip_prefix("| ZK")?.split('|').collect();
    let id = format!("ZK{}", cells.first()?.trim());
    let see = cells.get(2)?;
    let start = see.find("](")? + 2;
    let target = &see[start..start + see[start..].find(')')?];
    Some(format!("    ({id:?}, {:?}),\n", site_path(target)?))
}

/// The address on the site of a link from the error page: `learn/a/b.md`
/// is `/learn/a/b/`, `manual/a/b.md` is `/docs/a/b/`, and a `README.md` is
/// the address of its folder.
///
/// # Arguments
///
/// * `target` - The link, relative to the error page, with an optional `#mark`.
///
/// # Returns
///
/// The address, or `None` for a link outside the two books.
fn site_path(target: &str) -> Option<String>
{
    let (file, mark) = target.split_once('#').unwrap_or((target, ""));
    let parts = file.split('/').fold(PAGE_DIR.to_vec(), |mut parts, part| {
        match part
        {
            ".." => _ = parts.pop(),
            _ => parts.push(part),
        }
        parts
    });
    let (book, rest) = parts.split_first()?;
    let book = match *book
    {
        "learn" => "learn",
        "manual" => "docs",
        _ => return None,
    };
    let (last, folders) = rest.split_last()?;
    let last = last.strip_suffix(".md")?;
    let mut path: Vec<&str> =
        [book].into_iter().chain(folders.iter().copied()).collect();
    do_if!(last != "README", path.push(last));
    let mark = tern!(mark.is_empty(), String::new(), format!("#{mark}"));
    Some(format!("/{}/{mark}", path.join("/")))
}
