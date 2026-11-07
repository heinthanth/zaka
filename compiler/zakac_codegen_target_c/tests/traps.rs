mod common;

use std::path::Path;

use common::c_files;
use common::cc;
use common::compile_and_run;
use common::root;
use common::samples;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::Checks;

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
    assert!(run.stderr.ends_with(&line), "stderr: {}", run.stderr);
}

/// A division by zero stops the program, as in the VM.
#[test]
fn a_zero_divisor_stops_the_program()
{
    let path = samples().join("mistakes/zero_divide.zaka");
    let trap = "integer division or remainder with zero as divisor";
    stops_with(&path, "စားမည်\n", trap);
}

/// An overflow stops the program under `--overflow stop`, as in the VM.
#[test]
fn an_overflow_stops_the_program()
{
    let path = root().join("tests/inputs/overflow.zaka");
    let trap = "the result of + does not fit isize";
    stops_with(&path, "၉၂၂၃၃၇၂၀၃၆၈၅၄၇၇၅၈၀၇\n", trap);
}
