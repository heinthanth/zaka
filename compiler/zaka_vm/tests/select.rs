//! A wait on several channels at once: the arm of a channel that holds a
//! value runs, the arm of the time limit or of no data runs when none does,
//! and with each channel closed and drained no arm runs.

use zaka_vm::machine::HostData;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_and_keep;

use crate::common::compile;
use crate::common::output;

/// A function that sleeps 50 milliseconds, then sends 7 to its channel.
const LATE: &str = "နောက်ကျ ဟူသော လုပ်ဆောင်ချက် သည် လမ်း ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု \
                    ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅၀ မီလီစက္ကန့် စောင့်မည်။
လမ်း ထဲသို့ ၇ ကို ပို့မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// The opener of the wait.
const OPENER: &str = "အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။";

/// [`LATE`], then a start block that declares the channels `အလုပ်` and `ရပ်`
/// of numbers, then these lines.
fn program(lines: &str) -> String
{
    let pipe = |name: &str| {
        format!(
            "ကိန်းရှင် {name} သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး \
             ဖြစ်ပါစေ။"
        )
    };
    let (work, stop) = (pipe("အလုပ်"), pipe("ရပ်"));
    format!(
        "{LATE}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{work}\n{stop}\n{lines}\nပရိုဂရမ် \
         ပြီးဆုံးပါပြီး။\n"
    )
}

/// A wait on `အလုပ်` and `ရပ်` that prints the value it takes, then the line
/// of its last arm, when one is given.
fn wait(last: &str) -> String
{
    format!(
        "{OPENER}
အလုပ် ထဲတွင် ဒေတာ ရှိလျှင် \"အလုပ် {{အလုပ် ထဲမှ ဒေတာ}}\" ဟု ပြသမည်။
ရပ် ထဲတွင် ဒေတာ ရှိလျှင် \"ရပ် {{ရပ် ထဲမှ ဒေတာ}}\" ဟု ပြသမည်။
{last}"
    )
}

/// The arm of no data, which prints `မရှိ`.
const IDLE: &str = "မည်သည့်လမ်းကြောင်းတွင်မှ ဒေတာ မရှိလျှင် \"မရှိ\" ဟု ပြသမည်။";

/// The arm of a time limit of 10 milliseconds, which prints `ကုန်`.
const TIMEOUT: &str = "၁၀ မီလီစက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် \"ကုန်\" ဟု ပြသမည်။";

#[test]
fn the_arm_of_no_data_runs_when_no_channel_holds_a_value()
{
    assert_eq!(output(&program(&wait(IDLE))), "မရှိ\n");
}

#[test]
fn a_value_waiting_in_a_channel_wins_over_the_arm_of_no_data()
{
    let lines = format!("ရပ် ထဲသို့ ၄ ကို ပို့မည်။\n{}", wait(IDLE));
    assert_eq!(output(&program(&lines)), "ရပ် ၄\n");
}

#[test]
fn the_wait_parks_until_a_thread_sends()
{
    let lines = format!("နောက်ကျ ဟူသော လုပ်ဆောင်ချက်ကို ရပ် ဖြင့် အပြိုင် လုပ်ဆောင်မည်။\n{}", wait(""));
    assert_eq!(output(&program(&lines)), "ရပ် ၇\n");
}

#[test]
fn the_arm_of_the_time_limit_runs_when_no_value_came_in_time()
{
    let lines = format!(
        "နောက်ကျ ဟူသော လုပ်ဆောင်ချက်ကို ရပ် ဖြင့် အပြိုင် လုပ်ဆောင်မည်။\n{}\n{}",
        wait(TIMEOUT),
        wait("၁ စက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် \"ကုန်\" ဟု ပြသမည်။")
    );
    assert_eq!(output(&program(&lines)), "ကုန်\nရပ် ၇\n");
}

#[test]
fn no_arm_runs_once_each_channel_is_closed_and_drained()
{
    let lines = format!("အလုပ် ကို ပိတ်မည်။\nရပ် ကို ပိတ်မည်။\n{}\n\"ပြီး\" ဟု ပြသမည်။", wait(""));
    assert_eq!(output(&program(&lines)), "ပြီး\n");
}

#[test]
fn a_closed_channel_counts_as_no_value_while_another_is_open()
{
    let lines = format!("အလုပ် ကို ပိတ်မည်။\n{}", wait(IDLE));
    assert_eq!(output(&program(&lines)), "မရှိ\n");
}

#[test]
fn a_seed_picks_one_of_the_channels_that_hold_a_value()
{
    let lines = format!("အလုပ် ထဲသို့ ၁ ကို ပို့မည်။\nရပ် ထဲသို့ ၂ ကို ပို့မည်။\n{}", wait(""));
    let text = program(&lines);
    let module = compile(&text);
    let run = |seed: Option<u64>| {
        let mut out = Vec::new();
        let options = RunOptions::builder().maybe_shuffle(seed).build();
        let (_, result) =
            run_and_keep(&module, &mut out, &options, &HostData::default());
        assert!(result.is_ok(), "{result:?}");
        String::from_utf8(out).unwrap()
    };
    assert_eq!(run(None), "အလုပ် ၁\n");
    let picks: Vec<String> = (0..16).map(|s| run(Some(s))).collect();
    assert!(picks.iter().any(|p| p == "ရပ် ၂\n"), "{picks:?}");
    assert_eq!(run(Some(3)), run(Some(3)));
}

#[test]
fn text_taken_in_a_loop_of_waits_leaves_no_block_held()
{
    let text = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စာများ သည် စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် အခြား သည် စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
စာများ ထဲသို့ \"က\" ကို ပို့မည်။
အခြား ထဲသို့ \"ခ\" ကို ပို့မည်။
စာများ ထဲသို့ \"ဂ\" ကို ပို့မည်။
စာများ ကို ပိတ်မည်။
အခြား ကို ပိတ်မည်။
ကိန်းရှင် ဆက် သည် မှန် ဖြစ်ပါစေ။
ဆက် ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဆက် ကို မမှန် ဟု ထားမည်။
အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။
စာများ ထဲတွင် ဒေတာ ရှိလျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာများ ထဲမှ ဒေတာ ကို ပြသမည်။
ဆက် ကို မှန် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အခြား ထဲတွင် ဒေတာ ရှိလျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အခြား ထဲမှ ဒေတာ ကို ပြသမည်။
ဆက် ကို မှန် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";
    let mut out = Vec::new();
    let options = RunOptions::default();
    let (machine, result) =
        run_and_keep(&compile(text), &mut out, &options, &HostData::default());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(String::from_utf8(out).unwrap(), "က\nဂ\nခ\n");
    assert_eq!(machine.leaks(), Vec::new());
}
