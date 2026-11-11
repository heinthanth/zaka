mod common;

use std::mem::discriminant;
use std::path::PathBuf;

use common::compile;
use common::output;
use common::run_module;
use common::run_source;
use common::run_source_with;
use zaka_vm::machine::RunOptions;
use zaka_vm::trap::Trap;
use zaka_vm::trap::TrapKind;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_vm::bc::Module;

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
    let text = "ကိန်းရှင် သုည သည် ၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\n\"အရင်\" ဟု ပြသမည်။\n၁ / သုည ကို \
                ပြသမည်။\n\"နောက်\" ဟု ပြသမည်။";
    let (out, kind, func) = trapped(text, &RunOptions::default());
    assert_eq!(out, "အရင်\n");
    assert_eq!(kind, TrapKind::ZeroDivisor);
    assert_eq!(func, "ပရိုဂရမ်", "the trap names the start block");
    let (_, rem, _) = trapped(&text.replace(" / ", " % "), &RunOptions::default());
    assert_eq!(rem, TrapKind::ZeroDivisor);
}

#[test]
fn endless_recursion_hits_the_depth_limit()
{
    let text = "လုပ်ဆောင်ချက် ထပ် ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်ချက် ထပ် သည်\nအောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nလုပ်ဆောင်ချက် ထပ် ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let options = RunOptions::builder().max_depth(64).build();
    let (_, kind, func) = trapped(text, &options);
    assert_eq!(kind, TrapKind::StackOverflow { depth: 64 });
    assert_eq!(func, "ထပ်");
}

#[test]
fn endless_loops_hit_the_step_limit()
{
    let text = "ကိန်းရှင် က သည် ၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\nမှန် ဖြစ်နေသရွေ့ က ကို က + ၁ ဟု ထားမည်။";
    let options = RunOptions::builder().max_steps(1000).build();
    let (_, kind, _) = trapped(text, &options);
    assert_eq!(kind, TrapKind::StepLimit { steps: 1000 });
}

/// The output of a source text compiled with `--overflow wrap`; it must not
/// trap.
fn wrapped(text: &str) -> String
{
    let module = Module { overflow: OverflowMode::Wrap, ..compile(text) };
    let (out, result) = run_module(&module, &RunOptions::default());
    assert_eq!(result, Ok(()), "{text}\n{out}");
    out
}

#[test]
fn integer_arithmetic_wraps()
{
    let text = "ကိန်းရှင် က သည် ၂၅၅ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\nက + ၁ ကို \
                ပြသမည်။\nကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် -၁၂၈ ဖြစ်ပါစေ။\n-ခ ကို \
                ပြသမည်။\nခ / -၁ ကို ပြသမည်။\nခ - ၁ ကို ပြသမည်။\nကိန်းရှင် ဂ သည် ၁၀၀ ဟူသော ၈ ဘစ် \
                ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\nဂ * ၃ ကို ပြသမည်။";
    assert_eq!(wrapped(text), "၀\n-၁၂၈\n-၁၂၈\n၁၂၇\n၄၄\n");
}

#[test]
fn shifting_past_the_width_traps()
{
    let text = "ကိန်းရှင် ရွေ့ သည် ၈ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\nကိန်းရှင် က သည် ၁ ဟူသော \
                လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\nက << ရွေ့ ကို ပြသမည်။";
    let (_, kind, _) = trapped(text, &RunOptions::default());
    assert_eq!(kind, TrapKind::ShiftTooFar { distance: 8, bits: 8 });
}

#[test]
fn a_step_that_is_not_positive_traps()
{
    let text = "ကိန်းရှင် ခြား သည် ၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\n၁ မှ ၃ အထိ ခြား စီ ကိန်း အနေဖြင့် \
                တိုး၍ ကိန်း ကို ပြသမည်။";
    let (out, kind, _) = trapped(text, &RunOptions::default());
    assert_eq!(out, "");
    assert_eq!(kind, TrapKind::StepNotPositive);
}

#[test]
fn traps_name_their_place()
{
    let (_, result) =
        run_source("ကိန်းရှင် သုည သည် ၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\n၁ / သုည ကို ပြသမည်။");
    let trap = result.unwrap_err();
    let shown = trap.to_string();
    assert!(shown.contains("zero as divisor"), "{shown}");
    assert!(shown.contains("main"), "{shown}");
    assert!(shown.contains(&format!("op {}", trap.pc)), "{shown}");
}

#[test]
fn logic_short_circuits_at_run_time()
{
    let text = "ကိန်းရှင် သုည သည် ၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\nကိန်းရှင် ပိတ် သည် မှန်မှားတန်ဖိုး တစ်ခု \
                ဖြစ်၍ တန်ဖိုးသည် မမှန် ဖြစ်ပါစေ။\n(ပိတ် && ၁ / သုည သည် ၁ နှင့် ညီသည်) ကို ပြသမည်။\n(ပိတ် \
                မဟုတ် || ၁ / သုည သည် ၁ နှင့် ညီသည်) ကို ပြသမည်။";
    assert_eq!(output(text), "မမှန်\nမှန်\n");
}

/// The dir of the heap trap inputs.
fn heap_inputs() -> PathBuf
{
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/inputs/heap")
}

/// Run a heap trap input and take its trap.
fn heap_trap(stem: &str) -> Trap
{
    let text =
        std::fs::read_to_string(heap_inputs().join(format!("{stem}.zaka"))).unwrap();
    let options = match stem
    {
        "out_of_mem" => RunOptions::builder().max_heap(64).build(),
        _ => RunOptions::default(),
    };
    let (out, result) = run_source_with(&text, &options);
    let Err(trap) = result
    else
    {
        panic!("{stem} should trap; it printed:\n{out}");
    };
    trap
}

#[test]
fn each_heap_mistake_traps_with_its_kind()
{
    let table = [
        ("null_deref", TrapKind::NullDeref { addr: 0 }),
        ("null_step", TrapKind::NullDeref { addr: 0 }),
        ("out_of_bounds", TrapKind::OutOfBounds { addr: 0, size: 0 }),
        ("use_after_free", TrapKind::UseAfterFree { addr: 0 }),
        ("double_free", TrapKind::DoubleFree { addr: 0 }),
        ("bad_free", TrapKind::BadFree { addr: 0 }),
        ("misaligned", TrapKind::Misaligned { addr: 0, align: 0 }),
        ("bad_value", TrapKind::BadValue { addr: 0, kind: String::new() }),
        ("out_of_mem", TrapKind::OutOfMem { size: 0 }),
        ("mixed_blocks", TrapKind::MixedBlocks),
        ("overlap", TrapKind::Overlap { addr: 0 }),
        ("local_escape", TrapKind::UseAfterFree { addr: 0 }),
        (
            "index_out_of_range",
            TrapKind::IndexOutOfRange { index: 0, first: 0, len: 0 },
        ),
    ];
    let inputs = std::fs::read_dir(heap_inputs()).unwrap().count();
    assert_eq!(inputs, table.len(), "each input needs a row");
    for (stem, want) in table
    {
        let trap = heap_trap(stem);
        assert_eq!(discriminant(&trap.kind), discriminant(&want), "{stem}: {trap}");
    }
}

#[test]
fn a_local_address_that_escapes_names_its_variable_and_function()
{
    let trap = heap_trap("local_escape");
    let block = trap.block.clone().expect("a block");
    assert!(block.contains("x of ဂဏန်း"), "{block}");
    assert!(trap.to_string().ends_with(&format!("; {block}")), "{trap}");
}

/// A step off the null address makes no address of a block, even when the
/// number it adds is the address of a freed one.
#[test]
fn a_step_from_null_traps_as_a_null_access()
{
    let trap = heap_trap("null_step");
    assert_eq!(trap.kind, TrapKind::NullDeref { addr: 0 }, "{trap}");
    assert_eq!(trap.block, None);
}

#[test]
fn heap_traps_name_their_block()
{
    let trap = heap_trap("out_of_bounds");
    let block = trap.block.clone().expect("a block");
    assert!(block.contains("32 octets, heap"), "{block}");
    assert_eq!(heap_trap("null_deref").block, None);
    assert_eq!(heap_trap("mixed_blocks").block, None);
    let block = heap_trap("overlap").block.expect("a block");
    assert!(block.contains("32 octets, heap"), "{block}");
}

#[test]
fn a_cell_outside_its_list_traps_with_the_range()
{
    let trap = heap_trap("index_out_of_range");
    assert_eq!(trap.kind, TrapKind::IndexOutOfRange { index: 3, first: 0, len: 3 });
    assert_eq!(trap.block, None);
    let text = trap.to_string();
    assert!(text.contains("cell 3") && text.contains("0 to 2"), "{text}");
}

#[test]
fn a_remove_from_a_list_with_no_cells_traps()
{
    let text = "ကိန်းရှင် l သည် [၁] ဖြစ်ပါစေ။
l ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။
l ကို ပြသမည်။
l ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။";
    let (out, kind, _) = trapped(text, &RunOptions::default());
    assert_eq!(out, "[]\n");
    assert_eq!(kind, TrapKind::NoCellToRemove);
}

#[test]
fn a_remove_from_text_with_no_characters_traps()
{
    let text = "ကိန်းရှင် t သည် \"a\" ဖြစ်ပါစေ။
t ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။
t ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။";
    let (_, kind, _) = trapped(text, &RunOptions::default());
    assert_eq!(kind, TrapKind::NoCharToRemove);
}

#[test]
fn an_add_in_front_of_a_place_past_the_end_traps()
{
    let list = "ကိန်းရှင် l သည် [၁၊ ၂] ဖြစ်ပါစေ။\nl ၏ အကွက် ၃ ၏ အရှေ့တွင် ၉ ကို ထည့်မည်။";
    let (_, kind, _) = trapped(list, &RunOptions::default());
    assert_eq!(kind, TrapKind::NoPlaceToInsert { index: 3, first: 0, len: 2 });
    assert!(kind.to_string().contains("0 to 2, and 2 is the end"), "{kind}");
    let text = "ကိန်းရှင် t သည် \"ab\" ဖြစ်ပါစေ။\nt ၏ ၄ ခုမြောက်စာလုံး ၏ အရှေ့တွင် \"x\" ကို ထည့်မည်။";
    let (_, kind, _) = trapped(text, &RunOptions::default());
    assert_eq!(kind, TrapKind::NoPlaceToInsert { index: 4, first: 1, len: 2 });
}

#[test]
fn a_replace_of_a_bad_range_traps()
{
    let head = "ကိန်းရှင် t သည် \"ကခ\" ဖြစ်ပါစေ။\n";
    let trap_of =
        |line: &str| trapped(&format!("{head}{line}"), &RunOptions::default()).1;
    let backward = trap_of("t ၏ အကွက် ၂ မှ ၁ ထိကို \"x\" ဖြင့် အစားထိုးမည်။");
    assert_eq!(backward, TrapKind::EndBeforeStart { start: 2, end: 1 });
    assert!(backward.to_string().contains("from 2 to 1 ends before"), "{backward}");
    let past = trap_of("t ၏ အကွက် ၀ မှ ၉ ထိကို \"x\" ဖြင့် အစားထိုးမည်။");
    assert_eq!(past, TrapKind::IndexOutOfRange { index: 9, first: 0, len: 6 });
    let cut = trap_of("t ၏ အကွက် ၀ မှ ၀ ထိကို \"x\" ဖြင့် အစားထိုးမည်။");
    assert!(matches!(cut, TrapKind::TextCut { .. }), "{cut:?}");
    let chars = trap_of("t ၏ ၁ လုံးမြောက်စာလုံးမှ ၃ လုံးမြောက်စာလုံးထိကို \"x\" ဖြင့် အစားထိုးမည်။");
    assert_eq!(chars, TrapKind::CharOutOfRange { index: 3, first: 1, len: 2 });
    let turned = trap_of("t ၏ ၂ လုံးမြောက်စာလုံးမှ ၁ လုံးမြောက်စာလုံးထိကို \"x\" ဖြင့် အစားထိုးမည်။");
    assert_eq!(turned, TrapKind::EndBeforeStart { start: 2, end: 1 });
}

/// A list `l` of three cells, a count `n` of four, and a list `s` that holds
/// the second cell of `l`.
const SLICED: &str = "ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် n သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
n ကို n + ၂ ဟု ထားမည်။
ကိန်းရှင် s သည် l ၏ အကွက် ၁ မှစ၍ ၁ ကွက် ဖြစ်ပါစေ။
";

#[test]
fn a_range_past_its_source_traps()
{
    let options = RunOptions::default();
    let past_list = format!("{SLICED}l ၏ အကွက် ၀ မှစ၍ n ကွက် ကို ပြသမည်။");
    let (_, kind, _) = trapped(&past_list, &options);
    assert_eq!(kind, TrapKind::SliceOutOfRange { start: 0, len: 4, limit: 3 });
    let past_slice = format!("{SLICED}s ၏ အကွက် ၀ မှစ၍ n - ၂ ကွက် ကို ပြသမည်။");
    let (_, kind, _) = trapped(&past_slice, &options);
    assert_eq!(kind, TrapKind::SliceOutOfRange { start: 0, len: 2, limit: 1 });
    assert!(kind.to_string().contains("2 cells from cell 0"), "{kind}");
}

#[test]
fn a_cell_past_the_count_of_its_slice_traps()
{
    let (out, kind, _) = trapped(
        &format!("{SLICED}s ကို ပြသမည်။\ns ၏ အကွက် n - ၃ ကို ပြသမည်။"),
        &RunOptions::default(),
    );
    assert_eq!(out, "[၂]\n");
    assert_eq!(kind, TrapKind::IndexOutOfRange { index: 1, first: 0, len: 1 });
}

#[test]
fn a_range_past_its_heap_block_traps()
{
    let text = "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် n သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ပ ၏ အကွက် ၀ မှစ၍ n ကွက် ကို ပြသမည်။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။";
    let (_, kind, _) = trapped(text, &RunOptions::default());
    assert!(matches!(kind, TrapKind::OutOfBounds { size: 0, .. }), "{kind}");
}

#[test]
fn a_list_that_swaps_its_own_cells_reads_them_whole()
{
    let text = "ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
l ကို [(l ၏ အကွက် ၁)၊ (l ၏ အကွက် ၀)] ဟု ထားမည်။
l ကို ပြသမည်။
ကိန်းရှင် s သည် l ဖြစ်ပါစေ။
s ကို s ၏ အကွက် ၁ မှစ၍ ၁ ကွက် ဟု ထားမည်။
s ကို ပြသမည်။";
    assert_eq!(output(text), "[၂၊ ၁]\n[၁]\n");
}

#[test]
fn stop_mode_traps_on_overflow_and_wrap_mode_wraps()
{
    let text = "ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၅၀ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် -၁၂၈ ဖြစ်ပါစေ။
က + ၁၀ ကို ပြသမည်။
-ခ ကို ပြသမည်။";
    assert_eq!(wrapped(text), "၄\n-၁၂၈\n");
    let module = Module { overflow: OverflowMode::Stop, ..compile(text) };
    let (out, result) = run_module(&module, &RunOptions::default());
    assert_eq!(out, "");
    let kind = result.expect_err("the add overflows").kind;
    assert_eq!(kind, TrapKind::Overflow { op: "+".into(), kind: "u8".into() });
    assert!(kind.to_string().contains("does not fit u8"), "{kind}");
    let negate = text.replace("က + ၁၀ ကို ပြသမည်။\n", "");
    let module = Module { overflow: OverflowMode::Stop, ..compile(&negate) };
    let (_, result) = run_module(&module, &RunOptions::default());
    assert!(matches!(result.unwrap_err().kind, TrapKind::Overflow { .. }));
}

#[test]
fn stop_mode_leaves_counted_loops_alone()
{
    let text = "-၂၀ မှ ၂၀ အထိ ၂၀ စီ န အနေဖြင့် တိုး၍ န ကို ပြသမည်။";
    let module = Module { overflow: OverflowMode::Stop, ..compile(text) };
    let (out, result) = run_module(&module, &RunOptions::default());
    assert_eq!(result, Ok(()));
    assert_eq!(out, "-၂၀\n၀\n၂၀\n");
}

#[test]
fn operands_left_of_a_pass_on_run_first_whatever_their_kind()
{
    let text = "\
လုပ်ဆောင်ချက် စာ သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
\"စာ ပြေး\" ဟု ပြသမည်။
\"က\" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပြင် သည်
တ ဟူသော စာသား တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ကိန်းပြည့် ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
\"ပြင် ပြေး\" ဟု ပြသမည်။
တ ကို \"ပြင်ပြီး\" ဟု ထားမည်။
\"!\" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် အလုပ် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ကိန်းပြည့် ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို \
                ပြန်ပေးမည်။
ကိန်းရှင် စ သည် \"မူလ\" ဖြစ်ပါစေ။
စာ() + ပြင်(စ)? ကို ပြသမည်။
စ ကို \"မူလ\" ဟု ထားမည်။
စ + ပြင်(စ)? ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အလုပ်() ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။";
    let (out, result) = run_source(text);
    assert_eq!(result, Ok(()));
    assert_eq!(out, "စာ ပြေး\nပြင် ပြေး\nက!\nပြင် ပြေး\nမူလ!\nအောင်မြင်သည့်သတင်း\n");
}

/// A function that can fail with text: it fails on 0.
const CLOSE: &str = "\
လုပ်ဆောင်ချက် ပိတ် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် \"မပိတ်နိုင်ပါ\" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

#[test]
fn an_empty_optional_read_traps()
{
    let text =
        include_str!("../../zakac_codegen_target_c/tests/inputs/empty_optional.zaka");
    let (out, kind, _) = trapped(text, &RunOptions::default());
    assert_eq!(out, "၇\n");
    assert_eq!(kind, TrapKind::NoData);
}

#[test]
fn a_result_reads_its_part_and_the_other_part_traps()
{
    let text = format!(
        "{CLOSE}ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ပိတ်(၀) ဖြစ်ပါစေ။
ရ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
ရ ထဲမှ ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။"
    );
    let (out, kind, _) = trapped(&text, &RunOptions::default());
    assert_eq!(out, "မပိတ်နိုင်ပါ\n");
    assert_eq!(kind, TrapKind::NotGood { text: "မပိတ်နိုင်ပါ".to_string() });
    let good = text.replace("ပိတ်(၀)", "ပိတ်(၄)");
    let (out, kind, _) = trapped(&good, &RunOptions::default());
    assert_eq!(out, "");
    assert_eq!(kind, TrapKind::NotFault);
}

#[test]
fn a_whole_result_prints_as_its_part()
{
    let text = format!(
        "{CLOSE}ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပိတ်(၀) ကို ပြသမည်။
ကိန်းရှင် ရ သည် ပိတ်(၇) ဖြစ်ပါစေ။
\"ရ = {{ရ}}\" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။"
    );
    assert_eq!(output(&text), "အမှားအယွင်း(\"မပိတ်နိုင်ပါ\")\nရ = ရလဒ်ကောင်း(၇)\n");
}

#[test]
fn a_side_with_no_value_prints_as_news_and_its_bare_failure_traps()
{
    let find = "\
လုပ်ဆောင်ချက် ရှာ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ရေး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";
    let text = format!(
        "{find}ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရှာ(၀) ကို ပြသမည်။
ရေး() ကို ပြသမည်။
ရှာ(၀) ထဲမှ ဒေတာ ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။"
    );
    let (out, kind, _) = trapped(&text, &RunOptions::default());
    assert_eq!(out, "မအောင်မြင်သည့်သတင်း\nအောင်မြင်သည့်သတင်း\n");
    assert_eq!(kind, TrapKind::NotGood { text: String::new() });
    assert_eq!(kind.to_string(), "the result holds a failure, not a good result");
}
