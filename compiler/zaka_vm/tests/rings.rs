//! The statements of the ring collector, seen through a small heap: a loop
//! that leaves a ring behind on each round fits the heap when the rings are
//! collected, and runs out of room when they are not.

use zaka_vm::machine::RunOptions;
use zaka_vm::trap::TrapKind;

use crate::common::run_source_with;

/// A kind of node that two values can tie into a ring, and a function that
/// leaves one ring behind.
const RING: &str = "အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် နောက် ဟူသော \
                    အဆစ် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။

ကွင်းဆောက် ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အဆစ်(တန်: ၁၊ နောက်: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် အဆစ်(တန်: ၂၊ နောက်: က) ဖြစ်ပါစေ။
က ၏ နောက် ကို ခ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// The program: `setup` first, then a loop of 2,000 rounds that each leave a
/// ring behind and then run `each`.
///
/// # Arguments
///
/// * `setup` - The statements before the loop.
/// * `each` - The statements at the end of each round.
///
/// # Returns
///
/// The source.
fn program(setup: &str, each: &str) -> String
{
    format!(
        "{RING}
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
{setup}
၁ မှ ၂၀၀၀ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကွင်းဆောက် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
{each}
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
\"ပြီး\" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
"
    )
}

/// Run a program on a heap of 64 KiB.
///
/// # Arguments
///
/// * `text` - The source.
///
/// # Returns
///
/// Whether it ran to its end; `false` when it ran out of heap room.
fn fits(text: &str) -> bool
{
    let options = RunOptions::builder().max_heap(64 * 1024).build();
    let (out, result) = run_source_with(text, &options);
    let Err(trap) = result
    else
    {
        assert_eq!(out, "ပြီး\n");
        return true;
    };
    assert!(matches!(trap.kind, TrapKind::OutOfMem { .. }), "{trap}");
    false
}

/// The statement that pauses the collections.
const PAUSE: &str = "အလိုအလျောက် မင်မိုရီ ရှင်းလင်းခြင်းကို ခေတ္တရပ်မည်။";

/// The statement that runs the collector now.
const COLLECT: &str = "မင်မိုရီရှိ မလိုအပ်သော အရာများကို ရှင်းလင်းမည်။";

#[test]
fn paused_collections_leave_the_rings_until_the_heap_is_full()
{
    assert!(!fits(&program(PAUSE, "")));
}

#[test]
fn a_collection_on_each_round_keeps_the_heap_small()
{
    assert!(fits(&program(PAUSE, COLLECT)));
}

#[test]
fn the_glued_and_the_spaced_words_both_read()
{
    let spaced = "မင်မိုရီ ရှိ မလိုအပ်သော အရာများ ကို ရှင်းလင်းမည်။";
    assert!(fits(&program(PAUSE, spaced)));
}

#[test]
fn a_small_threshold_collects_often()
{
    let small = "အလိုအလျောက် မင်မိုရီ ရှင်းလင်းခြင်းကို ၅၁၂ ဘိုက် နှင့်အထက်မှ လုပ်ဆောင်မည်။";
    assert!(fits(&program(small, "")));
}

#[test]
fn a_threshold_past_the_heap_never_collects()
{
    let huge = "အလိုအလျောက် မင်မိုရီ ရှင်းလင်းခြင်းကို ၁၀၀၀၀၀၀၀၀၀ ဘိုက် နှင့်အထက်မှ လုပ်ဆောင်မည်။";
    assert!(!fits(&program(huge, "")));
}

#[test]
fn a_resume_lets_a_full_buffer_collect_again()
{
    let setup = format!("{PAUSE}\nအလိုအလျောက် မင်မိုရီ ရှင်းလင်းခြင်းကို ပြန်လည်စတင်မည်။");
    let small = "အလိုအလျောက် မင်မိုရီ ရှင်းလင်းခြင်းကို ၅၁၂ ဘိုက် နှင့်အထက်မှ လုပ်ဆောင်မည်။";
    assert!(fits(&program(&format!("{setup}\n{small}"), "")));
}

#[test]
fn the_default_threshold_collects_without_a_statement()
{
    assert!(fits(&program("", "")));
}
