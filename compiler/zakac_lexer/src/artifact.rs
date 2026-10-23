use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use zakac_diagnostics::Diagnostic;
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

use crate::lexer::LexerOptions;
use crate::lexer::lex;
use crate::token::Comment;
use crate::token::LexOutput;
use crate::token::StrLit;
use crate::token::Token;

const TEXT_HEADER: &str = "# zakac-lex-artifact v1";
const RAW_PHASE: &str = "lex";
const ARTIFACT_VERSION: u32 = 1;

/// Reloadable output of the lex phase. Spans are relative to the file start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct LexArtifact
{
    /// Diagnostic file name for the embedded source.
    pub name: FileName,
    /// Normalized source text the spans refer to.
    pub source: String,
    /// Tokens, ending with the end-of-input token.
    pub tokens: Vec<Token>,
    /// Comments.
    pub comments: Vec<Comment>,
    /// Decoded string literals.
    pub strings: Vec<StrLit>,
    /// Lexer diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

impl LexArtifact
{
    /// Lex a loaded source file into an artifact.
    ///
    /// # Arguments
    ///
    /// * `source` - The source file.
    /// * `options` - The lexer options.
    ///
    /// # Returns
    ///
    /// The artifact.
    pub fn from_source(source: &SourceFile, options: &LexerOptions) -> Self
    {
        Self::from_output(source, lex(source, options))
    }

    /// Build an artifact from lexer output with absolute spans.
    ///
    /// # Arguments
    ///
    /// * `source` - The source file the output belongs to.
    /// * `out` - The lexer output.
    ///
    /// # Returns
    ///
    /// The artifact, with spans made relative.
    pub fn from_output(source: &SourceFile, out: LexOutput) -> Self
    {
        let base = source.start_pos;
        let shifted = map_spans(out, |span| relative(span, base));
        LexArtifact::builder()
            .name(source.name.clone())
            .source(source.src.clone())
            .tokens(shifted.tokens)
            .comments(shifted.comments)
            .strings(shifted.strings)
            .diagnostics(shifted.diagnostics)
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

    /// The lexer output with spans made absolute against `source`.
    ///
    /// # Arguments
    ///
    /// * `source` - The registered source file.
    ///
    /// # Returns
    ///
    /// The lexer output.
    pub fn absolute_output(&self, source: &SourceFile) -> LexOutput
    {
        let base = source.start_pos;
        let out = LexOutput::builder()
            .tokens(self.tokens.clone())
            .comments(self.comments.clone())
            .strings(self.strings.clone())
            .diagnostics(self.diagnostics.clone())
            .build();
        map_spans(out, |span| absolute(span, base))
    }
}

/// Run `map` on each span in `out`.
///
/// # Arguments
///
/// * `out` - The lexer output.
/// * `map` - The span mapping.
///
/// # Returns
///
/// The output with mapped spans.
fn map_spans(mut out: LexOutput, map: impl Fn(Span) -> Span) -> LexOutput
{
    out.tokens.iter_mut().for_each(|tok| tok.span = map(tok.span));
    out.comments.iter_mut().for_each(|c| c.span = map(c.span));
    out.diagnostics.iter_mut().for_each(|d| d.span = map(d.span));
    out
}

/// Make an absolute span relative to `base`.
///
/// # Arguments
///
/// * `span` - The absolute span.
/// * `base` - The file start.
///
/// # Returns
///
/// The relative span.
fn relative(span: Span, base: Pos) -> Span
{
    Span::new(Pos::new(span.lo - base), Pos::new(span.hi - base))
}

/// Make a relative span absolute against `base`.
///
/// # Arguments
///
/// * `span` - The relative span.
/// * `base` - The file start.
///
/// # Returns
///
/// The absolute span.
fn absolute(span: Span, base: Pos) -> Span
{
    Span::new(base + span.lo.as_u32() as usize, base + span.hi.as_u32() as usize)
}

impl PhaseArtifact for LexArtifact
{
    /// Decode a lex artifact from an input.
    ///
    /// # Arguments
    ///
    /// * `format` - The format of the artifact.
    /// * `input` - The input to decode the artifact from.
    /// * `stdin` - The standard input to read from.
    /// * `source_map` - The source map to register the embedded source in.
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
        check_source(&artifact, source_map, &path)?;
        Ok(artifact)
    }

    /// Encode a lex artifact.
    ///
    /// # Arguments
    ///
    /// * `format` - The format of the artifact.
    ///
    /// # Returns
    ///
    /// The encoded artifact.
    fn encode(&self, format: ArtifactFormat) -> zakac_host_errors::Result<Vec<u8>>
    {
        ret_if!(format == ArtifactFormat::Text, encode_text(TEXT_HEADER, self));
        encode_raw(RAW_PHASE, ARTIFACT_VERSION, self)
    }
}

/// Create a decode error.
///
/// # Arguments
///
/// * `path` - The path to the artifact.
/// * `msg` - The message.
///
/// # Returns
///
/// The error.
fn decode_err(path: impl Into<PathBuf>, msg: impl AsRef<str>) -> Error
{
    Error::FailedToDecodeArtifact { path: path.into(), msg: msg.as_ref().to_string() }
}

/// Decode an artifact in the given format.
///
/// # Arguments
///
/// * `content` - The encoded artifact.
/// * `path` - The path to the artifact.
/// * `format` - The format.
///
/// # Returns
///
/// The decoded artifact.
fn decode_artifact(
    content: &[u8], path: &Path, format: ArtifactFormat,
) -> zakac_host_errors::Result<LexArtifact>
{
    ret_if!(format == ArtifactFormat::Text, decode_text(TEXT_HEADER, content, path));
    decode_raw(RAW_PHASE, ARTIFACT_VERSION, content, path)
}

/// Check that the embedded source survives re-ingest unchanged.
///
/// # Arguments
///
/// * `artifact` - The artifact.
/// * `source_map` - The source map to register the source in.
/// * `path` - The path to the artifact.
///
/// # Returns
///
/// `Ok` when the source is unchanged.
fn check_source(
    artifact: &LexArtifact, source_map: &mut SourceMap, path: &Path,
) -> zakac_host_errors::Result<()>
{
    let registered = artifact.register_source(source_map);
    ret_if!(registered.src == artifact.source, Ok(()));
    Err(decode_err(path, t!("lexer.artifact.source_changed")))
}
