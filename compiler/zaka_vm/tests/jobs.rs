//! Jobs on the VM: a call that runs in parallel starts a job, which takes
//! turns with the others over the one heap; a wait gives its result. A trap
//! ends its job alone under `--trap unwind`, and the whole program under
//! `--trap abort`.

use zaka_vm::machine::HostData;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::TrapMode;
use zaka_vm::machine::run_and_keep;
use zaka_vm::trap::TrapKind;

use crate::common::compile;
use crate::common::output;
use crate::common::run_source_with;

/// A function that sums a list, one that joins a mark to text, one that
/// divides 10, and one that prints its text.
const FNS: &str = "စုပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် ဂဏန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
ဂဏန်းများ ၏ အကွက်တိုင်း ကို ဂ အနေဖြင့် ယူ၍ စု ကို စု + ဂ ဟု ထားမည်။
စု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

အော် ဟူသော လုပ်ဆောင်ချက် သည် စာ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
\"{စာ}!\" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

စား ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၁၀ / က ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည် စာ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// The functions, then a start block of these lines.
fn program(lines: &str) -> String
{
    format!("{FNS}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{lines}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n")
}

#[test]
fn two_jobs_sum_two_halves()
{
    let text = program(
        "ကိန်းရှင် က သည် စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို [၁၊ ၂၊ ၃] ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို [၄၊ ၅၊ ၆] ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
(က ကို စောင့်သော ရလဒ်) + (ခ ကို စောင့်သော ရလဒ်) ကို ပြသမည်။",
    );
    assert_eq!(output(&text), "၂၁\n");
}

#[test]
fn a_text_result_comes_to_each_wait_and_leaves_no_block_held()
{
    let text = program(
        "ကိန်းရှင် က သည် အော် ဟူသော လုပ်ဆောင်ချက်ကို \"မင်္ဂလာ\" ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
က ကို စောင့်သော ရလဒ် ကို ပြသမည်။
က ကို စောင့်သော ရလဒ် ကို ပြသမည်။
က ကို စောင့်မည်။",
    );
    let mut out = Vec::new();
    let options = RunOptions::default();
    let (machine, result) =
        run_and_keep(&compile(&text), &mut out, &options, &HostData::default());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(String::from_utf8(out).unwrap(), "မင်္ဂလာ!\nမင်္ဂလာ!\n");
    assert_eq!(machine.leaks(), Vec::new());
}

#[test]
fn the_program_waits_at_its_end_for_each_job()
{
    let text = program(
        "ပြ ဟူသော လုပ်ဆောင်ချက်ကို \"နောက်ကွယ်\" ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
\"ရှေ့\" ဟု ပြသမည်။",
    );
    assert_eq!(output(&text), "ရှေ့\nနောက်ကွယ်\n");
}

/// A job that divides by 0, then a print and a job that divides by 2.
const TRAPS: &str = "ကိန်းရှင် က သည် စား ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း \
                     ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် စား ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
ခ ကို စောင့်သော ရလဒ် ကို ပြသမည်။";

#[test]
fn a_trap_ends_its_job_alone_and_the_program_ends_with_code_one()
{
    let mut out = Vec::new();
    let options = RunOptions::default();
    let module = compile(&program(TRAPS));
    let (machine, result) =
        run_and_keep(&module, &mut out, &options, &HostData::default());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(String::from_utf8(out).unwrap(), "၅\n");
    let traps = machine.job_traps();
    assert_eq!(traps.len(), 1);
    assert_eq!((traps[0].0, &traps[0].1.kind), (1, &TrapKind::ZeroDivisor));
    assert_eq!(machine.exit_code(), Some(1));
}

#[test]
fn a_wait_for_a_job_that_trapped_traps()
{
    let text = program(&format!("{TRAPS}\nက ကို စောင့်မည်။"));
    let (out, result) = run_source_with(&text, &RunOptions::default());
    assert_eq!(out, "၅\n");
    assert_eq!(result.map_err(|t| t.kind), Err(TrapKind::JobTrapped { job: 1 }));
}

#[test]
fn under_abort_a_trap_in_a_job_stops_the_program()
{
    let options = RunOptions::builder().trap(TrapMode::Abort).build();
    let (out, result) = run_source_with(&program(TRAPS), &options);
    assert_eq!(out, "");
    assert_eq!(result.map_err(|t| t.kind), Err(TrapKind::ZeroDivisor));
}
