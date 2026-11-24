use std::path::PathBuf;
use std::process::Command;

/// The test files of the VM crate: a module with a test of its own, a file of
/// tests that pass and a file with a test that fails.
fn fixture() -> PathBuf
{
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../zaka_vm/tests/inputs/test_blocks")
}

/// With `--jit`, each test runs as native code in a process of its own: a
/// stop fails that test alone, and the others still pass.
#[test]
fn jit_tests_run_each_in_a_process_of_its_own()
{
    let run = Command::new(env!("CARGO_BIN_EXE_zakac"))
        .args(["test", "--jit"])
        .arg(fixture().join("tests"))
        .env("ZAKAC_NO_CONFIG", "1")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(run.status.code(), Some(1), "{stdout}");
    assert!(stdout.contains("::ဆယ် ... ok"), "{stdout}");
    assert!(stdout.contains("::မှားသော ... FAILED"), "{stdout}");
    assert!(stdout.contains("did not hold: နှစ်ဆ မှားသည် (၄ == ၅)"), "{stdout}");
    assert!(stdout.ends_with("test result: FAILED. 3 passed, 1 failed\n"), "{stdout}");
}

/// A native test that runs past the time limit is killed and fails.
#[test]
fn a_jit_test_past_the_time_limit_is_killed()
{
    let run = Command::new(env!("CARGO_BIN_EXE_zakac"))
        .args(["test", "--jit", "--time-limit", "1"])
        .arg(fixture().join("more/endless.zaka"))
        .env("ZAKAC_NO_CONFIG", "1")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(run.status.code(), Some(1), "{stdout}");
    assert!(stdout.contains("ran past 1 seconds"), "{stdout}");
}
