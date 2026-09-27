use std::io;
use std::path::Path;

/// Abstraction over file system reads\
pub trait FileLoader: Send + Sync
{
    /// Return `true` if `path` exists.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check.
    ///
    /// # Returns
    ///
    /// `true` if `path` exists, `false` otherwise.
    fn file_exists(&self, path: &Path) -> bool;

    /// Read the UTF-8 contents of `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to read.
    ///
    /// # Returns
    ///
    /// The contents of `path`.
    fn read_file(&self, path: &Path) -> io::Result<String>;
}

/// A real [`std::fs`] based [`FileLoader`]
#[derive(Copy, Clone, Debug, Default)]
pub struct RealFileLoader;

impl FileLoader for RealFileLoader
{
    /// Return `true` if `path` exists.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check.
    ///
    /// # Returns
    ///
    /// `true` if `path` exists, `false` otherwise.
    fn file_exists(&self, path: &Path) -> bool { path.exists() }

    /// Read the UTF-8 contents of `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to read.
    ///
    /// # Returns
    ///
    /// The contents of `path`.
    fn read_file(&self, path: &Path) -> io::Result<String>
    {
        std::fs::read_to_string(path)
    }
}

/// In-memory file loader for tests.
#[derive(Clone, Debug, Default)]
pub struct MemFileLoader
{
    /// Path to contents.
    files: std::collections::HashMap<std::path::PathBuf, String>,
}

impl MemFileLoader
{
    /// Create a new file loader.
    ///
    /// # Returns
    ///
    /// The new file loader.
    pub fn new() -> Self { Self { files: std::collections::HashMap::new() } }

    /// Insert a file into the loader.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to insert.
    /// * `contents` - The contents to insert.
    ///
    /// # Returns
    ///
    /// The new file loader.
    pub fn insert(
        &mut self, path: impl Into<std::path::PathBuf>, contents: impl Into<String>,
    )
    {
        self.files.insert(path.into(), contents.into());
    }
}

impl FileLoader for MemFileLoader
{
    /// Return `true` if `path` exists.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check.
    ///
    /// # Returns
    ///
    /// `true` if `path` exists, `false` otherwise.
    fn file_exists(&self, path: &Path) -> bool { self.files.contains_key(path) }

    /// Read the UTF-8 contents of `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to read.
    ///
    /// # Returns
    ///
    /// The contents of `path`.
    fn read_file(&self, path: &Path) -> io::Result<String>
    {
        self.files.get(path).cloned().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("ERR_FILE_NOT_FOUND: {}", path.display()),
            )
        })
    }
}
