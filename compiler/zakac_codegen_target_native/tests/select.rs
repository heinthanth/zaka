//! A wait on several channels at once in a native program: light tasks and
//! the main thread wait on channels, up to a time limit, and pick at random
//! among the channels that hold a value. Each program runs through the JIT
//! and as an executable.

use std::path::Path;

use zakac_borrow_checker::options::OverflowMode;

use crate::common::Run;
use crate::common::run_aot;
use crate::common::run_jit;
use crate::common::stop_line;

/// Run a program through the JIT and as an executable, and check each run
/// with `check`.
fn both(text: &str, check: impl Fn(&str, &Run))
{
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("select.zaka");
    std::fs::write(&file, text).unwrap();
    check("jit", &run_jit(&file, &["--allow-all"]));
    let aot = run_aot(Path::new(&file), OverflowMode::Stop, &zaka_host::Grants::all());
    aot.into_iter().for_each(|run| check("aot", &run));
}

/// The opener of the wait.
const OPENER: &str = "အောက်ပါ ဒေတာလမ်းကြောင်းများထဲမှ ဦးစွာရောက်လာသော ဒေတာကို စောင့်မည်။";

/// A declaration of a channel of numbers.
fn pipe(name: &str) -> String
{
    format!(
        "ကိန်းရှင် {name} သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။"
    )
}

/// 200 light tasks that each wait 10 milliseconds on two channels no thread
/// sends to, then send 1 to `ပြီး`: the end of each time limit wakes its task.
fn waiters() -> String
{
    format!(
        "စောင့်သူ ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လည်းကောင်း ခ \
         ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လည်းကောင်း ပြီး ဟူသော ကိန်းပြည့် ပို့နိုင်သော \
         ဒေတာလမ်းကြောင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
{OPENER}
က ထဲတွင် ဒေတာ ရှိလျှင် \"က\" ဟု ပြသမည်။
ခ ထဲတွင် ဒေတာ ရှိလျှင် \"ခ\" ဟု ပြသမည်။
၁၀ မီလီစက္ကန့် အတွင်း ဒေတာ မရောက်လာလျှင် ပြီး ထဲသို့ ၁ ကို ပို့မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
{}
{}
{}
၁ မှ ၂၀၀ အထိ န အနေဖြင့် ရေတွက်၍ စောင့်သူ ဟူသော လုပ်ဆောင်ချက်ကို က၊ ခ နှင့် ပြီး တို့ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
၁ မှ ၂၀၀ အထိ ရ အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် တန် သည် ပြီး မှ ရရှိသော ဒေတာ ဖြစ်ပါစေ။
အကယ်၍ တန် ထဲတွင် ဒေတာ ရှိလျှင် စု ကို စု + တန် ထဲမှ ဒေတာ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
စု ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
",
        pipe("က"),
        pipe("ခ"),
        pipe("ပြီး")
    )
}

#[test]
fn the_time_limit_of_each_light_task_wakes_it()
{
    both(&waiters(), |how, run| {
        assert_eq!(run.status, Some(0), "{how}: {}", run.stderr);
        assert_eq!(run.stdout, "၂၀၀\n", "{how}");
    });
}

#[test]
fn a_wait_with_no_limit_that_no_thread_can_serve_is_a_deadlock()
{
    let text = format!(
        "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
{}
{}
{OPENER}
က ထဲတွင် ဒေတာ ရှိလျှင် \"က\" ဟု ပြသမည်။
ခ ထဲတွင် ဒေတာ ရှိလျှင် \"ခ\" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
",
        pipe("က"),
        pipe("ခ")
    );
    both(&text, |how, run| {
        let want = "the program stopped: each thread waits; none can go on\n";
        assert_eq!(stop_line(&run.stderr), want, "{how}: {}", run.stderr);
    });
}

#[test]
fn a_wait_picks_each_channel_that_holds_a_value_in_turn_at_random()
{
    // Each round sends to both channels and takes one value, so both hold a
    // value at each wait, and a fixed pick would take `က` each time.
    let text = format!(
        "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
{}
{}
ကိန်းရှင် ခရေ သည် ၀ ဖြစ်ပါစေ။
၁ မှ ၂၀၀ အထိ ရ အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ထဲသို့ ၁ ကို ပို့မည်။
ခ ထဲသို့ ၁ ကို ပို့မည်။
{OPENER}
က ထဲတွင် ဒေတာ ရှိလျှင် ခရေ ကို ခရေ + ၀ ဟု ထားမည်။
ခ ထဲတွင် ဒေတာ ရှိလျှင် ခရေ ကို ခရေ + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အကယ်၍ ခရေ > ၁၀ ဖြစ်လျှင် \"များ\" ဟု ပြသမည်။
အကယ်၍ ခရေ < ၁၉၀ ဖြစ်လျှင် \"နည်း\" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
",
        pipe("က"),
        pipe("ခ")
    );
    both(&text, |how, run| {
        assert_eq!(run.status, Some(0), "{how}: {}", run.stderr);
        assert_eq!(run.stdout, "များ\nနည်း\n", "{how}");
    });
}

#[test]
fn a_loop_of_waits_ends_once_a_light_task_closes_each_channel()
{
    let text = format!(
        "ထုတ် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လည်းကောင်း ခ \
         ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၁ မှ ၁၀၀ အထိ န အနေဖြင့် ရေတွက်၍ က ထဲသို့ န ကို ပို့မည်။
က ကို ပိတ်မည်။
ခ ကို ပိတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
{}
{}
ထုတ် ဟူသော လုပ်ဆောင်ချက်ကို က နှင့် ခ တို့ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ဆက် သည် မှန် ဖြစ်ပါစေ။
ဆက် ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
{OPENER}
က ထဲတွင် ဒေတာ ရှိလျှင် စု ကို စု + က ထဲမှ ဒေတာ ဟု ထားမည်။
ခ ထဲတွင် ဒေတာ ရှိလျှင် \"ခ\" ဟု ပြသမည်။
မည်သည့်လမ်းကြောင်းထံမှ ဒေတာမရောက်လာနိုင်တော့လျှင် ဆက် ကို မမှန် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
စု ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
",
        pipe("က"),
        pipe("ခ")
    );
    both(&text, |how, run| {
        assert_eq!(run.status, Some(0), "{how}: {}", run.stderr);
        assert_eq!(run.stdout, "၅၀၅၀\n", "{how}");
    });
}
