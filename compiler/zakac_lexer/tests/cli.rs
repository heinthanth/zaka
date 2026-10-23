use std::io::Cursor;
use std::path::PathBuf;

use termcolor::Buffer;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::ExitCode;
use zakac_lexer::cli::ZakacLexer;
use zakac_session::artifact::RAW_ARTIFACT_MAGIC;

const TEXT_HEADER: &str = "# zakac-lex-artifact v1\n";

/// Run the standalone lexer command.
fn run_cli(args: &[&str], stdin: &str) -> (ExitCode, String, String)
{
    let mut argv = vec![ZakacLexer::standalone_name().to_string()];
    argv.extend(args.iter().map(|arg| (*arg).to_string()));

    let command = ZakacLexer::create_as_standalone();
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
fn lex_stdin_without_input()
{
    let (code, stdout, stderr) = run_cli(&[], "အသက် ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stdout.contains("kw: print"));
    assert!(stderr.is_empty());
}

#[test]
fn lex_input_file()
{
    let (code, stdout, stderr) = run_cli(&[&sample("statements")], "");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stderr.is_empty());
}

#[test]
fn missing_input_file_reports_error()
{
    let (code, _stdout, stderr) = run_cli(&["/no/such/file.zaka"], "");
    assert_eq!(code, ExitCode::IoError);
    assert!(stderr.contains(ZakacLexer::name()));
    assert!(stderr.contains("can't find"));
}

#[test]
fn diagnostics_go_to_stderr_and_fail()
{
    let (code, stdout, stderr) = run_cli(&[], "\"abc\nx");
    assert_eq!(code, ExitCode::Failure);
    assert!(stdout.starts_with(TEXT_HEADER), "artifact is still written");
    let expected = "\
error[ZK0104]: string literal is missing its closing quote before the end of the line
 --> <stdin>:1:1
  |
1 | \"abc
  | ^^^^
";
    assert_eq!(stderr, expected);
}

#[test]
fn output_zktok_writes_raw_artifact()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.zktok");
    let (code, stdout, stderr) = run_cli(&["-o", out_path.to_str().unwrap()], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.is_empty() && stderr.is_empty());
    assert!(std::fs::read(&out_path).unwrap().starts_with(RAW_ARTIFACT_MAGIC));
}

#[test]
fn output_zktokt_writes_text_artifact()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.zktokt");
    let (code, _stdout, stderr) = run_cli(&["-o", out_path.to_str().unwrap()], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());
    assert!(std::fs::read_to_string(&out_path).unwrap().starts_with(TEXT_HEADER));
}

#[test]
fn output_dir_writes_basename_zktok()
{
    let dir = tempfile::tempdir().unwrap();
    let out_dir = dir.path().join("tokens");
    std::fs::create_dir(&out_dir).unwrap();
    let out = format!("{}/", out_dir.to_str().unwrap());
    let (code, _stdout, stderr) = run_cli(&[&sample("functions"), "-o", &out], "");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());
    let raw = std::fs::read(out_dir.join("functions.zktok")).unwrap();
    assert!(raw.starts_with(RAW_ARTIFACT_MAGIC));
}

#[test]
fn output_format_text_to_dir_writes_zktokt()
{
    let dir = tempfile::tempdir().unwrap();
    let out = format!("{}/", dir.path().to_str().unwrap());
    let (code, _stdout, stderr) =
        run_cli(&["--output-format", "text", "-o", &out], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());
    let text = std::fs::read_to_string(dir.path().join("stdin.zktokt")).unwrap();
    assert!(text.starts_with(TEXT_HEADER));
}

#[test]
fn output_txt_defaults_to_raw()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.txt");
    let (code, _stdout, stderr) = run_cli(&["-o", out_path.to_str().unwrap()], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());
    assert!(std::fs::read(&out_path).unwrap().starts_with(RAW_ARTIFACT_MAGIC));
}
