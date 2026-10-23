use std::io;
use std::path::Path;

/// Abstraction over FS reads
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

    /// Read the raw u8s of `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to read.
    ///
    /// # Returns
    ///
    /// The raw u8s of `path`.
    fn read_raw_file(&self, path: &Path) -> io::Result<Vec<u8>>;

    /// Read the UTF-8 source of `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to read.
    ///
    /// # Returns
    ///
    /// The UTF-8 source of `path`.
    fn read_file(&self, path: &Path) -> io::Result<String>
    {
        let u8s = self.read_raw_file(path)?;
        String::from_utf8(u8s)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
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

    /// Read the raw u8s of `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to read.
    ///
    /// # Returns
    ///
    /// The raw u8s of `path`.
    fn read_raw_file(&self, path: &Path) -> io::Result<Vec<u8>> { std::fs::read(path) }
}

/// MemFS loader for tests.
#[derive(Clone, Debug, Default)]
pub struct MemFileLoader
{
    /// Path to contents.
    files: std::collections::HashMap<std::path::PathBuf, Vec<u8>>,
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
        &mut self, path: impl Into<std::path::PathBuf>, contents: impl AsRef<[u8]>,
    )
    {
        self.files.insert(path.into(), contents.as_ref().to_vec());
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

    /// Read the raw u8s of `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to read.
    ///
    /// # Returns
    ///
    /// The raw u8s of `path`.
    fn read_raw_file(&self, path: &Path) -> io::Result<Vec<u8>>
    {
        self.files.get(path).cloned().ok_or_else(|| {
            let msg = format!("ERR_FILE_NOT_FOUND: {}", path.display());
            io::Error::new(io::ErrorKind::NotFound, msg)
        })
    }
}
