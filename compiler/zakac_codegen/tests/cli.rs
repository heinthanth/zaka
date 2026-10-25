use std::io::Cursor;

use termcolor::Buffer;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::ExitCode;
use zakac_codegen::cli::ZakacCodegen;
use zakac_codegen::listing::LISTING_HEADER;
use zakac_session::artifact::RAW_ARTIFACT_MAGIC;

const TEXT_HEADER: &str = "# zakac-codegen-artifact v1\n";

const SUM: &str = "(၂ + ၃) * ၄ ကို ပြသပါ။";

/// Run the command with arguments and stdin.
fn run_cli(args: &[&str], stdin: &str) -> (ExitCode, String, String)
{
    let mut argv = vec![ZakacCodegen::standalone_name().to_string()];
    argv.extend(args.iter().map(|arg| (*arg).to_string()));

    let command = ZakacCodegen::create_as_standalone();
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
fn stdin_gives_a_text_artifact()
{
    let (code, stdout, stderr) = run_cli(&[], SUM);
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER), "{stdout}");
    assert!(stdout.contains("push: 20:isize"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn asm_writes_the_listing()
{
    let (code, stdout, _) = run_cli(&["--asm"], SUM);
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(LISTING_HEADER), "{stdout}");
    assert!(stdout.contains("push 20:isize"), "{stdout}");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.zkasm");
    let (code, stdout, _) = run_cli(&["--asm", "-o", path.to_str().unwrap()], SUM);
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.is_empty());
    assert!(std::fs::read_to_string(&path).unwrap().starts_with(LISTING_HEADER));
}

#[test]
fn fold_none_keeps_the_operations()
{
    let (_, stdout, _) = run_cli(&["--asm", "--fold", "none"], SUM);
    assert!(stdout.contains("mul"), "{stdout}");
}

#[test]
fn check_errors_leave_no_module()
{
    let (code, stdout, stderr) = run_cli(&[], "က ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Failure);
    assert!(stdout.contains("module: null"), "{stdout}");
    assert!(stderr.starts_with("error[ZK0301]"), "{stderr}");
}

#[test]
fn output_extensions_pick_the_format()
{
    let dir = tempfile::tempdir().unwrap();
    let raw = dir.path().join("out.zkbc");
    let (code, stdout, stderr) = run_cli(&["-o", raw.to_str().unwrap()], SUM);
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.is_empty() && stderr.is_empty());
    assert!(std::fs::read(&raw).unwrap().starts_with(RAW_ARTIFACT_MAGIC));
    let text = dir.path().join("out.zkbct");
    let (code, _, _) = run_cli(&["-o", text.to_str().unwrap()], SUM);
    assert_eq!(code, ExitCode::Success);
    assert!(std::fs::read_to_string(&text).unwrap().starts_with(TEXT_HEADER));
}
