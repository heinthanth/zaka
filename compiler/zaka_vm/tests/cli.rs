use std::io::Cursor;

use termcolor::Buffer;
use zaka_vm::cli::ZakaVm;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::ExitCode;
use zakac_codegen::cli::ZakacCodegen;

const SUM: &str = "(၂ + ၃) * ၄ ကို ပြသပါ။\n\"ပြီး\" ဟု ပြသပါ။";

/// Run a command with arguments and stdin.
fn run_cmd(
    command: Box<dyn Command>, name: &str, args: &[&str], stdin: &[u8],
) -> (ExitCode, String, String)
{
    let mut argv = vec![name.to_string()];
    argv.extend(args.iter().map(|arg| (*arg).to_string()));
    let mut stdin = Cursor::new(stdin.to_vec());
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

/// Run the VM command.
fn run_vm(args: &[&str], stdin: &str) -> (ExitCode, String, String)
{
    run_cmd(
        ZakaVm::create_as_standalone(),
        ZakaVm::standalone_name(),
        args,
        stdin.as_bytes(),
    )
}

#[test]
fn source_on_stdin_runs()
{
    let (code, stdout, stderr) = run_vm(&[], SUM);
    assert_eq!(code, ExitCode::Success, "{stderr}");
    assert_eq!(stdout, "၂၀\nပြီး\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn codegen_artifacts_run_raw_or_text()
{
    let dir = tempfile::tempdir().unwrap();
    for ext in ["zkbc", "zkbct"]
    {
        let path = dir.path().join(format!("sum.{ext}"));
        let codegen = ZakacCodegen::create_as_standalone();
        let name = ZakacCodegen::standalone_name();
        let (code, _, _) =
            run_cmd(codegen, name, &["-o", path.to_str().unwrap()], SUM.as_bytes());
        assert_eq!(code, ExitCode::Success);
        let (code, stdout, stderr) = run_vm(&[path.to_str().unwrap()], "");
        assert_eq!(code, ExitCode::Success, "{ext}: {stderr}");
        assert_eq!(stdout, "၂၀\nပြီး\n", "{ext}");
        let content = std::fs::read(&path).unwrap();
        let vm = ZakaVm::create_as_standalone();
        let (code, stdout, _) = run_cmd(vm, ZakaVm::standalone_name(), &[], &content);
        assert_eq!(
            (code, stdout.as_str()),
            (ExitCode::Success, "၂၀\nပြီး\n"),
            "{ext} on stdin"
        );
    }
}

#[test]
fn compile_errors_stop_before_running()
{
    let (code, stdout, stderr) = run_vm(&[], "\"အရင်\" ဟု ပြသပါ။\nက ကို ပြသပါ။");
    assert_eq!(code, ExitCode::Failure);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.starts_with("error[ZK0301]"), "{stderr}");
}

#[test]
fn traps_report_on_stderr_and_fail()
{
    let text = "ကိန်းရှင် သုည သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\n\"အရင်\" ဟု ပြသပါ။\n၁ / သုည ကို ပြသပါ။";
    let (code, stdout, stderr) = run_vm(&[], text);
    assert_eq!(code, ExitCode::Failure);
    assert_eq!(stdout, "အရင်\n");
    assert!(stderr.contains("the program stopped"), "{stderr}");
    assert!(stderr.contains("zero as divisor"), "{stderr}");
}

#[test]
fn limits_come_from_flags()
{
    let text = "မှန် ဖြစ်နေသရွေ့ \"ထပ်\" ဟု ပြသပါ။";
    let (code, stdout, stderr) = run_vm(&["--max-steps", "40"], text);
    assert_eq!(code, ExitCode::Failure);
    assert!(stdout.lines().count() > 5, "{stdout}");
    assert!(stderr.contains("stopped after 40 ops"), "{stderr}");
    let (code, _, _) = run_vm(&["--max-steps", "0"], text);
    assert_ne!(code, ExitCode::Success);
}

#[test]
fn platform_bits_change_the_width()
{
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် ၁၂၇ ဖြစ်ပါစေ။\nက + ၁ ကို ပြသပါ။";
    let (code, stdout, _) = run_vm(&["--platform-bits", "8"], text);
    assert_eq!(code, ExitCode::Success);
    assert_eq!(stdout, "-၁၂၈\n");
}

/// The mistake sample that reads a block through a borrow after its owner
/// frees it.
const USE_AFTER_FREE: &str =
    include_str!("../../../samples/mistakes/use_after_free.zaka");

/// The trap text of a read inside a freed block.
const FREED_TRAP: &str = "inside a freed block";

#[test]
fn the_mem_check_flag_picks_error_warn_or_off()
{
    let (code, stdout, stderr) = run_vm(&[], USE_AFTER_FREE);
    assert_eq!(code, ExitCode::Failure);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("error[ZK0704]"), "{stderr}");
    assert!(!stderr.contains(FREED_TRAP), "{stderr}");
    let (code, _, stderr) = run_vm(&["--mem-check", "warn"], USE_AFTER_FREE);
    assert_eq!(code, ExitCode::Failure);
    assert!(stderr.contains("warning[ZK0704]"), "{stderr}");
    assert!(stderr.contains(FREED_TRAP), "{stderr}");
    let (code, _, stderr) = run_vm(&["--mem-check", "off"], USE_AFTER_FREE);
    assert_eq!(code, ExitCode::Failure);
    assert!(!stderr.contains("ZK07"), "{stderr}");
    assert!(stderr.contains(FREED_TRAP), "{stderr}");
}

#[test]
fn the_leaks_flag_lists_unfreed_blocks_and_keeps_success()
{
    let text = "ကိန်းရှင် p သည် လိပ်စာ ဖြစ်ပါစေ။\nမင်မိုရီ ၁၆ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။";
    let (code, stdout, stderr) = run_vm(&["--mem-check", "off", "--leaks"], text);
    assert_eq!(code, ExitCode::Success, "{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    let leaks: Vec<&str> = stderr.lines().filter(|l| l.starts_with("leak:")).collect();
    assert_eq!(leaks.len(), 1, "{stderr}");
    assert!(leaks[0].ends_with("16 octets"), "{stderr}");
    let (code, _, stderr) = run_vm(&["--mem-check", "off"], text);
    assert_eq!(code, ExitCode::Success);
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn the_max_heap_flag_caps_allocation()
{
    let text = "ကိန်းရှင် p သည် လိပ်စာ ဖြစ်ပါစေ။\nမင်မိုရီ ၆၄ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။\np \
                ကို ပြသပါ။";
    let (code, stdout, stderr) =
        run_vm(&["--mem-check", "off", "--max-heap", "16"], text);
    assert_eq!(code, ExitCode::Success, "{stderr}");
    assert_eq!(stdout, "ဗလာ\n");
    let (_, stdout, _) = run_vm(&["--mem-check", "off"], text);
    assert!(stdout.starts_with("0x"), "{stdout}");
}
