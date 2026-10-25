use std::collections::BTreeSet;
use std::path::PathBuf;

use zakac_diagnostics::Level;
use zakac_explorer::compile::Options;
use zakac_explorer::compile::PartKind;
use zakac_explorer::compile::Phase;
use zakac_explorer::compile::Status;
use zakac_explorer::compile::compile;
use zakac_explorer::compile::samples::DIAGNOSTICS;
use zakac_explorer::compile::samples::GROUPS;
use zakac_explorer::compile::samples::SAMPLES;
use zakac_explorer::compile::samples::find_sample;
use zakac_ir_builder::ir::FoldOptions;
use zakac_rust_sugars::tern;

#[test]
fn all_samples_compile()
{
    for sample in SAMPLES
    {
        let out = compile(sample.text, &Options::default());
        assert!(!out.tokens.is_empty(), "{}", sample.name);
        assert_eq!(out.source, sample.text);
    }
}

#[test]
fn clean_groups_have_no_diagnostics()
{
    let clean = SAMPLES.iter().filter(|s| s.group != DIAGNOSTICS);
    for sample in clean
    {
        let out = compile(sample.text, &Options::default());
        assert!(out.checked, "{}", sample.path());
        assert!(out.diagnostics.is_empty(), "{}: {:?}", sample.path(), out.diagnostics);
    }
}

#[test]
fn particle_warnings_belong_to_the_check_stage()
{
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။ က သည် ၁ နှင့် ညီသည် ကို ပြသပါ။";
    let out = compile(text, &Options::default());
    assert!(out.checked);
    assert_eq!(out.count(Level::Warning), 1);
    let row = out.diagnostics.iter().find(|d| d.level == Level::Warning).unwrap();
    assert_eq!(row.phase, Phase::Check);
    assert_eq!(row.code, "ZK0505");
    assert_eq!(out.status(Phase::Check), Status::Warned);
}

#[test]
fn diagnostics_group_reports_errors()
{
    for sample in SAMPLES.iter().filter(|s| s.group == DIAGNOSTICS)
    {
        // The platform sample is clean at 64 bits on purpose.
        let bits = tern!(sample.name == "platform_bits", 32, 64);
        let options = Options::builder().platform_bits(bits).build();
        let out = compile(sample.text, &options);
        assert!(out.count(Level::Error) > 0, "{}", sample.path());
    }
}

#[test]
fn all_sample_files_are_listed()
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../samples");
    let mut on_disk = BTreeSet::new();
    for group in GROUPS
    {
        let dir = std::fs::read_dir(root.join(group)).unwrap();
        for item in dir
        {
            let path = item.unwrap().path();
            assert_eq!(path.extension().and_then(|e| e.to_str()), Some("zaka"));
            let stem = path.file_stem().unwrap().to_str().unwrap();
            on_disk.insert(format!("{group}/{stem}"));
        }
    }
    let listed: BTreeSet<String> = SAMPLES.iter().map(|s| s.path()).collect();
    assert_eq!(on_disk, listed);
    assert_eq!(listed.len(), SAMPLES.len(), "a path is listed twice");
}

#[test]
fn token_ranges_cover_their_lexemes()
{
    // The samples are in normalized form, so the original text under each
    // range is the lexeme the lexer saw.
    for sample in SAMPLES
    {
        let out = compile(sample.text, &Options::default());
        for token in &out.tokens
        {
            let text = &sample.text[token.range.lo as usize..token.range.hi as usize];
            assert_eq!(text, token.lexeme, "{}", sample.name);
        }
    }
}

#[test]
fn token_ranges_are_sorted_and_apart()
{
    let out =
        compile(find_sample("basics/variables").unwrap().text, &Options::default());
    let ranges = out.token_ranges();
    assert!(ranges.windows(2).all(|w| w[0].hi <= w[1].lo));
}

#[test]
fn clean_program_is_checked()
{
    let out =
        compile(find_sample("basics/variables").unwrap().text, &Options::default());
    assert!(out.checked);
    assert_eq!(out.count(Level::Error), 0);
    assert!(out.sexpr.starts_with("(program "));
}

#[test]
fn lexer_errors_skip_the_check()
{
    let out = compile("က ; ခ", &Options::default());
    assert!(!out.checked);
    assert!(out.count(Level::Error) > 0);
    let diag = &out.diagnostics[0];
    assert_eq!(diag.code, "ZK0101");
    let joined: String = diag.pieces.iter().map(|p| p.text.as_str()).collect();
    assert!(joined.starts_with("error[ZK0101]"));
    assert!(joined.contains("explorer.zaka:1:3"));
}

#[test]
fn check_errors_are_reported()
{
    let out = compile("က ကို ပြသပါ။", &Options::default());
    assert!(out.checked);
    assert_eq!(out.diagnostics[0].code, "ZK0301");
    assert_eq!(out.diagnostics[0].range.map(|r| r.lo), Some(0));
}

#[test]
fn diagnostics_hold_their_phase()
{
    let lexer = compile("က ; ခ", &Options::default());
    assert_eq!(lexer.diagnostics[0].code, "ZK0101");
    assert_eq!(lexer.diagnostics[0].phase, Phase::Lex);
    let check = compile("က ကို ပြသပါ။", &Options::default());
    assert!(check.diagnostics.iter().all(|d| d.phase == Phase::Check));
    let parser = compile(
        find_sample("diagnostics/parser_errors").unwrap().text,
        &Options::default(),
    );
    assert!(parser.diagnostics.iter().any(|d| d.phase == Phase::Parse));
}

#[test]
fn phase_status_follows_the_diagnostics()
{
    let out = compile("က ကို ပြသပါ။", &Options::default());
    assert_eq!(out.status(Phase::Segment), Status::Clean);
    assert_eq!(out.status(Phase::Lex), Status::Clean);
    assert_eq!(out.status(Phase::Parse), Status::Clean);
    assert_eq!(out.status(Phase::Check), Status::Failed);
    assert_eq!(out.status(Phase::Lower), Status::Skipped);
}

#[test]
fn clean_samples_lower_without_diagnostics()
{
    for sample in SAMPLES.iter().filter(|s| s.group != DIAGNOSTICS)
    {
        let out = compile(sample.text, &Options::default());
        assert!(out.lowered, "{}", sample.path());
        assert_eq!(out.status(Phase::Lower), Status::Clean, "{}", sample.path());
        assert!(out.ir_folded_sexpr.starts_with("(module "), "{}", sample.path());
    }
}

#[test]
fn fold_errors_belong_to_the_lowering_stage()
{
    let text = find_sample("diagnostics/fold_errors").unwrap().text;
    let out = compile(text, &Options::default());
    assert!(out.lowered);
    assert_eq!(out.status(Phase::Check), Status::Clean);
    assert_eq!(out.status(Phase::Lower), Status::Failed);
    let codes: Vec<&str> = out.diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert_eq!(codes, ["ZK0601", "ZK0602", "ZK0602", "ZK0407"]);
    assert!(out.diagnostics.iter().all(|d| d.phase == Phase::Lower));
}

#[test]
fn fold_options_change_the_folded_ir()
{
    let text = "(၂ + ၃) * ၄ ကို ပြသပါ။";
    let all = compile(text, &Options::default());
    assert!(
        all.ir_folded_sexpr.contains("(print 20:isize)"),
        "{}",
        all.ir_folded_sexpr
    );
    assert!(all.ir_plain_sexpr.contains("(+ 2:isize 3:isize)"));
    let options = Options::builder().fold(FoldOptions::none()).build();
    let none = compile(text, &options);
    assert_eq!(none.ir_folded_sexpr, none.ir_plain_sexpr);
}

#[test]
fn check_status_is_skipped_after_lexer_errors()
{
    let out = compile("က ; ခ", &Options::default());
    assert_eq!(out.status(Phase::Lex), Status::Failed);
    assert_eq!(out.status(Phase::Check), Status::Skipped);
}

#[test]
fn debug_mode_opens_the_last_phase_before_the_first_failure()
{
    let clean = find_sample("programs/gcd").expect("the sample");
    assert_eq!(compile(clean.text, &Options::default()).last_passed(), Phase::Run);
    let lex = compile("က ; ခ", &Options::default());
    assert_eq!(lex.last_passed(), Phase::Segment);
    let parse = compile("ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ\nက ကို ပြသပါ။", &Options::default());
    assert_eq!(parse.status(Phase::Parse), Status::Failed);
    assert_eq!(parse.last_passed(), Phase::Lex);
}

#[test]
fn blank_source_compiles()
{
    let out = compile("", &Options::default());
    assert!(out.tokens.is_empty());
    assert!(out.diagnostics.is_empty());
    assert_eq!(out.tree.label, "program");
}

#[test]
fn strings_show_their_value_in_pieces_and_their_source()
{
    let out = compile("\"က\\nခ\" ကို ပြသပါ။", &Options::default());
    let row = &out.strings[0];
    let pieces: Vec<(PartKind, &str)> =
        row.value.iter().map(|p| (p.kind, p.text.as_str())).collect();
    assert_eq!(
        pieces,
        [(PartKind::Text, "က"), (PartKind::Escape, "\\n"), (PartKind::Text, "ခ")]
    );
    assert_eq!(row.source, "\"က\\nခ\"");
    let span = &out.source[row.range.lo as usize..row.range.hi as usize];
    assert_eq!(span, row.source);
}

#[test]
fn clean_samples_generate_ops_that_run()
{
    for sample in SAMPLES.iter().filter(|s| s.group != DIAGNOSTICS)
    {
        let out = compile(sample.text, &Options::default());
        assert!(out.emitted, "{}", sample.path());
        assert_eq!(out.status(Phase::Codegen), Status::Clean, "{}", sample.path());
        assert_eq!(out.status(Phase::Run), Status::Clean, "{}", sample.path());
        let ops = out.ops.as_ref().expect("ops");
        assert_eq!(out.asm.len(), ops.funcs.len(), "{}", sample.path());
        for (func, asm) in ops.funcs.iter().zip(&out.asm)
        {
            assert_eq!(asm.rows.len(), func.code.len(), "{}", sample.path());
            assert_eq!(asm.locals.len(), func.locals.len(), "{}", sample.path());
        }
        assert_eq!(out.globals.len(), ops.globals.len(), "{}", sample.path());
    }
}

#[test]
fn op_rows_point_at_their_source()
{
    let text = "\"က\" ဟု ပြသပါ။\n၁ + ၂ ကို ပြသပါ။";
    let out = compile(text, &Options::default());
    let main = out.asm.last().expect("main");
    assert_eq!(main.head(), "f0 main (params 0)");
    assert_eq!(out.texts, ["က"]);
    let first = main.rows[0].range.expect("range");
    assert_eq!(&text[first.lo as usize..first.hi as usize], "\"က\" ဟု ပြသပါ။");
    assert_eq!(main.rows[0].line, Some(1));
    assert_eq!(main.rows[1].line, Some(2));
    let last = main.rows.last().expect("ret");
    assert_eq!(last.text, "ret");
    assert!(last.range.is_some_and(|r| r.is_blank()));
}

#[test]
fn jump_targets_get_labels()
{
    let text = "ကိန်းရှင် က သည် မှန်မှားတန်ဖိုး မှန် ဖြစ်ပါစေ။\nအကယ်၍ က ဖြစ်လျှင် \"ဟုတ်\" ဟု ပြသပါ။";
    let out = compile(text, &Options::default());
    let main = out.asm.last().expect("main");
    let jump = main.rows.iter().find(|r| r.text.starts_with("jump_if_false"));
    let jump = jump.expect("jump");
    let target = main.rows.iter().find(|r| r.label.is_some()).expect("target");
    assert_eq!(jump.text, format!("jump_if_false L{}", target.label.unwrap()));
}

#[test]
fn codegen_skips_after_fold_errors()
{
    let text = find_sample("diagnostics/fold_errors").unwrap().text;
    let out = compile(text, &Options::default());
    assert!(!out.emitted);
    assert_eq!(out.status(Phase::Codegen), Status::Skipped);
    assert_eq!(out.status(Phase::Run), Status::Skipped);
    assert!(out.ops.is_none() && out.asm.is_empty());
}

#[test]
fn compiling_does_not_run_the_ops()
{
    let text = "ကိန်းရှင် သုည သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\n\"အရင်\" ဟု ပြသပါ။\n၁ / သုည ကို ပြသပါ။";
    let out = compile(text, &Options::default());
    assert_eq!(out.status(Phase::Codegen), Status::Clean);
    assert_eq!(out.status(Phase::Run), Status::Clean);
}
