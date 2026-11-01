use std::path::PathBuf;

use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_borrow_checker::options::MemCheck;
use zakac_codegen::artifact::CodegenArtifact;
use zakac_codegen::artifact::run_loaded;
use zakac_codegen::bc::FmtArg;
use zakac_codegen::bc::Func;
use zakac_codegen::bc::Module;
use zakac_codegen::bc::Op;
use zakac_codegen::listing::listing;
use zakac_codegen::options::ResultMode;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::load;
use zakac_rust_sugars::tern;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::PhaseArtifact;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Inputs that must give no diagnostics.
const CLEAN_INPUTS: &[&str] =
    &["folding", "functions", "heap", "lists", "loops", "programs"];

/// The crate dir.
fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// Generate the ops of a test input.
fn emit_input(name: &str, folding: &FoldOptions) -> CodegenArtifact
{
    let path = root().join(format!("tests/inputs/{name}.zaka"));
    let text = std::fs::read_to_string(path).unwrap();
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom(format!("{name}.zaka")), text);
    let options = SemcheckOptions::default();
    let mem = BorrowCheckOptions::default();
    let lexer = LexerOptions::default();
    CodegenArtifact::from_source(
        &file,
        &lexer,
        &options,
        folding,
        &mem,
        ResultMode::default(),
    )
}

/// Compare text with an expected file; `ZAKA_BLESS=1` rewrites the file.
fn check_file(name: &str, text: &str)
{
    let path = root().join(format!("tests/expected/{name}"));
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::write(&path, text).unwrap();
    }
    let expected = std::fs::read_to_string(&path).unwrap();
    assert_eq!(text, expected, "output differs from the expected file {name}");
}

/// Check that each index an op names is in range and that each function
/// ends in `ret`.
fn assert_well_formed(name: &str, module: &Module)
{
    assert_eq!(module.main as usize, module.funcs.len() - 1, "{name}");
    for func in &module.funcs
    {
        let bad: Vec<(usize, Op)> = func
            .code
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, op)| !fits(*op, module, func))
            .collect();
        assert!(bad.is_empty(), "{name} {}: {bad:?}", func.name);
        assert_eq!(func.code.last(), Some(&Op::Ret), "{name} {}", func.name);
        assert_eq!(func.spans.len(), func.code.len(), "{name} {}", func.name);
    }
}

/// Whether the indices an op names are in range.
fn fits(op: Op, module: &Module, func: &Func) -> bool
{
    let below = |n: u32, len: usize| (n as usize) < len;
    match op
    {
        Op::LocalAddr(n) => below(n, func.locals.len()),
        Op::GlobalAddr(n) => below(n, module.globals.len()),
        Op::Jump(t) | Op::JumpIfFalse(t) | Op::JumpIfTrue(t) =>
        {
            below(t, func.code.len())
        }
        Op::Call(f) => below(f, module.funcs.len()),
        Op::PrintText(t) | Op::Text(t) => below(t, module.texts.len()),
        Op::Format(f) => module.formats.get(f as usize).is_some_and(|args| {
            args.iter().all(
                |a| !matches!(a, FmtArg::Text(t) if !below(*t, module.texts.len())),
            )
        }),
        _ => true,
    }
}

#[test]
fn clean_inputs_match_their_expected_output()
{
    for name in CLEAN_INPUTS
    {
        let artifact = emit_input(name, &FoldOptions::default());
        let msgs: Vec<&str> =
            artifact.diagnostics.iter().map(|d| d.message.as_str()).collect();
        assert!(artifact.diagnostics.is_empty(), "{name}: {msgs:?}");
        let module = artifact.module.as_ref().expect("emitted");
        assert_well_formed(name, module);
        let encoded = artifact.encode(ArtifactFormat::Text).unwrap();
        check_file(&format!("{name}.zkbct"), &String::from_utf8(encoded).unwrap());
        check_file(&format!("{name}.zkasm"), &listing(module));
    }
}

#[test]
fn unfolded_inputs_are_well_formed()
{
    for name in CLEAN_INPUTS
    {
        let artifact = emit_input(name, &FoldOptions::none());
        assert_well_formed(name, artifact.module.as_ref().expect("emitted"));
    }
}

#[test]
fn repo_samples_are_well_formed()
{
    let samples = root().join("../../samples");
    for group in std::fs::read_dir(&samples).unwrap().flatten()
    {
        let skip = group.file_name() == "diagnostics" || !group.path().is_dir();
        if skip
        {
            continue;
        }
        let files = std::fs::read_dir(group.path()).unwrap().flatten();
        // The files a sample imports sit in a folder below.
        for file in files.filter(|f| f.path().extension().is_some_and(|e| e == "zaka"))
        {
            let mut sm = SourceMap::new();
            let source = sm.load_file(&file.path()).unwrap();
            let loaded = load(&source, &mut sm, &LexerOptions::default()).unwrap();
            let options = SemcheckOptions::default();
            let folding = FoldOptions::default();
            let mistake = group.file_name() == "mistakes";
            let check = tern!(mistake, MemCheck::Off, MemCheck::Error);
            let mem = BorrowCheckOptions::builder().check(check).build();
            let results = ResultMode::default();
            let emitted = run_loaded(loaded, &options, &folding, &mem, results);
            let artifact = CodegenArtifact::from_emitted(&source, emitted);
            let name = file.path().display().to_string();
            let module = artifact.module.as_ref().unwrap_or_else(|| panic!("{name}"));
            assert_well_formed(&name, module);
        }
    }
}
