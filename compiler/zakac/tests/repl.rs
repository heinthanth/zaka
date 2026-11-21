//! `zakac repl`: each chunk prints its own output once, an open block takes
//! more lines, and a chunk with an error is dropped.

use std::io::Write;
use std::process::Command;
use std::process::Stdio;

/// Run a session of `zakac repl` with the lines of `input`.
///
/// # Returns
///
/// What it wrote to stdout and to stderr.
fn session(input: &str) -> (String, String)
{
    let mut child = Command::new(env!("CARGO_BIN_EXE_zakac"))
        .env("ZAKAC_NO_CONFIG", "1")
        .arg("repl")
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    let text = |v: Vec<u8>| String::from_utf8(v).unwrap();
    (text(out.stdout), text(out.stderr))
}

#[test]
fn each_chunk_prints_its_output_once()
{
    let input = "ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
က ကို ပြသမည်။
အကယ်၍ က == ၁ ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
\"တူ\" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
၂ ကို ပြသမည်။
";
    let (out, err) = session(input);
    assert_eq!(out, "၁\nတူ\n၂\n", "{err}");
}

#[test]
fn a_chunk_with_an_error_or_a_trap_is_dropped()
{
    let input = "ကိန်းရှင် က သည် ၀ ဖြစ်ပါစေ။
မသိ ကို ပြသမည်။
၁ / က ကို ပြသမည်။
\"ဆက်\" ဟု ပြသမည်။
";
    let (out, err) = session(input);
    assert_eq!(out, "ဆက်\n");
    assert!(err.contains("ZK0301"), "{err}");
    // The dropped chunk leaves no line: the trap is on line 2 of the session.
    assert!(err.contains("<repl>:2:1"), "{err}");
}

#[test]
fn reset_forgets_the_session_and_exit_ends_it()
{
    let input = "ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
:clear
:reset
က ကို ပြသမည်။
:huh
၃ ကို ပြသမည်။
:exit
၄ ကို ပြသမည်။
";
    let (out, err) = session(input);
    assert_eq!(out, "၃\n");
    assert!(err.contains("ZK0301"), "{err}");
    assert!(err.contains(":huh"), "{err}");
}
