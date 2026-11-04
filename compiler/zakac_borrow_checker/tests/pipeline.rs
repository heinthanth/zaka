mod common;

use common::ok;
use common::run;
use zakac_ast::stmt::START_NAME;
use zakac_borrow_checker::digest::FnDigest;
use zakac_borrow_checker::digest::Root;
use zakac_borrow_checker::digest::Seg;
use zakac_ir_builder::ir::Place;

#[test]
fn a_start_block_alone_runs_through_borrow_check_with_one_digest()
{
    let run = run("၁ ကို ပြသပါ။");
    assert!(run.out.diagnostics.is_empty());
    let names: Vec<&str> = run.out.digests.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, [START_NAME]);
}

#[test]
fn digests_are_blank_per_function()
{
    let run = run(include_str!("inputs/clean.zaka"));
    assert!(run.out.diagnostics.is_empty(), "{:?}", run.out.diagnostics);
    let module = run.out.lowered.folded.as_ref().expect("folded");
    let blanks: Vec<FnDigest> = module.funcs.iter().map(FnDigest::blank).collect();
    assert_eq!(module.funcs.len(), 4);
    assert_eq!(run.out.digests, blanks);
    assert!(run.out.digests.iter().all(|d| d.frees.is_empty()));
}

#[test]
fn a_blank_digest_prints_as_one_line()
{
    let run = run(include_str!("inputs/clean.zaka"));
    let lines: Vec<String> = run.out.digests.iter().map(|d| d.to_string()).collect();
    let name = &run.out.digests[0].name;
    assert_eq!(lines[0], format!("{name}: params none; globals none"));
    assert!(lines.iter().all(|line| !line.contains('\n')));
}

#[test]
fn a_digest_prints_the_params_then_the_globals_it_frees()
{
    let param = Root { place: Place::Local(1), path: vec![Seg::Block] };
    let local = Root { place: Place::Local(3), path: vec![] };
    let global = Root {
        place: Place::Global(2),
        path: vec![Seg::Block, Seg::Cell(Some(4)), Seg::Cell(None)],
    };
    let frees = vec![param, local, global];
    let digest = FnDigest::builder().name("f").params(2).frees(frees).build();
    let want = "f: params l1.block; globals g2.block.cell[4].cell[*]";
    assert_eq!(digest.to_string(), want);
}

#[test]
fn earlier_errors_skip_the_check()
{
    let run = run("က ကို ပြသပါ။");
    assert!(!run.out.diagnostics.is_empty());
    assert!(run.out.digests.is_empty());
    assert!(run.out.lowered.folded.is_none());
}

#[test]
fn clean_programs_have_no_diagnostics() { ok(include_str!("inputs/clean.zaka")); }
