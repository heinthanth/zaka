use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use zakac_rust_sugars::ret_if;

use crate::file_loader::FileLoader;
use crate::file_loader::RealFileLoader;
use crate::filename::FileName;
use crate::filename::FilePathMapping;
use crate::hash::SourceFileHashAlgorithm;
use crate::pos::CharPos;
use crate::pos::Pos;
use crate::source_file::SourceFile;
use crate::span::Span;

/// A source location (file, line, column).
#[derive(Clone, Debug)]
pub struct Loc
{
    /// Source file.
    pub file: Arc<SourceFile>,
    /// 1-based line number.
    pub line: usize,
    /// 0-based column (Unicode scalar values).
    pub col: CharPos,
}

/// Error extracting a span snippet.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum SpanSnippetError
{
    /// The span is a placeholder.
    Placeholder,
    /// The span crosses file boundaries or is out of range.
    OutOfRange,
}

impl fmt::Display for SpanSnippetError
{
    /// Format the span snippet error.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to format the span snippet error to.
    ///
    /// # Returns
    ///
    /// The result of formatting the span snippet error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            SpanSnippetError::Placeholder => f.write_str("ERR_SPAN_PLACEHOLDER"),
            SpanSnippetError::OutOfRange => f.write_str("ERR_SPAN_OUT_OF_RANGE"),
        }
    }
}

/// Check if a file contains a position.
///
/// # Arguments
///
/// * `file` - The file to check.
/// * `pos` - The position to check.
///
/// # Returns
///
/// `true` if the file contains the position, `false` otherwise.
fn file_contains_pos(file: &SourceFile, pos: &Pos) -> bool
{
    if file.src.is_empty()
    {
        pos == &file.start_pos
    }
    else
    {
        pos >= &file.start_pos && pos < &file.end_pos()
    }
}

/// Tracks all source files and maps absolute byte positions to locations.
#[derive(Clone)]
pub struct SourceMap
{
    /// Registered source files.
    files: Vec<Arc<SourceFile>>,
    /// File loader.
    file_loader: Arc<dyn FileLoader>,
    /// Path prefix remapping.
    path_mapping: FilePathMapping,
    /// Hash algorithm for new files.
    hash_kind: SourceFileHashAlgorithm,
    /// Working directory for resolving relative paths.
    working_dir: Option<PathBuf>,
}

impl Default for SourceMap
{
    /// Create a new source map with the default real file loader.
    ///
    /// # Returns
    ///
    /// The new source map.
    fn default() -> Self { Self::new() }
}

impl fmt::Debug for SourceMap
{
    /// Format the source map.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to format the source map to.
    ///
    /// # Returns
    ///
    /// The result of formatting the source map.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("SourceMap")
            .field("files", &self.files)
            .field("path_mapping", &self.path_mapping)
            .field("hash_kind", &self.hash_kind)
            .field("working_dir", &self.working_dir)
            .finish_non_exhaustive()
    }
}

impl PartialEq for SourceMap
{
    /// Check if the source maps are equal.
    ///
    /// # Arguments
    ///
    /// * `other` - The other source map to compare to.
    ///
    /// # Returns
    ///
    /// `true` if the source maps are equal, `false` otherwise.
    fn eq(&self, other: &Self) -> bool
    {
        self.files == other.files
            && self.path_mapping == other.path_mapping
            && self.hash_kind == other.hash_kind
            && self.working_dir == other.working_dir
    }
}

impl SourceMap
{
    /// Create a source map with the default real file loader.
    ///
    /// # Returns
    ///
    /// The new source map.
    pub fn new() -> Self { Self::with_loader(Arc::new(RealFileLoader)) }

    /// Create a source map with a custom file loader.
    ///
    /// # Arguments
    ///
    /// * `loader` - The file loader to use.
    ///
    /// # Returns
    ///
    /// The new source map.
    pub fn with_loader(loader: Arc<dyn FileLoader>) -> Self
    {
        SourceMap {
            files: Vec::new(),
            file_loader: loader,
            path_mapping: FilePathMapping::new(),
            hash_kind: SourceFileHashAlgorithm::Sha256,
            working_dir: None,
        }
    }

    /// Return the file loader.
    ///
    /// # Returns
    ///
    /// The file loader.
    pub fn file_loader(&self) -> &dyn FileLoader { self.file_loader.as_ref() }

    /// Return the path mapping.
    ///
    /// # Returns
    ///
    /// The path mapping.
    pub fn path_mapping(&self) -> &FilePathMapping { &self.path_mapping }

    /// Set path prefix remapping.
    ///
    /// # Arguments
    ///
    /// * `mapping` - The path mapping to set.
    ///
    /// # Returns
    ///
    /// The new source map.
    pub fn set_path_mapping(&mut self, mapping: FilePathMapping)
    {
        self.path_mapping = mapping;
    }

    /// Set the working directory.
    ///
    /// # Arguments
    ///
    /// * `dir` - The working directory to set.
    ///
    /// # Returns
    ///
    /// The new source map.
    pub fn set_working_dir(&mut self, dir: PathBuf) { self.working_dir = Some(dir); }

    /// Return registered source files.
    ///
    /// # Returns
    ///
    /// The registered source files.
    pub fn files(&self) -> &[Arc<SourceFile>] { &self.files }

    /// Register a new source file from original text.
    ///
    /// # Arguments
    ///
    /// * `name` - File name.
    /// * `original_src` - Raw source text (hashed and normalized on ingest).
    ///
    /// # Returns
    ///
    /// The registered source file.
    pub fn new_source_file(
        &mut self, name: FileName, original_src: String,
    ) -> Arc<SourceFile>
    {
        let start_pos = self.next_start_pos();
        let file =
            Arc::new(SourceFile::new(name, original_src, start_pos, self.hash_kind));
        self.files.push(file.clone());
        file
    }

    /// Load a file from disk via the file loader.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file.
    ///
    /// # Returns
    ///
    /// The registered source file.
    pub fn load_file(&mut self, path: &Path) -> std::io::Result<Arc<SourceFile>>
    {
        let resolved = self.resolve_path(path);
        let original = self.file_loader.read_file(&resolved)?;
        let (remapped, _) = self.path_mapping.remap_path(&resolved);
        let name = FileName::real_remapped(resolved, remapped);
        Ok(self.new_source_file(name, original))
    }

    /// Look up the source file containing `pos`.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position to look up.
    ///
    /// # Returns
    ///
    /// The source file containing `pos`.
    pub fn lookup_source_file(&self, pos: &Pos) -> Arc<SourceFile>
    {
        let idx = self.lookup_source_file_idx(pos);
        self.files[idx].clone()
    }

    /// Borrow the source file containing `pos`.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position to look up.
    ///
    /// # Returns
    ///
    /// The source file containing `pos`.
    pub fn source_file(&self, pos: &Pos) -> &SourceFile
    {
        let idx = self.lookup_source_file_idx(pos);
        &self.files[idx]
    }

    /// Look up line/column information for `pos`.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position to look up.
    ///
    /// # Returns
    ///
    /// The line/column information for `pos`.
    pub fn lookup_char_pos(&self, pos: &Pos) -> Loc
    {
        let file = self.lookup_source_file(pos);
        let relative = file.relative_position(pos);
        let (line, col) = file.lookup_file_pos(&relative);
        Loc { file, line, col }
    }

    /// Format a span as `file:line:col`.
    ///
    /// # Arguments
    ///
    /// * `sp` - The span to format.
    ///
    /// # Returns
    ///
    /// The formatted span.
    pub fn span_to_string(&self, sp: &Span) -> String
    {
        ret_if!(sp.is_placeholder(), String::from("<PLACEHOLDER>"));
        let loc = self.lookup_char_pos(&sp.lo);
        format!(
            "{}:{}:{}",
            loc.file.name.virtual_path_string(),
            loc.line,
            loc.col.as_u32()
        )
    }

    /// Return the next start position.
    ///
    /// # Returns
    ///
    /// The next start position.
    fn next_start_pos(&self) -> Pos
    {
        match self.files.last()
        {
            None => Pos::ONE,
            Some(file) => file.end_pos().advanced(1),
        }
    }

    /// Look up the index of the source file containing `pos`.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position to look up.
    ///
    /// # Returns
    ///
    /// The index of the source file containing `pos`.
    fn lookup_source_file_idx(&self, pos: &Pos) -> usize
    {
        self.files
            .binary_search_by(|file| {
                ret_if!(pos < &file.start_pos, std::cmp::Ordering::Greater);
                ret_if!(file_contains_pos(file, pos), std::cmp::Ordering::Equal);
                return std::cmp::Ordering::Less;
            })
            .unwrap_or_else(|idx| idx.saturating_sub(1))
    }

    /// Resolve a path relative to the working directory.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to resolve.
    ///
    /// # Returns
    ///
    /// The resolved path.
    fn resolve_path(&self, path: &Path) -> PathBuf
    {
        if path.is_absolute()
        {
            path.to_path_buf()
        }
        else if let Some(dir) = &self.working_dir
        {
            dir.join(path)
        }
        else
        {
            path.to_path_buf()
        }
    }

    /// Extract the source text covered by `sp`.
    ///
    /// # Arguments
    ///
    /// * `sp` - The span to extract.
    ///
    /// # Returns
    ///
    /// The source text covered by `sp`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the span is a placeholder or out of range.
    pub fn span_to_snippet(&self, sp: &Span) -> Result<String, SpanSnippetError>
    {
        ret_if!(sp.is_placeholder(), Err(SpanSnippetError::Placeholder));

        let lo_file = self.source_file(&sp.lo);
        let hi_file = self.source_file(&sp.hi);
        ret_if!(!std::ptr::eq(lo_file, hi_file), Err(SpanSnippetError::OutOfRange));

        let lo = lo_file.relative_position(&sp.lo);
        let hi = lo_file.relative_position(&sp.hi);
        lo_file.snippet(&lo, &hi).map(str::to_owned).ok_or(SpanSnippetError::OutOfRange)
    }
}
