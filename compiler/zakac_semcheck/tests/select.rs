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
