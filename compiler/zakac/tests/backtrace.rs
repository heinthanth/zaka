//! How much of the stack a stop lists, as `ZAKA_BACKTRACE` picks it.

use std::process::Command;

/// A recursion that stops at its tenth call.
const DEEP: &str = "\
လုပ်ဆောင်ချက် နက် သည် န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် ၁ / န ကို ပြန်ပေးမည်။
(နက် ဟူသော လုပ်ဆောင်ချက်ကို န - ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်) + ၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
နက် ဟူသော လုပ်ဆောင်ချက်ကို ၉ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// What `zakac run` writes to standard error for the program, with the
/// variable set to a value, or unset.
fn stderr_with(value: Option<&str>) -> String
{
    let dir =
        std::env::temp_dir().join(format!("zakac-backtrace-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("deep-{}.zaka", value.unwrap_or("unset")));
    std::fs::write(&file, DEEP).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zakac"));
    cmd.arg("run").arg(&file).env_remove("ZAKA_BACKTRACE");
    if let Some(v) = value
    {
        cmd.env("ZAKA_BACKTRACE", v);
    }
    let out = cmd.output().unwrap();
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// When the variable is not set, a run of one call shows once, with a note on
/// the variable.
#[test]
fn a_stop_lists_a_short_backtrace_by_default()
{
    let err = stderr_with(None);
    assert!(err.contains("stack backtrace"), "{err}");
    assert!(err.contains("the call above, 8 times more"), "{err}");
    assert!(err.contains("ZAKA_BACKTRACE=full"), "{err}");
    assert!(!err.contains("  main"), "{err}");
}

/// `full` lists each call, the main function the compiler adds too.
#[test]
fn full_lists_each_call()
{
    let err = stderr_with(Some("full"));
    assert_eq!(err.matches("  နက် at ").count(), 10, "{err}");
    assert!(err.contains("  main"), "{err}");
    assert!(!err.contains("times more"), "{err}");
}

/// `0` leaves the calls out: the line of the stop alone.
#[test]
fn zero_leaves_the_calls_out()
{
    let err = stderr_with(Some("0"));
    assert!(err.contains("the program stopped"), "{err}");
    assert!(!err.contains("stack backtrace"), "{err}");
}
