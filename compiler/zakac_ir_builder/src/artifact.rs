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
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::Loaded;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::artifact::Checked;
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
use zakac_span::pos::Pos;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

use crate::fold::fold;
use crate::ir::FoldOptions;
use crate::ir::LowerOptions;
use crate::ir::Module;
use crate::lower::lower;
use crate::spans::map_spans;

const TEXT_HEADER: &str = "# zakac-ir-artifact v1";
const RAW_PHASE: &str = "irgen";
const ARTIFACT_VERSION: u32 = 1;

/// The check output with the lowered module before and after folding, and
/// all diagnostics, with absolute spans.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct Lowered
{
    /// The tree, kinds and front-end diagnostics.
    pub checked: Checked,
    /// The module as lowered, when the earlier phases reported no error.
    pub plain: Option<Module>,
    /// The module after folding, when the earlier phases reported no
    /// error.
    pub folded: Option<Module>,
    /// The front-end diagnostics, then those of folding.
    #[builder(default)]
    pub diagnostics: Vec<Diagnostic>,
}

/// Lex, parse, check, lower and fold a source file. Lowering is left out
/// when an earlier phase reports an error.
///
/// # Arguments
///
/// * `source` - The source file.
/// * `lexer` - The lexer options.
/// * `options` - The check options; their platform width also drives lowering
///   and folding.
/// * `folding` - Which foldings rewrite the module.
///
/// # Returns
///
/// The output of all phases.
pub fn run_phases(
    source: &SourceFile, lexer: &LexerOptions, options: &SemcheckOptions,
    folding: &FoldOptions,
) -> Lowered
{
    run_loaded(Loaded::single(source, lexer), options, folding)
}

/// Check, lower and fold a program read with its imports. Lowering is left
/// out when an earlier phase reports an error.
///
/// # Arguments
///
/// * `loaded` - The program and the lexer and parser diagnostics.
/// * `options` - The check options; their platform width also drives lowering
///   and folding.
/// * `folding` - Which foldings rewrite the module.
///
/// # Returns
///
/// The output of all phases.
pub fn run_loaded(
    loaded: Loaded, options: &SemcheckOptions, folding: &FoldOptions,
) -> Lowered
{
    let checked = zakac_semcheck::artifact::check_loaded(loaded, options);
    let mut diagnostics = checked.diagnostics.clone();
    let builder = Lowered::builder();
    ret_if!(
        has_errors(&diagnostics),
        builder.checked(checked).diagnostics(diagnostics).build()
    );
    let lower_options =
        LowerOptions::builder().platform_bits(options.platform_bits).build();
    let plain =
        lower(&checked.program, &checked.kinds, &checked.structs, &lower_options);
    let mut folded = plain.clone();
    diagnostics.extend(fold(&mut folded, folding, options.platform_bits));
    #[rustfmt::skip]
    return builder.checked(checked).plain(plain).folded(folded).diagnostics(diagnostics)
        .build();
}

/// Reloadable output of the lowering phase: the folded module. Spans are
/// relative to the file start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct IrArtifact
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
    /// All diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

impl IrArtifact
{
    /// Run all phases on a loaded source file into an artifact.
    ///
    /// # Arguments
    ///
    /// * `source` - The source file.
    /// * `lexer` - The lexer options.
    /// * `options` - The check options.
    /// * `folding` - Which foldings rewrite the module.
    ///
    /// # Returns
    ///
    /// The artifact.
    pub fn from_source(
        source: &SourceFile, lexer: &LexerOptions, options: &SemcheckOptions,
        folding: &FoldOptions,
    ) -> Self
    {
        Self::from_lowered(source, run_phases(source, lexer, options, folding))
    }

    /// Build an artifact from output with absolute spans.
    ///
    /// # Arguments
    ///
    /// * `source` - The source file of the output.
    /// * `lowered` - The output.
    ///
    /// # Returns
    ///
    /// The artifact, with spans made relative.
    pub fn from_lowered(source: &SourceFile, lowered: Lowered) -> Self
    {
        let base = source.start_pos;
        let shift = |span: Span| relative(span, base);
        let Lowered { folded: mut module, mut diagnostics, checked, .. } = lowered;
        do_map(module.as_mut(), &shift);
        diagnostics.iter_mut().for_each(|d| d.map_spans(&shift));
        #[rustfmt::skip]
        return IrArtifact::builder().name(source.name.clone())
            .source(source.src.clone()).imports(checked.imports)
            .maybe_module(module).diagnostics(diagnostics).build();
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
pub fn relative(span: Span, base: Pos) -> Span
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
pub fn absolute(span: Span, base: Pos) -> Span
{
    Span::new(base + span.lo.as_u32() as usize, base + span.hi.as_u32() as usize)
}

impl PhaseArtifact for IrArtifact
{
    /// Decode an IR artifact from an input.
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

    /// Encode an IR artifact.
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
) -> zakac_host_errors::Result<IrArtifact>
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
    artifact: &IrArtifact, source_map: &mut SourceMap, path: &Path,
) -> zakac_host_errors::Result<()>
{
    let registered = artifact.register_source(source_map);
    ret_if!(registered.src == artifact.source, Ok(()));
    let msg = t!("ir.artifact.source_changed").to_string();
    Err(Error::FailedToDecodeArtifact { path: PathBuf::from(path), msg })
}
