use std::path::PathBuf;

use zakac_diagnostics::Code;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::artifact::ParseArtifact;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::PhaseArtifact;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Inputs that must parse without diagnostics.
const CLEAN_INPUTS: &[&str] = &["clauses", "functions", "literals", "statements"];

/// Parse `tests/inputs/{name}.zaka` and compare its text artifact with
/// `tests/expected/{name}.zkastt`. With `ZAKA_BLESS=1` the expected file is
/// rewritten instead.
fn check_expected(name: &str) -> ParseArtifact
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(root.join(format!("tests/inputs/{name}.zaka")))
        .unwrap();
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom(format!("{name}.zaka")), text);
    let artifact = ParseArtifact::from_source(&file, &LexerOptions::default());
    let encoded =
        String::from_utf8(artifact.encode(ArtifactFormat::Text).unwrap()).unwrap();

    let expected_path = root.join(format!("tests/expected/{name}.zkastt"));
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::write(&expected_path, &encoded).unwrap();
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap();
    assert_eq!(encoded, expected, "output differs from the expected file for {name}");
    artifact
}

#[test]
fn clean_inputs_match_their_expected_trees()
{
    for name in CLEAN_INPUTS
    {
        let artifact = check_expected(name);
        let msgs: Vec<&str> =
            artifact.diagnostics.iter().map(|d| d.message.as_str()).collect();
        assert!(artifact.diagnostics.is_empty(), "{name}: {msgs:?}");
        assert!(!artifact.program.stmts.is_empty(), "{name}");
    }
}

#[test]
fn invalid_input_reports_one_error_per_bad_line()
{
    let artifact = check_expected("invalid");
    let found: Vec<Code> = artifact.diagnostics.iter().map(|d| d.code).collect();
    let want = [
        Code::DoubleParticle,
        Code::DanglingOperands,
        Code::MissingOperand,
        Code::ChainedComparison,
        Code::SlotMismatch,
        Code::InvalidDigit,
        Code::ElseWithoutIf,
        Code::ExpectedStmtVerb,
        Code::UnmatchedBlockClose,
    ];
    assert_eq!(found, want);
    assert_eq!(artifact.program.stmts.len(), 1, "the last line still parses");
}
