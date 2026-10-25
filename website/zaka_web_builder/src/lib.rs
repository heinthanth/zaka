use std::path::Path;

use xxhash_rust::xxh3::xxh3_64;

/// Write each file under `<stem>.<hash>.<ext>` into each folder in `dirs`,
/// emptied first, and name its address in Rust.
///
/// # Arguments
///
/// * `files` - The Rust name of each address and its file, from the package.
/// * `base` - What each address starts with, such as `/assets/`.
/// * `dirs` - Folders that get the hashed files.
///
/// # Returns
///
/// The Rust code: one `pub const` per file.
pub fn hash_files(files: &[(&str, &str)], base: &str, dirs: &[&Path]) -> String
{
    dirs.iter().for_each(|dir| fresh(dir));
    files.iter().map(|&(name, src)| hashed(name, src, base, dirs)).collect()
}

/// Clear a folder, making it when missing.
///
/// # Arguments
///
/// * `dir` - The folder.
fn fresh(dir: &Path)
{
    _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
}

/// Write one file under its hashed name.
///
/// # Arguments
///
/// * `name` - Rust name of the address.
/// * `src` - The file, from the package.
/// * `base` - What the address starts with.
/// * `dirs` - Folders that get the file.
///
/// # Returns
///
/// The Rust line that names its address.
fn hashed(name: &str, src: &str, base: &str, dirs: &[&Path]) -> String
{
    cargo_build::rerun_if_changed([src]);
    let raw = std::fs::read(src).unwrap_or_else(|e| panic!("{src}: {e}"));
    let path = Path::new(src);
    let stem = path.file_stem().and_then(|s| s.to_str()).expect("a file name");
    let ext = path.extension().and_then(|s| s.to_str()).expect("an extension");
    let file = format!("{stem}.{:08x}.{ext}", xxh3_64(&raw) as u32);
    dirs.iter().for_each(|dir| {
        std::fs::write(dir.join(&file), &raw).unwrap_or_else(|e| panic!("{file}: {e}"));
    });
    format!(
        "/// `{src}`, named after a hash of its content.\npub const {name}: &str = \
         \"{base}{file}\";\n"
    )
}
