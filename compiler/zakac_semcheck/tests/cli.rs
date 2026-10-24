use std::io::Cursor;
use std::path::PathBuf;

use termcolor::Buffer;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::ExitCode;
use zakac_semcheck::cli::ZakacSemcheck;
use zakac_session::artifact::RAW_ARTIFACT_MAGIC;

const TEXT_HEADER: &str = "# zakac-semcheck-artifact v1\n";

/// An `i32` value stored in a platform-sized integer.
const PLATFORM_WIDENING: &str = "ကိန်းရှင် က သည် ၃၂ ဘစ် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် က ဖြစ်ပါစေ။";

/// Run the standalone check command.
fn run_cli(args: &[&str], stdin: &str) -> (ExitCode, String, String)
{
    let mut argv = vec![ZakacSemcheck::standalone_name().to_string()];
    argv.extend(args.iter().map(|arg| (*arg).to_string()));

    let command = ZakacSemcheck::create_as_standalone();
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
fn check_stdin_without_input()
{
    let (code, stdout, stderr) = run_cli(&[], "ကိန်းရှင် က သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stdout.contains("isize"));
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn check_input_file()
{
    let (code, stdout, stderr) = run_cli(&[&sample("programs")], "");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn diagnostics_go_to_stderr_and_fail()
{
    let (code, stdout, stderr) = run_cli(&[], "က ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Failure);
    assert!(stdout.starts_with(TEXT_HEADER), "the artifact is still written");
    assert!(stderr.starts_with("error[ZK0301]"), "{stderr}");
    assert!(stderr.contains("<stdin>:1:"), "{stderr}");
}

#[test]
fn parser_errors_stop_the_check()
{
    let (code, _stdout, stderr) = run_cli(&[], "က ကို ကို ပြသပါ။\nခ ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Failure);
    assert!(stderr.starts_with("error[ZK0207]"), "{stderr}");
    assert!(!stderr.contains("ZK0301"), "{stderr}");
}

#[test]
fn platform_bits_change_widening()
{
    let (code, _, stderr) = run_cli(&[], PLATFORM_WIDENING);
    assert_eq!(code, ExitCode::Success, "{stderr}");
    let (code, _, stderr) = run_cli(&["--platform-bits", "32"], PLATFORM_WIDENING);
    assert_eq!(code, ExitCode::Failure);
    assert!(stderr.starts_with("error[ZK0401]"), "{stderr}");
}

#[test]
fn platform_bits_must_be_a_kind_width()
{
    let (code, _, _) = run_cli(&["--platform-bits", "7"], PLATFORM_WIDENING);
    assert_ne!(code, ExitCode::Success);
}

#[test]
fn output_zksem_writes_raw_artifact()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.zksem");
    let (code, stdout, stderr) =
        run_cli(&["-o", out_path.to_str().unwrap()], PLATFORM_WIDENING);
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.is_empty() && stderr.is_empty());
    assert!(std::fs::read(&out_path).unwrap().starts_with(RAW_ARTIFACT_MAGIC));
}

#[test]
fn output_zksemt_writes_text_artifact()
{
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.zksemt");
    let (code, _stdout, stderr) =
        run_cli(&["-o", out_path.to_str().unwrap()], PLATFORM_WIDENING);
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty());
    assert!(std::fs::read_to_string(&out_path).unwrap().starts_with(TEXT_HEADER));
}
