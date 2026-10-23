use std::io::Cursor;
use std::path::PathBuf;

use termcolor::Buffer;
use zakac_burmese_segmenter::cli::ZakacBurmeseSegmenter;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::ExitCode;
use zakac_session::artifact::RAW_ARTIFACT_MAGIC;

const TEXT_HEADER: &str = "# zakac-segment-artifact v1\n";

fn run_cli(args: &[&str], stdin: &str) -> (ExitCode, String, String)
{
    let mut argv = vec![ZakacBurmeseSegmenter::standalone_name().to_string()];
    argv.extend(args.iter().map(|arg| (*arg).to_string()));

    let command = ZakacBurmeseSegmenter::create_as_standalone();

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

#[test]
fn segment_stdin_without_input()
{
    let (code, stdout, stderr) = run_cli(&[], "ကာကား");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stdout.contains("ကာကား"));
    assert!(stderr.is_empty());
}

#[test]
fn segment_input_file()
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/paragraph1.in");
    let (code, stdout, stderr) = run_cli(&[path.to_str().unwrap()], "");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stderr.is_empty());
}

#[test]
fn missing_input_file_reports_error()
{
    let (code, _stdout, stderr) = run_cli(&["/no/such/file.zaka"], "");
    assert_eq!(code, ExitCode::IoError);
    assert!(stderr.contains(ZakacBurmeseSegmenter::name()));
    assert!(stderr.contains("can't find"));
}

#[test]
fn coalesce_non_burmese_flag()
{
    let (code, stdout, stderr) = run_cli(&["--coalesce-non-burmese"], "hello");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stdout.contains("hello"));
    assert!(stderr.is_empty());
}

#[test]
fn stdout_dash_writes_text_artifact()
{
    let (code, stdout, stderr) = run_cli(&["-"], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stderr.is_empty());
}

#[test]
fn output_zksegt_writes_text_artifact()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.zksegt");
    let out = out_path.to_str().unwrap();

    let (code, stdout, stderr) = run_cli(&["-o", out], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());

    let contents = std::fs::read_to_string(&out_path).unwrap();
    assert!(contents.starts_with(TEXT_HEADER));
}

#[test]
fn output_json_defaults_to_raw()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.json");
    let out = out_path.to_str().unwrap();

    let (code, _stdout, stderr) = run_cli(&["-o", out], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());

    let raw = std::fs::read(&out_path).unwrap();
    assert!(raw.starts_with(RAW_ARTIFACT_MAGIC));
}

#[test]
fn output_without_extension_writes_raw_artifact()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out");
    let out = out_path.to_str().unwrap();

    let (code, _stdout, stderr) = run_cli(&["-o", out], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());

    let contents = std::fs::read_to_string(&out_path).unwrap();
    let raw = std::fs::read(&out_path).unwrap();
    assert_eq!(contents.len(), raw.len());
    assert!(raw.starts_with(RAW_ARTIFACT_MAGIC));
}

#[test]
fn output_zkseg_writes_raw_artifact()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.zkseg");
    let out = out_path.to_str().unwrap();

    let (code, _stdout, stderr) = run_cli(&["-o", out], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());

    let raw = std::fs::read(&out_path).unwrap();
    assert!(raw.starts_with(RAW_ARTIFACT_MAGIC));
}

#[test]
fn unknown_extension_defaults_to_raw()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.bin");
    let out = out_path.to_str().unwrap();

    let (code, _stdout, stderr) = run_cli(&["-o", out], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());

    let raw = std::fs::read(&out_path).unwrap();
    assert!(raw.starts_with(RAW_ARTIFACT_MAGIC));
}

#[test]
fn output_dir_writes_basename_zkseg()
{
    let dir = tempfile::tempdir().unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/paragraph1.in");
    let out_dir = dir.path().join("segments");
    std::fs::create_dir(&out_dir).unwrap();
    let out = format!("{}/", out_dir.to_str().unwrap());

    let (code, _stdout, stderr) = run_cli(&[path.to_str().unwrap(), "-o", &out], "");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());

    let expected = out_dir.join("paragraph1.zkseg");
    assert!(expected.is_file());
    let raw = std::fs::read(&expected).unwrap();
    assert!(raw.starts_with(RAW_ARTIFACT_MAGIC));
}

#[test]
fn output_format_raw_overrides_zksegt_extension()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.zksegt");
    let out = out_path.to_str().unwrap();

    let (code, _stdout, stderr) = run_cli(&["--output-format", "raw", "-o", out], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());

    let raw = std::fs::read(&out_path).unwrap();
    assert!(raw.starts_with(RAW_ARTIFACT_MAGIC));
}

#[test]
fn output_format_text_to_dir_writes_zksegt()
{
    let dir = tempfile::tempdir().unwrap();
    let out_dir = dir.path().join("text_out");
    std::fs::create_dir(&out_dir).unwrap();
    let out = format!("{}/", out_dir.to_str().unwrap());

    let (code, _stdout, stderr) =
        run_cli(&["--output-format", "text", "-o", &out], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());

    let expected = out_dir.join("stdin.zksegt");
    assert!(expected.is_file());
    let contents = std::fs::read_to_string(&expected).unwrap();
    assert!(contents.starts_with(TEXT_HEADER));
}

#[test]
fn output_txt_defaults_to_raw()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.txt");
    let out = out_path.to_str().unwrap();

    let (code, _stdout, stderr) = run_cli(&["-o", out], "က");
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());

    let raw = std::fs::read(&out_path).unwrap();
    assert!(raw.starts_with(RAW_ARTIFACT_MAGIC));
}
