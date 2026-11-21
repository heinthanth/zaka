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

/// A chain of functions that each keep a list of 1500 cells, 12 KiB, and call
/// the next: no function calls itself again, so a thread of the first runs as
/// a light task, and the chain needs more than its stack of 1 MiB.
fn deep_chain(count: u32) -> String
{
    let list =
        "ကိန်းရှင် ကိန်းများ သည် အကွက် ၁၅၀၀ ရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။";
    let link = |i: u32| {
        let ret = match i == count
        {
            true => "ကိန်းများ ၏ အကွက် ၀ ကို ပြန်ပေးမည်။".to_string(),
            false => format!("f{}() + (ကိန်းများ ၏ အကွက် ၀) ကို ပြန်ပေးမည်။", i + 1),
        };
        format!(
            "f{i} ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို \
             ပြန်ပေးမည်။\n{list}\n{ret}\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n"
        )
    };
    let fns: String = (1..=count).map(link).collect();
    format!(
        "{fns}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် f1 ဟူသော လုပ်ဆောင်ချက်ကို အပြိုင် လုပ်ဆောင်သော လုပ်ငန်း ဖြစ်ပါစေ။
က ကို စောင့်သော ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
"
    )
}

#[test]
fn a_light_task_whose_stack_runs_full_stops_with_a_trap()
{
    both(&deep_chain(120), |how, run| {
        let want = "the program stopped: the stack of a light task, 1048576 octets, \
                    ran full\n";
        assert_eq!(stop_line(&run.stderr), want, "{how}: {}", run.stderr);
        assert_eq!(run.status, Some(1), "{how}");
    });
}

#[test]
fn the_same_chain_on_an_os_thread_of_its_own_has_room()
{
    let text = deep_chain(120).replace(" အပြိုင် လုပ်ဆောင်သော", " သီးသန့် အပြိုင် လုပ်ဆောင်သော");
    both(&text, |how, run| {
        assert_eq!(run.status, Some(0), "{how}: {}", run.stderr);
        assert_eq!(run.stdout, "၀\n", "{how}");
    });
}

/// Readers of named pipes, more than the OS threads of the pool, and one
/// writer that sleeps, then writes to each pipe: opening a pipe waits for the
/// other end, so readers that held the OS threads of the pool would keep the
/// writer from running. Each call on a file of a light task runs on an OS
/// thread for I/O, so the pool goes on.
#[cfg(unix)]
#[test]
fn file_calls_of_light_tasks_leave_the_pool_free()
{
    let dir = tempfile::tempdir().unwrap();
    let cores = std::thread::available_parallelism().map_or(1, usize::from);
    let count = (cores + 4).min(48);
    // Text shows a number in Burmese digits, so the names of the pipes use them.
    let pipe = |n: usize| dir.path().join(format!("p{}", burmese(n as u64)));
    for n in 1..=count
    {
        let made = std::process::Command::new("mkfifo").arg(pipe(n)).status().unwrap();
        assert!(made.success());
    }
    let at = dir.path().display().to_string().replace('\\', "/");
    let text = format!(
        "\"စံကား/အဝင်အထွက်\" မှ ရေး၊ ဖိုင်ကိုဖတ် နှင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း တို့ကို ထည့်သွင်းမည်။

ဖတ်ပို့ ဟူသော လုပ်ဆောင်ချက် သည် လမ်း ဟူသော စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ကို လည်းကောင်း န ဟူသော ကိန်းပြည့် \
         တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖတ်ပြီး သည် \"{at}/p{{န}}\" ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ် ဖြစ်ပါစေ။
အကယ်၍ ဖတ်ပြီး ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် လမ်း ထဲသို့ ဖတ်ပြီး ထဲမှ ဒေတာ ကို ပို့မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရေးသူ ဟူသော လုပ်ဆောင်ချက် သည် အရေအတွက် ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
၅၀ မီလီစက္ကန့် စောင့်မည်။
၁ မှ အရေအတွက် အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရေးပြီး သည် \"x\" ကို \"{at}/p{{န}}\" သို့ ရေး သည့် ရလဒ် ဖြစ်ပါစေ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် လမ်း သည် စာသား ပို့နိုင်သော ဒေတာလမ်းကြောင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
၁ မှ {count} အထိ န အနေဖြင့် ရေတွက်၍ ဖတ်ပို့ ဟူသော လုပ်ဆောင်ချက်ကို လမ်း နှင့် န တို့ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ရေးသူ ဟူသော လုပ်ဆောင်ချက်ကို {count} ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ၀ ဖြစ်ပါစေ။
၁ မှ {count} အထိ မ အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် တန် သည် လမ်း မှ ရရှိသော ဒေတာ ဖြစ်ပါစေ။
အကယ်၍ တန် ထဲတွင် ဒေတာ ရှိလျှင် ရ ကို ရ + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ရ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
"
    );
    let file = dir.path().join("pipes.zaka");
    std::fs::write(&file, text).unwrap();
    // A run past the deadline is a pool that the readers hold: it ends here.
    let exe = env!("CARGO_BIN_EXE_zakac_codegen_target_native");
    let mut child = std::process::Command::new(exe)
        .env("ZAKAC_NO_CONFIG", "1")
        .args(["--jit", "--allow-all"])
        .arg(&file)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while child.try_wait().unwrap().is_none() && std::time::Instant::now() < deadline
    {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let ended = child.try_wait().unwrap().is_some();
    if !ended
    {
        let _ = child.kill();
    }
    let out = child.wait_with_output().unwrap();
    assert!(ended, "the readers held the pool: the run did not end");
    let printed = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(printed, format!("{}\n", burmese(count as u64)), "{stderr}");
}
