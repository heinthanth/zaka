use std::io::Cursor;
use std::path::PathBuf;

use zakac_burmese_segmenter::artifact::SegmentArtifact;
use zakac_burmese_segmenter::segmenter::SegmenterOptions;
use zakac_rust_sugars::extract;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::PhaseArtifact;
use zakac_session::artifact::dump_artifact;
use zakac_session::artifact::load_artifact;
use zakac_session::config::Input;
use zakac_session::config::Output;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

fn sample_source() -> std::sync::Arc<SourceFile>
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/paragraph1.in");
    let mut source_map = SourceMap::new();
    source_map.load_file(&path).unwrap()
}

fn round_trip(format: ArtifactFormat)
{
    let source = sample_source();
    let options = SegmenterOptions::default();
    let artifact = SegmentArtifact::from_source(&source, &options);

    let mut output = Output::Data { name: "dump".to_string(), content: Vec::new() };
    let mut stdout = termcolor::Buffer::no_color();
    dump_artifact(&mut output, &mut stdout, &artifact, format).unwrap();

    extract!(Output::Data { content, .. } = output);
    let input = Input::Data { name: "reload".to_string(), content: content.clone() };
    let mut stdin = Cursor::new(Vec::<u8>::new());
    let mut source_map = SourceMap::new();
    let reloaded: SegmentArtifact =
        load_artifact(&input, format, &mut stdin, &mut source_map).unwrap();

    let input_path = artifact.name.virtual_path_string();
    let reloaded_path = reloaded.name.virtual_path_string();
    assert_eq!(input_path, reloaded_path, "input path mismatch");
    assert_eq!(reloaded.source, artifact.source);
    assert_eq!(reloaded.tokens, artifact.tokens);
}

#[test]
fn segment_artifact_text_round_trip() { round_trip(ArtifactFormat::Text); }

#[test]
fn segment_artifact_raw_round_trip() { round_trip(ArtifactFormat::Raw); }

#[test]
fn segment_artifact_burmese_sample_round_trip()
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/metta_sutta.in");
    let mut source_map = SourceMap::new();
    let source = source_map.load_file(&path).unwrap();
    let artifact = SegmentArtifact::from_source(&source, &SegmenterOptions::default());

    let encoded = artifact.encode(ArtifactFormat::Raw).unwrap();
    let input = Input::Data { name: "metta".to_string(), content: encoded };
    let mut stdin = Cursor::new(Vec::<u8>::new());
    let reloaded: SegmentArtifact =
        load_artifact(&input, ArtifactFormat::Raw, &mut stdin, &mut source_map)
            .unwrap();
    assert_eq!(reloaded.tokens.len(), artifact.tokens.len());
    assert_eq!(reloaded.source, artifact.source);
}
