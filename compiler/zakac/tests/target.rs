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
    Command::new(env!("CARGO_BIN_EXE_zakac"))
        .env("ZAKAC_NO_CONFIG", "1")
        .args(args)
        .output()
        .unwrap()
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

/// A build for another OS is refused with the runtime lib `zakac` carries,
/// and goes through with one `--runtime-lib` names; `true` stands in for a C
/// compiler of that OS.
#[cfg(unix)]
#[test]
fn a_runtime_lib_for_another_os_lets_a_build_through()
{
    let file = program_file("cross");
    let dir = file.parent().unwrap().to_path_buf();
    let lib = dir.join("libzaka_runtime.a");
    std::fs::write(&lib, b"!<arch>\n").unwrap();
    let cache = dir.join("cross-cache");
    let path = file.to_str().unwrap();
    let base = ["build", "--backend", "c", "--target-os", other_os(), "--cc", "true"];
    let mut args: Vec<&str> = base.to_vec();
    args.extend(["--cache", cache.to_str().unwrap(), path]);
    let refused = zakac(&args);
    assert!(!refused.status.success());
    args.extend(["--runtime-lib", lib.to_str().unwrap()]);
    let built = zakac(&args);
    let stderr = String::from_utf8_lossy(&built.stderr);
    assert!(built.status.success(), "{stderr}");
}
