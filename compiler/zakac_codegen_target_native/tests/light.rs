//! Light tasks: a thread that calls no C, reads no standard input and calls no
//! function again runs on a small stack that a pool of OS threads takes turns
//! on. Each program runs through the JIT and as an executable.

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
    let file = dir.path().join("light.zaka");
    std::fs::write(&file, text).unwrap();
    check("jit", &run_jit(&file, &["--allow-all"]));
    let aot = run_aot(Path::new(&file), OverflowMode::Stop, &zaka_host::Grants::all());
    do_if_some(aot, |run| check("aot", &run));
}

/// Run `f` on a value when there is one: a host with no C compiler links no
/// executable.
fn do_if_some(run: Option<Run>, f: impl Fn(Run)) { run.into_iter().for_each(f); }

/// A program of a count of threads that each sleep, then send their number;
/// the program start block adds the numbers.
fn sleepers(count: u32) -> String
{
    format!(
        "နောက်မှပို့ ဟူသော လုပ်ဆောင်ချက် သည် လမ်း ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လည်းကောင်း \
         န ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၂၀ မီလီစက္ကန့် စောင့်မည်။
လမ်း ထဲသို့ န ကို ပို့မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၁ မှ {count} အထိ န အနေဖြင့် ရေတွက်၍ နောက်မှပို့ ဟူသော လုပ်ဆောင်ချက်ကို လမ်း နှင့် န တို့ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
၁ မှ {count} အထိ ရ အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် တန် သည် လမ်း မှ ရရှိသော ဒေတာ ဖြစ်ပါစေ။
အကယ်၍ တန် ထဲတွင် ဒေတာ ရှိလျှင် စု ကို စု + တန် ထဲမှ ဒေတာ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
စု ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
"
    )
}

/// The digits of a number in Burmese.
fn burmese(n: u64) -> String
{
    n.to_string()
        .chars()
        .map(|c| char::from_u32(0x1040 + c.to_digit(10).unwrap()).unwrap())
        .collect()
}

#[test]
fn two_thousand_sleeping_threads_run_as_light_tasks()
{
    // Two thousand OS threads would each take a stack of 64 MiB.
    both(&sleepers(2000), |how, run| {
        assert_eq!(run.status, Some(0), "{how}: {}", run.stderr);
        assert_eq!(run.stdout, format!("{}\n", burmese(2000 * 2001 / 2)), "{how}");
    });
}

/// 64 threads, more than the OS threads of the pool, that each count
/// themselves in, then wait in a loop until each has: a task whose turn is
/// over gives its OS thread on, so each gets to count itself in. A loop gives
/// up after a bound, so a pool with no turns ends too, with a lower sum.
const TURNS: &str = "ကောင်တာ ဟူသော အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် \
                     ကြိမ် ဟူသော အက်တမ်မစ် ဖြစ်သော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

စောင့်လည် ဟူသော လုပ်ဆောင်ချက် သည် ရေတွက် ဟူသော ကောင်တာ တစ်ခု ကို လည်းကောင်း လမ်း ဟူသော ကိန်းပြည့် ပို့နိုင်သော \
                     ဒေတာလမ်းကြောင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရေတွက် ၏ ကြိမ် ထဲသို့ ၁ ကို ပေါင်းထည့်မည်။
ကိန်းရှင် လည် သည် ၀ ဖြစ်ပါစေ။
(ရေတွက် ၏ ကြိမ်) < ၆၄ && လည် < ၅၀၀၀၀၀၀ ဖြစ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လည် ကို လည် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အကယ်၍ (ရေတွက် ၏ ကြိမ်) < ၆၄ ဖြစ်လျှင် လမ်း ထဲသို့ ၀ ကို ပို့မည်။
အကယ်၍ (ရေတွက် ၏ ကြိမ်) >= ၆၄ ဖြစ်လျှင် လမ်း ထဲသို့ ၁ ကို ပို့မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရေတွက် သည် ကောင်တာ(ကြိမ်: ၀) ဖြစ်ပါစေ။
ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၁ မှ ၆၄ အထိ န အနေဖြင့် ရေတွက်၍ စောင့်လည် ဟူသော လုပ်ဆောင်ချက်ကို ရေတွက် နှင့် လမ်း တို့ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ကိန်းရှင် စု သည် ၀ ဖြစ်ပါစေ။
၁ မှ ၆၄ အထိ ရ အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် တန် သည် လမ်း မှ ရရှိသော ဒေတာ ဖြစ်ပါစေ။
အကယ်၍ တန် ထဲတွင် ဒေတာ ရှိလျှင် စု ကို စု + တန် ထဲမှ ဒေတာ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
စု ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

#[test]
fn a_task_whose_turn_is_over_gives_its_os_thread_to_the_next()
{
    both(TURNS, |how, run| {
        assert_eq!(run.status, Some(0), "{how}: {}", run.stderr);
        assert_eq!(run.stdout, "၆၄\n", "{how}");
    });
}

/// A thread that waits for a value no thread sends, and a program start block
/// that waits for it.
const STUCK: &str = "ယူ ဟူသော လုပ်ဆောင်ချက် သည် လမ်း ဟူသော ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို \
                     လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လမ်း မှ ရရှိသော ဒေတာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လမ်း သည် ကိန်းပြည့် ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ယူ ဟူသော လုပ်ဆောင်ချက်ကို လမ်း ဖြင့် အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
က ကို စောင့်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

#[test]
fn a_light_task_that_no_thread_can_serve_stops_the_program_as_a_deadlock()
{
    both(STUCK, |how, run| {
        let want = "the program stopped: each thread waits; none can go on\n";
        assert_eq!(stop_line(&run.stderr), want, "{how}: {}", run.stderr);
    });
}

/// The program of [`sleepers`] with each thread written `သီးသန့် အပြိုင်`.
#[test]
fn a_thread_written_apart_runs_on_an_os_thread_of_its_own()
{
    let text = sleepers(4).replace("တို့ဖြင့် အပြိုင်", "တို့ဖြင့် သီးသန့် အပြိုင်");
    assert!(text.contains("သီးသန့်"));
    both(&text, |how, run| {
        assert_eq!(run.status, Some(0), "{how}: {}", run.stderr);
        assert_eq!(run.stdout, "၁၀\n", "{how}");
    });
}
