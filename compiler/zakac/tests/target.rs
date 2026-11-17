//! The target a program is built for: the facts it prints, the branch a
//! compile-time conditional keeps, and the refusal to run or build a program
//! for another OS on this host.

use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

/// A program that prints the OS it was built for, and names an
/// undeclared name in an arm the host never keeps.
const PROGRAM: &str = "\
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလုပ်လုပ်မည့်ကွန်ပျူတာစနစ် ကို ပြသမည်။
အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ် == ၈ ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
မရှိသောနာမည် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// The program, written into a folder of its own.
fn program_file(name: &str) -> PathBuf
{
    let dir = std::env::temp_dir().join(format!("zakac-target-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("{name}.zaka"));
    std::fs::write(&file, PROGRAM).unwrap();
    file
}

/// Run `zakac` with some arguments.
fn zakac(args: &[&str]) -> Output
{
    Command::new(env!("CARGO_BIN_EXE_zakac")).args(args).output().unwrap()
}

/// An OS that is not the host's.
fn other_os() -> &'static str
{
    if cfg!(target_os = "linux")
    {
        "windows"
    }
    else
    {
        "linux"
    }
}

/// A run prints the host's OS, and the arm the host does not keep is
/// never checked.
#[test]
fn a_run_holds_the_host_and_skips_the_dropped_arm()
{
    let file = program_file("host");
    let out = zakac(&["run", file.to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), std::env::consts::OS);
}

/// The VM refuses to run a program built for another OS, and the C
/// target refuses to build an executable for one.
#[test]
fn another_os_is_refused_on_this_host()
{
    let file = program_file("other");
    let path = file.to_str().unwrap();
    let run = zakac(&["run", "--target-os", other_os(), path]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("built for"));
    let exe = file.with_extension("");
    let build = zakac(&[
        "build",
        "--backend",
        "c",
        "--target-os",
        other_os(),
        path,
        "-o",
        exe.to_str().unwrap(),
    ]);
    assert!(!build.status.success());
    assert!(String::from_utf8_lossy(&build.stderr).contains("--emit c"));
}
