//! The formatters of the debuggers: `zakac debug-script` prints them, and lldb
//! or gdb, when the host has one, shows text as a string and a plain list as
//! its cells in a native build.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::time::Duration;
use std::time::Instant;

use zakac_rust_sugars::ret_if;

/// The program of the test: locals of each common kind, among them text and
/// a plain list.
fn program() -> PathBuf
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.join("../zakac_codegen_target_c/tests/inputs/debug/vars.zaka")
}

/// Run `zakac` with arguments in a folder.
///
/// # Arguments
///
/// * `dir` - The folder.
/// * `args` - The arguments.
///
/// # Returns
///
/// What it printed.
fn zakac(dir: &Path, args: &[&str]) -> Output
{
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zakac"));
    cmd.current_dir(dir).env("ZAKAC_NO_CONFIG", "1").args(args);
    cmd.output().unwrap()
}

/// Run a debugger, stopped after a minute.
///
/// # Arguments
///
/// * `cmd` - The debugger and its arguments.
///
/// # Returns
///
/// What it printed.
fn within_a_minute(cmd: &mut Command) -> String
{
    use std::process::Stdio;

    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let end = Instant::now() + Duration::from_secs(60);
    while child.try_wait().unwrap().is_none() && Instant::now() < end
    {
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = child.kill();
    let out = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Whether the host runs a tool.
///
/// # Arguments
///
/// * `tool` - The name of the tool.
///
/// # Returns
///
/// `true` when `<tool> --version` runs.
fn has(tool: &str) -> bool
{
    let ran = Command::new(tool).arg("--version").output();
    ran.is_ok_and(|o| o.status.success())
}

/// Build the program of the test with the native backend in a folder.
///
/// # Arguments
///
/// * `dir` - The folder.
///
/// # Returns
///
/// The executable and the line of the program to stop on, or `None` when
/// the host links no native build.
fn built(dir: &Path) -> Option<(PathBuf, usize)>
{
    let text = std::fs::read_to_string(program()).unwrap();
    std::fs::write(dir.join("vars.zaka"), &text).unwrap();
    let args =
        ["build", "--backend", "native", "--cache", "cache", "-o", "vars", "vars.zaka"];
    let out = zakac(dir, &args);
    ret_if!(!out.status.success(), None);
    let line = text.lines().position(|l| l.starts_with("ခ ကို ပြသမည်"))? + 1;
    Some((dir.join(format!("vars{}", std::env::consts::EXE_SUFFIX)), line))
}

#[test]
fn debug_script_prints_the_script_of_each_debugger()
{
    let dir = tempfile::tempdir().unwrap();
    let lldb = zakac(dir.path(), &["debug-script", "lldb"]);
    assert!(String::from_utf8_lossy(&lldb.stdout).contains("__lldb_init_module"));
    let gdb = zakac(dir.path(), &["debug-script", "gdb"]);
    assert!(String::from_utf8_lossy(&gdb.stdout).contains("gdb.pretty_printers"));
}

#[test]
fn lldb_shows_text_and_a_plain_list_through_the_script()
{
    ret_if!(!has("lldb"));
    let dir = tempfile::tempdir().unwrap();
    let Some((exe, line)) = built(dir.path())
    else
    {
        return;
    };
    let script = dir.path().join("zaka_lldb.py");
    std::fs::write(&script, zakac(dir.path(), &["debug-script", "lldb"]).stdout)
        .unwrap();
    let mut cmd = Command::new("lldb");
    cmd.arg("--batch")
        .args(["-o", &format!("command script import {}", script.display())]);
    cmd.args([
        "-o",
        &format!("b vars.zaka:{line}"),
        "-o",
        "run",
        "-o",
        "frame variable",
    ]);
    let shown = within_a_minute(cmd.args(["-o", "kill", "-o", "quit"]).arg(&exe));
    assert!(shown.contains("င = \"မင်္ဂလာပါ\""), "{shown}");
    assert!(shown.contains("ဆ = ([0] = 4, [1] = 5)"), "{shown}");
}

#[test]
fn gdb_shows_text_and_a_plain_list_through_the_script_of_the_build()
{
    ret_if!(!has("gdb") || cfg!(target_os = "macos"));
    let dir = tempfile::tempdir().unwrap();
    let Some((exe, line)) = built(dir.path())
    else
    {
        return;
    };
    let mut cmd = Command::new("gdb");
    let safe = format!("add-auto-load-safe-path {}", dir.path().display());
    cmd.arg("-batch").args(["-iex", &safe, "-ex", &format!("break vars.zaka:{line}")]);
    let shown =
        within_a_minute(cmd.args(["-ex", "run", "-ex", "info locals"]).arg(&exe));
    assert!(shown.contains("င = \"မင်္ဂလာပါ\""), "{shown}");
    assert!(shown.contains("ဆ = 2 cells = {4, 5}"), "{shown}");
}
