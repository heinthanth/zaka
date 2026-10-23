use std::io::Cursor;
use std::path::PathBuf;

use termcolor::Buffer;
use zakac_rust_sugars::extract;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::load_artifact;
use zakac_session::config::Input;
use zakac_session::config::Output;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

#[test]
fn input_data_read_returns_content()
{
    let input = Input::Data { name: "inline".to_string(), content: b"hello".to_vec() };
    let mut stdin = Cursor::new(Vec::<u8>::new());
    let (name, content) = input.read(&mut stdin).unwrap();
    assert_eq!(content, b"hello");
    assert_eq!(name.virtual_path_string(), "inline");
}

#[test]
fn output_data_write_appends()
{
    let mut output = Output::Data { name: "out".to_string(), content: b"ab".to_vec() };
    let mut stdout = Buffer::no_color();
    output.write(&mut stdout, b"cd").unwrap();
    extract!(Output::Data { content, .. } = output);
    assert_eq!(content, b"abcd");
}

#[test]
fn load_source_text_from_data()
{
    let input = Input::Data {
        name: "test.zaka".to_string(),
        content: "က".as_bytes().to_vec(),
    };
    let mut stdin = Cursor::new(Vec::<u8>::new());
    let mut source_map = SourceMap::new();
    let file: std::sync::Arc<SourceFile> =
        load_artifact(&input, ArtifactFormat::Text, &mut stdin, &mut source_map)
            .unwrap();
    assert_eq!(file.src, "က");
}

#[test]
fn load_source_text_rejects_invalid_utf8()
{
    let input = Input::Data { name: "bad".to_string(), content: vec![0xff] };
    let mut stdin = Cursor::new(Vec::<u8>::new());
    let mut source_map = SourceMap::new();
    let err = load_artifact::<std::sync::Arc<SourceFile>>(
        &input,
        ArtifactFormat::Text,
        &mut stdin,
        &mut source_map,
    )
    .unwrap_err();
    assert!(err.to_string().contains("bad"));
}

#[test]
fn load_source_from_file_uses_source_map()
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../zakac_burmese_segmenter/samples/paragraph1.in");
    let input = Input::File(path);
    let mut stdin = Cursor::new(Vec::<u8>::new());
    let mut source_map = SourceMap::new();
    let file: std::sync::Arc<SourceFile> =
        load_artifact(&input, ArtifactFormat::Text, &mut stdin, &mut source_map)
            .unwrap();
    assert!(!file.src.is_empty());
    assert_eq!(source_map.files().len(), 1);
}
