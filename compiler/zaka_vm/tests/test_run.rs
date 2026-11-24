use std::io::Cursor;
use std::path::PathBuf;

use termcolor::Buffer;
use zaka_vm::test_run::ZakaTest;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::ExitCode;

/// The folder of the test files: a module with a test of its own, a file of
/// tests that pass and a file with a test that fails.
fn fixture() -> PathBuf
{
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/inputs/test_blocks")
}

/// Run `zakac test` with arguments.
///
/// # Arguments
///
/// * `args` - The arguments.
///
/// # Returns
///
/// The exit code, standard output and standard error.
fn run_tests(args: &[&str]) -> (ExitCode, String, String)
{
    let mut argv = vec![ZakaTest::standalone_name().to_string()];
    argv.extend(args.iter().map(|a| (*a).to_string()));
    let mut stdin = Cursor::new(Vec::new());
    let mut stdout = Buffer::no_color();
    let mut stderr = Buffer::no_color();
    let mut io = CommandIO::builder()
        .stdin(&mut stdin)
        .stdout(&mut stdout)
        .stderr(&mut stderr)
        .build();
    let code = ZakaTest::create_as_standalone().execute(&argv, &mut io);
    let stdout = String::from_utf8(stdout.into_inner()).unwrap();
    let stderr = String::from_utf8(stderr.into_inner()).unwrap();
    (code, stdout, stderr)
}

/// Each test of the files and of the module these import runs once; a failed
/// test shows what it printed and the stop, and fails the run.
#[test]
fn each_test_runs_once_and_a_failure_fails_the_run()
{
    let folder = fixture().join("tests");
    let (code, stdout, stderr) = run_tests(&[folder.to_str().unwrap()]);
    assert_eq!(code, ExitCode::Failure, "{stdout}{stderr}");
    assert_eq!(stdout.matches("နှစ်ဆ_အတွင်း ... ok").count(), 1, "{stdout}");
    assert!(stdout.contains("::ဆယ် ... ok"), "{stdout}");
    assert!(stdout.contains("::အတွင်းလုပ်ဆောင်ချက် ... ok"), "{stdout}");
    assert!(stdout.contains("::မှားသော ... FAILED"), "{stdout}");
    assert!(stdout.contains("ဤစာ ပေါ်ရမည်"), "{stdout}");
    assert!(stdout.contains("did not hold: နှစ်ဆ မှားသည် (၄ == ၅)"), "{stdout}");
    assert!(stdout.ends_with("test result: FAILED. 3 passed, 1 failed\n"), "{stdout}");
}

/// The filter keeps the tests whose names hold its word, and a run with no
/// failure ends with success.
#[test]
fn the_filter_keeps_the_tests_it_names()
{
    let folder = fixture().join("tests");
    let (code, stdout, _) = run_tests(&[folder.to_str().unwrap(), "--filter", "ဆယ်"]);
    assert_eq!(code, ExitCode::Success, "{stdout}");
    assert!(stdout.ends_with("test result: ok. 1 passed, 0 failed\n"), "{stdout}");
}

/// A path that does not exist stops the command before a test runs.
#[test]
fn a_missing_path_is_an_error()
{
    let missing = fixture().join("no_such_folder");
    let (code, stdout, stderr) = run_tests(&[missing.to_str().unwrap()]);
    assert_ne!(code, ExitCode::Success);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(!stderr.is_empty());
}

/// A test that runs past the most ops fails, and the run goes on.
#[test]
fn a_test_past_the_step_limit_fails()
{
    let file = fixture().join("more/endless.zaka");
    let (code, stdout, _) =
        run_tests(&[file.to_str().unwrap(), "--max-steps", "10000"]);
    assert_eq!(code, ExitCode::Failure, "{stdout}");
    assert!(stdout.contains("::အဆုံးမဲ့ ... FAILED"), "{stdout}");
    assert!(stdout.contains("stopped after 10000 ops"), "{stdout}");
}

/// A file whose parse errors hide its tests fails, and shows the errors.
#[test]
fn a_file_with_parse_errors_fails()
{
    let file = fixture().join("more/hidden_under_errors.zaka");
    let (code, stdout, stderr) = run_tests(&[file.to_str().unwrap()]);
    assert_eq!(code, ExitCode::Failure, "{stdout}");
    assert!(stdout.contains("hidden_under_errors.zaka ... FAILED"), "{stdout}");
    assert!(stderr.contains("error[ZK0203]"), "{stderr}");
}
