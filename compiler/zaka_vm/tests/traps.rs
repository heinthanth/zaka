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
use zakac_codegen::bc::Module;

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
    let text = "ကိန်းရှင် သုည သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\n\"အရင်\" ဟု ပြသပါ။\n၁ / သုည ကို \
                ပြသပါ။\n\"နောက်\" ဟု ပြသပါ။";
    let (out, kind, func) = trapped(text, &RunOptions::default());
    assert_eq!(out, "အရင်\n");
    assert_eq!(kind, TrapKind::ZeroDivisor);
    assert_eq!(func, "main");
    let (_, rem, _) = trapped(&text.replace(" / ", " % "), &RunOptions::default());
    assert_eq!(rem, TrapKind::ZeroDivisor);
}

#[test]
fn endless_recursion_hits_the_depth_limit()
{
    let text = "လုပ်ဆောင်ချက် ထပ် ကို လုပ်ဆောင်ပါ။\nလုပ်ဆောင်ချက် ထပ် သည်\nအောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nလုပ်ဆောင်ချက် ထပ် ကို လုပ်ဆောင်ပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let options = RunOptions::builder().max_depth(64).build();
    let (_, kind, func) = trapped(text, &options);
    assert_eq!(kind, TrapKind::StackOverflow { depth: 64 });
    assert_eq!(func, "ထပ်");
}

#[test]
fn endless_loops_hit_the_step_limit()
{
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\nမှန် ဖြစ်နေသရွေ့ က ကို က + ၁ ဟု ထားပါ။";
    let options = RunOptions::builder().max_steps(1000).build();
    let (_, kind, _) = trapped(text, &options);
    assert_eq!(kind, TrapKind::StepLimit { steps: 1000 });
}

#[test]
fn integer_arithmetic_wraps()
{
    let text = "ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ၂၅၅ ဖြစ်ပါစေ။\nက + ၁ ကို ပြသပါ။\nကိန်းရှင် ခ \
                သည် ၈ ဘစ် ကိန်းပြည့် -၁၂၈ ဖြစ်ပါစေ။\n-ခ ကို ပြသပါ။\nခ / -၁ ကို ပြသပါ။\nခ - ၁ ကို \
                ပြသပါ။\nကိန်းရှင် ဂ သည် ၈ ဘစ် ကိန်းပြည့် ၁၀၀ ဖြစ်ပါစေ။\nဂ * ၃ ကို ပြသပါ။";
    assert_eq!(output(text), "၀\n-၁၂၈\n-၁၂၈\n၁၂၇\n၄၄\n");
}

#[test]
fn shifting_past_the_width_traps()
{
    let text = "ကိန်းရှင် ရွေ့ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ၈ ဖြစ်ပါစေ။\nကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် \
                ကိန်းပြည့် ၁ ဖြစ်ပါစေ။\nက << ရွေ့ ကို ပြသပါ။";
    let (_, kind, _) = trapped(text, &RunOptions::default());
    assert_eq!(kind, TrapKind::ShiftTooFar { distance: 8, bits: 8 });
}

#[test]
fn a_step_that_is_not_positive_traps()
{
    let text =
        "ကိန်းရှင် ခြား သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\n၁ မှ ၃ အထိ ခြား စီ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသပါ။";
    let (out, kind, _) = trapped(text, &RunOptions::default());
    assert_eq!(out, "");
    assert_eq!(kind, TrapKind::StepNotPositive);
}

#[test]
fn traps_name_their_place()
{
    let (_, result) = run_source("ကိန်းရှင် သုည သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\n၁ / သုည ကို ပြသပါ။");
    let trap = result.unwrap_err();
    let shown = trap.to_string();
    assert!(shown.contains("zero as divisor"), "{shown}");
    assert!(shown.contains("main"), "{shown}");
    assert!(shown.contains(&format!("op {}", trap.pc)), "{shown}");
}

#[test]
fn logic_short_circuits_at_run_time()
{
    let text = "ကိန်းရှင် သုည သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\nကိန်းရှင် ပိတ် သည် မှန်မှားတန်ဖိုး မမှန် \
                ဖြစ်ပါစေ။\n(ပိတ် && ၁ / သုည သည် ၁ နှင့် ညီသည်) ကို ပြသပါ။\n(ပိတ် မဟုတ် || ၁ / သုည သည် ၁ \
                နှင့် ညီသည်) ကို ပြသပါ။";
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

/// A step off `ဗလာ` makes no address of a block, even when the
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

/// A list `l` of three cells, a count `n` of four, and a slice `s` of the
/// second cell of `l`.
const SLICED: &str = "ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် n သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂ ဖြစ်ပါစေ။
n ကို n + ၂ ဟု ထားပါ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် l ၏ အကွက် ၁ မှစ၍ ၁ ကွက် ဖြစ်ပါစေ။
";

#[test]
fn a_range_past_its_source_traps()
{
    let options = RunOptions::default();
    let past_list = format!("{SLICED}l ၏ အကွက် ၀ မှစ၍ n ကွက် ကို ပြသပါ။");
    let (_, kind, _) = trapped(&past_list, &options);
    assert_eq!(kind, TrapKind::SliceOutOfRange { start: 0, len: 4, limit: 3 });
    let past_slice = format!("{SLICED}s ၏ အကွက် ၀ မှစ၍ n - ၂ ကွက် ကို ပြသပါ။");
    let (_, kind, _) = trapped(&past_slice, &options);
    assert_eq!(kind, TrapKind::SliceOutOfRange { start: 0, len: 2, limit: 1 });
    assert!(kind.to_string().contains("2 cells from cell 0"), "{kind}");
}

#[test]
fn a_cell_past_the_count_of_its_slice_traps()
{
    let (out, kind, _) = trapped(
        &format!("{SLICED}s ကို ပြသပါ။\ns ၏ အကွက် n - ၃ ကို ပြသပါ။"),
        &RunOptions::default(),
    );
    assert_eq!(out, "[၂]\n");
    assert_eq!(kind, TrapKind::IndexOutOfRange { index: 1, first: 0, len: 1 });
}

#[test]
fn a_range_past_its_heap_block_traps()
{
    let text = "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် n သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။
ပ ၏ အကွက် ၀ မှစ၍ n ကွက် ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။";
    let (_, kind, _) = trapped(text, &RunOptions::default());
    assert!(matches!(kind, TrapKind::OutOfBounds { size: 0, .. }), "{kind}");
}

#[test]
fn a_list_that_swaps_its_own_cells_reads_them_whole()
{
    let text = "ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
l ကို [(l ၏ အကွက် ၁)၊ (l ၏ အကွက် ၀)] ဟု ထားပါ။
l ကို ပြသပါ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် l ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
s ကို s ၏ အကွက် ၁ မှစ၍ ၁ ကွက် ဟု ထားပါ။
s ကို ပြသပါ။";
    assert_eq!(output(text), "[၂၊ ၁]\n[၁]\n");
}

#[test]
fn stop_mode_traps_on_overflow_and_wrap_mode_wraps()
{
    let text = "ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၂၅၀ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် -၁၂၈ ဖြစ်ပါစေ။
က + ၁၀ ကို ပြသပါ။
-ခ ကို ပြသပါ။";
    assert_eq!(output(text), "၄\n-၁၂၈\n");
    let module = Module { overflow: OverflowMode::Stop, ..compile(text) };
    let (out, result) = run_module(&module, &RunOptions::default());
    assert_eq!(out, "");
    let kind = result.expect_err("the add overflows").kind;
    assert_eq!(kind, TrapKind::Overflow { op: "+".into(), kind: "u8".into() });
    assert!(kind.to_string().contains("does not fit u8"), "{kind}");
    let negate = text.replace("က + ၁၀ ကို ပြသပါ။\n", "");
    let module = Module { overflow: OverflowMode::Stop, ..compile(&negate) };
    let (_, result) = run_module(&module, &RunOptions::default());
    assert!(matches!(result.unwrap_err().kind, TrapKind::Overflow { .. }));
}

#[test]
fn stop_mode_leaves_counted_loops_alone()
{
    let text = "-၂၀ မှ ၂၀ အထိ ၂၀ စီ န အနေဖြင့် တိုး၍ န ကို ပြသပါ။";
    let module = Module { overflow: OverflowMode::Stop, ..compile(text) };
    let (out, result) = run_module(&module, &RunOptions::default());
    assert_eq!(result, Ok(()));
    assert_eq!(out, "-၂၀\n၀\n၂၀\n");
}
