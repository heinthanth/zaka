use std::time::Duration;

use strum::IntoEnumIterator;
use zaka_explorer::compile::Options;
use zaka_explorer::compile::Phase;
use zaka_explorer::compile::compile;
use zaka_explorer::compile::samples::find_sample;
use zaka_explorer::compile::timing::PhaseTimes;
use zaka_explorer::compile::timing::time_text;

#[test]
fn a_clean_compile_times_each_phase_but_the_run()
{
    let text = find_sample("basics/variables").unwrap().text;
    let times = compile(text, &Options::default()).times;
    let timed: Vec<Phase> = Phase::iter().filter(|p| times.of(*p).is_some()).collect();
    let want: Vec<Phase> = Phase::iter().filter(|p| *p != Phase::Run).collect();
    assert_eq!(timed, want);
}

#[test]
fn skipped_phases_take_no_time()
{
    let times = compile("က ; ခ", &Options::default()).times;
    assert!(times.of(Phase::Lex).is_some());
    assert_eq!(times.of(Phase::Check), None);
    assert_eq!(times.of(Phase::Lower), None);
    assert_eq!(times.of(Phase::BorrowCheck), None);
    assert_eq!(times.of(Phase::Codegen), None);
}

#[test]
fn the_total_adds_the_phases_that_ran()
{
    let ms = Duration::from_millis;
    let times = PhaseTimes::builder().segment(ms(1)).lex(ms(2)).parse(ms(4)).build();
    assert_eq!(times.total(), ms(7));
}

#[test]
fn a_time_reads_in_a_fitting_unit()
{
    assert_eq!(time_text(Duration::from_micros(420)), "420 µs");
    assert_eq!(time_text(Duration::from_micros(1_240)), "1.24 ms");
    assert_eq!(time_text(Duration::from_micros(12_400)), "12.4 ms");
    assert_eq!(time_text(Duration::from_millis(240)), "240 ms");
    assert_eq!(time_text(Duration::from_millis(1_240)), "1.24 s");
    assert_eq!(time_text(Duration::ZERO), "0 µs");
}
