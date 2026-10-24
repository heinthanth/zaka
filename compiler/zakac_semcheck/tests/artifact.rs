use std::io::Cursor;
use std::path::PathBuf;

use zakac_ast::visit::Node;
use zakac_lexer::lexer::LexerOptions;
use zakac_rust_sugars::extract;
use zakac_semcheck::artifact::SemcheckArtifact;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::dump_artifact;
use zakac_session::artifact::load_artifact;
use zakac_session::config::Input;
use zakac_session::config::Output;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Check a sample into an artifact. A file before it moves its start
/// position off zero, so relative spans are tested.
fn sample(name: &str) -> (SemcheckArtifact, u32)
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/inputs/{name}.zaka"));
    let mut sm = SourceMap::new();
    sm.new_source_file(FileName::Custom("pad".into()), "pad".into());
    let file = sm.load_file(&path).unwrap();
    let options = SemcheckOptions::default();
    let artifact =
        SemcheckArtifact::from_source(&file, &LexerOptions::default(), &options);
    (artifact, file.start_pos.as_u32())
}

/// Dump and reload an artifact in `format`.
fn round_trip(artifact: &SemcheckArtifact, format: ArtifactFormat) -> SemcheckArtifact
{
    let mut output = Output::Data { name: "dump".to_string(), content: Vec::new() };
    let mut stdout = termcolor::Buffer::no_color();
    dump_artifact(&mut output, &mut stdout, artifact, format).unwrap();
    extract!(Output::Data { content, .. } = output);
    let input = Input::Data { name: "reload".to_string(), content };
    let mut stdin = Cursor::new(Vec::<u8>::new());
    load_artifact(&input, format, &mut stdin, &mut SourceMap::new()).unwrap()
}

#[test]
fn text_round_trip()
{
    let (artifact, _) = sample("programs");
    assert!(!artifact.kinds.is_empty());
    assert_eq!(round_trip(&artifact, ArtifactFormat::Text), artifact);
}

#[test]
fn raw_round_trip()
{
    let (artifact, _) = sample("functions");
    assert!(!artifact.kinds.is_empty());
    assert_eq!(round_trip(&artifact, ArtifactFormat::Raw), artifact);
}

#[test]
fn spans_are_relative_to_the_file()
{
    let (artifact, base) = sample("loops");
    assert!(base > 0);
    assert_eq!(artifact.program.span.lo.as_u32(), 0);
    assert_eq!(artifact.program.stmts[0].span().lo.as_u32(), 0);
    let (invalid, _) = sample("invalid");
    assert_eq!(invalid.diagnostics[0].span.lo.as_u32(), 0);
}

#[test]
fn absolute_output_restores_spans()
{
    let (artifact, _) = sample("invalid");
    let mut sm = SourceMap::new();
    sm.new_source_file(FileName::Custom("pad".into()), "pad".into());
    let file = artifact.register_source(&mut sm);
    let (program, diags) = artifact.absolute_output(&file);
    assert_eq!(program.span.lo, file.start_pos);
    assert_eq!(diags[0].span.lo, file.start_pos);
}
