use std::io::Cursor;
use std::path::PathBuf;

use termcolor::Buffer;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::ExitCode;
use zakac_parser::cli::ZakacParser;
use zakac_session::artifact::RAW_ARTIFACT_MAGIC;

const TEXT_HEADER: &str = "# zakac-parse-artifact v1\n";

/// Run the standalone parser command.
fn run_cli(args: &[&str], stdin: &str) -> (ExitCode, String, String)
{
    let mut argv = vec![ZakacParser::standalone_name().to_string()];
    argv.extend(args.iter().map(|arg| (*arg).to_string()));

    let command = ZakacParser::create_as_standalone();
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

/// Path of a sample file.
fn sample(name: &str) -> String
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/inputs/{name}.zaka"));
    path.to_str().unwrap().to_string()
}

#[test]
fn parse_stdin_without_input()
{
    let (code, stdout, stderr) = run_cli(&[], "အသက် ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stdout.contains("print"));
    assert!(stderr.is_empty());
}

#[test]
fn parse_input_file()
{
    let (code, stdout, stderr) = run_cli(&[&sample("statements")], "");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stderr.is_empty());
}

#[test]
fn diagnostics_go_to_stderr_and_fail()
{
    let (code, stdout, stderr) = run_cli(&[], "က ကို ကို ပြသပါ။\nခ ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Failure);
    assert!(stdout.starts_with(TEXT_HEADER), "the partial tree is still written");
    assert!(stderr.starts_with("error[ZK0207]"), "{stderr}");
    assert!(stderr.contains("<stdin>:1:"), "{stderr}");
}

#[test]
fn lexer_diagnostics_are_reported_too()
{
    let (code, _stdout, stderr) = run_cli(&[], "\"abc\n");
    assert_eq!(code, ExitCode::Failure);
    assert!(stderr.starts_with("error[ZK0104]"), "{stderr}");
}

#[test]
fn output_zkast_writes_raw_artifact()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.zkast");
    let (code, stdout, stderr) =
        run_cli(&["-o", out_path.to_str().unwrap()], "က ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.is_empty() && stderr.is_empty());
    assert!(std::fs::read(&out_path).unwrap().starts_with(RAW_ARTIFACT_MAGIC));
}

#[test]
fn output_zkastt_writes_text_artifact()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.zkastt");
    let (code, _stdout, stderr) =
        run_cli(&["-o", out_path.to_str().unwrap()], "က ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());
    assert!(std::fs::read_to_string(&out_path).unwrap().starts_with(TEXT_HEADER));
}
