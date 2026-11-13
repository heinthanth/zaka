use zaka_vm::machine::FrameMode;
use zaka_vm::machine::Held;
use zaka_vm::machine::Machine;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::State;
use zaka_vm::mem::Block;
use zaka_vm::mem::Origin;
use zaka_vm::print::burmese_digits;
use zaka_vm::trap::TrapKind;
use zakac_rust_sugars::do_while;

use crate::common;
use crate::common::output;
use crate::common::output_on;

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
    let text = "ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ၈ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
p ကို ပြသမည်။";
    assert_eq!(output(text), "မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ\n");
}

#[test]
fn free_through_a_dereference_clears_the_pointee()
{
    let text = "ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် pp သည် ကိန်းပြည့် ၏ လိပ်စာ ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် p ၏ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ၈ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို pp ၏ တန်ဖိုး တွင် သိမ်းထားမည်။
လိပ်စာ pp ၏ တန်ဖိုး ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
p ကို ပြသမည်။
(pp သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် မညီသည်) ကို ပြသမည်။";
    assert_eq!(output(text), "မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ\nမှန်\n");
}

#[test]
fn a_global_address_outlives_a_call()
{
    let text = "ကိန်းရှင် g သည် ၁ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ယူ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။
p ၏ တန်ဖိုး ကို ၉ ဟု ထားမည်။
g ကို ပြသမည်။
လုပ်ဆောင်ချက် ယူ သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
p ကို g ၏ လိပ်စာ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    assert_eq!(output(text), "၉\n");
}

#[test]
fn ptr_add_wraps_at_the_platform_width()
{
    // 255 as a u8 literal (an 8-bit isize cannot hold it), cast to a pointer
    // at the top of the 256-address space; one element past it wraps to 0,
    // which prints as the null address.
    let text = "ကိန်းရှင် p သည် ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၅၅ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် \
                ကိန်းပြည့် တစ်ခု ကို ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
(p + ၁) ကို ပြသမည်။";
    assert_eq!(output_on(text, 8), "မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ\n");
}

#[test]
fn an_offset_stored_in_an_opaque_place_prints_its_address()
{
    let text = "ကိန်းရှင် x သည် ၅ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် x ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် q သည် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် p + ၁ ဖြစ်ပါစေ။
p ကို ပြသမည်။
q ကို ပြသမည်။";
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
fn deep_calls_give_their_stack_room_back_on_return()
{
    let text = "ကိန်းရှင် g သည် ၁ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် gp သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် g ၏ လိပ်စာ ဖြစ်ပါစေ။
ဆင်း ဟူသော လုပ်ဆောင်ချက်ကို ၂၀၀၀ တို့ဖြင့် လုပ်ဆောင်မည်။
gp ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်ချက် ဆင်း သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် န ၏ လိပ်စာ ဖြစ်ပါစေ။
အကယ်၍ p ၏ တန်ဖိုး သည် ၀ ထက် ကြီးလျှင် ဆင်း ဟူသော လုပ်ဆောင်ချက်ကို န - ၁ တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let limits = RunOptions::builder().max_steps(1_000_000).build();
    let mut m = Machine::new(common::compile(text), limits);
    do_while!(m.frames().len() < 2, let _ = m.step());
    m.step_out();
    assert_eq!(m.frames().len(), 1, "back in the main function");
    let live: Vec<&Block> = m.mem().blocks().filter(|b| b.live).collect();
    assert_eq!(live.len(), 1, "the block of the global alone stands");
    assert!(live.iter().all(|b| b.origin == Origin::Static));
    let stack = m.mem().blocks().filter(|b| b.origin == Origin::Stack).count();
    // Each call takes two blocks, and the start block one for its pointer.
    assert!(stack <= 2 * 2001 + 1, "each call took two blocks: {stack}");
    do_while!(m.running(), let _ = m.step());
    assert_eq!(m.state(), &State::Finished);
    assert_eq!(m.output(), "၁\n");
    let live = m.mem().blocks().filter(|b| b.live);
    assert!(live.map(|b| b.origin).all(|o| o == Origin::Static));
}

#[test]
fn the_peak_heap_outlasts_the_frees()
{
    let text = include_str!("../../../samples/heap/growable.zaka");
    let mut m = Machine::new(common::compile(text), RunOptions::default());
    let mut most = 0;
    do_while!(m.state() == &State::Running, {
        m.step();
        most = most.max(m.mem().live_heap());
    });
    assert_eq!(m.state(), &State::Finished);
    assert!(most > 0, "the list takes heap");
    // A resize holds the old block and the new one at once, inside one op.
    assert!(m.mem().peak_heap() >= most);
    assert!(m.mem().peak_heap() > m.mem().live_heap(), "the frees leave the peak");
}

/// Two globals, and a read one integer past the first.
const PAST_THE_FIRST: &str = "ကိန်းရှင် က သည် ၁ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၂ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ + ၁ ၏ တန်ဖိုး ကို ပြသမည်။";

/// Limits with the variables set out in one form.
fn laid_out(frames: FrameMode) -> RunOptions
{
    RunOptions::builder().frames(frames).build()
}

#[test]
fn a_read_past_a_variable_traps_when_checked_and_reaches_the_next_when_packed()
{
    let (_, checked) =
        common::run_source_with(PAST_THE_FIRST, &laid_out(FrameMode::Checked));
    let trap = checked.expect_err("a checked read past က traps");
    assert!(matches!(trap.kind, TrapKind::OutOfBounds { .. }), "{trap}");
    assert!(trap.block.is_some_and(|b| b.ends_with("global က")));
    let (out, packed) =
        common::run_source_with(PAST_THE_FIRST, &laid_out(FrameMode::Packed));
    assert_eq!((out.as_str(), packed), ("၂\n", Ok(())), "a packed read finds ခ");
}

#[test]
fn a_packed_call_takes_one_block_with_its_locals_at_c_offsets()
{
    let text = "ပြ ဟူသော လုပ်ဆောင်ချက်ကို မှန်၊ ၇ တို့ဖြင့် လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် ပြ သည်
ဖွင့် ဟူသော မှန်မှားတန်ဖိုး တစ်ခု ကို လည်းကောင်း န ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let mut m = Machine::new(common::compile(text), laid_out(FrameMode::Packed));
    do_while!(m.frames().len() < 3, let _ = m.step());
    let frame = &m.frames()[2];
    let base = frame.blocks[0];
    assert_eq!(frame.blocks.len(), 1);
    assert_eq!(frame.locals, [base, base + 8], "the integer is aligned past the bool");
    let block = m.mem().blocks().find(|b| b.start == base).expect("the frame block");
    assert_eq!((block.origin, block.len), (Origin::Stack, 16));
    assert_eq!(m.owner(base), Some(Held::Frame { func: "ပြ" }));
    let vars: Vec<(u64, u64, &str)> = m
        .packed_vars(base)
        .expect("a packed frame")
        .iter()
        .map(|v| (v.offset, v.len, v.name))
        .collect();
    assert_eq!(vars, [(0, 1, "ဖွင့်"), (8, 8, "န")]);
    do_while!(m.running(), let _ = m.step());
    assert_eq!(m.output(), "၇\n");
}

#[test]
fn a_packed_local_that_escapes_names_its_frame()
{
    let text = "ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဂဏန်း ဟူသော လုပ်ဆောင်ချက်ကို \
                လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
p ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်ချက် ဂဏန်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် x သည် ၅ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
x ၏ လိပ်စာ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let (_, result) = common::run_source_with(text, &laid_out(FrameMode::Packed));
    let trap = result.expect_err("the frame was given back");
    assert!(matches!(trap.kind, TrapKind::UseAfterFree { .. }), "{trap}");
    assert!(trap.block.is_some_and(|b| b.ends_with("frame of ဂဏန်း")));
}

#[test]
fn a_lent_owner_keeps_its_block()
{
    let text = "ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
p ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
ကိန်းရှင် q သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် p ဖြစ်ပါစေ။
(p သည် q နှင့် ညီသည်) ကို ပြသမည်။
q ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။";
    assert_eq!(output(text), "မှန်\n၇\n");
}

/// A step that leaves the block of its base stops at the step, though one
/// past the end is a fine address to form.
#[test]
fn a_step_out_of_its_block_traps_at_the_step()
{
    let text = "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် နောက် သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ + ၂ ဖြစ်ပါစေ။
\"ပြီး\" ဟု ပြသမည်။
ကိန်းရှင် ကျော် သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ + ၃ ဖြစ်ပါစေ။
\"မရောက်\" ဟု ပြသမည်။";
    let (out, result) = common::run_source(text);
    assert_eq!(out, "ပြီး\n");
    let trap = result.expect_err("a step two past the end traps");
    assert!(matches!(trap.kind, TrapKind::OutOfBounds { size: 0, .. }), "{trap}");
    assert!(trap.to_string().contains("pointer step to"), "{trap}");
}

/// A live base plus a number that lands in another block, a freed one
/// here, stops at the step as out of bounds, not at the read.
#[test]
fn a_step_into_another_block_traps_at_the_step()
{
    let text = "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (ပ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်) / ၈ ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ကိန်းရှင် သ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို သ တွင် သိမ်းထားမည်။
ကိန်းရှင် ဆ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် သ ဖြစ်ပါစေ။
ကိန်းရှင် မ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (ဆ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်) / ၈ ဖြစ်ပါစေ။
(ဆ + (န - မ)) ၏ တန်ဖိုး ကို ပြသမည်။
လိပ်စာ သ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။";
    let (_, result) = common::run_source(text);
    let trap = result.expect_err("the step leaves the block of သ");
    assert!(matches!(trap.kind, TrapKind::OutOfBounds { .. }), "{trap}");
}

/// A list literal of known cells sits in a frozen static list block, and the
/// list takes a heap block at its first edit alone; a slice and a cell
/// address of it write a block of its own, never the static one.
#[test]
fn a_static_list_takes_no_heap_block_until_its_first_edit()
{
    let text = "ကိန်းရှင် l သည် [\"က\"၊ \"ခ\"] ဖြစ်ပါစေ။\nl ကို ပြသမည်။\nl ထဲသို့ \"ဂ\" ကို ထည့်မည်။\nl \
                ကို ပြသမည်။";
    let mut m = Machine::new(common::compile(text), RunOptions::default());
    do_while!(!m.output().contains('\n'), let _ = m.step());
    let heap = m.mem().blocks().filter(|b| b.live && b.origin == Origin::Heap).count();
    assert_eq!(heap, 0, "the literal takes no heap block");
    let named = m.mem().blocks().filter_map(|b| m.owner(b.start)).collect::<Vec<_>>();
    assert!(named.contains(&Held::Seq(0)), "{named:?}");
    do_while!(m.running(), let _ = m.step());
    assert_eq!(m.state(), &State::Finished);
    assert_eq!(m.output(), "[\"က\"၊ \"ခ\"]\n[\"က\"၊ \"ခ\"၊ \"ဂ\"]\n");
    assert!(m.leaks().is_empty());
    let writes = "ကိန်းရှင် l သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။\nကိန်းရှင် p သည် l ၏ အကွက် ၁ ၏ လိပ်စာ \
                  ဖြစ်ပါစေ။\np ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။\nl ကို ပြသမည်။";
    assert_eq!(output(writes), "[၁၊ ၇၊ ၃]\n");
}

/// A static list lent to a function that reads it, and walked in a loop that
/// reads it, takes no heap block: the parameter and the loop read the static
/// cells.
#[test]
fn a_static_list_lent_to_a_reader_takes_no_heap_block()
{
    let text = "လုပ်ဆောင်ချက် စုစုပေါင်း သည်\nကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို \
                လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\nကိန်းရှင် အဖြေ သည် \
                ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။\nကိန်းရှင် န သည် လက္ခဏာမဲ့ ကိန်းပြည့် \
                တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ န သည် ကိန်းများ ၏ အကွက်အရေအတွက် အောက် ငယ်နေသရွေ့ \
                အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nအဖြေ ကို အဖြေ + (ကိန်းများ ၏ အကွက် န) ဟု ထားမည်။\nပြီးလျှင် \
                န ကို န + ၁ ဟု ထားမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။\nအဖြေ ကို \
                ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။\n\nကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု \
                ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။\nစုစုပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို l တို့ဖြင့် \
                လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။\nl ၏ အကွက်တိုင်း ကို က အနေဖြင့် ယူ၍ က ကို ပြသမည်။";
    let mut m = Machine::new(common::compile(text), RunOptions::default());
    let heap = |m: &Machine| {
        m.mem().blocks().filter(|b| b.live && b.origin == Origin::Heap).count()
    };
    while m.running()
    {
        let _ = m.step();
        assert_eq!(heap(&m), 0, "a heap block after the output {:?}", m.output());
    }
    assert_eq!(m.state(), &State::Finished);
    assert_eq!(m.output(), "၆\n၁\n၂\n၃\n");
}

/// A counted list literal of known cells takes them from a static list, so
/// an edit of the list leaves the static cells unchanged for the next
/// call, and an edited text cell takes a block of its own.
#[test]
fn a_counted_list_copies_its_static_cells()
{
    let text = "လုပ်ဆောင်ချက် ပြ သည်\nန ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nကိန်းရှင် ခ သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈] \
                ဖြစ်ပါစေ။\nခ ကို ပြသမည်။\nခ ၏ အကွက် ၀ ကို န ဟု ထားမည်။\nခ ကို \
                ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီ။\n\nပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁ တို့ဖြင့် \
                လုပ်ဆောင်မည်။\nပြ ဟူသော လုပ်ဆောင်ချက်ကို ၂ တို့ဖြင့် လုပ်ဆောင်မည်။\nကိန်းရှင် ဂ သည် အကွက် ၂ \
                ခုရှိသော စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [\"က\"၊ \"ခ\"] ဖြစ်ပါစေ။\nဂ ၏ အကွက် ၁ \
                ထဲသို့ \"x\" ကို ထည့်မည်။\nဂ ကို ပြသမည်။";
    let shown = "[၇၊ ၈]\n[၁၊ ၈]\n[၇၊ ၈]\n[၂၊ ၈]\n[\"က\"၊ \"ခx\"]\n";
    let mut m = Machine::new(common::compile(text), RunOptions::default());
    do_while!(m.running(), let _ = m.step());
    assert_eq!(m.state(), &State::Finished);
    assert_eq!(m.output(), shown);
    assert!(m.leaks().is_empty());
}

/// A struct literal of known fields takes them from a static list, each at
/// its offset, so an edit of the struct leaves the static fields unchanged
/// for the next call, and an edited text field takes a block of its own.
#[test]
fn a_struct_literal_copies_its_static_fields()
{
    let text = "လူ ဟူသော ဒေတာအစုအဝေးတွင် အောင် ဟူသော မှန်မှားတန်ဖိုး တစ်ခု၊ အမည် ဟူသော စာသား တစ်ခု၊ အသက် \
                ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။\n\nလုပ်ဆောင်ချက် ပြ သည်\nန ဟူသော ကိန်းပြည့် တစ်ခု ကို \
                လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းရှင် ခ သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အောင် မှန်၊ အမည် \
                \"မမ\"၊ အသက် ၂၀ ရှိသော လူတစ်ခု ဖြစ်ပါစေ။\nခ ကို ပြသမည်။\nခ ၏ အမည် ထဲသို့ \"x\" ကို \
                ထည့်မည်။\nခ ၏ အသက် ကို န ဟု ထားမည်။\nခ ကို ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီ။\n\nပြ \
                ဟူသော လုပ်ဆောင်ချက်ကို ၁ တို့ဖြင့် လုပ်ဆောင်မည်။\nပြ ဟူသော လုပ်ဆောင်ချက်ကို ၂ တို့ဖြင့် လုပ်ဆောင်မည်။";
    let shown = "လူ(အောင်: မှန်၊ အမည်: \"မမ\"၊ အသက်: ၂၀)\nလူ(အောင်: မှန်၊ အမည်: \"မမx\"၊ အသက်: \
                 ၁)\nလူ(အောင်: မှန်၊ အမည်: \"မမ\"၊ အသက်: ၂၀)\nလူ(အောင်: မှန်၊ အမည်: \"မမx\"၊ \
                 အသက်: ၂)\n";
    let mut m = Machine::new(common::compile(text), RunOptions::default());
    do_while!(m.running(), let _ = m.step());
    assert_eq!(m.state(), &State::Finished);
    assert_eq!(m.output(), shown);
    assert!(m.leaks().is_empty());
}
