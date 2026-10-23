use std::io::Cursor;
use std::path::PathBuf;

use zakac_lexer::artifact::LexArtifact;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_rust_sugars::extract;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::dump_artifact;
use zakac_session::artifact::load_artifact;
use zakac_session::config::Input;
use zakac_session::config::Output;
use zakac_span::source_map::SourceMap;

/// Lex a sample into an artifact.
fn sample(name: &str) -> LexArtifact
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/inputs/{name}.zaka"));
    let mut sm = SourceMap::new();
    // a second file moves the start position awa from the first valid one
    sm.new_source_file(
        zakac_span::filename::FileName::Custom("pad".into()),
        "pad".into(),
    );
    let file = sm.load_file(&path).unwrap();
    LexArtifact::from_source(&file, &LexerOptions::default())
}

/// Dump and reload an artifact in `format`.
fn round_trip(artifact: &LexArtifact, format: ArtifactFormat) -> LexArtifact
{
    let mut output = Output::Data { name: "dump".to_string(), content: Vec::new() };
    let mut stdout = termcolor::Buffer::no_color();
    dump_artifact(&mut output, &mut stdout, artifact, format).unwrap();

    extract!(Output::Data { content, .. } = output);
    let input = Input::Data { name: "reload".to_string(), content };
    let mut stdin = Cursor::new(Vec::<u8>::new());
    let mut sm = SourceMap::new();
    load_artifact(&input, format, &mut stdin, &mut sm).unwrap()
}

#[test]
fn text_round_trip()
{
    let artifact = sample("statements");
    assert_eq!(round_trip(&artifact, ArtifactFormat::Text), artifact);
}

#[test]
fn raw_round_trip()
{
    let artifact = sample("literals");
    assert_eq!(round_trip(&artifact, ArtifactFormat::Raw), artifact);
}

#[test]
fn diagnostics_survive_round_trip()
{
    let artifact = sample("invalid");
    assert!(!artifact.diagnostics.is_empty());
    assert_eq!(
        round_trip(&artifact, ArtifactFormat::Raw).diagnostics,
        artifact.diagnostics
    );
}

#[test]
fn spans_are_relative_and_map_back()
{
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/inputs/functions.zaka");
    let mut sm = SourceMap::new();
    sm.new_source_file(
        zakac_span::filename::FileName::Custom("pad".into()),
        "pad".into(),
    );
    let file = sm.load_file(&path).unwrap();
    let out = lex(&file, &LexerOptions::default());
    let artifact = LexArtifact::from_output(&file, out.clone());

    assert_eq!(artifact.tokens[0].span.lo.as_u32(), 0);
    let mut sm2 = SourceMap::new();
    sm2.new_source_file(
        zakac_span::filename::FileName::Custom("pad".into()),
        "pad".into(),
    );
    let registered = artifact.register_source(&mut sm2);
    assert_eq!(registered.start_pos, file.start_pos);
    assert_eq!(artifact.absolute_output(&registered), out);
}

#[test]
fn text_header_is_checked()
{
    let input = Input::Data { name: "bad".to_string(), content: b"{}".to_vec() };
    let mut stdin = Cursor::new(Vec::<u8>::new());
    let mut sm = SourceMap::new();
    let res: zakac_host_errors::Result<LexArtifact> =
        load_artifact(&input, ArtifactFormat::Text, &mut stdin, &mut sm);
    assert!(res.is_err());
}

#[test]
fn raw_phase_is_checked()
{
    let raw = zakac_session::artifact::encode_raw("segment", 1, &0u8).unwrap();
    let input = Input::Data { name: "bad".to_string(), content: raw };
    let mut stdin = Cursor::new(Vec::<u8>::new());
    let mut sm = SourceMap::new();
    let res: zakac_host_errors::Result<LexArtifact> =
        load_artifact(&input, ArtifactFormat::Raw, &mut stdin, &mut sm);
    assert!(res.is_err());
}
