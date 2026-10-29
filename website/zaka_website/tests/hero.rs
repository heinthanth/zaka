use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_with;
use zaka_website::components::hero::LINE;
use zaka_website::components::hero::PRINTED;
use zaka_website::components::hero::syllables;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_codegen::artifact::run_phases;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

#[test]
fn the_hero_line_prints_what_the_hero_shows()
{
    let mut map = SourceMap::new();
    let file =
        map.new_source_file(FileName::Custom("hero.zaka".into()), LINE.to_string());
    let emitted = run_phases(
        &file,
        &LexerOptions::default(),
        &SemcheckOptions::default(),
        &FoldOptions::default(),
        &BorrowCheckOptions::default(),
    );
    let module = emitted.module.expect("the hero line compiles");
    let mut out = Vec::new();
    run_with(&module, &mut out, &RunOptions::default()).expect("the hero line runs");
    assert_eq!(String::from_utf8(out).unwrap(), format!("{PRINTED}\n"));
}

#[test]
fn the_hero_line_is_written_out_in_whole_syllables()
{
    let pieces = syllables(LINE);
    assert_eq!(pieces.concat(), LINE);
    assert!(pieces.contains(&"င်္ဂ".to_string()), "{pieces:?}");
    assert!(pieces.contains(&"လာ".to_string()), "{pieces:?}");
    assert!(pieces.contains(&"ပြ".to_string()), "{pieces:?}");
}
