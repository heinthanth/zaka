use std::io::Cursor;
use std::path::PathBuf;

use termcolor::Buffer;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::ExitCode;
use zakac_ir_builder::cli::ZakacIrBuilder;
use zakac_session::artifact::RAW_ARTIFACT_MAGIC;

const TEXT_HEADER: &str = "# zakac-ir-artifact v1\n";

/// Literal arithmetic that folds.
const SUM: &str = "(၂ + ၃) * ၄ ကို ပြသပါ။";

/// Run the standalone lowering command.
fn run_cli(args: &[&str], stdin: &str) -> (ExitCode, String, String)
{
    let mut argv = vec![ZakacIrBuilder::standalone_name().to_string()];
    argv.extend(args.iter().map(|arg| (*arg).to_string()));

    let command = ZakacIrBuilder::create_as_standalone();
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

/// Path of an input file.
fn input(name: &str) -> String
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/inputs/{name}.zaka"));
    path.to_str().unwrap().to_string()
}

#[test]
fn lower_stdin_without_input()
{
    let (code, stdout, stderr) = run_cli(&[], SUM);
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stdout.contains("20:isize"), "{stdout}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn lower_input_file()
{
    let (code, stdout, stderr) = run_cli(&[&input("programs")], "");
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.starts_with(TEXT_HEADER));
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn fold_none_keeps_the_operations()
{
    let (code, stdout, _) = run_cli(&["--fold", "none"], SUM);
    assert_eq!(code, ExitCode::Success);
    assert!(!stdout.contains("20:isize"), "{stdout}");
    assert!(stdout.contains("2:isize"), "{stdout}");
    let (_, listed, _) = run_cli(&["--fold", "consts,branches"], SUM);
    assert_eq!(listed, stdout);
    let (_, all, _) = run_cli(&["--fold", "exprs"], SUM);
    assert!(all.contains("20:isize"), "{all}");
}

#[test]
fn fold_errors_fail_whatever_the_options()
{
    for fold in ["all", "none"]
    {
        let (code, stdout, stderr) = run_cli(&["--fold", fold], "၁ / ၀ ကို ပြသပါ။");
        assert_eq!(code, ExitCode::Failure);
        assert!(stdout.starts_with(TEXT_HEADER), "the artifact is still written");
        assert!(stderr.starts_with("error[ZK0602]"), "{stderr}");
    }
}

#[test]
fn check_errors_stop_lowering()
{
    let (code, stdout, stderr) = run_cli(&[], "က ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Failure);
    assert!(stdout.contains("module: null"), "{stdout}");
    assert!(stderr.starts_with("error[ZK0301]"), "{stderr}");
}

#[test]
fn unknown_fold_words_are_rejected()
{
    let (code, _, _) = run_cli(&["--fold", "loops"], SUM);
    assert_ne!(code, ExitCode::Success);
}

#[test]
fn output_extensions_pick_the_format()
{
    let dir = tempfile::tempdir().unwrap();
    let raw = dir.path().join("out.zkir");
    let (code, stdout, stderr) = run_cli(&["-o", raw.to_str().unwrap()], SUM);
    assert_eq!(code, ExitCode::Success);
    assert!(stdout.is_empty() && stderr.is_empty());
    assert!(std::fs::read(&raw).unwrap().starts_with(RAW_ARTIFACT_MAGIC));
    let text = dir.path().join("out.zkirt");
    let (code, _, _) = run_cli(&["-o", text.to_str().unwrap()], SUM);
    assert_eq!(code, ExitCode::Success);
    assert!(std::fs::read_to_string(&text).unwrap().starts_with(TEXT_HEADER));
}

#[test]
fn no_raw_rejects_raw_pointer_kinds()
{
    let source = "ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်ပါစေ။";
    let (code, _, stderr) = run_cli(&["--no-raw"], source);
    assert_ne!(code, ExitCode::Success);
    assert!(stderr.contains("ZK0717"), "{stderr}");
    let (code, _, stderr) = run_cli(&[], source);
    assert_eq!(code, ExitCode::Success, "{stderr}");
}
