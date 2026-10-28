use std::path::PathBuf;

use zakac_ast::expr::Expr;
use zakac_ast::expr::NodeId;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_expr;
use zakac_diagnostics::Code;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::artifact::SemcheckArtifact;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::PhaseArtifact;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Inputs that must check without diagnostics.
const CLEAN_INPUTS: &[&str] = &["functions", "heap", "lists", "loops", "programs"];

/// Check `tests/inputs/{name}.zaka` and compare its text artifact with
/// `tests/expected/{name}.zksemt`. With `ZAKA_BLESS=1` the expected file is
/// rewritten instead.
fn check_expected(name: &str) -> SemcheckArtifact
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(root.join(format!("tests/inputs/{name}.zaka")))
        .unwrap();
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom(format!("{name}.zaka")), text);
    let options = SemcheckOptions::default();
    let artifact =
        SemcheckArtifact::from_source(&file, &LexerOptions::default(), &options);
    let encoded =
        String::from_utf8(artifact.encode(ArtifactFormat::Text).unwrap()).unwrap();

    let expected_path = root.join(format!("tests/expected/{name}.zksemt"));
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::write(&expected_path, &encoded).unwrap();
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap();
    assert_eq!(encoded, expected, "output differs from the expected file for {name}");
    artifact
}

/// Collects the node number of each expression.
struct Ids(Vec<NodeId>);

impl Visitor for Ids
{
    fn visit_expr(&mut self, n: &Expr)
    {
        self.0.push(n.id());
        walk_expr(self, n);
    }
}

#[test]
fn clean_inputs_record_a_kind_for_each_expression()
{
    for name in CLEAN_INPUTS
    {
        let artifact = check_expected(name);
        let mut ids = Ids(Vec::new());
        ids.visit_program(&artifact.program);
        let missing: Vec<&NodeId> =
            ids.0.iter().filter(|id| !artifact.kinds.contains_key(id)).collect();
        assert!(missing.is_empty(), "{name}: {missing:?}");
    }
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
        assert!(!artifact.kinds.is_empty(), "{name}");
    }
}

#[test]
fn invalid_input_reports_one_error_per_bad_line()
{
    let artifact = check_expected("invalid");
    let found: Vec<Code> = artifact.diagnostics.iter().map(|d| d.code).collect();
    let want = [
        Code::UndeclaredName,
        Code::LiteralDoesNotFit,
        Code::Redeclaration,
        Code::ConstWithoutInit,
        Code::AssignToConst,
        Code::ConditionNotBool,
        Code::OperandKind,
        Code::BreakOutsideLoop,
        Code::ReturnOutsideFunction,
        Code::InvalidCast,
    ];
    assert_eq!(found, want);
}
