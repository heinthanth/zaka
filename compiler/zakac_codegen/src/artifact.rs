use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::has_errors;
use zakac_host_errors::Error;
use zakac_i18n::t;
use zakac_ir_builder::artifact::Lowered;
use zakac_ir_builder::artifact::absolute;
use zakac_ir_builder::artifact::relative;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::PhaseArtifact;
use zakac_session::artifact::decode_raw;
use zakac_session::artifact::decode_text;
use zakac_session::artifact::encode_raw;
use zakac_session::artifact::encode_text;
use zakac_session::config::Input;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

use crate::bc::Module;
use crate::emit::emit;

/// The first line of a text artifact.
pub const TEXT_HEADER: &str = "# zakac-codegen-artifact v1";
/// The phase name in a raw artifact.
pub const RAW_PHASE: &str = "codegen";
/// The schema version of the artifact.
pub const ARTIFACT_VERSION: u32 = 1;

/// The output of the phases up to code generation.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct Emitted
{
    /// The output of the phases up to lowering.
    pub lowered: Lowered,
    /// The ops, or `None` when an earlier phase reported an error.
    pub module: Option<Module>,
}

impl Emitted
{
    /// The diagnostics of each phase.
    ///
    /// # Returns
    ///
    /// The diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] { &self.lowered.diagnostics }
}

/// Run each phase from lexing to code generation. Code generation runs
/// when no earlier phase reported an error.
///
/// # Arguments
///
/// * `source` - The source file.
/// * `lexer` - The lexer options.
/// * `options` - The check options.
/// * `folding` - The foldings to run.
///
/// # Returns
///
/// The output of each phase.
pub fn run_phases(
    source: &SourceFile, lexer: &LexerOptions, options: &SemcheckOptions,
    folding: &FoldOptions,
) -> Emitted
{
    let lowered =
        zakac_ir_builder::artifact::run_phases(source, lexer, options, folding);
    let clean = !has_errors(&lowered.diagnostics);
    let bits = options.platform_bits;
    let module = lowered.folded.as_ref().filter(|_| clean).map(|m| emit(m, bits));
    Emitted::builder().lowered(lowered).maybe_module(module).build()
}

/// The code generation artifact: the source, the ops and the diagnostics.
/// Spans are relative to the start of the file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct CodegenArtifact
{
    /// The source file name.
    pub name: FileName,
    /// The normalized source text the spans refer to.
    pub source: String,
    /// The ops, or `None` when an earlier phase reported an error.
    pub module: Option<Module>,
    /// The diagnostics of each phase.
    pub diagnostics: Vec<Diagnostic>,
}

impl CodegenArtifact
{
    /// Run each phase on a source file and keep the output.
    ///
    /// # Arguments
    ///
    /// * `source` - The source file.
    /// * `lexer` - The lexer options.
    /// * `options` - The check options.
    /// * `folding` - The foldings to run.
    ///
    /// # Returns
    ///
    /// The artifact.
    pub fn from_source(
        source: &SourceFile, lexer: &LexerOptions, options: &SemcheckOptions,
        folding: &FoldOptions,
    ) -> Self
    {
        Self::from_emitted(source, run_phases(source, lexer, options, folding))
    }

    /// Keep the output of the phases, with spans made relative to the file.
    ///
    /// # Arguments
    ///
    /// * `source` - The source file.
    /// * `emitted` - The output of the phases.
    ///
    /// # Returns
    ///
    /// The artifact.
    pub fn from_emitted(source: &SourceFile, emitted: Emitted) -> Self
    {
        let base = source.start_pos;
        let Emitted { lowered, mut module } = emitted;
        let mut diagnostics = lowered.diagnostics;
        diagnostics.iter_mut().for_each(|d| d.span = relative(d.span, base));
        module.iter_mut().for_each(|m| m.map_spans(&|s| relative(s, base)));
        #[rustfmt::skip]
        return CodegenArtifact::builder().name(source.name.clone())
            .source(source.src.clone())
            .maybe_module(module).diagnostics(diagnostics).build();
    }

    /// Add the embedded source to a source map.
    ///
    /// # Arguments
    ///
    /// * `source_map` - The source map.
    ///
    /// # Returns
    ///
    /// The registered file.
    pub fn register_source(&self, source_map: &mut SourceMap) -> Arc<SourceFile>
    {
        source_map.new_source_file(self.name.clone(), self.source.clone())
    }

    /// The diagnostics with spans made absolute for a registered file.
    ///
    /// # Arguments
    ///
    /// * `source` - The file the embedded source was registered as.
    ///
    /// # Returns
    ///
    /// The diagnostics.
    pub fn absolute_diagnostics(&self, source: &SourceFile) -> Vec<Diagnostic>
    {
        let base = source.start_pos;
        let shift = |span: Span| absolute(span, base);
        let mut diagnostics = self.diagnostics.clone();
        diagnostics.iter_mut().for_each(|d| d.span = shift(d.span));
        diagnostics
    }

    /// The ops with source regions made absolute for a registered file.
    ///
    /// # Arguments
    ///
    /// * `source` - The file the embedded source was registered as.
    ///
    /// # Returns
    ///
    /// The ops, or `None` when an earlier phase reported an error.
    pub fn absolute_module(&self, source: &SourceFile) -> Option<Module>
    {
        let base = source.start_pos;
        let mut module = self.module.clone()?;
        module.map_spans(&|span| absolute(span, base));
        Some(module)
    }
}

impl PhaseArtifact for CodegenArtifact
{
    /// Read an artifact and register its source.
    ///
    /// # Arguments
    ///
    /// * `format` - The format.
    /// * `input` - Where to read from.
    /// * `stdin` - The standard input.
    /// * `source_map` - The source map.
    ///
    /// # Returns
    ///
    /// The artifact.
    fn decode(
        format: ArtifactFormat, input: &Input, stdin: &mut dyn BufRead,
        source_map: &mut SourceMap,
    ) -> zakac_host_errors::Result<Self>
    {
        let path = input.error_path();
        let (_, content) = input.read(stdin)?;
        let artifact = decode_artifact(&content, &path, format)?;
        check_embedded_source(&artifact, source_map, &path)?;
        Ok(artifact)
    }

    /// Encode the artifact.
    ///
    /// # Arguments
    ///
    /// * `format` - The format.
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

/// Decode an artifact from its text or raw form.
///
/// # Arguments
///
/// * `content` - The encoded artifact.
/// * `path` - The path for errors.
/// * `format` - The format.
///
/// # Returns
///
/// The artifact.
pub fn decode_artifact(
    content: &[u8], path: &Path, format: ArtifactFormat,
) -> zakac_host_errors::Result<CodegenArtifact>
{
    tern!(
        format == ArtifactFormat::Text,
        decode_text(TEXT_HEADER, content, path),
        decode_raw(RAW_PHASE, ARTIFACT_VERSION, content, path)
    )
}

/// Register the embedded source and check that it reads back unchanged.
///
/// # Arguments
///
/// * `artifact` - The artifact.
/// * `source_map` - The source map.
/// * `path` - The path for errors.
///
/// # Returns
///
/// An error when the source text changed on re-ingest.
fn check_embedded_source(
    artifact: &CodegenArtifact, source_map: &mut SourceMap, path: &Path,
) -> zakac_host_errors::Result<()>
{
    let registered = artifact.register_source(source_map);
    ret_if!(registered.src == artifact.source, Ok(()));
    let msg = t!("codegen.artifact.source_changed").to_string();
    Err(Error::FailedToDecodeArtifact { path: PathBuf::from(path), msg })
}
