use std::io::Cursor;
use std::path::PathBuf;

use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_codegen::artifact::CodegenArtifact;
use zakac_ir_builder::ir::FoldOptions;
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

/// Generate the ops of a test input, after a padding file so spans do not
/// start at zero.
fn sample(name: &str) -> (CodegenArtifact, u32)
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/inputs/{name}.zaka"));
    let mut sm = SourceMap::new();
    sm.new_source_file(FileName::Custom("pad".into()), "pad".into());
    let file = sm.load_file(&path).unwrap();
    let options = SemcheckOptions::default();
    let artifact = CodegenArtifact::from_source(
        &file,
        &LexerOptions::default(),
        &options,
        &FoldOptions::default(),
        &BorrowCheckOptions::default(),
    );
    (artifact, file.start_pos.as_u32())
}

/// Encode an artifact and read it back.
fn round_trip(artifact: &CodegenArtifact, format: ArtifactFormat) -> CodegenArtifact
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
    assert!(artifact.module.is_some());
    assert_eq!(round_trip(&artifact, ArtifactFormat::Text), artifact);
}

#[test]
fn raw_round_trip()
{
    let (artifact, _) = sample("functions");
    assert!(artifact.module.is_some());
    assert_eq!(round_trip(&artifact, ArtifactFormat::Raw), artifact);
}

#[test]
fn errors_leave_no_module_and_keep_relative_spans()
{
    let mut sm = SourceMap::new();
    sm.new_source_file(FileName::Custom("pad".into()), "pad".into());
    let file =
        sm.new_source_file(FileName::Custom("bad.zaka".into()), "က ကို ပြသပါ။".into());
    let options = SemcheckOptions::default();
    let folding = FoldOptions::default();
    let artifact = CodegenArtifact::from_source(
        &file,
        &LexerOptions::default(),
        &options,
        &folding,
        &BorrowCheckOptions::default(),
    );
    assert!(artifact.module.is_none());
    assert_eq!(artifact.diagnostics[0].span.lo.as_u32(), 0);
    let mut fresh = SourceMap::new();
    fresh.new_source_file(FileName::Custom("pad".into()), "pad".into());
    let again = artifact.register_source(&mut fresh);
    let diags = artifact.absolute_diagnostics(&again);
    assert_eq!(diags[0].span.lo, again.start_pos);
    assert_eq!(round_trip(&artifact, ArtifactFormat::Text), artifact);
}

#[test]
fn op_spans_are_relative_in_the_artifact()
{
    let (artifact, base) = sample("programs");
    let module = artifact.module.as_ref().expect("emitted");
    let spans = module.funcs.iter().flat_map(|f| f.spans.iter());
    let len = artifact.source.len() as u32;
    assert!(spans.clone().all(|s| s.hi.as_u32() <= len), "spans past the text");
    let mut fresh = SourceMap::new();
    fresh.new_source_file(FileName::Custom("pad".into()), "pad".into());
    let again = artifact.register_source(&mut fresh);
    let moved = artifact.absolute_module(&again).expect("emitted");
    let shifted = moved.funcs.iter().flat_map(|f| f.spans.iter());
    for (rel, abs) in spans.zip(shifted)
    {
        assert_eq!(abs.lo.as_u32(), rel.lo.as_u32() + base);
    }
}
