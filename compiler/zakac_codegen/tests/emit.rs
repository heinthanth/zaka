mod common;

use common::emit_text;
use common::main_func;
use common::main_ops;
use common::ops;
use zakac_codegen::bc::Op;
use zakac_codegen::bc::TrapCode;
use zakac_codegen::emit::MAIN_NAME;

const FLAGS: &str =
    "ကိန်းရှင် က သည် မှန်မှားတန်ဖိုး မှန် ဖြစ်ပါစေ။\nကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး မမှန် ဖြစ်ပါစေ။\n";

fn with_flags(line: &str) -> Vec<String> { main_ops(&format!("{FLAGS}{line}")) }

#[test]
fn main_comes_last_and_funcs_keep_their_order()
{
    let text = "လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်ပါ။\nလုပ်ဆောင်ချက် နှုတ်ဆက် သည်\nအောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\n\"မင်္ဂလာပါ\" ဟု ပြသပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။\nလုပ်ဆောင်ချက် \
                ဘာမှ သည်\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let module = emit_text(text);
    let names: Vec<&str> = module.funcs.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["နှုတ်ဆက်", "ဘာမှ", MAIN_NAME]);
    assert_eq!(module.main, 2);
    assert_eq!(ops(&module.funcs[0]), ["print_text t0", "ret"]);
    assert_eq!(ops(&module.funcs[1]), ["ret"]);
    assert_eq!(ops(main_func(&module)), ["call f0", "ret"]);
}

#[test]
fn texts_are_pooled_once()
{
    let module = emit_text("\"က\" ဟု ပြသပါ။\n\"ခ\" ဟု ပြသပါ။\n\"က\" ဟု ပြသပါ။");
    assert_eq!(module.texts, ["က", "ခ"]);
    let want = ["print_text t0", "print_text t1", "print_text t0", "ret"];
    assert_eq!(ops(main_func(&module)), want);
}

#[test]
fn and_runs_the_right_side_just_when_needed()
{
    let found = with_flags("(က && ခ) ကို ပြသပါ။");
    let want = [
        "push true:bool",
        "store_global g0",
        "push false:bool",
        "store_global g1",
        "load_global g0",
        "jump_if_false @8",
        "load_global g1",
        "jump @9",
        "push false:bool",
        "print",
        "ret",
    ];
    assert_eq!(found, want);
}

#[test]
fn or_runs_the_right_side_just_when_needed()
{
    let found = with_flags("(က || ခ) ကို ပြသပါ။");
    assert_eq!(found[5], "jump_if_true @8");
    assert_eq!(found[8], "push true:bool");
}

#[test]
fn if_without_else_has_no_jump_over_it()
{
    let found = with_flags("အကယ်၍ က ဖြစ်လျှင် \"ဟုတ်\" ဟု ပြသပါ။");
    let tail = &found[4..];
    assert_eq!(tail, ["load_global g0", "jump_if_false @7", "print_text t0", "ret"]);
}

#[test]
fn if_with_else_jumps_over_the_else_branch()
{
    let found = with_flags("အကယ်၍ က ဖြစ်လျှင် \"ဟုတ်\" ဟု ပြသပါ။\nသို့မဟုတ်ပါက \"မဟုတ်\" ဟု ပြသပါ။");
    let tail = &found[4..];
    let want = [
        "load_global g0",
        "jump_if_false @8",
        "print_text t0",
        "jump @9",
        "print_text t1",
        "ret",
    ];
    assert_eq!(tail, want);
}

#[test]
fn break_and_continue_go_to_the_exit_and_the_update()
{
    let text = "ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ၀ ဖြစ်ပြီး ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို \
                လုပ်ဆောင်ပါ။\nအကယ်၍ ကိန်း % ၂ သည် ၁ နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။\nအကယ်၍ ကိန်း သည် \
                ၇ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်ပါ။\nပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု \
                ထားပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let found = main_ops(text);
    let update = found.iter().rposition(|op| op == "push 1:isize").unwrap() - 1;
    let exit = found.len() - 1;
    assert_eq!(found[update], "load_global g0");
    assert!(found.contains(&format!("jump @{update}")), "{found:?}");
    assert!(found.contains(&format!("jump @{exit}")), "{found:?}");
    assert_eq!(found[exit - 1], "jump @2");
}

#[test]
fn call_statements_drop_a_result()
{
    let text = "လုပ်ဆောင်ချက် တစ် ကို လုပ်ဆောင်ပါ။\nလုပ်ဆောင်ချက် တစ် သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် \
                ရလဒ် ကို ပြန်ပေးမည်။\n၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let module = emit_text(text);
    assert_eq!(ops(main_func(&module)), ["call f0", "pop", "ret"]);
    let f = &module.funcs[0];
    assert_eq!(f.code.last(), Some(&Op::Ret));
    assert_eq!(f.code.iter().filter(|op| **op == Op::Ret).count(), 1);
}

#[test]
fn constant_bounds_fix_the_direction()
{
    let up = main_ops("၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။");
    let count = |list: &[String], op: &str| list.iter().filter(|o| *o == op).count();
    assert_eq!((count(&up, "add"), count(&up, "sub")), (1, 1), "{up:?}");
    let module = emit_text("၃ မှ ၁ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။");
    let main = main_func(&module);
    assert!(main.locals.is_empty(), "{:?}", main.locals);
    assert_eq!(ops(main).iter().filter(|o| *o == "sub").count(), 2);
}

#[test]
fn runtime_bounds_pick_the_direction_once()
{
    let text =
        "ကိန်းရှင် အဆုံး သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။\n၁ မှ အဆုံး အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။";
    let module = emit_text(text);
    let names: Vec<&str> =
        main_func(&module).locals.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["%to", "%dir"], "a top-level loop variable is a global");
    let found = ops(main_func(&module));
    assert_eq!(found.iter().filter(|o| *o == "load_local l1").count(), 2);
}

#[test]
fn a_runtime_step_is_checked()
{
    let text =
        "ကိန်းရှင် ခြား သည် ကိန်းပြည့် ၂ ဖြစ်ပါစေ။\n၀ မှ ၁၀ အထိ ခြား စီ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသပါ။";
    let module = emit_text(text);
    let main = main_func(&module);
    assert!(main.code.contains(&Op::Trap(TrapCode::StepNotPositive)));
    let names: Vec<&str> = main.locals.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["%step"]);
    let constant = emit_text("၀ မှ ၁၀ အထိ ၂ စီ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသပါ။");
    assert!(!main_func(&constant).code.iter().any(|op| matches!(op, Op::Trap(_))));
}

#[test]
fn signed_loops_measure_the_distance_unsigned()
{
    let found = main_ops("၁ မှ ၃ အထိ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသပါ။");
    assert!(found.contains(&"convert usize".to_string()), "{found:?}");
    assert!(found.contains(&"push 3:usize".to_string()), "{found:?}");
    let unsigned = main_ops(
        "၁ မှ ၃ အထိ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသပါ။"
            .replace("၁ မှ", "လက္ခဏာမဲ့ ကိန်းပြည့် ၁ မှ")
            .as_str(),
    );
    assert!(!unsigned.iter().any(|o| o.starts_with("convert")), "{unsigned:?}");
}

/// The ops and their source text of a clean program, with no folding, so
/// constant expressions keep their ops.
fn unfolded(text: &str) -> (zakac_codegen::bc::Module, String, u32)
{
    use zakac_codegen::artifact::run_phases;
    use zakac_ir_builder::ir::FoldOptions;
    use zakac_lexer::lexer::LexerOptions;
    use zakac_semcheck::semcheck::SemcheckOptions;
    use zakac_span::filename::FileName;
    use zakac_span::source_map::SourceMap;

    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let options = SemcheckOptions::default();
    let emitted =
        run_phases(&file, &LexerOptions::default(), &options, &FoldOptions::none());
    let base = file.start_pos.as_u32();
    (emitted.module.expect("emitted"), file.src.to_string(), base)
}

/// The source text a span covers, in a file that starts at `base`.
fn text_of(src: &str, base: u32, span: zakac_span::span::Span) -> &str
{
    &src[(span.lo.as_u32() - base) as usize..(span.hi.as_u32() - base) as usize]
}

#[test]
fn each_op_has_a_span()
{
    let text = "လုပ်ဆောင်ချက် တစ် ကို လုပ်ဆောင်ပါ။\nလုပ်ဆောင်ချက် တစ် သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် \
                ရလဒ် ကို ပြန်ပေးမည်။\n၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။\nလုပ်ဆောင်စရာများ \
                ပြီးပါပြီး။\n\"က\" ဟု ပြသပါ။";
    let module = emit_text(text);
    for func in &module.funcs
    {
        assert_eq!(func.spans.len(), func.code.len(), "{}", func.name);
    }
    let main = main_func(&module);
    let print = main.code.iter().position(|op| matches!(op, Op::PrintText(_))).unwrap();
    assert!(!main.spans[print].is_placeholder());
    assert!(!main.spans[print].is_blank());
}

#[test]
fn expression_ops_take_the_region_of_their_expression()
{
    let (module, src, base) = unfolded("(၂ + ၃) * ၄ ကို ပြသပါ။");
    let main = main_func(&module);
    let at = |name: &str| ops(main).iter().position(|op| op == name).unwrap();
    let span = |name: &str| main.spans[at(name)];
    let (add, mul, print) = (span("add"), span("mul"), span("print"));
    assert!(mul.contains(&add) && add != mul, "{add:?} {mul:?}");
    assert!(print.contains(&mul) && mul != print, "{mul:?} {print:?}");
    assert_eq!(text_of(&src, base, main.spans[0]), "၂");
    assert!(text_of(&src, base, add).contains("၂ + ၃"));
}

#[test]
fn the_implicit_ret_sits_at_the_end_of_the_declaration()
{
    let text = "လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်ပါ။\nလုပ်ဆောင်ချက် နှုတ်ဆက် သည်\nအောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\n\"မင်္ဂလာပါ\" ဟု ပြသပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let module = emit_text(text);
    for func in &module.funcs
    {
        let last = *func.spans.last().unwrap();
        assert!(last.is_blank(), "{}", func.name);
        let before = &func.spans[..func.spans.len() - 1];
        assert!(before.iter().all(|s| s.hi <= last.lo), "{}", func.name);
    }
}

#[test]
fn jumps_of_an_if_take_the_if_span()
{
    let text =
        format!("{FLAGS}အကယ်၍ က ဖြစ်လျှင် \"ဟုတ်\" ဟု ပြသပါ။\nသို့မဟုတ်ပါက \"မဟုတ်\" ဟု ပြသပါ။");
    let main_mod = emit_text(&text);
    let main = main_func(&main_mod);
    let found = ops(main);
    let branch = found.iter().position(|op| op.starts_with("jump_if_false")).unwrap();
    let over = found.iter().position(|op| op.starts_with("jump @")).unwrap();
    assert_eq!(main.spans[branch], main.spans[over]);
    assert!(main.spans[branch].contains(&main.spans[branch - 1]));
    assert!(main.spans[branch].contains(&main.spans[branch + 1]));
}

#[test]
fn addressed_locals_live_in_cells()
{
    let text = "ကိန်းရှင် x သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။\nကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ x ၏ လိပ်စာ \
                ဖြစ်ပါစေ။\np ၏ တန်ဖိုး ကို ပြသပါ။";
    let found = main_ops(text);
    let want = [
        "cell 8 8",
        "store_global g0", // prologue: x gets a cell
        "load_global g0",
        "push 5:isize",
        "store isize", // x = 5 through the cell
        "load_global g0",
        "store_global g1", // p = &x
        "load_global g1",
        "load isize",
        "print", // *p
        "ret",
    ];
    assert_eq!(found, want);
    let module = emit_text(text);
    assert!(module.globals[0].addressed && !module.globals[1].addressed);
}

#[test]
fn addressed_parameters_go_into_their_cell()
{
    let text = "လုပ်ဆောင်ချက် ပြ သည်\nကိန်းပြည့် က ကို လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\nက ၏ လိပ်စာ ၏ \
                တန်ဖိုး ကို ပြသပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let module = emit_text(text);
    let found = ops(&module.funcs[0]);
    assert_eq!(
        found[..5],
        ["cell 8 8", "dup", "load_local l0", "store isize", "store_local l0"]
    );
}

#[test]
fn heap_statements_and_pointer_arithmetic_emit_heap_ops()
{
    let text = "ကိန်းရှင် p သည် ၃၂ ဘစ် ကိန်းပြည့် ၏ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ၃၂ ဘစ် ကိန်းပြည့် ၁၀ ခုစာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
p + ၁ ၏ တန်ဖိုး ကို ၇ ဟု ထားပါ။
p လိပ်စာရှိကို မင်မိုရီ ၆၄ ဘိုက်စာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
p လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။";
    let found = main_ops(text);
    let want = [
        "push 0x0:ptr",
        "store_global g0",
        "push 10:usize",
        "alloc 4",
        "store_global g0",
        "load_global g0",
        "push 1:isize",
        "ptr_add 4",
        "push 7:i32",
        "store i32",
        "load_global g0",
        "push 64:usize",
        "realloc 1",
        "store_global g0",
        "load_global g0",
        "free",
        "push 0x0:ptr",
        "store_global g0",
        "ret",
    ];
    assert_eq!(found, want);
}

#[test]
fn a_deref_destination_uses_dup()
{
    let text =
        "ကိန်းရှင် pp သည် ကိန်းပြည့် ၏ လိပ်စာ ၏ လိပ်စာ ဖြစ်ပါစေ။\npp ၏ တန်ဖိုး လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။";
    let found = main_ops(text);
    assert_eq!(
        found[2..],
        [
            "load_global g0",
            "dup",
            "load *isize",
            "free",
            "push 0x0:ptr",
            "store *isize",
            "ret"
        ]
    );
}

#[test]
fn an_offset_to_an_opaque_place_steps_then_converts()
{
    let text = "ကိန်းရှင် x သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ x ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် q သည် လိပ်စာ p + ၁ ဖြစ်ပါစေ။
q ကို p + ၁ ဟု ထားပါ။
q == p + ၁ ကို ပြသပါ။
လုပ်ဆောင်ချက် ယူ ကို p + ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ယူ သည် လိပ်စာ o ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
o ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် ရှေ့ သည် ကိန်းပြည့် ၏ လိပ်စာ r ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ လိပ်စာ ရလဒ် ကို ပြန်ပေးမည်။
r + ၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let module = emit_text(text);
    let found = ops(main_func(&module));
    let at = found.iter().position(|op| op == "ptr_add 8").expect("a ptr_add 8");
    assert_eq!(found[at + 1], "convert ptr");
    assert_eq!(found[at + 2], "store_global g2");
    let steps = found.iter().filter(|op| *op == "ptr_add 8").count();
    assert_eq!(steps, 4);
    let ret = ops(&module.funcs[1]);
    let at = ret.iter().position(|op| op == "ptr_add 8").expect("a ptr_add 8");
    assert_eq!(ret[at + 1], "convert ptr");
}
