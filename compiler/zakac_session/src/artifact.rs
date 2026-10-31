use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use termcolor::WriteColor;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_i18n::t;
use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_if;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

use crate::config::Input;
use crate::config::Output;

/// A file an artifact carries besides the file that is run: a file the
/// program imports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddedFile
{
    /// Diagnostic file name.
    pub name: FileName,
    /// Normalized source text.
    pub source: String,
}

impl EmbeddedFile
{
    /// The name and the text of a source file.
    ///
    /// # Arguments
    ///
    /// * `file` - The file.
    ///
    /// # Returns
    ///
    /// The embedded file.
    pub fn of(file: &SourceFile) -> Self
    {
        Self { name: file.name.clone(), source: file.src.clone() }
    }
}

/// Register the file that is run, then the files it imports in the order
/// reading order, so spans made relative to the first file remain valid for
/// each of them.
///
/// # Arguments
///
/// * `sm` - The source map.
/// * `name` - The name of the file that is run.
/// * `source` - Its text.
/// * `imports` - The imported files.
///
/// # Returns
///
/// The registered file that is run.
pub fn register_embedded(
    sm: &mut SourceMap, name: &FileName, source: &str, imports: &[EmbeddedFile],
) -> Arc<SourceFile>
{
    let root = sm.new_source_file(name.clone(), source.to_string());
    imports
        .iter()
        .for_each(|f| _ = sm.new_source_file(f.name.clone(), f.source.clone()));
    root
}

/// Magic prefix shared across all raw phase artifacts.
pub const RAW_ARTIFACT_MAGIC: &[u8; 4] = b"ZKAR";

/// Length of the fixed raw header part: magic, version, and phase name length.
const RAW_HEADER_PREFIX_LEN: usize = RAW_ARTIFACT_MAGIC.len() + 4 + 4;

/// Serialized form of a phase artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactFormat
{
    /// Human-readable text.
    Text,
    /// Raw u8s.
    Raw,
}

/// A compiler phase input or output.
pub trait PhaseArtifact: Sized
{
    /// Decode an artifact from a session input carrier.
    ///
    /// # Arguments
    ///
    /// * `format` - The format of the artifact.
    /// * `input` - The input to decode the artifact from.
    /// * `stdin` - The standard input to read from.
    /// * `sm` - The source map to use.
    ///
    /// # Returns
    ///
    /// The decoded artifact.
    fn decode(
        format: ArtifactFormat, input: &Input, stdin: &mut dyn BufRead,
        sm: &mut SourceMap,
    ) -> zakac_host_errors::Result<Self>;

    /// Encode this artifact into u8s.
    ///
    /// # Arguments
    ///
    /// * `format` - The format of the artifact.
    ///
    /// # Returns
    ///
    /// The encoded artifact.
    fn encode(&self, format: ArtifactFormat) -> zakac_host_errors::Result<Vec<u8>>;
}

/// Decode a phase artifact from `input`.
///
/// # Arguments
///
/// * `input` - The input to decode the artifact from.
/// * `format` - The format of the artifact.
/// * `stdin` - The standard input to read from.
/// * `sm` - The source map to use.
///
/// # Returns
///
/// The decoded artifact.
pub fn load_artifact<T: PhaseArtifact>(
    input: &Input, format: ArtifactFormat, stdin: &mut dyn BufRead, sm: &mut SourceMap,
) -> zakac_host_errors::Result<T>
{
    T::decode(format, input, stdin, sm)
}

/// Encode `value` and write it to `output`.
///
/// # Arguments
///
/// * `output` - The output to write the artifact to.
/// * `stdout` - The standard output to write to.
/// * `value` - The artifact to encode.
/// * `format` - The format of the artifact.
///
/// # Returns
///
/// The result of the operation.
pub fn dump_artifact<T: PhaseArtifact>(
    output: &mut Output, stdout: &mut dyn WriteColor, value: &T, format: ArtifactFormat,
) -> zakac_host_errors::Result<()>
{
    let content = value.encode(format)?;
    output.write(stdout, &content)
}

/// Encode a value as a raw artifact with the shared envelope.
///
/// # Arguments
///
/// * `phase` - Phase name stored in the file header.
/// * `version` - Artifact schema version.
/// * `value` - Value to serialize with Postcard after the header.
///
/// # Returns
///
/// The full raw artifact u8s.
pub fn encode_raw<T: Serialize>(
    phase: &str, version: u32, value: &T,
) -> zakac_host_errors::Result<Vec<u8>>
{
    let data = postcard::to_allocvec(value).map_err(raw_encode_err)?;
    let phase_u8s = phase.as_bytes();
    let mut out =
        Vec::with_capacity(RAW_HEADER_PREFIX_LEN + phase_u8s.len() + data.len());
    out.extend_from_slice(RAW_ARTIFACT_MAGIC);
    out.extend_from_slice(&version.to_le_bytes());
    out.extend_from_slice(&(phase_u8s.len() as u32).to_le_bytes());
    out.extend_from_slice(phase_u8s);
    out.extend(data);
    Ok(out)
}

/// Decode a raw artifact using the shared envelope.
///
/// # Arguments
///
/// * `phase` - Expected phase name in the file header.
/// * `version` - Expected artifact schema version.
/// * `content` - Full file contents.
/// * `path` - Diagnostic path for decode errors.
///
/// # Returns
///
/// The decoded value.
pub fn decode_raw<T: DeserializeOwned>(
    phase: &str, version: u32, content: &[u8], path: &Path,
) -> zakac_host_errors::Result<T>
{
    if content.len() < RAW_HEADER_PREFIX_LEN
    {
        return Err(artifact_decode_err(path, t!("session.artifact.truncated_raw")));
    }
    if content[..RAW_ARTIFACT_MAGIC.len()] != *RAW_ARTIFACT_MAGIC
    {
        return Err(artifact_decode_err(
            path,
            t!("session.artifact.invalid_raw_magic"),
        ));
    }

    let file_version = read_u32(path, &content[RAW_ARTIFACT_MAGIC.len()..])?;
    if file_version != version
    {
        let msg =
            t!("session.artifact.unsupported_version", v = file_version.to_string());
        return Err(artifact_decode_err(path, msg));
    }

    let phase_len = read_u32(path, &content[RAW_ARTIFACT_MAGIC.len() + 4..])? as usize;
    let phase_start = RAW_HEADER_PREFIX_LEN;
    let phase_end = phase_start + phase_len;
    if content.len() < phase_end
    {
        return Err(artifact_decode_err(
            path,
            t!("session.artifact.invalid_raw_header"),
        ));
    }

    let stored_phase =
        std::str::from_utf8(&content[phase_start..phase_end]).map_err(|_| {
            artifact_decode_err(path, t!("session.artifact.invalid_raw_header"))
        })?;
    if stored_phase != phase
    {
        let msg =
            t!("session.artifact.wrong_phase", got = stored_phase, expected = phase);
        return Err(artifact_decode_err(path, msg));
    }

    let res = postcard::from_bytes(&content[phase_end..]);
    ret_if!(let Ok(value) = res, Ok(value));
    let err = res.err().unwrap();
    Err(artifact_decode_err(
        path,
        t!("session.artifact.invalid_raw", e = err.to_string()),
    ))
}

/// Encode `value` as a text artifact: the header comment line, then YAML.
///
/// # Arguments
///
/// * `header` - The header comment line without the line break, such as `#
///   zakac-lex-artifact v1`.
/// * `value` - The value to serialize.
///
/// # Returns
///
/// The UTF-8 text artifact.
pub fn encode_text<T: Serialize>(
    header: &str, value: &T,
) -> zakac_host_errors::Result<Vec<u8>>
{
    // Block scalars (`|`) do not round-trip a string that is a lone line
    // break in serde-saphyr 1.2, so multi-line strings are written quoted.
    let options = serde_saphyr::ser_options! { prefer_block_scalars: false };
    let encoded = serde_saphyr::to_string_with_options(value, options)
        .map_err(|err| Error::InternalError { error: Box::new(err) })?;
    Ok(format!("{header}\n{encoded}").into_bytes())
}

/// Decode a text artifact: check the header comment line, then parse YAML.
///
/// # Arguments
///
/// * `header` - The expected header comment line without the line break.
/// * `content` - Full file contents.
/// * `path` - Diagnostic path for decode errors.
///
/// # Returns
///
/// The decoded value.
pub fn decode_text<T: DeserializeOwned>(
    header: &str, content: &[u8], path: &Path,
) -> zakac_host_errors::Result<T>
{
    let text = std::str::from_utf8(content).map_err(|err| {
        artifact_decode_err(
            path,
            t!("session.artifact.invalid_utf8", e = err.to_string()),
        )
    })?;
    let rest = text.strip_prefix(header).and_then(|after| after.strip_prefix('\n'));
    let Some(rest) = rest
    else
    {
        return Err(artifact_decode_err(
            path,
            t!("session.artifact.missing_text_header"),
        ));
    };
    serde_saphyr::from_str(rest).map_err(|err| {
        artifact_decode_err(
            path,
            t!("session.artifact.invalid_text", e = err.to_string()),
        )
    })
}

/// Read a little-endian `u32` from the start of `u8s`.
///
/// # Arguments
///
/// * `path` - The path to the file.
/// * `u8s` - The u8s to read the `u32` from.
///
/// # Returns
///
/// The read `u32`.
///
/// # Errors
///
/// Returns an error if the u8s is not long enough to read the `u32`.
fn read_u32(path: &Path, u8s: &[u8]) -> zakac_host_errors::Result<u32>
{
    let u8s: [u8; 4] = u8s
        .get(..4)
        .ok_or_else(|| {
            artifact_decode_err(path, t!("session.artifact.invalid_raw_header"))
        })?
        .try_into()
        .map_err(|_| {
            artifact_decode_err(path, t!("session.artifact.invalid_raw_header"))
        })?;
    Ok(u32::from_le_bytes(u8s))
}

/// Build a failed-to-decode-artifact error.
fn artifact_decode_err(path: &Path, msg: impl AsRef<str>) -> Error
{
    Error::FailedToDecodeArtifact {
        path: path.to_path_buf(),
        msg: msg.as_ref().to_string(),
    }
}

/// Wrap a Postcard encode failure as an internal error.
///
/// # Arguments
///
/// * `err` - The Postcard encode error.
///
/// # Returns
///
/// The wrapped error.
fn raw_encode_err(err: postcard::Error) -> Error
{
    Error::InternalError { error: Box::new(err) }
}

/// Decode a UTF-8 source from an input.
///
/// # Arguments
///
/// * `input` - The input to decode the source from.
/// * `stdin` - The standard input to read from.
/// * `sm` - The source map to use.
///
/// # Returns
///
/// The decoded source file.
fn decode_utf_src(
    input: &Input, stdin: &mut dyn BufRead, sm: &mut SourceMap,
) -> zakac_host_errors::Result<Arc<SourceFile>>
{
    ret_if!(let Input::File(path) = input, decode_utf8_source_from_file(path, stdin, sm));

    let data_name = input.file_name();
    let path = input.error_path();
    ret_if!(let Input::Stdin = input, decode_utf8_source_from_stdin(stdin, &data_name, &path, sm));
    extract!(Input::Data { content, .. } = input);
    decode_utf8_source_from_data(content, &data_name, &path, sm)
}

/// Decode a UTF-8 source from a file.
///
/// # Arguments
///
/// * `path` - The path to the file.
/// * `stdin` - The standard input to read from.
/// * `sm` - The source map to use.
///
/// # Returns
///
/// The decoded artifact.
fn decode_utf8_source_from_file(
    path: &PathBuf, _stdin: &mut dyn BufRead, sm: &mut SourceMap,
) -> zakac_host_errors::Result<Arc<SourceFile>>
{
    let result = sm.load_file(path.as_path());
    ret_if!(let Ok(file) = result, Ok(file));
    let ctx = IOErrorContext::new(path.clone(), IOErrorAction::Read);
    Err(Error::from_io(result.err().unwrap(), ctx))
}

/// Decode a UTF-8 source from data.
///
/// # Arguments
///
/// * `data` - The data to decode the source from.
/// * `sm` - The source map to use.
///
/// # Returns
///
/// The decoded source file.
fn decode_utf8_source_from_data(
    data: &[u8], filename: &FileName, path: &PathBuf, sm: &mut SourceMap,
) -> zakac_host_errors::Result<Arc<SourceFile>>
{
    let res = String::from_utf8(data.to_vec());
    ret_if!(let Ok(text) = res, Ok(sm.new_source_file(filename.clone(), text)));

    let ctx = IOErrorContext::new(path.clone(), IOErrorAction::Read);
    let source =
        std::io::Error::new(std::io::ErrorKind::InvalidData, res.err().unwrap());
    Err(Error::from_io(source, ctx))
}

/// Decode a UTF-8 source from stdin.
///
/// # Arguments
///
/// * `stdin` - The standard input to read from.
/// * `sm` - The source map to use.
///
/// # Returns
///
/// The decoded source file.
fn decode_utf8_source_from_stdin(
    stdin: &mut dyn BufRead, filename: &FileName, path: &PathBuf, sm: &mut SourceMap,
) -> zakac_host_errors::Result<Arc<SourceFile>>
{
    let mut content = String::new();
    let res = stdin.read_to_string(&mut content);
    ret_if!(let Ok(_) = res, Ok(sm.new_source_file(filename.clone(), content)));

    let ctx = IOErrorContext::new(path.clone(), IOErrorAction::Read);
    let source =
        std::io::Error::new(std::io::ErrorKind::InvalidData, res.err().unwrap());
    Err(Error::from_io(source, ctx))
}

impl PhaseArtifact for Arc<SourceFile>
{
    /// Decode an artifact from a session input carrier into a source file.
    ///
    /// # Arguments
    ///
    /// * `format` - The format of the artifact.
    /// * `input` - The input to decode the artifact from.
    /// * `stdin` - The standard input to read from.
    /// * `sm` - The source map to use.
    ///
    /// # Returns
    ///
    /// The decoded source file.
    fn decode(
        format: ArtifactFormat, input: &Input, stdin: &mut dyn BufRead,
        sm: &mut SourceMap,
    ) -> zakac_host_errors::Result<Self>
    {
        ret_if!(format == ArtifactFormat::Text, decode_utf_src(input, stdin, sm));
        Err(Error::UnsupportedArtifactFormat { path: input.error_path() })
    }

    /// Encode this source file into artifact.
    ///
    /// # Arguments
    ///
    /// * `format` - The format of the artifact.
    ///
    /// # Returns
    ///
    /// The encoded source file.
    fn encode(&self, format: ArtifactFormat) -> zakac_host_errors::Result<Vec<u8>>
    {
        ret_if!(format == ArtifactFormat::Text, Ok(self.src.as_bytes().to_vec()));
        let path = PathBuf::from(self.name.virtual_path_string());
        Err(Error::UnsupportedArtifactFormat { path })
    }
}
