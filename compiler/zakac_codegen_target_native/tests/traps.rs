use std::path::Path;

use crate::common::c_inputs;
use crate::common::run_jit;
use crate::common::samples;
use crate::common::stop_line;

/// Run a program that stops through the JIT, and check what it printed
/// before the stop and the trap line after it.
///
/// # Arguments
///
/// * `path` - The program.
/// * `flags` - The flags of `zakac run`.
/// * `printed` - What it prints before it stops.
/// * `trap` - The text of the trap.
fn stops_with(path: &Path, flags: &[&str], printed: &str, trap: &str)
{
    let run = run_jit(path, flags);
    assert_eq!(run.status, Some(1), "{}", run.stderr);
    assert_eq!(run.stdout, printed);
    let line = format!("the program stopped: {trap}\n");
    assert_eq!(stop_line(&run.stderr), line, "stderr: {}", run.stderr);
}

/// A zero divisor stops the program, as in the VM.
#[test]
fn a_zero_divisor_stops_the_program()
{
    let path = samples().join("mistakes/zero_divide.zaka");
    let trap = "integer division or remainder with zero as divisor";
    stops_with(&path, &[], "စားမည်\n", trap);
}

/// A read of the data of an optional that holds none stops the program.
#[test]
fn an_optional_with_no_data_stops_the_program_when_read()
{
    let path = c_inputs().join("empty_optional.zaka");
    stops_with(&path, &[], "၇\n", "the optional holds no data");
}

/// An overflow stops the program under `--overflow stop`.
#[test]
fn an_overflow_stops_the_program()
{
    let path = c_inputs().join("overflow.zaka");
    let trap = "the result of + does not fit isize";
    stops_with(&path, &[], "၉၂၂၃၃၇၂၀၃၆၈၅၄၇၇၅၈၀၇\n", trap);
}

/// A cell past the end of a list stops the program.
#[test]
fn a_cell_past_the_end_stops_the_program()
{
    let path = c_inputs().join("cell_past_end.zaka");
    let trap = "cell 3 is outside the list, whose cells run from 0 to 2";
    stops_with(&path, &[], "", trap);
}

/// A recursion deeper than the depth limit stops the program, where the
/// native stack would overflow.
#[test]
fn a_deep_recursion_stops_the_program()
{
    let path = c_inputs().join("deep_recursion.zaka");
    stops_with(&path, &[], "", "more than 4096 nested calls");
}

/// A stop names the function and the line of the statement, then the calls
/// in progress, the most recent first; the main function is left out.
#[test]
fn a_stop_names_its_line_and_the_calls()
{
    let run = run_jit(&c_inputs().join("stop_in_a_call.zaka"), &[]);
    assert!(run.stderr.contains("(in စား, at "), "{}", run.stderr);
    assert!(run.stderr.contains("stop_in_a_call.zaka:3:1)"), "{}", run.stderr);
    assert!(run.stderr.contains("stack backtrace"), "{}", run.stderr);
    assert!(run.stderr.contains("stop_in_a_call.zaka:6:1"), "{}", run.stderr);
    assert!(!run.stderr.contains("  main"), "{}", run.stderr);
}

/// A list that grows past `--max-heap` stops the program.
#[test]
fn a_heap_past_the_cap_stops_the_program()
{
    let path = c_inputs().join("grow.zaka");
    let line = "no heap room for 4096 octets";
    stops_with(&path, &["--max-heap", "4000"], "", line);
}

/// A loop that leaves a ring behind on each round fills a small heap while the
/// collections are paused, and fits it when each round runs the collector.
#[test]
fn the_ring_statements_reach_the_runtime()
{
    let flags = ["--max-heap", "65536"];
    let paused = c_inputs().join("rings_paused.zaka");
    stops_with(&paused, &flags, "", "no heap room for 40 octets");
    let run = run_jit(&c_inputs().join("rings_collected.zaka"), &flags);
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    // Each round collects once, and the rings it frees count.
    assert_eq!(run.stdout, "ပြီး\n၂၀၀၀\nမှန်\n");
}

/// What `threads.zaka` prints.
const THREADS_OUT: &str = "၁၀၁၀၀\nမင်္ဂလာ!\nမင်္ဂလာ!\nစာကြောင်း ၁\nစာကြောင်း ၂\nစာကြောင်း \
                           ၃\nစာကြောင်း ၄\nစာကြောင်း ၅\n";

/// Threads run on OS threads of their own in an AOT build and through the JIT,
/// and a trap in one ends it alone.
#[test]
fn threads_run_on_os_threads_in_an_aot_build()
{
    use zakac_borrow_checker::options::OverflowMode;
    let grants = zaka_host::Grants::default();
    let Some(run) = crate::common::run_aot(
        &c_inputs().join("threads.zaka"),
        OverflowMode::Stop,
        &grants,
    )
    else
    {
        return;
    };
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, THREADS_OUT);
    let trap = crate::common::run_aot(
        &c_inputs().join("thread_trap.zaka"),
        OverflowMode::Stop,
        &grants,
    );
    let trap = trap.expect("the first build linked");
    assert_eq!(trap.status, Some(1), "{}", trap.stderr);
    assert_eq!(trap.stdout, "၅\n");
    assert!(
        trap.stderr.contains("thread 1 stopped: integer division"),
        "{}",
        trap.stderr
    );
    let jit = run_jit(&c_inputs().join("threads.zaka"), &[]);
    assert_eq!(
        (jit.status, jit.stdout.as_str()),
        (Some(0), THREADS_OUT),
        "{}",
        jit.stderr
    );
}

/// Two threads change one shared object inside lock blocks in an AOT build, and
/// lose no change.
#[test]
fn lock_blocks_keep_the_changes_of_two_threads_apart()
{
    use zakac_borrow_checker::options::OverflowMode;
    let grants = zaka_host::Grants::default();
    let run = crate::common::run_aot(
        &c_inputs().join("locks.zaka"),
        OverflowMode::Stop,
        &grants,
    );
    let Some(run) = run
    else
    {
        return;
    };
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "၂၀၀၀\n၂၀၀၀\n");
}

/// Two threads add to one atomic number with no lock in an AOT build, and
/// lose no add.
#[test]
fn atomic_adds_of_two_threads_lose_nothing()
{
    use zakac_borrow_checker::options::OverflowMode;
    let grants = zaka_host::Grants::default();
    let run = crate::common::run_aot(
        &c_inputs().join("atomics.zaka"),
        OverflowMode::Stop,
        &grants,
    );
    let Some(run) = run
    else
    {
        return;
    };
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "၁၉၈၀\n၇\n၄\nရလဒ်ကောင်း(၃)\nအမှားအယွင်း(၉)\nရလဒ်ကောင်း(၇)\n၈\n");
}

/// A parallel map runs its rounds on threads in an AOT build and through the
/// JIT, and gives its results in the order of the cells.
/// The ring collector halts a thread that spins in a loop with no
/// allocation, through the JIT and as an executable.
#[test]
fn the_ring_collector_halts_a_loop_with_no_allocation()
{
    use zakac_borrow_checker::options::OverflowMode;
    let path = c_inputs().join("ring_loop.zaka");
    let want = (Some(0), "collected beside a loop\n");
    let jit = run_jit(&path, &[]);
    assert_eq!((jit.status, jit.stdout.as_str()), want, "{}", jit.stderr);
    let grants = zaka_host::Grants::default();
    let Some(run) = crate::common::run_aot(&path, OverflowMode::Stop, &grants)
    else
    {
        return;
    };
    assert_eq!((run.status, run.stdout.as_str()), want, "{}", run.stderr);
}

/// A second wait for a thread stops the program, through the JIT and as an
/// executable: the first wait took its result.
#[test]
fn a_second_wait_for_a_thread_stops_the_program()
{
    use zakac_borrow_checker::options::OverflowMode;
    let path = c_inputs().join("wait_twice.zaka");
    let said = "the thread 1 this waits for gave its result to an earlier wait";
    let jit = run_jit(&path, &[]);
    assert_eq!((jit.status, jit.stdout.as_str()), (Some(1), "က!\n"), "{}", jit.stderr);
    assert!(jit.stderr.contains(said), "{}", jit.stderr);
    let grants = zaka_host::Grants::default();
    let Some(run) = crate::common::run_aot(&path, OverflowMode::Stop, &grants)
    else
    {
        return;
    };
    assert_eq!((run.status, run.stdout.as_str()), (Some(1), "က!\n"), "{}", run.stderr);
    assert!(run.stderr.contains(said), "{}", run.stderr);
}

/// The ring collector runs while threads run, through the JIT and as an
/// executable.
#[test]
fn the_ring_collector_runs_while_threads_run()
{
    use zakac_borrow_checker::options::OverflowMode;
    let path = c_inputs().join("ring_threads.zaka");
    let jit = run_jit(&path, &[]);
    assert_eq!(
        (jit.status, jit.stdout.as_str()),
        (Some(0), "collected while threads ran\n"),
        "{}",
        jit.stderr
    );
    let grants = zaka_host::Grants::default();
    let Some(run) = crate::common::run_aot(&path, OverflowMode::Stop, &grants)
    else
    {
        return;
    };
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "collected while threads ran\n");
}

/// Threads that print at once never mix their lines, through the JIT and as an
/// executable.
#[test]
fn the_prints_of_threads_come_out_as_whole_lines()
{
    use zakac_borrow_checker::options::OverflowMode;
    let path = c_inputs().join("thread_lines.zaka");
    let jit = run_jit(&path, &[]);
    assert_eq!(jit.status, Some(0), "{}", jit.stderr);
    assert_eq!(sorted_lines(&jit.stdout), thread_lines());
    let grants = zaka_host::Grants::default();
    let Some(run) = crate::common::run_aot(&path, OverflowMode::Stop, &grants)
    else
    {
        return;
    };
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(sorted_lines(&run.stdout), thread_lines());
}

/// The lines `thread_lines.zaka` prints, sorted: each of four labels with each
/// number from 1 to 300 in Burmese digits.
fn thread_lines() -> Vec<String>
{
    let burmese = |n: u32| {
        let digit = |c: char| char::from_u32(0x1040 + c.to_digit(10).unwrap_or(0));
        n.to_string().chars().filter_map(digit).collect::<String>()
    };
    let labels = ["က", "ခ", "ဂ", "ဃ"];
    let mut lines: Vec<String> = labels
        .iter()
        .flat_map(|l| (1..=300).map(move |n| format!("{l}-{}", burmese(n))))
        .collect();
    lines.sort();
    lines
}

/// The lines of an output, sorted.
fn sorted_lines(out: &str) -> Vec<String>
{
    let mut lines: Vec<String> = out.lines().map(str::to_string).collect();
    lines.sort();
    lines
}

#[test]
fn a_parallel_map_runs_its_rounds_on_threads()
{
    use zakac_borrow_checker::options::OverflowMode;
    let path = c_inputs().join("par_map.zaka");
    let jit = run_jit(&path, &[]);
    assert_eq!(
        (jit.status, jit.stdout.as_str()),
        (Some(0), "၁၀၈၈၉၄\n၁၂၃၄၆!\n၃၃၈၃၅၀\n၁၀၀၀၀\n"),
        "{}",
        jit.stderr
    );
    let grants = zaka_host::Grants::default();
    let Some(run) = crate::common::run_aot(&path, OverflowMode::Stop, &grants)
    else
    {
        return;
    };
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "၁၀၈၈၉၄\n၁၂၃၄၆!\n၃၃၈၃၅၀\n၁၀၀၀၀\n");
}

/// What `par_loop.zaka` prints.
const PAR_LOOP_OUT: &str = "၁၄၉၉၈၅၀၀၀\n[၂၊ ၄၊ ၆၊ ၈၊ ၁၀၊ ၁၂၊ ၁၄၊ ၁၆၊ ၁၈၊ ၂၀၊ ၂၂၊ ၂၄၊ \
                            ၂၆၊ ၂၈၊ ၃၀၊ ၃၂၊ ၃၄၊ ၃၆၊ ၃၈၊ ၄၀]\n[၁၊ ၂၊ ၃၊ ၄၊ ၅၊ ၆၊ ၇၊ ၈၊ \
                            ၉၊ ၁၀၊ ၁၁၊ ၁၂၊ ၁၃၊ ၁၄၊ ၁၅၊ ၁၆၊ ၁၇၊ ၁၈၊ ၁၉၊ ၂၀]\n";

/// The rounds of parallel loops run on threads in an AOT build and through
/// the JIT, and a trap in a round ends its thread: the loop then stops with the
/// trap of the thread.
#[test]
fn the_rounds_of_a_parallel_loop_run_on_threads()
{
    use zakac_borrow_checker::options::OverflowMode;
    let grants = zaka_host::Grants::default();
    let run = crate::common::run_aot(
        &c_inputs().join("par_loop.zaka"),
        OverflowMode::Stop,
        &grants,
    );
    let Some(run) = run
    else
    {
        return;
    };
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, PAR_LOOP_OUT);
    let trap = crate::common::run_aot(
        &c_inputs().join("par_loop_trap.zaka"),
        OverflowMode::Stop,
        &grants,
    );
    let trap = trap.expect("the first build linked");
    assert_eq!(trap.status, Some(1), "{}", trap.stderr);
    assert_eq!(trap.stdout, "");
    assert!(trap.stderr.contains("stopped: integer division"), "{}", trap.stderr);
    assert!(
        trap.stderr.contains("this waits for stopped at a trap"),
        "{}",
        trap.stderr
    );
    let jit = run_jit(&c_inputs().join("par_loop.zaka"), &[]);
    assert_eq!(
        (jit.status, jit.stdout.as_str()),
        (Some(0), PAR_LOOP_OUT),
        "{}",
        jit.stderr
    );
    let jit_trap = run_jit(&c_inputs().join("par_loop_trap.zaka"), &[]);
    assert_eq!(jit_trap.status, Some(1), "{}", jit_trap.stderr);
    assert!(
        jit_trap.stderr.contains("this waits for stopped at a trap"),
        "{}",
        jit_trap.stderr
    );
}

/// What `par_loop_steps.zaka` prints.
const PAR_STEPS_OUT: &str = "၅၁၆၉၉\n[မ၊ င၊ ်၊ ္၊ ဂ၊ လ၊ ာ၊ ပ၊ ါ၊  ၊ a၊ b၊ c]\n";

/// What `par_loop_rows.zaka` prints.
const PAR_ROWS_OUT: &str = "[၁၊ ၁၁၊ ၂၁၊ ၃၁၊ ၄၁၊ ၅၁၊ ၆၁၊ ၀၊ ၈၁၊ ၉၁၊ ၁၀၁၊ ၁၁၁၊ ၁၂၁၊ \
                            ၁၃၁၊ ၁၄၁၊ ၁၅၁၊ ၁၆၁၊ ၁၇၁၊ ၁၈၁၊ ၁၉၁၊ ၂၀၁၊ ၂၁၁၊ ၂၂၁၊ ၂၃၁၊ \
                            ၂၄၁၊ ၂၅၁၊ ၂၆၁၊ ၂၇၁၊ ၂၈၁၊ ၂၉၁]\n";

/// Counted loops with a step, up and down, a loop over characters and one
/// over the rows of a ဇယား spread their rounds in an AOT build too.
#[test]
fn steps_characters_and_rows_spread_their_rounds()
{
    use zakac_borrow_checker::options::OverflowMode;
    let grants = zaka_host::Grants::default();
    for (input, want) in
        [("par_loop_steps.zaka", PAR_STEPS_OUT), ("par_loop_rows.zaka", PAR_ROWS_OUT)]
    {
        let run = crate::common::run_aot(
            &c_inputs().join(input),
            OverflowMode::Stop,
            &grants,
        );
        let Some(run) = run
        else
        {
            return;
        };
        assert_eq!(run.status, Some(0), "{input}: {}", run.stderr);
        assert_eq!(run.stdout, want, "{input}");
    }
}

/// A loop and a map run on the count of threads given to each in an AOT
/// build: three threads each stop when each round traps.
#[test]
fn a_count_of_threads_sets_the_threads_of_a_loop()
{
    use zakac_borrow_checker::options::OverflowMode;
    let grants = zaka_host::Grants::default();
    let run = crate::common::run_aot(
        &c_inputs().join("par_threads.zaka"),
        OverflowMode::Stop,
        &grants,
    );
    let Some(run) = run
    else
    {
        return;
    };
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "၉၉၀၀\n[၁၊ ၄၊ ၉၊ ၁၆၊ ၂၅]\n");
    let trap = crate::common::run_aot(
        &c_inputs().join("par_threads_trap.zaka"),
        OverflowMode::Stop,
        &grants,
    );
    let trap = trap.expect("the first build linked");
    assert_eq!(trap.status, Some(1), "{}", trap.stderr);
    assert_eq!(
        trap.stderr.matches("stopped: integer division").count(),
        3,
        "{}",
        trap.stderr
    );
}
