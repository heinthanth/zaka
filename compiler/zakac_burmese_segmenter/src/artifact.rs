use std::io::BufRead;
use std::path::PathBuf;
use std::sync::Arc;

use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use zakac_host_errors::Error;
use zakac_i18n::t;
use zakac_rust_sugars::ret_if;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::PhaseArtifact;
use zakac_session::artifact::decode_raw;
use zakac_session::artifact::decode_text;
use zakac_session::artifact::encode_raw;
use zakac_session::artifact::encode_text;
use zakac_session::config::Input;
use zakac_span::filename::FileName;
use zakac_span::pos::Pos;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

use crate::segmenter::PreLexicalToken;
use crate::segmenter::PreLexicalTokenKind;
use crate::segmenter::SegmenterOptions;
use crate::segmenter::segment_at;

const TEXT_HEADER: &str = "# zakac-segment-artifact v1";
const RAW_PHASE: &str = "segment";
const ARTIFACT_VERSION: u32 = 1;

/// Owned pre-lexical token with relative spans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct OwnedPreLexicalToken
{
    /// Kind of the token.
    pub kind: PreLexicalTokenKind,
    /// Lexeme of the token.
    pub lexeme: String,
    /// Span relative to the source file start.
    pub span: Span,
}

/// Reloadable output of the segment phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct SegmentArtifact
{
    /// Diagnostic file name for the embedded source.
    pub name: FileName,
    /// Normalized source text the spans refer to.
    pub source: String,
    /// Segmented tokens.
    pub tokens: Vec<OwnedPreLexicalToken>,
}

impl SegmentArtifact
{
    /// Build an artifact from a loaded source file and segmenter options.
    ///
    /// # Arguments
    ///
    /// * `source` - The source file to build the artifact from.
    /// * `options` - The segmenter options to use.
    ///
    /// # Returns
    ///
    /// The built artifact.
    pub fn from_source(source: &SourceFile, options: &SegmenterOptions) -> Self
    {
        let base = source.start_pos;
        let tokens = segment_at(&source.src, base, options)
            .into_iter()
            .map(|token| token_to_relative_owned_token(token, source))
            .collect();
        SegmentArtifact::builder()
            .name(source.name.clone())
            .source(source.src.clone())
            .tokens(tokens)
            .build()
    }

    /// Register the embedded source in `source_map` and return the file.
    ///
    /// # Arguments
    ///
    /// * `source_map` - The source map to register the source in.
    ///
    /// # Returns
    ///
    /// The registered source file.
    pub fn register_source(&self, source_map: &mut SourceMap) -> Arc<SourceFile>
    {
        source_map.new_source_file(self.name.clone(), self.source.clone())
    }

    /// Absolute spans for tokens against `source`.
    ///
    /// # Arguments
    ///
    /// * `source` - The source file to get the absolute spans for.
    ///
    /// # Returns
    ///
    /// The absolute spans for tokens against `source`.
    pub fn absolute_tokens(&self, source: &SourceFile) -> Vec<OwnedPreLexicalToken>
    {
        self.tokens
            .iter()
            .map(|token| token_to_absolute_owned_token(token, source))
            .collect()
    }
}

/// Convert a pre-lexical token to an owned pre-lexical token.
///
/// # Arguments
///
/// * `token` - The pre-lexical token to convert.
/// * `src` - The source file the token belongs to.
///
/// # Returns
///
/// The owned pre-lexical token.
fn token_to_relative_owned_token(
    token: PreLexicalToken, src: &SourceFile,
) -> OwnedPreLexicalToken
{
    let rel_lo = src.relative_position(&token.span.lo);
    let rel_hi = src.relative_position(&token.span.hi);
    OwnedPreLexicalToken::builder()
        .kind(token.kind)
        .lexeme(token.lexeme.to_string())
        .span(Span::new(Pos::new(rel_lo.as_u32()), Pos::new(rel_hi.as_u32())))
        .build()
}

/// Convert a pre-lexical token to an absolute owned pre-lexical token.
///
/// # Arguments
///
/// * `token` - The pre-lexical token to convert.
/// * `src` - The source file the token belongs to.
///
/// # Returns
///
/// The absolute owned pre-lexical token.
fn token_to_absolute_owned_token(
    token: &OwnedPreLexicalToken, src: &SourceFile,
) -> OwnedPreLexicalToken
{
    let base = src.start_pos;
    let lo = base.advanced(token.span.lo.as_u32() as usize);
    let hi = base.advanced(token.span.hi.as_u32() as usize);
    OwnedPreLexicalToken::builder()
        .kind(token.kind)
        .lexeme(token.lexeme.clone())
        .span(Span::new(lo, hi))
        .build()
}

impl PhaseArtifact for SegmentArtifact
{
    /// Decode a segment artifact from an input.
    ///
    /// # Arguments
    ///
    /// * `format` - The format of the artifact.
    /// * `input` - The input to decode the artifact from.
    /// * `stdin` - The standard input to read from.
    /// * `source_map` - The source map to use.
    ///
    /// # Returns
    ///
    /// The decoded artifact.
    fn decode(
        format: ArtifactFormat, input: &Input, stdin: &mut dyn BufRead,
        source_map: &mut SourceMap,
    ) -> zakac_host_errors::Result<Self>
    {
        let path = input.error_path();
        let (_, content) = input.read(stdin)?;
        let artifact = decode_artifact(&content, &path, format)?;
        run_src_verification(&artifact, source_map, &path)?;
        Ok(artifact)
    }

    /// Encode a segment artifact into u8s.
    ///
    /// # Arguments
    ///
    /// * `format` - The format of the artifact.
    ///
    /// # Returns
    ///
    /// The encoded artifact.
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact cannot be encoded.
    fn encode(&self, format: ArtifactFormat) -> zakac_host_errors::Result<Vec<u8>>
    {
        encode_artifact(self, format)
    }
}

/// Create a decode error.
///
/// # Arguments
///
/// * `path` - The path to the artifact.
/// * `msg` - The message to create the error with.
///
/// # Returns
///
/// The created error.
fn decode_err(path: impl Into<PathBuf>, msg: impl AsRef<str>) -> Error
{
    Error::FailedToDecodeArtifact { path: path.into(), msg: msg.as_ref().to_string() }
}

/// Encode an artifact into given format.
///
/// # Arguments
///
/// * `artifact` - The artifact to encode.
/// * `format` - The format to encode the artifact into.
///
/// # Returns
///
/// The encoded artifact.
fn encode_artifact(
    artifact: &SegmentArtifact, format: ArtifactFormat,
) -> zakac_host_errors::Result<Vec<u8>>
{
    ret_if!(format == ArtifactFormat::Text, encode_text(TEXT_HEADER, artifact));
    encode_raw_artifact(artifact)
}

/// Decode a segment artifact from given format.
///
/// # Arguments
///
/// * `content` - The content to decode the artifact from.
/// * `path` - The path to the artifact.
/// * `format` - The format to decode the artifact from.
///
/// # Returns
///
/// The decoded artifact.
fn decode_artifact(
    content: &[u8], path: &std::path::Path, format: ArtifactFormat,
) -> zakac_host_errors::Result<SegmentArtifact>
{
    ret_if!(format == ArtifactFormat::Text, decode_text(TEXT_HEADER, content, path));
    decode_raw_artifact(content, path)
}

/// Decode a segment artifact from raw u8s.
///
/// # Arguments
///
/// * `content` - The content to decode the artifact from.
/// * `path` - The path to the artifact.
///
/// # Returns
///
/// The decoded artifact.
///
/// # Errors
///
/// Returns an error if the artifact cannot be decoded.
fn decode_raw_artifact(
    content: &[u8], path: &std::path::Path,
) -> zakac_host_errors::Result<SegmentArtifact>
{
    decode_raw(RAW_PHASE, ARTIFACT_VERSION, content, path)
}

/// Encode a segment artifact into raw u8s.
///
/// # Arguments
///
/// * `artifact` - The artifact to encode.
///
/// # Returns
///
/// The encoded artifact.
///
/// # Errors
///
/// Returns an error if the artifact cannot be encoded.
fn encode_raw_artifact(artifact: &SegmentArtifact)
-> zakac_host_errors::Result<Vec<u8>>
{
    encode_raw(RAW_PHASE, ARTIFACT_VERSION, artifact)
}

/// Run the source verification.
///
/// # Arguments
///
/// * `artifact` - The artifact to check.
/// * `source_map` - The source map to use.
/// * `path` - The path to the artifact.
///
/// # Returns
///
/// The result of the verification.
fn run_src_verification(
    artifact: &SegmentArtifact, source_map: &mut SourceMap, path: &std::path::Path,
) -> zakac_host_errors::Result<()>
{
    let registered = artifact.register_source(source_map);
    ret_if!(registered.src == artifact.source, Ok(()));
    Err(decode_err(path, t!("segmenter.artifact.source_changed")))
}
