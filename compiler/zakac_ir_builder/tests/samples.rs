use std::path::PathBuf;

use zakac_diagnostics::Code;
use zakac_ir_builder::artifact::IrArtifact;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::FoldOptions;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::PrintValue;
use zakac_ir_builder::ir::Stmt;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::PhaseArtifact;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Inputs that must lower without diagnostics.
const CLEAN_INPUTS: &[&str] = &["folding", "functions", "loops", "programs"];

/// Lower `tests/inputs/{name}.zaka` with `folding`.
fn lower_input(name: &str, folding: &FoldOptions) -> IrArtifact
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(root.join(format!("tests/inputs/{name}.zaka")))
        .unwrap();
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom(format!("{name}.zaka")), text);
    let options = SemcheckOptions::default();
    IrArtifact::from_source(&file, &LexerOptions::default(), &options, folding)
}

/// Lower an input and compare its text artifact with
/// `tests/expected/{name}.zkirt`. With `ZAKA_BLESS=1` the expected file is
/// rewritten instead.
fn check_expected(name: &str) -> IrArtifact
{
    let artifact = lower_input(name, &FoldOptions::default());
    let encoded =
        String::from_utf8(artifact.encode(ArtifactFormat::Text).unwrap()).unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let expected_path = root.join(format!("tests/expected/{name}.zkirt"));
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::write(&expected_path, &encoded).unwrap();
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap();
    assert_eq!(encoded, expected, "output differs from the expected file for {name}");
    artifact
}

/// The places read or written anywhere in a statement list.
fn places(stmts: &[Stmt], out: &mut Vec<Place>)
{
    for s in stmts
    {
        match s
        {
            Stmt::Set { place, value, .. } =>
            {
                out.push(*place);
                expr_places(value, out);
            }
            Stmt::Print { value: PrintValue::Expr(e), .. } => expr_places(e, out),
            Stmt::If { cond, then, els, .. } =>
            {
                expr_places(cond, out);
                places(then, out);
                places(els, out);
            }
            Stmt::Loop { init, cond, update, block, .. } =>
            {
                places(init, out);
                expr_places(cond, out);
                places(update, out);
                places(block, out);
            }
            Stmt::Counted { var, from, to, step, block, .. } =>
            {
                out.push(*var);
                expr_places(from, out);
                expr_places(to, out);
                step.iter().for_each(|s| expr_places(s, out));
                places(block, out);
            }
            Stmt::Return { value: Some(v), .. } => expr_places(v, out),
            Stmt::Call { args, .. } => args.iter().for_each(|a| expr_places(a, out)),
            _ => (),
        }
    }
}

/// The places read in an expression.
fn expr_places(e: &Expr, out: &mut Vec<Place>)
{
    match &e.node
    {
        ExprNode::Read(place) => out.push(*place),
        ExprNode::Const(_) => (),
        ExprNode::Prefix { operand, .. } | ExprNode::Convert { operand, .. } =>
        {
            expr_places(operand, out)
        }
        ExprNode::Infix { lhs, rhs, .. }
        | ExprNode::Compare { lhs, rhs, .. }
        | ExprNode::Logic { lhs, rhs, .. } =>
        {
            expr_places(lhs, out);
            expr_places(rhs, out);
        }
        ExprNode::Call { args, .. } => args.iter().for_each(|a| expr_places(a, out)),
    }
}

/// Assert that every place names a slot and every call a function.
fn assert_well_formed(name: &str, module: &Module)
{
    let mut found = Vec::new();
    places(&module.main, &mut found);
    let bad: Vec<&Place> = found
        .iter()
        .filter(
            |p| !matches!(p, Place::Global(n) if (*n as usize) < module.globals.len()),
        )
        .collect();
    assert!(bad.is_empty(), "{name}: {bad:?}");
    for func in &module.funcs
    {
        let mut found = Vec::new();
        places(&func.block, &mut found);
        let fits = |p: &&Place| match p
        {
            Place::Global(n) => (*n as usize) < module.globals.len(),
            Place::Local(n) => (*n as usize) < func.locals.len(),
        };
        let bad: Vec<&Place> = found.iter().filter(|p| !fits(p)).collect();
        assert!(bad.is_empty(), "{name} {}: {bad:?}", func.name);
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
        assert_well_formed(name, artifact.module.as_ref().expect("lowered"));
    }
}

#[test]
fn unfolded_inputs_are_well_formed()
{
    for name in CLEAN_INPUTS
    {
        let artifact = lower_input(name, &FoldOptions::none());
        assert_well_formed(name, artifact.module.as_ref().expect("lowered"));
    }
}

#[test]
fn fold_errors_report_one_error_per_bad_line()
{
    let artifact = check_expected("fold_errors");
    let found: Vec<Code> = artifact.diagnostics.iter().map(|d| d.code).collect();
    let want = [
        Code::ConstantOverflow,
        Code::ConstantZeroDivisor,
        Code::ConstantZeroDivisor,
        Code::ShiftTooFar,
    ];
    assert_eq!(found, want);
}
