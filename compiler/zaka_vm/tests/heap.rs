mod common;

use common::output;
use common::output_on;
use zaka_vm::machine::Machine;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::State;
use zaka_vm::mem::Block;
use zaka_vm::mem::Origin;
use zaka_vm::print::burmese_digits;
use zakac_rust_sugars::do_while;

#[test]
fn swap_through_pointers()
{
    let text = include_str!("../../../samples/heap/swap.zaka");
    assert_eq!(output(text), "၂\n၁\n");
}

#[test]
fn a_growable_list_resizes_and_frees()
{
    let text = include_str!("../../../samples/heap/growable.zaka");
    let want: String = (1..=10)
        .map(|i| format!("{}\n", burmese_digits(&(i * i).to_string())))
        .collect();
    assert_eq!(output(text), want);
}

#[test]
fn a_record_built_from_offsets()
{
    assert_eq!(output(include_str!("../../../samples/heap/record.zaka")), "၁၅၀၀\n");
}

#[test]
fn free_clears_the_variable_and_a_null_free_is_harmless()
{
    let text = "ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ၈ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
p လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
p လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
p ကို ပြသပါ။";
    assert_eq!(output(text), "ဗလာ\n");
}

#[test]
fn free_through_a_dereference_clears_the_pointee()
{
    let text = "ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် pp သည် ကိန်းပြည့် ၏ လိပ်စာ ၏ လိပ်စာ p ၏ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ၈ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို pp ၏ တန်ဖိုး တွင် သိမ်းထားမည်။
pp ၏ တန်ဖိုး လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
p ကို ပြသပါ။
(pp သည် ဗလာ နှင့် မညီသည်) ကို ပြသပါ။";
    assert_eq!(output(text), "ဗလာ\nမှန်\n");
}

#[test]
fn a_global_address_outlives_a_call()
{
    let text = "ကိန်းရှင် g သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ယူ ကို လုပ်ဆောင်ပါ။
p ၏ တန်ဖိုး ကို ၉ ဟု ထားပါ။
g ကို ပြသပါ။
လုပ်ဆောင်ချက် ယူ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
p ကို g ၏ လိပ်စာ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    assert_eq!(output(text), "၉\n");
}

#[test]
fn ptr_add_wraps_at_the_platform_width()
{
    // 255 as a u8 literal (an 8-bit isize cannot hold it), cast to a pointer
    // at the top of the 256-address space; one element past it wraps to 0,
    // which prints as ဗလာ.
    let text = "ကိန်းရှင် p သည် ၈ ဘစ် ကိန်းပြည့် ၏ လိပ်စာ လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ၂၅၅ ကို ၈ ဘစ် ကိန်းပြည့် ၏ \
                လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
(p + ၁) ကို ပြသပါ။";
    assert_eq!(output_on(text, 8), "ဗလာ\n");
}

#[test]
fn an_offset_stored_in_an_opaque_place_prints_its_address()
{
    let text = "ကိန်းရှင် x သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ x ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် q သည် လိပ်စာ p + ၁ ဖြစ်ပါစေ။
p ကို ပြသပါ။
q ကို ပြသပါ။";
    let found = output(text);
    let lines: Vec<&str> = found.lines().collect();
    let [p, q] = lines[..]
    else
    {
        panic!("two lines expected: {found:?}");
    };
    let hex = |s: &str| u64::from_str_radix(s.trim_start_matches("0x"), 16).expect(s);
    assert!(q.starts_with("0x"), "{q}");
    assert_eq!(hex(q), hex(p) + 8);
}

#[test]
fn deep_calls_with_a_cell_free_each_cell_on_return()
{
    let text = "ကိန်းရှင် g သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် gp သည် ကိန်းပြည့် ၏ လိပ်စာ g ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဆင်း ကို ၂၀၀၀ တို့ဖြင့် လုပ်ဆောင်ပါ။
gp ၏ တန်ဖိုး ကို ပြသပါ။
လုပ်ဆောင်ချက် ဆင်း သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ န ၏ လိပ်စာ ဖြစ်ပါစေ။
အကယ်၍ p ၏ တန်ဖိုး သည် ၀ ထက် ကြီးလျှင် လုပ်ဆောင်ချက် ဆင်း ကို န - ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let limits = RunOptions::builder().max_steps(1_000_000).build();
    let mut m = Machine::new(common::compile(text), limits);
    do_while!(m.frames().len() < 2, let _ = m.step());
    m.step_out();
    assert_eq!(m.frames().len(), 1, "back in the main function");
    let live: Vec<&Block> = m.mem().blocks().filter(|b| b.live).collect();
    assert_eq!(live.len(), 1, "the global cell alone stands");
    assert_eq!(live[0].origin, Origin::Cell);
    assert!(m.mem().blocks().count() <= 2001 + 1, "one block per call");
    do_while!(m.running(), let _ = m.step());
    assert_eq!(m.state(), &State::Finished);
    assert_eq!(m.output(), "၁\n");
    assert_eq!(m.mem().blocks().filter(|b| b.live).count(), 0);
}
