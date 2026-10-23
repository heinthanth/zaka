use std::path::Path;
use std::path::PathBuf;

use clap::ArgMatches;
use zakac_errors::Error;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::config::Input;
use zakac_session::config::Output;

/// Which artifact formats a phase can write and how paths are interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseArtifactOutput
{
    /// Formats this phase can emit (for example `[ArtifactFormat::Text,
    /// ArtifactFormat::Raw]`).
    pub formats: &'static [ArtifactFormat],
    /// Default format when the path does not select another supported format.
    pub default_format: ArtifactFormat,
    /// Raw file extension including the dot (for example `.zksegments`).
    pub raw_extension: &'static str,
    /// Text file extension including the dot (for example `.txt`).
    pub text_extension: &'static str,
}

/// Resolved output destination and artifact format.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedArtifactOutput
{
    /// Where to write the artifact.
    pub output: Output,
    /// How to encode the artifact.
    pub format: ArtifactFormat,
}

/// Read `--output-format` from parsed CLI matches.
///
/// # Arguments
///
/// * `matches` - Parsed argument matches.
///
/// # Returns
///
/// `Some` when the flag was passed, otherwise `None`.
pub fn output_format_from_matches(
    matches: &ArgMatches, caps: &PhaseArtifactOutput,
) -> Option<ArtifactFormat>
{
    matches.get_one::<String>("output_format").map(|value| match value.as_str()
    {
        "text" => ArtifactFormat::Text,
        "raw" => ArtifactFormat::Raw,
        _ => caps.default_format,
    })
}

/// Resolve the artifact output path and format from CLI arguments.
///
/// # Arguments
///
/// * `input` - Session input (for basename when writing into a dir).
/// * `output_arg` - Value of `-o` / `--output`, if set.
/// * `format_flag` - Explicit `--output-format`, if set.
/// * `caps` - Formats this phase supports.
///
/// # Returns
///
/// The resolved destination and format, or an error when the requested format
/// is not supported.
pub fn resolve_artifact_output(
    input: &Input, output_arg: Option<&str>, format_flag: Option<ArtifactFormat>,
    caps: &PhaseArtifactOutput,
) -> zakac_errors::Result<ResolvedArtifactOutput>
{
    ret_if!(let Some(format) = format_flag, resolve_with_exp_format(input, output_arg, format, caps));
    resolve_without_format_flag(input, output_arg, caps)
}

/// Resolve output when `--output-format` was set.
///
/// # Arguments
///
/// * `input` - Session input (for basename when writing into a dir).
/// * `output_arg` - Value of `-o` / `--output`, if set.
/// * `format` - Explicit `--output-format` value.
/// * `caps` - Formats this phase supports.
///
/// # Returns
///
/// A `ResolvedArtifactOutput` with the resolved output path and format.
fn resolve_with_exp_format(
    input: &Input, output_arg: Option<&str>, format: ArtifactFormat,
    caps: &PhaseArtifactOutput,
) -> zakac_errors::Result<ResolvedArtifactOutput>
{
    ensure_format_supported(format, caps, output_arg)?;
    let output = build_output_path(input, output_arg, format, caps);
    Ok(ResolvedArtifactOutput { output, format })
}

/// Resolve output when format comes from path hints and phase defaults.
fn resolve_without_format_flag(
    input: &Input, output_arg: Option<&str>, caps: &PhaseArtifactOutput,
) -> zakac_errors::Result<ResolvedArtifactOutput>
{
    let format = infer_format(output_arg, caps);
    let output = build_output_path(input, output_arg, format, caps);
    Ok(ResolvedArtifactOutput { output, format })
}

/// Infer artifact format from the output path and phase capabilities.
///
/// # Arguments
///
/// * `output_arg` - Value of `-o` / `--output`.
/// * `caps` - Formats this phase supports.
///
/// # Returns
///
/// The inferred `ArtifactFormat`.
fn infer_format(output_arg: Option<&str>, caps: &PhaseArtifactOutput)
-> ArtifactFormat
{
    ret_if!(let Some(format) = sole_supported_format(caps), format);
    ret_if!(output_is_stdout(output_arg), stdout_default_format(caps));

    let Some(arg) = output_arg
    else
    {
        return phase_default_format(caps);
    };
    let path = Path::new(arg);
    ret_if!(is_path_dir(path, arg), phase_default_format(caps));
    format_from_file_extension(path, caps)
}

/// When the phase supports just one format, just use it.
///
/// # Arguments
///
/// * `caps` - Formats this phase supports.
///
/// # Returns
///
/// The sole supported `ArtifactFormat`, if set.
fn sole_supported_format(caps: &PhaseArtifactOutput) -> Option<ArtifactFormat>
{
    tern!(caps.formats.len() == 1, Some(caps.formats[0]), None)
}

/// Default artifact format.
///
/// # Arguments
///
/// * `caps` - Formats this phase supports.
///
/// # Returns
///
/// The default `ArtifactFormat` for this phase.
fn phase_default_format(caps: &PhaseArtifactOutput) -> ArtifactFormat
{
    caps.default_format
}

/// Default format for stdout when more than one format is available.
///
/// # Arguments
///
/// * `caps` - Formats this phase supports.
///
/// # Returns
///
/// The default `ArtifactFormat` for this phase.
fn stdout_default_format(caps: &PhaseArtifactOutput) -> ArtifactFormat
{
    ret_if!(supports_format(caps, ArtifactFormat::Text), ArtifactFormat::Text);
    phase_default_format(caps)
}

/// Whether the phase can write `format`.
///
/// # Arguments
///
/// * `caps` - The phase's artifact output capabilities.
/// * `format` - The format to check.
///
/// # Returns
///
/// `true` if the phase supports `format`, `false` otherwise.
fn supports_format(caps: &PhaseArtifactOutput, format: ArtifactFormat) -> bool
{
    caps.formats.contains(&format)
}

/// Map a file extension to an artifact format when the phase supports both.
///
/// # Arguments
///
/// * `path` - The file path to map.
/// * `caps` - The phase's artifact output capabilities.
///
/// # Returns
///
/// The `ArtifactFormat` corresponding to `path`.
fn format_from_file_extension(path: &Path, caps: &PhaseArtifactOutput)
-> ArtifactFormat
{
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    ret_if!(ext.is_empty(), caps.default_format);
    if extension_matches_text(ext, caps) && supports_format(caps, ArtifactFormat::Text)
    {
        return ArtifactFormat::Text;
    }
    ret_if!(extension_matches_raw(ext, caps), ArtifactFormat::Raw);
    phase_default_format(caps)
}

/// Whether `ext` is this phase's raw artifact extension (without the dot).
///
/// # Arguments
///
/// * `ext` - The file extension to check.
/// * `caps` - The phase's artifact output capabilities.
///
/// # Returns
///
/// `true` if `ext` matches the raw extension, `false` otherwise.
fn extension_matches_raw(ext: &str, caps: &PhaseArtifactOutput) -> bool
{
    extension_without_dot(caps.raw_extension) == ext
}

/// Whether `ext` is this phase's text artifact extension (without the dot).
///
/// # Arguments
///
/// * `ext` - The file extension to check.
/// * `caps` - The phase's artifact output capabilities.
///
/// # Returns
///
/// `true` if `ext` matches the text extension, `false` otherwise.
fn extension_matches_text(ext: &str, caps: &PhaseArtifactOutput) -> bool
{
    extension_without_dot(caps.text_extension) == ext
}

/// Strip a leading dot from a phase extension constant.
///
/// # Arguments
///
/// * `extension` - The extension to strip.
///
/// # Returns
///
/// The extension with the leading dot stripped, or the original if no dot was
/// present.
fn extension_without_dot(extension: &str) -> &str
{
    extension.strip_prefix('.').unwrap_or(extension)
}

/// Build the final [`Output`] for a resolved format.
///
/// # Arguments
///
/// * `input` - The input to the phase.
/// * `output_arg` - The output path argument, if set.
/// * `format` - The resolved artifact format.
/// * `caps` - The phase's artifact output capabilities.
///
/// # Returns
///
/// The resolved [`Output`] path.
fn build_output_path(
    input: &Input, output_arg: Option<&str>, format: ArtifactFormat,
    caps: &PhaseArtifactOutput,
) -> Output
{
    ret_if!(output_is_stdout(output_arg), Output::Stdout);

    let Some(arg) = output_arg
    else
    {
        return Output::Stdout;
    };
    let path = PathBuf::from(arg);

    ret_if!(!is_path_dir(&path, arg), Output::File(path));

    let base = input_basename(input);
    let ext = match format
    {
        ArtifactFormat::Raw => caps.raw_extension,
        ArtifactFormat::Text => caps.text_extension,
    };
    return Output::File(path.join(format!("{base}{ext}")));
}

/// Basename used when expanding a dir output path.
///
/// # Arguments
///
/// * `input` - The input to the phase.
///
/// # Returns
///
/// The basename of the input, suitable for use in artifact output paths.
fn input_basename(input: &Input) -> String
{
    match input
    {
        Input::Stdin => "stdin".to_string(),
        Input::File(path) => input_basename_from_file(path),
        Input::Data { name, .. } => input_basename_from_data(name),
    }
}

/// Returns the basename of a file path, suitable for use in artifact output
/// paths.
///
/// # Arguments
///
/// * `path` - The file path.
///
/// # Returns
///
/// The basename of the file path.
fn input_basename_from_file(path: &Path) -> String
{
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "input".to_string())
}

/// Returns the basename of a data input, suitable for use in artifact output
/// paths.
///
/// # Arguments
///
/// * `name` - The data input name.
///
/// # Returns
///
/// The basename of the data input.
fn input_basename_from_data(name: &str) -> String
{
    Path::new(name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "input".to_string())
}

/// Whether the CLI output target is stdout.
///
/// # Arguments
///
/// * `output_arg` - The output path argument, if set.
///
/// # Returns
///
/// `true` if the output target is stdout, `false` otherwise.
fn output_is_stdout(output_arg: Option<&str>) -> bool
{
    matches!(output_arg, None | Some("-"))
}

/// Whether the path `arg` is path to a folder.
///
/// # Arguments
///
/// * `path` - The path to check.
/// * `arg` - The original argument string.
///
/// # Returns
///
/// `true` if the path is a folder or ends with slash,
/// `false` otherwise.
fn is_path_dir(path: &Path, arg: &str) -> bool
{
    path.is_dir() || arg.ends_with('/') || arg.ends_with('\\')
}

/// Fail when the user requested a format this phase cannot write.
///
/// # Arguments
///
/// * `format` - The format from the user.
/// * `caps` - The capabilities of the phase.
/// * `output_arg` - The output argument from the user.
///
/// # Returns
///
/// An error if the format is not supported.
fn ensure_format_supported(
    format: ArtifactFormat, caps: &PhaseArtifactOutput, output_arg: Option<&str>,
) -> zakac_errors::Result<()>
{
    let path = output_error_path(output_arg);
    let supported = supports_format(caps, format);
    tern!(supported, Ok(()), Err(Error::UnsupportedArtifactFormat { path }))
}

/// Diagnostic path for unsupported-format errors.
///
/// # Arguments
///
/// * `output_arg` - The output argument from the user.
///
/// # Returns
///
/// The path to use for diagnostic output.
fn output_error_path(output_arg: Option<&str>) -> PathBuf
{
    ret_if!(let Some(o) = output_arg && o != "-", PathBuf::from(o));
    PathBuf::from("<stdout>")
}
