use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use zakac_ast::stmt::Program;
use zakac_ast::visit::VisitorMut;
use zakac_diagnostics::Diagnostic;
use zakac_host_errors::Error;
use zakac_i18n::t;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
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

use crate::parser::parse;

const TEXT_HEADER: &str = "# zakac-parse-artifact v1";
const RAW_PHASE: &str = "parse";
const ARTIFACT_VERSION: u32 = 1;

/// Reloadable output of the parse phase. Spans are relative to the file
/// start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct ParseArtifact
{
    /// Diagnostic file name for the embedded source.
    pub name: FileName,
    /// Normalized source text the spans refer to.
    pub source: String,
    /// The tree.
    pub program: Program,
    /// Lexer then parser diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

/// Moves spans with a fixed function.
struct MapSpans<F: Fn(Span) -> Span>(F);

impl<F: Fn(Span) -> Span> VisitorMut for MapSpans<F>
{
    fn visit_span_mut(&mut self, span: &mut Span) { *span = (self.0)(*span) }
}

impl ParseArtifact
{
    /// Lex and parse a loaded source file into an artifact.
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
        let lexed = lex(source, options);
        let parsed = parse(source, &lexed);
        let mut diagnostics = lexed.diagnostics;
        diagnostics.extend(parsed.diagnostics);
        Self::from_output(source, parsed.program, diagnostics)
    }

    /// Build an artifact from a tree and diagnostics with absolute spans.
    ///
    /// # Arguments
    ///
    /// * `source` - The source file of the tree and diagnostics.
    /// * `program` - The tree.
    /// * `diagnostics` - The lexer and parser diagnostics.
    ///
    /// # Returns
    ///
    /// The artifact, with spans made relative.
    pub fn from_output(
        source: &SourceFile, mut program: Program, mut diagnostics: Vec<Diagnostic>,
    ) -> Self
    {
        let base = source.start_pos;
        let shift = |span: Span| relative(span, base);
        MapSpans(shift).visit_program_mut(&mut program);
        diagnostics.iter_mut().for_each(|d| d.span = shift(d.span));
        #[rustfmt::skip]
        return ParseArtifact::builder().name(source.name.clone()).source(source.src.clone())
            .program(program).diagnostics(diagnostics).build();
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

    /// The tree and diagnostics with spans made absolute against `source`.
    ///
    /// # Arguments
    ///
    /// * `source` - The registered source file.
    ///
    /// # Returns
    ///
    /// The tree and the diagnostics.
    pub fn absolute_output(&self, source: &SourceFile) -> (Program, Vec<Diagnostic>)
    {
        let base = source.start_pos;
        let shift = |span: Span| absolute(span, base);
        let mut program = self.program.clone();
        MapSpans(shift).visit_program_mut(&mut program);
        let mut diagnostics = self.diagnostics.clone();
        diagnostics.iter_mut().for_each(|d| d.span = shift(d.span));
        (program, diagnostics)
    }
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

impl PhaseArtifact for ParseArtifact
{
    /// Decode a parse artifact from an input.
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

    /// Encode a parse artifact.
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
) -> zakac_host_errors::Result<ParseArtifact>
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
    artifact: &ParseArtifact, source_map: &mut SourceMap, path: &Path,
) -> zakac_host_errors::Result<()>
{
    let registered = artifact.register_source(source_map);
    ret_if!(registered.src == artifact.source, Ok(()));
    let msg = t!("parser.artifact.source_changed").to_string();
    Err(Error::FailedToDecodeArtifact { path: PathBuf::from(path), msg })
}
