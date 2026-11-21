//! A wait on several channels at once: each arm tests a channel, the read of
//! its data stands inside its arm, and the time limit is a count.

use zakac_diagnostics::Code;

use crate::common::codes;

/// The opener of the wait.
const OPENER: &str = "အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။";

/// The channels `က` and `ခ` of numbers, the optional `ဂ`, then a wait with
/// these arms.
fn wait(arms: &str) -> String
{
    let pipe = |name: &str| {
        format!(
            "ကိန်းရှင် {name} သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး \
             ဖြစ်ပါစေ။"
        )
    };
    let optional = "ကိန်းရှင် ဂ သည် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။";
    format!("{}\n{}\n{optional}\n{OPENER}\n{arms}", pipe("က"), pipe("ခ"))
}

#[test]
fn each_arm_reads_the_data_of_its_own_channel()
{
    let arms = "က ထဲတွင် ဒေတာ ရှိလျှင် က ထဲမှ ဒေတာ ကို ပြသမည်။
ခ ထဲတွင် ဒေတာ ရှိလျှင် ခ ထဲမှ ဒေတာ + ၁ ကို ပြသမည်။
၂ စက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် \"ကုန်\" ဟု ပြသမည်။";
    assert_eq!(codes(&wait(arms)), []);
}

#[test]
fn an_arm_that_tests_no_channel_is_zk0495()
{
    let arms = "က ထဲတွင် ဒေတာ ရှိလျှင် \"က\" ဟု ပြသမည်။
ဂ ထဲတွင် ဒေတာ ရှိလျှင် \"ဂ\" ဟု ပြသမည်။";
    assert_eq!(codes(&wait(arms)), [Code::NotAPipe]);
}

#[test]
fn a_read_of_the_channel_of_another_arm_is_zk0496()
{
    let arms = "က ထဲတွင် ဒေတာ ရှိလျှင် ခ ထဲမှ ဒေတာ ကို ပြသမည်။
ခ ထဲတွင် ဒေတာ ရှိလျှင် \"ခ\" ဟု ပြသမည်။";
    assert_eq!(codes(&wait(arms)), [Code::PipeDataOutside]);
}

#[test]
fn a_time_limit_that_is_no_count_is_refused()
{
    let arms = "က ထဲတွင် ဒေတာ ရှိလျှင် \"က\" ဟု ပြသမည်။
\"၂\" မီလီစက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် \"ကုန်\" ဟု ပြသမည်။";
    assert!(!codes(&wait(arms)).is_empty());
}

/// A function that returns from a wait on the channel `လမ်း`, with these end
/// arms after the arm of the channel.
fn returning(ends: &str) -> String
{
    format!(
        "ယူ ဟူသော လုပ်ဆောင်ချက် သည် လမ်း ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
{OPENER}
လမ်း ထဲတွင် ဒေတာ ရှိလျှင် လမ်း ထဲမှ ဒေတာ ကို ပြန်ပေးမည်။
{ends}
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။"
    )
}

#[test]
fn a_wait_with_the_closed_arm_returns_on_each_path()
{
    let closed = "မည်သည့်လမ်းကြောင်းထံမှ ဒေတာမရောက်လာနိုင်တော့လျှင် ၀ ကို ပြန်ပေးမည်။";
    assert_eq!(codes(&returning(closed)), []);
    let timed = format!("{closed}\n၁ စက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် ၁ ကို ပြန်ပေးမည်။");
    assert_eq!(codes(&returning(&timed)), []);
}

#[test]
fn a_wait_with_neither_the_closed_arm_nor_the_arm_of_no_data_can_end_with_no_return()
{
    let timed = "၁ စက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် ၁ ကို ပြန်ပေးမည်။";
    assert_eq!(codes(&returning(timed)), [Code::EndWithoutReturn]);
}
