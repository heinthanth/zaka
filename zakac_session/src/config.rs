use std::io::BufRead;
use std::path::PathBuf;

use bon::Builder;
use termcolor::ColorChoice;
use termcolor::WriteColor;
use zakac_errors::errors::IOErrorAction;
use zakac_errors::errors::IOErrorContext;
use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::filename::FileName;

/// Session options.
#[derive(Builder, Default, Debug, Clone, PartialEq)]
pub struct SessionOptions
{
    /// Working dir of the session.
    #[builder(default)]
    pub working_dir: PathBuf,
    /// Color config
    #[builder(default)]
    pub color: ColorChoice,
    /// Path where the compiler should write its output.
    pub output_dir: Option<PathBuf>,
    /// The path where temp files should be stored.
    pub temp_dir: Option<PathBuf>,
}

/// An input source.
#[derive(Default, Debug, Clone, PartialEq)]
pub enum Input
{
    /// Stdin
    #[default]
    Stdin,
    /// Input from a file.
    File(PathBuf),
    /// In-process u8s with a diagnostic name.
    Data
    {
        name: String, content: Vec<u8>
    },
}

/// An output destination.
#[derive(Default, Debug, Clone, PartialEq)]
pub enum Output
{
    /// Stdout
    #[default]
    Stdout,
    /// Output to a file.
    File(PathBuf),
    /// In-process u8s with a diagnostic name.
    Data
    {
        name: String, content: Vec<u8>
    },
}

impl Input
{
    /// Diagnostic label for errors tied to this input.
    ///
    /// # Returns
    ///
    /// The diagnostic path.
    pub fn error_path(&self) -> PathBuf
    {
        ret_if!(let Input::Stdin = self, PathBuf::from("<stdin>"));
        ret_if!(let Input::File(path) = self, path.clone());

        extract!(Input::Data { name, .. } = self);
        PathBuf::from(name.as_str())
    }

    /// File name used when this input is registered in a
    /// [`SourceMap`](zakac_span::source_map::SourceMap).
    ///
    /// # Returns
    ///
    /// The file name.
    pub fn file_name(&self) -> FileName
    {
        ret_if!(let Input::Stdin = self, FileName::Stdin);
        ret_if!(let Input::File(path) = self, FileName::real(path.clone()));

        extract!(Input::Data { name, .. } = self);
        FileName::Custom(name.clone())
    }

    /// Read the full input.
    ///
    /// # Arguments
    ///
    /// * `stdin` - The standard input to use when reading from stdin.
    ///
    /// # Returns
    ///
    /// The file name and content.
    pub fn read(
        &self, stdin: &mut dyn BufRead,
    ) -> zakac_errors::Result<(FileName, Vec<u8>)>
    {
        let name = self.file_name();
        let path = self.error_path();

        ret_if!(let Input::Stdin = self, read_stdin_input(stdin, &name, &path));
        ret_if!(let Input::File(path) = self, read_file_input(&path, &name));
        extract!(Input::Data { content, .. } = self);
        Ok((name, content.clone()))
    }
}

/// Read input from stdin.
///
/// # Arguments
///
/// * `stdin` - The standard input to read from.
/// * `name` - The file name.
/// * `path` - The file path.
///
/// # Returns
///
/// The file name and content.
fn read_stdin_input(
    stdin: &mut dyn BufRead, name: &FileName, path: &PathBuf,
) -> zakac_errors::Result<(FileName, Vec<u8>)>
{
    let mut content = Vec::new();
    let res = stdin.read_to_end(&mut content);
    ret_if!(let Ok(_) = res, Ok((name.clone(), content)));
    let ctx = IOErrorContext::new(path.clone(), IOErrorAction::Read);
    Err(zakac_errors::Error::from_io(res.err().unwrap(), ctx))
}

/// Read input from a file.
///
/// # Arguments
///
/// * `path` - The file path.
/// * `name` - The file name.
///
/// # Returns
///
/// The file name and content.
fn read_file_input(
    path: &PathBuf, name: &FileName,
) -> zakac_errors::Result<(FileName, Vec<u8>)>
{
    let res = std::fs::read(path);
    ret_if!(let Ok(content) = res, Ok((name.clone(), content)));
    let ctx = IOErrorContext::new(path.clone(), IOErrorAction::Read);
    Err(zakac_errors::Error::from_io(res.err().unwrap(), ctx))
}

impl From<&str> for Input
{
    /// Convert a input-like string to an [`Input`].
    ///
    /// # Arguments
    ///
    /// * `value` - The input-like string.
    ///
    /// # Returns
    ///
    /// The corresponding [`Input`].
    fn from(value: &str) -> Self
    {
        tern!(value == "-", Input::Stdin, Input::File(PathBuf::from(value)))
    }
}

impl From<Option<&str>> for Input
{
    /// Convert a optional input-like string to an [`Input`].
    ///
    /// # Arguments
    ///
    /// * `value` - The optional input-like string.
    ///
    /// # Returns
    ///
    /// The corresponding [`Input`].
    fn from(value: Option<&str>) -> Self
    {
        tern!(let Some(value) = value, Input::from(value), Input::Stdin)
    }
}

impl From<&str> for Output
{
    /// Convert a output-like string to an [`Output`].
    ///
    /// # Arguments
    ///
    /// * `value` - The output-like string.
    ///
    /// # Returns
    ///
    /// The corresponding [`Output`].
    fn from(value: &str) -> Self
    {
        tern!(value == "-", Output::Stdout, Output::File(PathBuf::from(value)))
    }
}

impl From<Option<&str>> for Output
{
    /// Convert a optional output-like string to an [`Output`].
    ///
    /// # Arguments
    ///
    /// * `value` - The optional output-like string.
    ///
    /// # Returns
    ///
    /// The corresponding [`Output`].
    fn from(value: Option<&str>) -> Self
    {
        tern!(let Some(value) = value, Output::from(value), Output::Stdout)
    }
}

impl Output
{
    /// Diagnostic label for errors tied to this output.
    ///
    /// # Returns
    ///
    /// The diagnostic path.
    pub fn error_path(&self) -> PathBuf
    {
        ret_if!(let Output::Stdout = self, PathBuf::from("<stdout>"));
        ret_if!(let Output::File(path) = self, path.clone());

        extract!(Output::Data { name, .. } = self);
        PathBuf::from(name.as_str())
    }

    /// Write u8s to this output destination.
    ///
    /// # Arguments
    ///
    /// * `stdout` - The standard output to write to.
    /// * `content` - The content to write.
    ///
    /// # Returns
    ///
    /// The result of the operation.
    pub fn write(
        &mut self, stdout: &mut dyn WriteColor, content: &[u8],
    ) -> zakac_errors::Result<()>
    {
        ret_if!(let Output::Stdout = self, write_stdout_output(stdout, content));
        ret_if!(let Output::File(path) = self, write_file_output(&path, content));
        extract!(Output::Data { content: buf, .. } = self);
        buf.extend_from_slice(content);
        Ok(())
    }

    /// Write UTF-8 text to this output destination.
    ///
    /// # Arguments
    ///
    /// * `stdout` - The standard output to write to.
    /// * `text` - The text to write.
    ///
    /// # Returns
    ///
    /// The result of the operation.
    pub fn write_text(
        &mut self, stdout: &mut dyn WriteColor, text: &str,
    ) -> zakac_errors::Result<()>
    {
        self.write(stdout, text.as_bytes())
    }
}

/// Write u8s to stdout.
///
/// # Arguments
///
/// * `stdout` - The standard output to write to.
/// * `content` - The content to write.
///
/// # Returns
///
/// The result of the operation.
fn write_stdout_output(
    stdout: &mut dyn WriteColor, content: &[u8],
) -> zakac_errors::Result<()>
{
    let res = stdout.write_all(content);
    ret_if!(let Ok(_) = res, Ok(()));
    let ctx = IOErrorContext::new(PathBuf::from("<stdout>"), IOErrorAction::Write);
    Err(zakac_errors::Error::from_io(res.err().unwrap(), ctx))
}

/// Write u8s to a file.
///
/// # Arguments
///
/// * `path` - The file path to write to.
/// * `content` - The content to write.
///
/// # Returns
///
/// The result of the operation.
fn write_file_output(path: &PathBuf, content: &[u8]) -> zakac_errors::Result<()>
{
    let res = std::fs::write(path, content);
    ret_if!(let Ok(_) = res, Ok(()));
    let ctx = IOErrorContext::new(path.clone(), IOErrorAction::Write);
    Err(zakac_errors::Error::from_io(res.err().unwrap(), ctx))
}
