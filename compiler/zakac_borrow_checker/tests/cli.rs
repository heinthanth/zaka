use std::io::Cursor;
use std::path::PathBuf;

use termcolor::Buffer;
use zakac_borrow_checker::cli::ZakacBorrowChecker;
use zakac_borrow_checker::cli::attach_borrow_check_args;
use zakac_borrow_checker::cli::borrow_check_options;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_borrow_checker::options::MemCheck;
use zakac_cli_common::clap;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::ExitCode;
use zakac_session::artifact::RAW_ARTIFACT_MAGIC;

const TEXT_HEADER: &str = "# zakac-borrow-check-artifact v1\n";

/// Run the standalone ownership check command.
fn run_cli(args: &[&str], stdin: &str) -> (ExitCode, String, String)
{
    let mut argv = vec![ZakacBorrowChecker::standalone_name().to_string()];
    argv.extend(args.iter().map(|arg| (*arg).to_string()));

    let command = ZakacBorrowChecker::create_as_standalone();
    let mut stdin = Cursor::new(stdin.to_string());
    let mut stdout = Buffer::no_color();
    let mut stderr = Buffer::no_color();
    let mut io = CommandIO::builder()
        .stdin(&mut stdin)
        .stdout(&mut stdout)
        .stderr(&mut stderr)
        .build();

    let code = command.execute(&argv, &mut io);
    let stdout = String::from_utf8(stdout.into_inner()).unwrap();
    let stderr = String::from_utf8(stderr.into_inner()).unwrap();
    (code, stdout, stderr)
}

/// Parse a command line with just the ownership check flags.
fn parse(args: &[&str]) -> Result<BorrowCheckOptions, clap::Error>
{
    let cmd = attach_borrow_check_args(clap::Command::new("t"));
    let argv = std::iter::once("t").chain(args.iter().copied());
    cmd.try_get_matches_from(argv).map(|m| borrow_check_options(&m))
}

#[test]
fn borrow_check_stdin_without_input()
{
    let (code, stdout, stderr) = run_cli(&[], "၁ ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn borrow_check_input_file()
{
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/inputs/clean.zaka");
    let (code, stdout, stderr) = run_cli(&[path.to_str().unwrap()], "");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn mem_check_words_are_parsed()
{
    let options = parse(&["--mem-check", "warn", "--mem-strict"]).unwrap();
    assert_eq!(options.check, MemCheck::Warn);
    assert!(options.strict);
    assert_eq!(parse(&["--mem-check", "off"]).unwrap().check, MemCheck::Off);
    assert_eq!(parse(&["--mem-check", "error"]).unwrap().check, MemCheck::Error);
}

#[test]
fn mem_check_defaults_to_error_and_not_strict()
{
    let options = parse(&[]).unwrap();
    assert_eq!(options, BorrowCheckOptions::default());
    assert_eq!(options.check, MemCheck::Error);
    assert!(!options.strict);
}

#[test]
fn mem_check_rejects_other_words()
{
    assert!(parse(&["--mem-check", "loud"]).is_err());
    let (code, _, _) = run_cli(&["--mem-check", "loud"], "၁ ကို ပြသပါ။");
    assert_ne!(code, ExitCode::Success);
}

#[test]
fn earlier_errors_fail_but_still_write_the_artifact()
{
    let (code, stdout, stderr) = run_cli(&[], "က ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Failure);
    assert!(stdout.starts_with(TEXT_HEADER), "{stdout}");
    assert!(stderr.starts_with("error[ZK0301]"), "{stderr}");
}

#[test]
fn output_extensions_pick_the_format()
{
    let dir = tempfile::tempdir().unwrap();
    let raw = dir.path().join("out.zkbck");
    let (code, stdout, stderr) = run_cli(&["-o", raw.to_str().unwrap()], "၁ ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.is_empty() && stderr.is_empty());
    assert!(std::fs::read(&raw).unwrap().starts_with(RAW_ARTIFACT_MAGIC));
    let text = dir.path().join("out.zkbckt");
    let (code, _, _) = run_cli(&["-o", text.to_str().unwrap()], "၁ ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Success);
    assert!(std::fs::read_to_string(&text).unwrap().starts_with(TEXT_HEADER));
}
