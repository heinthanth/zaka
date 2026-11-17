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
