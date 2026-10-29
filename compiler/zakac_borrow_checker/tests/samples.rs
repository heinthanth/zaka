mod common;

use std::path::PathBuf;

use common::lines;
use zakac_borrow_checker::artifact::BorrowCheckArtifact;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_diagnostics::Code;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::PhaseArtifact;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Inputs that must pass the check with no diagnostic.
const CLEAN_INPUTS: [&str; 3] = ["loans", "calls", "lists"];

/// The crate dir.
fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// The text of `tests/inputs/{name}.zaka`.
fn input_text(name: &str) -> String
{
    std::fs::read_to_string(root().join(format!("tests/inputs/{name}.zaka"))).unwrap()
}

/// Check `tests/inputs/{name}.zaka` into an artifact with the default
/// options.
fn check_input(name: &str) -> BorrowCheckArtifact
{
    let mut sm = SourceMap::new();
    let file =
        sm.new_source_file(FileName::Custom(format!("{name}.zaka")), input_text(name));
    BorrowCheckArtifact::from_source(
        &file,
        &LexerOptions::default(),
        &SemcheckOptions::default(),
        &FoldOptions::default(),
        &BorrowCheckOptions::default(),
    )
}

/// Check an input and compare its text artifact with
/// `tests/expected/{name}.zkbckt`. With `ZAKA_BLESS=1` the expected file is
/// rewritten instead.
fn check_expected(name: &str) -> BorrowCheckArtifact
{
    let artifact = check_input(name);
    let encoded =
        String::from_utf8(artifact.encode(ArtifactFormat::Text).unwrap()).unwrap();
    let expected_path = root().join(format!("tests/expected/{name}.zkbckt"));
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::create_dir_all(expected_path.parent().unwrap()).unwrap();
        std::fs::write(&expected_path, &encoded).unwrap();
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap();
    assert_eq!(encoded, expected, "output differs from the expected file for {name}");
    artifact
}

#[test]
fn clean_inputs_match_their_expected_output()
{
    for name in CLEAN_INPUTS
    {
        let artifact = check_expected(name);
        let msgs: Vec<&str> =
            artifact.diagnostics.iter().map(|d| d.message.as_str()).collect();
        assert!(artifact.diagnostics.is_empty(), "{name}: {msgs:?}");
        assert!(artifact.module.is_some(), "{name}");
    }
}

/// The scenarios of the spec, each in a function of its own: a read after
/// the free, a free through the borrow, a read after the resize, a read
/// after a move into a call, a lost block and one kept at the end, two
/// reads of `ဗလာ`, and a counted loop one cell too far.
#[test]
fn mistakes_report_one_code_per_scenario()
{
    let want = [
        (Code::InvalidatedLoan, 7),
        (Code::InvalidatedLoan, 17),
        (Code::FreedBorrow, 18),
        (Code::InvalidatedLoan, 27),
        (Code::MovedOut, 44),
        (Code::BlockLeaked, 52),
        (Code::BlockLeaked, 54),
        (Code::NullDeref, 60),
        (Code::NullDeref, 64),
        (Code::OffsetPastEnd, 72),
    ];
    assert_eq!(lines(&input_text("mistakes")), want);
}
