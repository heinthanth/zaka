use std::io::Cursor;
use std::path::PathBuf;

use zakac_borrow_checker::artifact::BorrowCheckArtifact;
use zakac_borrow_checker::digest::FnDigest;
use zakac_borrow_checker::digest::Root;
use zakac_borrow_checker::digest::Seg;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_ir_builder::ir::FoldOptions;
use zakac_ir_builder::ir::Place;
use zakac_lexer::lexer::LexerOptions;
use zakac_rust_sugars::extract;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::dump_artifact;
use zakac_session::artifact::load_artifact;
use zakac_session::config::Input;
use zakac_session::config::Output;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Check an input into an artifact. A file before it moves its start
/// position off zero, so relative spans are tested.
fn sample(name: &str) -> (BorrowCheckArtifact, u32)
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/inputs/{name}.zaka"));
    let mut sm = SourceMap::new();
    sm.new_source_file(FileName::Custom("pad".into()), "pad".into());
    let file = sm.load_file(&path).unwrap();
    let artifact = BorrowCheckArtifact::from_source(
        &file,
        &LexerOptions::default(),
        &SemcheckOptions::default(),
        &FoldOptions::default(),
        &BorrowCheckOptions::default(),
    );
    (artifact, file.start_pos.as_u32())
}

/// Dump and reload an artifact in `format`.
fn round_trip(
    artifact: &BorrowCheckArtifact, format: ArtifactFormat,
) -> BorrowCheckArtifact
{
    let mut output = Output::Data { name: "dump".to_string(), content: Vec::new() };
    let mut stdout = termcolor::Buffer::no_color();
    dump_artifact(&mut output, &mut stdout, artifact, format).unwrap();
    extract!(Output::Data { content, .. } = output);
    let input = Input::Data { name: "reload".to_string(), content };
    let mut stdin = Cursor::new(Vec::<u8>::new());
    load_artifact(&input, format, &mut stdin, &mut SourceMap::new()).unwrap()
}

/// A digest that frees through a parameter and a global, so the round trip
/// carries each step.
fn frees_digest() -> FnDigest
{
    let param = Root { place: Place::Local(0), path: vec![Seg::Block] };
    let global = Root {
        place: Place::Global(1),
        path: vec![Seg::Block, Seg::Cell(Some(2)), Seg::Cell(None)],
    };
    let result = Root { place: Place::Local(0), path: vec![Seg::Block, Seg::Block] };
    FnDigest::builder()
        .name("frees")
        .params(1)
        .frees(vec![param, global])
        .result_from(vec![0])
        .result_roots(vec![result.clone()])
        .renewed(vec![result])
        .result_roots_ok(false)
        .build()
}

#[test]
fn text_round_trip()
{
    let (artifact, _) = sample("clean");
    assert!(artifact.module.is_some());
    assert_eq!(artifact.digests.len(), 3);
    assert_eq!(round_trip(&artifact, ArtifactFormat::Text), artifact);
}

#[test]
fn raw_round_trip()
{
    let (mut artifact, _) = sample("clean");
    artifact.digests.push(frees_digest());
    assert!(artifact.module.is_some());
    assert_eq!(round_trip(&artifact, ArtifactFormat::Raw), artifact);
}

#[test]
fn text_round_trip_keeps_the_digest_paths()
{
    let (mut artifact, _) = sample("clean");
    artifact.digests.push(frees_digest());
    assert_eq!(round_trip(&artifact, ArtifactFormat::Text), artifact);
}

#[test]
fn spans_are_relative_to_the_file()
{
    let (artifact, base) = sample("clean");
    assert!(base > 0);
    let module = artifact.module.expect("checked");
    assert_eq!(module.span.lo.as_u32(), 0);
    assert_eq!(module.main[0].span().lo.as_u32(), 0);
    let (errors, _) = sample("fold_errors");
    assert!(errors.diagnostics[0].span.lo.as_u32() < 200);
}

#[test]
fn absolute_output_restores_spans()
{
    let (artifact, _) = sample("fold_errors");
    let mut sm = SourceMap::new();
    sm.new_source_file(FileName::Custom("pad".into()), "pad".into());
    let file = artifact.register_source(&mut sm);
    let (module, diags) = artifact.absolute_output(&file);
    assert_eq!(module.expect("folded").span.lo, file.start_pos);
    assert!(diags[0].span.lo > file.start_pos);
    assert_eq!(
        diags[0].span.lo,
        file.start_pos + artifact.diagnostics[0].span.lo.as_u32() as usize
    );
}

#[test]
fn absolute_output_restores_the_module_spans()
{
    let (artifact, _) = sample("clean");
    let mut sm = SourceMap::new();
    sm.new_source_file(FileName::Custom("pad".into()), "pad".into());
    let file = artifact.register_source(&mut sm);
    let (module, _) = artifact.absolute_output(&file);
    assert_eq!(module.expect("checked").span.lo, file.start_pos);
}

#[test]
fn label_spans_are_relative_and_absolute_again()
{
    let (artifact, base) = sample("invalidated_loan");
    assert!(base > 0);
    let label = artifact.diagnostics[0].labels[0].span;
    assert_eq!(round_trip(&artifact, ArtifactFormat::Text), artifact);
    let lo = label.lo.as_u32() as usize;
    let rest = artifact.source.get(lo..).expect("the label starts on a char");
    assert!(rest.starts_with("မိတ္တူ ၏ တန်ဖိုး"), "{rest}");
    assert_eq!(artifact.source[..lo].lines().count() + 1, 7);
    let mut sm = SourceMap::new();
    sm.new_source_file(FileName::Custom("pad".into()), "pad".into());
    let file = artifact.register_source(&mut sm);
    let (_, diags) = artifact.absolute_output(&file);
    assert_eq!(diags[0].labels[0].span.lo, file.start_pos + lo);
}
