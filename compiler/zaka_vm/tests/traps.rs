mod common;

use common::output;
use common::run_source;
use common::run_source_with;
use zaka_vm::machine::RunOptions;
use zaka_vm::trap::TrapKind;

/// The trap kind and function of a source text that must trap.
fn trapped(text: &str, options: &RunOptions) -> (String, TrapKind, String)
{
    let (out, result) = run_source_with(text, options);
    let trap = result.expect_err("the program should trap");
    (out, trap.kind, trap.func)
}

#[test]
fn zero_divisor_at_run_time_traps_after_earlier_output()
{
    let text = "ကိန်းရှင် သုည သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\n\"အရင်\" ဟု ပြသပါ။\n၁ / သုည ကို \
                ပြသပါ။\n\"နောက်\" ဟု ပြသပါ။";
    let (out, kind, func) = trapped(text, &RunOptions::default());
    assert_eq!(out, "အရင်\n");
    assert_eq!(kind, TrapKind::ZeroDivisor);
    assert_eq!(func, "main");
    let (_, rem, _) = trapped(&text.replace(" / ", " % "), &RunOptions::default());
    assert_eq!(rem, TrapKind::ZeroDivisor);
}

#[test]
fn endless_recursion_hits_the_depth_limit()
{
    let text = "လုပ်ဆောင်ချက် ထပ် ကို လုပ်ဆောင်ပါ။\nလုပ်ဆောင်ချက် ထပ် သည်\nအောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nလုပ်ဆောင်ချက် ထပ် ကို လုပ်ဆောင်ပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let options = RunOptions::builder().max_depth(64).build();
    let (_, kind, func) = trapped(text, &options);
    assert_eq!(kind, TrapKind::StackOverflow { depth: 64 });
    assert_eq!(func, "ထပ်");
}

#[test]
fn endless_loops_hit_the_step_limit()
{
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\nမှန် ဖြစ်နေသရွေ့ က ကို က + ၁ ဟု ထားပါ။";
    let options = RunOptions::builder().max_steps(1000).build();
    let (_, kind, _) = trapped(text, &options);
    assert_eq!(kind, TrapKind::StepLimit { steps: 1000 });
}

#[test]
fn integer_arithmetic_wraps()
{
    let text = "ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ၂၅၅ ဖြစ်ပါစေ။\nက + ၁ ကို ပြသပါ။\nကိန်းရှင် ခ \
                သည် ၈ ဘစ် ကိန်းပြည့် -၁၂၈ ဖြစ်ပါစေ။\n-ခ ကို ပြသပါ။\nခ / -၁ ကို ပြသပါ။\nခ - ၁ ကို \
                ပြသပါ။\nကိန်းရှင် ဂ သည် ၈ ဘစ် ကိန်းပြည့် ၁၀၀ ဖြစ်ပါစေ။\nဂ * ၃ ကို ပြသပါ။";
    assert_eq!(output(text), "0\n-128\n-128\n127\n44\n");
}

#[test]
fn shifting_past_the_width_traps()
{
    let text = "ကိန်းရှင် ရွေ့ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ၈ ဖြစ်ပါစေ။\nကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် \
                ကိန်းပြည့် ၁ ဖြစ်ပါစေ။\nက << ရွေ့ ကို ပြသပါ။";
    let (_, kind, _) = trapped(text, &RunOptions::default());
    assert_eq!(kind, TrapKind::ShiftTooFar { distance: 8, bits: 8 });
}

#[test]
fn a_step_that_is_not_positive_traps()
{
    let text =
        "ကိန်းရှင် ခြား သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\n၁ မှ ၃ အထိ ခြား စီ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသပါ။";
    let (out, kind, _) = trapped(text, &RunOptions::default());
    assert_eq!(out, "");
    assert_eq!(kind, TrapKind::StepNotPositive);
}

#[test]
fn traps_name_their_place()
{
    let (_, result) = run_source("ကိန်းရှင် သုည သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\n၁ / သုည ကို ပြသပါ။");
    let trap = result.unwrap_err();
    let shown = trap.to_string();
    assert!(shown.contains("zero as divisor"), "{shown}");
    assert!(shown.contains("main"), "{shown}");
    assert!(shown.contains(&format!("op {}", trap.pc)), "{shown}");
}

#[test]
fn logic_short_circuits_at_run_time()
{
    let text = "ကိန်းရှင် သုည သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\nကိန်းရှင် ပိတ် သည် မှန်မှားတန်ဖိုး မမှန် \
                ဖြစ်ပါစေ။\n(ပိတ် && ၁ / သုည သည် ၁ နှင့် ညီသည်) ကို ပြသပါ။\n(ပိတ် မဟုတ် || ၁ / သုည သည် ၁ \
                နှင့် ညီသည်) ကို ပြသပါ။";
    assert_eq!(output(text), "မမှန်\nမှန်\n");
}
