use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_if;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileName
{
    /// A real file on the FS
    Real(RealFileName),
    /// Standard Input
    Stdin,
    /// A custom name (e.g. string input)
    Custom(String),
}

/// A real file path, with optional remapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealFileName
{
    /// Path on disk.
    pub local_path: PathBuf,
    /// Remapped path shown in diagnostics.
    pub remapped_path: Option<PathBuf>,
}

impl FileName
{
    /// Create a real file name from a local path.
    ///
    /// # Arguments
    ///
    /// * `local_path` - The local path to create a real file name from.
    ///
    /// # Returns
    ///
    /// The real file name.
    pub fn real(local_path: PathBuf) -> Self
    {
        FileName::Real(RealFileName { local_path, remapped_path: None })
    }

    /// Create a real file name with a remapped display path.
    ///
    /// # Arguments
    ///
    /// * `local_path` - The local path to create a real file name from.
    /// * `remapped_path` - The remapped path to show in diagnostics.
    ///
    /// # Returns
    ///
    /// The real file name.
    pub fn real_remapped(local_path: PathBuf, remapped_path: PathBuf) -> Self
    {
        let remapped_path = Some(remapped_path);
        FileName::Real(RealFileName { local_path, remapped_path })
    }

    /// Get the virtual path string for this file name.
    ///
    /// # Returns
    ///
    /// The virtual path.
    pub fn virtual_path_string(&self) -> String
    {
        ret_if!(let FileName::Stdin = self, String::from("<stdin>"));
        ret_if!(let FileName::Custom(name) = self, name.clone());

        extract!(FileName::Real(real) = self);
        real.remapped_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| real.local_path.display().to_string())
    }
}

impl fmt::Display for FileName
{
    /// Format the file name.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to format the file name to.
    ///
    /// # Returns
    ///
    /// The result of formatting the file name.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.write_str(&self.virtual_path_string())
    }
}

/// Prefix-based path remapping (e.g. `--remap-path-prefix`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilePathMapping
{
    /// `(from, to)` prefix pairs applied in order.
    mapping: Vec<(PathBuf, PathBuf)>,
}

impl FilePathMapping
{
    /// Create an blank mapping.
    ///
    /// # Returns
    ///
    /// The blank mapping.
    pub fn new() -> Self { Self { mapping: Vec::new() } }

    /// Create a mapping from prefix pairs.
    ///
    /// # Arguments
    ///
    /// * `mapping` - The mapping to create a mapping from.
    ///
    /// # Returns
    ///
    /// The mapping from the prefix pairs.
    pub fn from_pairs(mapping: Vec<(PathBuf, PathBuf)>) -> Self { Self { mapping } }

    /// Return the mapping pairs.
    ///
    /// # Returns
    ///
    /// The mapping pairs.
    pub fn pairs(&self) -> &[(PathBuf, PathBuf)] { &self.mapping }

    /// Add prefix remapping to `path`.
    ///
    /// # Arguments
    ///
    /// * `p` - The path to remap.
    ///
    /// Returns the remapped path and whether there's a mapping applied.
    pub fn remap_path(&self, p: &Path) -> (PathBuf, bool)
    {
        for (f, t) in &self.mapping
        {
            ret_if!(p.starts_with(f), (t.join(p.strip_prefix(f).unwrap_or(p)), true));
        }
        (p.to_path_buf(), false)
    }
}
