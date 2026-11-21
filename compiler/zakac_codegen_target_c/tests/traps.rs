use std::path::Path;

use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::Checks;
use zakac_codegen_target_c::options::TargetCOptions;

use crate::common::c_files;
use crate::common::c_files_with;
use crate::common::cc;
use crate::common::compile_and_run;
use crate::common::root;
use crate::common::samples;
use crate::common::stop_line;

/// Build a program that stops, and check what it printed before the stop
/// and the trap line after it.
///
/// # Arguments
///
/// * `path` - The program.
/// * `printed` - What it prints before it stops.
/// * `trap` - The text of the trap.
fn stops_with(path: &Path, printed: &str, trap: &str)
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let files = c_files(path, OverflowMode::Stop, Checks::Default).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
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
    stops_with(&path, "စားမည်\n", trap);
}

/// A read of the data of an empty optional stops the program, as in the VM.
#[test]
fn an_empty_optional_read_stops_the_program()
{
    let path = root().join("tests/inputs/empty_optional.zaka");
    stops_with(&path, "၇\n", "the optional holds no data");
}

/// An overflow stops the program under `--overflow stop`, as in the VM.
#[test]
fn an_overflow_stops_the_program()
{
    let path = root().join("tests/inputs/overflow.zaka");
    let trap = "the result of + does not fit isize";
    stops_with(&path, "၉၂၂၃၃၇၂၀၃၆၈၅၄၇၇၅၈၀၇\n", trap);
}

/// A cell past the end of a list stops the program, as in the VM: the
/// runtime files keep the checks of the build.
#[test]
fn a_cell_past_the_end_stops_the_program()
{
    let path = root().join("tests/inputs/cell_past_end.zaka");
    let trap = "cell 3 is outside the list, whose cells run from 0 to 2";
    stops_with(&path, "", trap);
}

/// A recursion deeper than the depth limit stops the program, as in the VM,
/// where the native stack would overflow.
#[test]
fn a_deep_recursion_stops_the_program()
{
    let path = root().join("tests/inputs/deep_recursion.zaka");
    stops_with(&path, "", "more than 4096 nested calls");
}

/// A stop names the function and the line of the statement, then the calls
/// in progress, the most recent first; the main function C starts with is
/// left out.
#[test]
fn a_stop_names_its_line_and_the_calls()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/stop_in_a_call.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert!(run.stderr.contains("(in စား, at "), "{}", run.stderr);
    assert!(run.stderr.contains("stop_in_a_call.zaka:3:1)"), "{}", run.stderr);
    assert!(run.stderr.contains("stack backtrace"), "{}", run.stderr);
    assert!(run.stderr.contains("stop_in_a_call.zaka:6:1"), "{}", run.stderr);
    assert!(!run.stderr.contains("  main"), "{}", run.stderr);
}

/// A release build keeps no lines: a stop names the function alone.
#[test]
fn a_release_build_names_the_function()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/stop_in_a_call.zaka");
    let target = TargetCOptions::builder().debug_info(false).build();
    let files = c_files_with(&path, OverflowMode::Stop, target).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert!(run.stderr.contains("(in စား)"), "{}", run.stderr);
    assert!(!run.stderr.contains(".zaka:"), "{}", run.stderr);
}

/// A list that grows past `--max-heap` stops the program, as in the VM.
#[test]
fn a_heap_past_the_cap_stops_the_program()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/grow.zaka");
    let target = TargetCOptions::builder().max_heap(4000).build();
    let files = c_files_with(&path, OverflowMode::Stop, target).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(run.status, Some(1), "{}", run.stderr);
    let line = "the program stopped: no heap room for 4096 octets\n";
    assert_eq!(stop_line(&run.stderr), line, "{}", run.stderr);
}

/// A loop that leaves a ring behind on each round fills a small heap while the
/// collections are paused, and fits it when each round runs the collector, as
/// in the VM.
#[test]
fn the_ring_statements_reach_the_runtime()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let target = TargetCOptions::builder().max_heap(64 * 1024).build();
    let run_input = |name: &str| {
        let files = c_files_with(
            &root().join("tests/inputs").join(name),
            OverflowMode::Stop,
            target,
        );
        compile_and_run(&cc, &files.unwrap(), Checks::Default)
    };
    let paused = run_input("rings_paused.zaka");
    assert_eq!(paused.status, Some(1), "{}", paused.stderr);
    assert!(stop_line(&paused.stderr).contains("no heap room"), "{}", paused.stderr);
    let collected = run_input("rings_collected.zaka");
    assert_eq!(collected.status, Some(0), "{}", collected.stderr);
    // Each round collects once, and the rings it frees count.
    assert_eq!(collected.stdout, "ပြီး\n၂၀၀၀\nမှန်\n");
}

/// What `threads.zaka` prints.
const THREADS_OUT: &str = "၁၀၁၀၀\nမင်္ဂလာ!\nမင်္ဂလာ!\nစာကြောင်း ၁\nစာကြောင်း ၂\nစာကြောင်း \
                           ၃\nစာကြောင်း ၄\nစာကြောင်း ၅\n";

/// Threads run on OS threads of their own and channels take values between
/// them, as in the VM.
#[test]
fn threads_and_channels_run_on_os_threads()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/threads.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, THREADS_OUT);
}

/// A trap in a thread ends that thread alone, and the program ends with code 1.
#[test]
fn a_trap_in_a_thread_ends_it_alone()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/thread_trap.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(run.status, Some(1), "{}", run.stderr);
    assert_eq!(run.stdout, "၅\n");
    assert!(
        run.stderr.contains("thread 1 stopped: integer division"),
        "{}",
        run.stderr
    );
}

/// A second wait for a thread stops the program: the first took its result.
#[test]
fn a_second_wait_for_a_thread_stops_the_program()
{
    let path = root().join("tests/inputs/wait_twice.zaka");
    stops_with(
        &path,
        "က!\n",
        "the thread 1 this waits for gave its result to an earlier wait",
    );
}

/// A receive no thread can serve stops the program.
#[test]
fn a_wait_no_thread_can_end_stops_the_program()
{
    let path = root().join("tests/inputs/deadlock.zaka");
    stops_with(&path, "", "each thread waits; none can go on");
}

/// Two threads change one shared object inside lock blocks, and lose no change.
#[test]
fn lock_blocks_keep_the_changes_of_two_threads_apart()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/locks.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "၂၀၀၀\n၂၀၀၀\n");
}

/// Two threads add to one atomic number with no lock, and lose no add.
#[test]
fn atomic_adds_of_two_threads_lose_nothing()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/atomics.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout,
        "၁၉၈၀\n၇\n၄\nရလဒ်ကောင်း(၃)\nအမှားအယွင်း(၉)\nရလဒ်ကောင်း(၇)\n၈\n၅၀၀၅၀၀\n"
    );
}

/// A parallel map runs its rounds on threads and gives its results in the
/// order of the cells.
#[test]
fn a_parallel_map_runs_its_rounds_on_threads()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/par_map.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "၁၀၈၈၉၄\n၁၂၃၄၆!\n၃၃၈၃၅၀\n၁၀၀၀၀\n၃၉၆၉\n[၂၅၊ ၃၆]\n");
}

/// The ring collector runs while threads run: it halts the OS threads at safe
/// points.
#[test]
fn the_ring_collector_runs_while_threads_run()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/ring_threads.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "collected while threads ran\n");
}

/// The ring collector halts a thread that spins in a loop with no
/// allocation: each round of the loop passes a safe point.
#[test]
fn the_ring_collector_halts_a_loop_with_no_allocation()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/ring_loop.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "collected beside a loop\n");
}

/// Threads that print at once never mix their lines.
#[test]
fn the_prints_of_threads_come_out_as_whole_lines()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/thread_lines.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
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

/// What `par_loop.zaka` prints.
const PAR_LOOP_OUT: &str = "၁၄၉၉၈၅၀၀၀\n[၂၊ ၄၊ ၆၊ ၈၊ ၁၀၊ ၁၂၊ ၁၄၊ ၁၆၊ ၁၈၊ ၂၀၊ ၂၂၊ ၂၄၊ \
                            ၂၆၊ ၂၈၊ ၃၀၊ ၃၂၊ ၃၄၊ ၃၆၊ ၃၈၊ ၄၀]\n[၁၊ ၂၊ ၃၊ ၄၊ ၅၊ ၆၊ ၇၊ ၈၊ \
                            ၉၊ ၁၀၊ ၁၁၊ ၁၂၊ ၁၃၊ ၁၄၊ ၁၅၊ ၁၆၊ ၁၇၊ ၁၈၊ ၁၉၊ ၂၀]\n";

/// The rounds of parallel loops run on threads, and a trap in a round ends
/// its thread: the loop then stops with the trap of the thread.
#[test]
fn the_rounds_of_a_parallel_loop_run_on_threads()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/par_loop.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, PAR_LOOP_OUT);
    let path = root().join("tests/inputs/par_loop_trap.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let trap = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(trap.status, Some(1), "{}", trap.stderr);
    assert_eq!(trap.stdout, "");
    assert!(trap.stderr.contains("stopped: integer division"), "{}", trap.stderr);
    assert!(
        trap.stderr.contains("this waits for stopped at a trap"),
        "{}",
        trap.stderr
    );
}

/// What `par_loop_steps.zaka` prints.
const PAR_STEPS_OUT: &str = "၅၁၆၉၉\n[မ၊ င၊ ်၊ ္၊ ဂ၊ လ၊ ာ၊ ပ၊ ါ၊  ၊ a၊ b၊ c]\n";

/// What `par_loop_rows.zaka` prints.
const PAR_ROWS_OUT: &str = "[၁၊ ၁၁၊ ၂၁၊ ၃၁၊ ၄၁၊ ၅၁၊ ၆၁၊ ၀၊ ၈၁၊ ၉၁၊ ၁၀၁၊ ၁၁၁၊ ၁၂၁၊ \
                            ၁၃၁၊ ၁၄၁၊ ၁၅၁၊ ၁၆၁၊ ၁၇၁၊ ၁၈၁၊ ၁၉၁၊ ၂၀၁၊ ၂၁၁၊ ၂၂၁၊ ၂၃၁၊ \
                            ၂၄၁၊ ၂၅၁၊ ၂၆၁၊ ၂၇၁၊ ၂၈၁၊ ၂၉၁]\n";

/// Counted loops with a step, up and down, a loop over characters and one
/// over the rows of a ဇယား spread their rounds too.
#[test]
fn steps_characters_and_rows_spread_their_rounds()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    for (input, want) in
        [("par_loop_steps.zaka", PAR_STEPS_OUT), ("par_loop_rows.zaka", PAR_ROWS_OUT)]
    {
        let path = root().join("tests/inputs").join(input);
        let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
        let run = compile_and_run(&cc, &files, Checks::Default);
        assert_eq!(run.status, Some(0), "{input}: {}", run.stderr);
        assert_eq!(run.stdout, want, "{input}");
    }
}

/// A loop and a map run on the count of threads given to each: three threads
/// each stop when each round traps.
#[test]
fn a_count_of_threads_sets_the_threads_of_a_loop()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let path = root().join("tests/inputs/par_threads.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let run = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "၉၉၀၀\n[၁၊ ၄၊ ၉၊ ၁၆၊ ၂၅]\n");
    let path = root().join("tests/inputs/par_threads_trap.zaka");
    let files = c_files(&path, OverflowMode::Stop, Checks::Default).unwrap();
    let trap = compile_and_run(&cc, &files, Checks::Default);
    assert_eq!(trap.status, Some(1), "{}", trap.stderr);
    assert_eq!(
        trap.stderr.matches("stopped: integer division").count(),
        3,
        "{}",
        trap.stderr
    );
}
