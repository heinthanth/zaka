//! Closures C calls back: a closure is live while the C call that got it
//! runs, and after that while C has leave from `ခေါ်ဆိုခွင့် ပြုမည်`; a later
//! call stops the program. libc's `signal` keeps a handler, and `raise` calls
//! it on the thread that runs the program.
#![cfg(all(feature = "ffi", unix))]

mod common;

use common::compile;
use common::run_source_with;
use zaka_vm::machine::HostData;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_and_keep;
use zaka_vm::trap::TrapKind;

/// `signal` and `raise` of libc, then a start block that holds `steps`, with
/// a counter `ကြိမ်` and a handler `ဖမ်း` that adds one to it.
fn program(steps: &str) -> String
{
    format!(
        "လုပ်ဆောင်ချက် signal သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
sig ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း handler ဟူသော (၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် \
         အမျိုးအစား) တစ်ခု ကို လည်းကောင်း လက်ခံမည်။
လုပ်ဆောင်ချက် raise သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ sig ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ကြိမ် သည် ၀ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဖမ်း သည် s ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
{steps}
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
"
    )
}

/// Hand `ဖမ်း` to `signal` for the signal 15.
const SET: &str = "လုပ်ဆောင်ချက် signal ကို ၁၅၊ ဖမ်း ဟူသော လုပ်ဆောင်ချက် တို့ဖြင့် လုပ်ဆောင်မည်။";
/// Raise the signal 15.
const RAISE: &str = "လုပ်ဆောင်ချက် raise ကို ၁၅ တို့ဖြင့် လုပ်ဆောင်မည်။";
/// Give C leave to call `ဖမ်း`.
const LEAVE: &str = "ဖမ်း ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။";
/// Take that leave back.
const REVOKE: &str = "ဖမ်း ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်။";

#[test]
fn a_closure_with_leave_runs_when_c_calls_it_later()
{
    let steps = format!("{LEAVE}\n{SET}\n{RAISE}\n{RAISE}\nကြိမ် ကို ပြသမည်။\n{REVOKE}");
    let (out, result) = run_source_with(&program(&steps), &RunOptions::default());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(out, "၂\n");
}

#[test]
fn a_call_after_the_leave_ends_stops_the_program()
{
    let steps = format!("{LEAVE}\n{SET}\n{RAISE}\n{REVOKE}\n{RAISE}\n\"no\" ဟု ပြသမည်။");
    let (out, result) = run_source_with(&program(&steps), &RunOptions::default());
    let trap = result.expect_err("the late call stops the program");
    assert_eq!(trap.kind, TrapKind::ClosureEscape, "{trap}");
    assert_eq!(out, "");
}

#[test]
fn a_closure_with_no_leave_lives_for_its_call_alone()
{
    let steps = format!("{SET}\n{RAISE}");
    let (_, result) = run_source_with(&program(&steps), &RunOptions::default());
    let trap = result.expect_err("C has no leave after signal returns");
    assert_eq!(trap.kind, TrapKind::ClosureEscape, "{trap}");
}

#[test]
fn a_second_leave_and_a_revoke_with_no_leave_change_nothing()
{
    let steps = format!(
        "{REVOKE}\n{LEAVE}\n{LEAVE}\n{SET}\n{RAISE}\n{REVOKE}\n{REVOKE}\nကြိမ် ကို ပြသမည်။"
    );
    let (out, result) = run_source_with(&program(&steps), &RunOptions::default());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(out, "၁\n");
}

/// A leave still open when the program ends ends with it, so the record it
/// held goes back and no block leaks.
#[test]
fn a_leave_left_open_ends_with_the_run()
{
    let module = compile(&program(&format!("{LEAVE}\n{SET}\n{RAISE}")));
    let mut out = Vec::new();
    let options = RunOptions::default();
    let (machine, result) =
        run_and_keep(&module, &mut out, &options, &HostData::default());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(machine.leaks(), []);
}
