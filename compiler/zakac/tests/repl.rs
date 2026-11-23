//! `zakac repl`: each chunk prints its own output once, an open block takes
//! more lines, a chunk with an error is dropped, the VM goes on from where
//! the last chunk left it, and a chunk that is one expression prints its
//! value.

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

#[test]
fn a_later_chunk_sees_the_values_the_earlier_ones_left()
{
    // A run of the whole session would set ခ before the start block, from the
    // first value of က.
    let input = "ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
က ကို ၂ ဟု ထားမည်။
ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
ပြ ဟူသော လုပ်ဆောင်ချက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ခ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပြ မည်။
";
    let (out, err) = session(input);
    assert_eq!(out, "၂\n", "{err}");
}

#[test]
fn a_chunk_sets_its_variables_in_the_order_of_its_statements()
{
    let input = "ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
က ကို ၅ ဟု ထားမည်။ ကိန်းရှင် ခ သည် က ဖြစ်ပါစေ။
ခ ကို ပြသမည်။
";
    let (out, err) = session(input);
    assert_eq!(out, "၅\n", "{err}");
}

#[test]
fn a_chunk_that_traps_puts_the_values_back()
{
    let input = "ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် သုည သည် ၀ ဖြစ်ပါစေ။
က ကို ၅ ဟု ထားမည်။ ၁ / သုည ကို ပြသမည်။
က ကို ပြသမည်။
";
    let (out, err) = session(input);
    assert_eq!(out, "၁\n", "{err}");
    assert!(err.contains("<repl>:3:"), "{err}");
}

#[test]
fn lists_text_and_closures_live_on_into_later_chunks()
{
    let input = "ကိန်းရှင် ဂဏန်းများ သည် [၁၊ ၂] ဖြစ်ပါစေ။
ဂဏန်းများ ထဲသို့ ၃ ကို ထည့်မည်။
ဂဏန်းများ ကို ပြသမည်။
ကိန်းရှင် စာ သည် \"က\" ဖြစ်ပါစေ။
စာ ကို \"{စာ}ခ\" ဟု ထားမည်။
စာ ကို ပြသမည်။
ရေတွက်စက် ဟူသော လုပ်ဆောင်ချက် သည် စ ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် ကြိမ် သည် စ ဖြစ်ပါစေ။
တိုး ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
ကြိမ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
တိုး ဟူသော လုပ်ဆောင်ချက် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် ရေတွက် သည် ရေတွက်စက် ဟူသော လုပ်ဆောင်ချက်ကို ၁၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ရေတွက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
ရေတွက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
";
    let (out, err) = session(input);
    assert_eq!(out, "[၁၊ ၂၊ ၃]\nကခ\n၁၁\n၁၂\n", "{err}");
}

#[test]
fn a_parallel_loop_reads_and_adds_to_the_variables_of_the_session()
{
    let input = "ကိန်းရှင် ထွက် သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး \
                 ဖြစ်ပါစေ။
ကိန်းရှင် တိုး သည် ၃ ဖြစ်ပါစေ။
၀ မှ ၂ အထိ i အနေဖြင့် အပြိုင် ရေတွက်၍ ထွက် ၏ အကွက် i ကို i * တိုး ဟု ထားမည်။
ထွက် ကို ပြသမည်။
ကိန်းရှင် ပေါင်း သည် အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
၁ မှ ၁၀၀ အထိ ခ အနေဖြင့် အပြိုင် ရေတွက်၍ ပေါင်း ထဲသို့ ခ ကို ပေါင်းထည့်မည်။
ပေါင်း ကို ပြသမည်။
";
    let (out, err) = session(input);
    assert_eq!(out, "[၀၊ ၃၊ ၆]\n၅၀၅၀\n", "{err}");
}

#[test]
fn a_chunk_that_is_one_expression_prints_its_value()
{
    let input = "ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
က + ၂
က == ၁
";
    let (out, err) = session(input);
    assert_eq!(out, "၃\nမှန်\n", "{err}");
    assert!(!err.contains("..."), "{err}");
    assert!(!err.contains("ZK0507"), "{err}");
}

/// The globals of the imported files are set with the first chunk and keep
/// their values through the later ones, a module imported in a later chunk
/// among them, and the facts of the target from the core too.
#[test]
fn the_globals_of_imported_files_live_on_into_later_chunks()
{
    let input = "\"စံကား/အဝင်အထွက်\" မှ စံပြုအထွက် ကို ထည့်သွင်းမည်။
\"စံကား/သင်္ချာ\" မှ ပိုင် ကို ထည့်သွင်းမည်။
စံပြုအထွက် ကို ပြသမည်။
ကိန်းရှင် က သည် ၁ ဖြစ်ပါစေ။
ပိုင် ကို ပြသမည်။
စံပြုအထွက် ကို ပြသမည်။
က ကို ပြသမည်။
";
    let (out, err) = session(input);
    let stream = "အချက်အလက်အထွက်စီးကြောင်း(ဖိုင်အညွှန်း: ၁)";
    assert_eq!(out, format!("{stream}\n၃.၁၄၁၅၉၂၆၅၃၅၈၉၇၉၃\n{stream}\n၁\n"), "{err}");
}
