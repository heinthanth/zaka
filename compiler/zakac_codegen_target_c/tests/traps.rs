mod common;

use std::path::Path;

use common::c_files;
use common::c_files_with;
use common::cc;
use common::compile_and_run;
use common::root;
use common::samples;
use common::stop_line;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::Checks;
use zakac_codegen_target_c::options::TargetCOptions;

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
