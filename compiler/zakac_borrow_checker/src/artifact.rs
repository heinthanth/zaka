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
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::spans::map_spans;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::Loaded;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::EmbeddedFile;
use zakac_session::artifact::PhaseArtifact;
use zakac_session::artifact::decode_raw;
use zakac_session::artifact::decode_text;
use zakac_session::artifact::encode_raw;
use zakac_session::artifact::encode_text;
use zakac_session::artifact::register_embedded;
use zakac_session::config::Input;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

use crate::borrow_check;
use crate::digest::FnDigest;
use crate::options::BorrowCheckOptions;

/// The first line of a text artifact.
pub const TEXT_HEADER: &str = "# zakac-borrow-check-artifact v1";
/// The phase name in a raw artifact.
pub const RAW_PHASE: &str = "borrow_check";
/// The schema version of the artifact.
pub const ARTIFACT_VERSION: u32 = 1;

/// The output of the phases up to the ownership check, with absolute spans.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct Borrowchecked
{
    /// The output of the phases up to lowering; its folded module is the
    /// one the check marked.
    pub lowered: Lowered,
    /// The digest of each function, none when an earlier phase reported
    /// an error.
    #[builder(default)]
    pub digests: Vec<FnDigest>,
    /// The diagnostics of the earlier phases, then those of the check.
    #[builder(default)]
    pub diagnostics: Vec<Diagnostic>,
}

/// Lex, parse, check, lower, fold and borrow check a source file. The
/// ownership check is left out when an earlier phase reports an error.
///
/// # Arguments
///
/// * `source` - The source file.
/// * `lexer` - The lexer options.
/// * `options` - The check options; their platform width also drives the later
///   phases.
/// * `folding` - Which foldings rewrite the module.
/// * `mem` - How the ownership check reports.
///
/// # Returns
///
/// The output of all phases.
pub fn run_phases(
    source: &SourceFile, lexer: &LexerOptions, options: &SemcheckOptions,
    folding: &FoldOptions, mem: &BorrowCheckOptions,
) -> Borrowchecked
{
    run_loaded(Loaded::single(source, lexer), options, folding, mem)
}

/// Check, lower, fold and borrow check a program read with its imports. The
/// ownership check is left out when an earlier phase reports an error.
///
/// # Arguments
///
/// * `loaded` - The program and the lexer and parser diagnostics.
/// * `options` - The check options; their platform width also drives the later
///   phases.
/// * `folding` - Which foldings rewrite the module.
/// * `mem` - How the ownership check reports.
///
/// # Returns
///
/// The output of all phases.
pub fn run_loaded(
    loaded: Loaded, options: &SemcheckOptions, folding: &FoldOptions,
    mem: &BorrowCheckOptions,
) -> Borrowchecked
{
    let mut lowered = zakac_ir_builder::artifact::run_loaded(loaded, options, folding);
    let mut diagnostics = lowered.diagnostics.clone();
    let builder = Borrowchecked::builder();
    let clean = !has_errors(&diagnostics);
    let Some(module) = lowered.folded.as_mut().filter(|_| clean)
    else
    {
        return builder.lowered(lowered).diagnostics(diagnostics).build();
    };
    let output = borrow_check(module, mem, options.platform_bits);
    diagnostics.extend(output.diagnostics);
    #[rustfmt::skip]
    return builder.lowered(lowered).digests(output.digests).diagnostics(diagnostics)
        .build();
}

/// Reloadable output of the ownership check: the folded module and the
/// digests. Spans are relative to the file start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct BorrowCheckArtifact
{
    /// Diagnostic file name for the embedded source.
    pub name: FileName,
    /// Normalized source text the spans refer to.
    pub source: String,
    /// The imported files, in reading order.
    #[builder(default)]
    #[serde(default)]
    pub imports: Vec<EmbeddedFile>,
    /// The folded module, when the earlier phases reported no error.
    pub module: Option<Module>,
    /// The digest of each function.
    pub digests: Vec<FnDigest>,
    /// All diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

impl BorrowCheckArtifact
{
    /// Run all phases on a loaded source file into an artifact.
    ///
    /// # Arguments
    ///
    /// * `source` - The source file.
    /// * `lexer` - The lexer options.
    /// * `options` - The check options.
    /// * `folding` - Which foldings rewrite the module.
    /// * `mem` - How the ownership check reports.
    ///
    /// # Returns
    ///
    /// The artifact.
    pub fn from_source(
        source: &SourceFile, lexer: &LexerOptions, options: &SemcheckOptions,
        folding: &FoldOptions, mem: &BorrowCheckOptions,
    ) -> Self
    {
        let output = run_phases(source, lexer, options, folding, mem);
        Self::from_borrowchecked(source, output)
    }

    /// Build an artifact from output with absolute spans.
    ///
    /// # Arguments
    ///
    /// * `source` - The source file of the output.
    /// * `output` - The output.
    ///
    /// # Returns
    ///
    /// The artifact, with spans made relative.
    pub fn from_borrowchecked(source: &SourceFile, output: Borrowchecked) -> Self
    {
        let base = source.start_pos;
        let shift = |span: Span| relative(span, base);
        let Borrowchecked { lowered, digests, mut diagnostics } = output;
        let (mut module, imports) = (lowered.folded, lowered.checked.imports);
        do_map(module.as_mut(), &shift);
        diagnostics.iter_mut().for_each(|d| d.map_spans(&shift));
        #[rustfmt::skip]
        return BorrowCheckArtifact::builder().name(source.name.clone())
            .source(source.src.clone()).imports(imports)
            .maybe_module(module).digests(digests).diagnostics(diagnostics).build();
    }

    /// Register the embedded source, then the imported files, in
    /// `source_map`, and return the file of the embedded source.
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
        register_embedded(source_map, &self.name, &self.source, &self.imports)
    }

    /// The module and diagnostics with spans made absolute against
    /// `source`.
    ///
    /// # Arguments
    ///
    /// * `source` - The registered source file.
    ///
    /// # Returns
    ///
    /// The module and the diagnostics.
    pub fn absolute_output(
        &self, source: &SourceFile,
    ) -> (Option<Module>, Vec<Diagnostic>)
    {
        let base = source.start_pos;
        let shift = |span: Span| absolute(span, base);
        let mut module = self.module.clone();
        do_map(module.as_mut(), &shift);
        let mut diagnostics = self.diagnostics.clone();
        diagnostics.iter_mut().for_each(|d| d.map_spans(&shift));
        (module, diagnostics)
    }
}

/// Move the spans of a module, when there is one.
///
/// # Arguments
///
/// * `module` - The module.
/// * `f` - Maps one span.
fn do_map(module: Option<&mut Module>, f: &impl Fn(Span) -> Span)
{
    do_if!(let Some(m) = module, map_spans(m, f));
}

impl PhaseArtifact for BorrowCheckArtifact
{
    /// Decode an ownership check artifact from an input.
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
        check_embedded_source(&artifact, source_map, &path)?;
        Ok(artifact)
    }

    /// Encode an ownership check artifact.
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
pub fn decode_artifact(
    content: &[u8], path: &Path, format: ArtifactFormat,
) -> zakac_host_errors::Result<BorrowCheckArtifact>
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
fn check_embedded_source(
    artifact: &BorrowCheckArtifact, source_map: &mut SourceMap, path: &Path,
) -> zakac_host_errors::Result<()>
{
    let registered = artifact.register_source(source_map);
    ret_if!(registered.src == artifact.source, Ok(()));
    let msg = t!("borrow_check.artifact.source_changed").to_string();
    Err(Error::FailedToDecodeArtifact { path: PathBuf::from(path), msg })
}
