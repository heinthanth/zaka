mod common;

use common::emit_mode;
use common::emit_text;
use common::in_start;
use common::main_func;
use common::main_ops;
use common::ops;
use zakac_ast::stmt::START_NAME;
use zakac_codegen_target_vm::bc::Op;
use zakac_codegen_target_vm::bc::TrapCode;
use zakac_codegen_target_vm::emit::MAIN_NAME;
use zakac_codegen_target_vm::options::EmitOptions;
use zakac_codegen_target_vm::options::ResultMode;
use zakac_codegen_target_vm::options::ScratchMode;

const FLAGS: &str = "ကိန်းရှင် က သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မှန် ဖြစ်ပါစေ။\nကိန်းရှင် ခ သည် \
                     မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မမှန် ဖြစ်ပါစေ။\n";

fn with_flags(line: &str) -> Vec<String>
{
    main_ops(&in_start(&format!("{FLAGS}{line}")))
}

#[test]
fn main_comes_last_and_funcs_keep_their_order()
{
    let text = "ကိန်းရှင် ဂ သည် ၇ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\nလုပ်ဆောင်ချက် နှုတ်ဆက် သည်\nအောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\n\"မင်္ဂလာပါ\" ဟု ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။\nလုပ်ဆောင်ချက် \
                ဘာမှ သည်\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။\n";
    let module =
        emit_text(&format!("{text}{}", in_start("လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်မည်။")));
    let names: Vec<&str> = module.funcs.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["နှုတ်ဆက်", "ဘာမှ", START_NAME, MAIN_NAME]);
    assert_eq!(module.main, 3);
    assert_eq!(ops(&module.funcs[0]), ["print_text t0", "ret"]);
    assert_eq!(ops(&module.funcs[1]), ["ret"]);
    assert_eq!(ops(main_func(&module)), ["call f0", "ret"]);
    let main = ["global_addr g0", "push 7:isize", "store isize", "call f2", "ret"];
    assert_eq!(
        ops(&module.funcs[3]),
        main,
        "main sets the globals, then runs the start block"
    );
}

#[test]
fn texts_are_pooled_once()
{
    let module = emit_text("\"က\" ဟု ပြသမည်။\n\"ခ\" ဟု ပြသမည်။\n\"က\" ဟု ပြသမည်။");
    assert_eq!(module.texts, ["က", "ခ"]);
    let want = ["print_text t0", "print_text t1", "print_text t0", "ret"];
    assert_eq!(ops(main_func(&module)), want);
}

#[test]
fn and_runs_the_right_side_just_when_needed()
{
    let found = with_flags("(က && ခ) ကို ပြသမည်။");
    let want = [
        "local_addr l0",
        "push true:bool",
        "store bool",
        "local_addr l1",
        "push false:bool",
        "store bool",
        "local_addr l0",
        "load bool",
        "jump_if_false @12",
        "local_addr l1",
        "load bool",
        "jump @13",
        "push false:bool",
        "print",
        "ret",
    ];
    assert_eq!(found, want);
}

#[test]
fn or_runs_the_right_side_just_when_needed()
{
    let found = with_flags("(က || ခ) ကို ပြသမည်။");
    assert_eq!(found[8], "jump_if_true @12");
    assert_eq!(found[12], "push true:bool");
}

#[test]
fn if_without_else_has_no_jump_over_it()
{
    let found = with_flags("အကယ်၍ က ဖြစ်လျှင် \"ဟုတ်\" ဟု ပြသမည်။");
    let tail = &found[6..];
    let want =
        ["local_addr l0", "load bool", "jump_if_false @10", "print_text t0", "ret"];
    assert_eq!(tail, want);
}

#[test]
fn if_with_else_jumps_over_the_else_branch()
{
    let found = with_flags("အကယ်၍ က ဖြစ်လျှင် \"ဟုတ်\" ဟု ပြသမည်။\nသို့မဟုတ်ပါက \"မဟုတ်\" ဟု ပြသမည်။");
    let tail = &found[6..];
    let want = [
        "local_addr l0",
        "load bool",
        "jump_if_false @11",
        "print_text t0",
        "jump @12",
        "print_text t1",
        "ret",
    ];
    assert_eq!(tail, want);
}

#[test]
fn break_and_continue_go_to_the_exit_and_the_update()
{
    let text = "ကိန်းရှင် ကိန်း သည် ၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပြီး ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nအကယ်၍ ကိန်း % ၂ သည် ၁ နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။\nအကယ်၍ ကိန်း သည် \
                ၇ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်မည်။\nပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု \
                ထားမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let found = main_ops(text);
    let update = found.iter().rposition(|op| op == "push 1:isize").unwrap() - 3;
    let exit = found.len() - 1;
    assert_eq!(
        found[update..update + 3],
        ["local_addr l0", "local_addr l0", "load isize"]
    );
    assert!(found.contains(&format!("jump @{update}")), "{found:?}");
    assert!(found.contains(&format!("jump @{exit}")), "{found:?}");
    assert_eq!(found[exit - 1], "jump @3");
}

#[test]
fn call_statements_drop_a_result()
{
    let text = "လုပ်ဆောင်ချက် တစ် ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်ချက် တစ် သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် \
                တစ်ခု ကို ပြန်ပေးမည်။\n၁ ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let module = emit_text(text);
    assert_eq!(ops(main_func(&module)), ["call f0", "pop", "ret"]);
    let f = &module.funcs[0];
    assert_eq!(f.code.last(), Some(&Op::Ret));
    assert_eq!(f.code.iter().filter(|op| **op == Op::Ret).count(), 1);
}

#[test]
fn constant_bounds_fix_the_direction()
{
    let up = main_ops("၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။");
    let count = |list: &[String], op: &str| list.iter().filter(|o| *o == op).count();
    assert_eq!((count(&up, "wrap add"), count(&up, "wrap sub")), (1, 1), "{up:?}");
    let module = emit_text("၃ မှ ၁ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။");
    let main = main_func(&module);
    assert!(main.locals.iter().all(|l| !l.name.starts_with('%')), "{:?}", main.locals);
    assert_eq!(ops(main).iter().filter(|o| *o == "wrap sub").count(), 2);
}

#[test]
fn runtime_bounds_pick_the_direction_once()
{
    let text = "ကိန်းရှင် အဆုံး သည် ၃ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\n၁ မှ အဆုံး အထိ ကိန်း အနေဖြင့် ရေတွက်၍ \
                ကိန်း ကို ပြသမည်။";
    let module = emit_text(text);
    let names: Vec<&str> =
        main_func(&module).locals.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["ကိန်း", "%to", "%dir"], "the loop variable comes first");
    let found = ops(main_func(&module));
    let dir = found.iter().filter(|o| *o == "local_addr l2").count();
    assert_eq!(dir, 3, "the direction is set once and read twice");
}

#[test]
fn a_runtime_step_is_checked()
{
    let text = "ကိန်းရှင် ခြား သည် ၂ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\n၀ မှ ၁၀ အထိ ခြား စီ ကိန်း အနေဖြင့် \
                တိုး၍ ကိန်း ကို ပြသမည်။";
    let module = emit_text(text);
    let main = main_func(&module);
    assert!(main.code.contains(&Op::Trap(TrapCode::StepNotPositive)));
    let names: Vec<&str> = main.locals.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["ကိန်း", "%step"]);
    let constant = emit_text("၀ မှ ၁၀ အထိ ၂ စီ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသမည်။");
    assert!(!main_func(&constant).code.iter().any(|op| matches!(op, Op::Trap(_))));
}

#[test]
fn signed_loops_measure_the_distance_unsigned()
{
    let found = main_ops("၁ မှ ၃ အထိ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသမည်။");
    assert!(found.contains(&"convert usize".to_string()), "{found:?}");
    assert!(found.contains(&"push 3:usize".to_string()), "{found:?}");
    let unsigned = main_ops(
        "၁ မှ ၃ အထိ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသမည်။"
            .replace("၁ မှ", "၁ ဟူသော လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု မှ")
            .as_str(),
    );
    assert!(!unsigned.iter().any(|o| o.starts_with("convert")), "{unsigned:?}");
}

/// The ops and their source text of a clean program, with no folding, so
/// constant expressions keep their ops.
fn unfolded(text: &str) -> (zakac_codegen_target_vm::bc::Module, String, u32)
{
    use zakac_borrow_checker::options::BorrowCheckOptions;
    use zakac_codegen_target_vm::artifact::run_phases;
    use zakac_codegen_target_vm::options::ResultMode;
    use zakac_ir_builder::ir::FoldOptions;
    use zakac_lexer::lexer::LexerOptions;
    use zakac_semcheck::semcheck::SemcheckOptions;
    use zakac_span::filename::FileName;
    use zakac_span::source_map::SourceMap;

    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let options = SemcheckOptions::default();
    let mem = BorrowCheckOptions::default();
    let lexer = LexerOptions::default();
    let emitted = run_phases(
        &file,
        &lexer,
        &options,
        &FoldOptions::none(),
        &mem,
        ResultMode::default(),
    );
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
    let text = "လုပ်ဆောင်ချက် တစ် ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်ချက် တစ် သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် \
                တစ်ခု ကို ပြန်ပေးမည်။\n၁ ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။\n\"က\" ဟု \
                ပြသမည်။";
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
    let (module, src, base) = unfolded(&in_start("(၂ + ၃) * ၄ ကို ပြသမည်။"));
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
    let text = "လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်ချက် နှုတ်ဆက် သည်\nအောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\n\"မင်္ဂလာပါ\" ဟု ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
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
        format!("{FLAGS}အကယ်၍ က ဖြစ်လျှင် \"ဟုတ်\" ဟု ပြသမည်။\nသို့မဟုတ်ပါက \"မဟုတ်\" ဟု ပြသမည်။");
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
fn each_variable_is_read_and_written_through_its_block()
{
    let text = "ကိန်းရှင် x သည် ၅ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\nကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု \
                ဖြစ်၍ တန်ဖိုးသည် x ၏ လိပ်စာ ဖြစ်ပါစေ။\np ၏ တန်ဖိုး ကို ပြသမည်။";
    let module = emit_text(text);
    let main = [
        "global_addr g0",
        "push 5:isize",
        "store isize", // x = 5
        "call f0",
        "ret",
    ];
    assert_eq!(ops(&module.funcs[module.main as usize]), main);
    let start = [
        "local_addr l0",
        "global_addr g0",
        "store isize*", // p = &x
        "local_addr l0",
        "load isize*",
        "load isize",
        "print",
        "ret",
    ];
    assert_eq!(ops(main_func(&module)), start, "*p");
}

#[test]
fn a_parameter_is_read_from_its_block()
{
    let text = "လုပ်ဆောင်ချက် ပြ သည်\nက ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\nက ကို \
                ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let module = emit_text(text);
    let found = ops(&module.funcs[0]);
    assert_eq!(found[..3], ["local_addr l0", "load isize", "print"]);
}

#[test]
fn heap_statements_and_pointer_arithmetic_emit_heap_ops()
{
    let text = "ကိန်းရှင် p သည် ၃၂ ဘစ် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ၃၂ ဘစ် ကိန်းပြည့် ၁၀ ခုစာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
p + ၁ ၏ တန်ဖိုး ကို ၇ ဟု ထားမည်။
လိပ်စာ p ရှိ မင်မိုရီကို ၆၄ ဘိုက်စာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။";
    let found = main_ops(text);
    let want = [
        "global_addr g0",
        "push 10:usize",
        "alloc 4",
        "store i32*",
        "global_addr g0",
        "load i32*",
        "push 1:isize",
        "ptr_add 4",
        "push 7:i32",
        "store i32",
        "global_addr g0",
        "global_addr g0",
        "load i32*",
        "push 64:usize",
        "realloc 1",
        "store i32*",
        "global_addr g0",
        "global_addr g0",
        "load i32*",
        "free",
        "push 0x0:ptr",
        "store i32*",
        "ret",
    ];
    assert_eq!(found, want);
}

#[test]
fn a_deref_destination_uses_dup()
{
    let text = "ကိန်းရှင် pp သည် ကိန်းပြည့် ၏ လိပ်စာ ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် လိပ်စာ ၁ ခုစာ နေရာယူ၍ လိပ်စာကို pp တွင် သိမ်းထားမည်။
လိပ်စာ pp ၏ တန်ဖိုး ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။";
    let found = main_ops(text);
    assert_eq!(
        found[found.len() - 8..],
        [
            "global_addr g0",
            "load isize**",
            "dup",
            "load isize*",
            "free",
            "push 0x0:ptr",
            "store isize*",
            "ret"
        ]
    );
}

#[test]
fn an_offset_to_an_opaque_place_steps_then_converts()
{
    let stmts = "ကိန်းရှင် x သည် ၅ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် x ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် q သည် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် p + ၁ ဖြစ်ပါစေ။
q ကို p + ၁ ဟု ထားမည်။
q == p + ၁ ကို ပြသမည်။
လုပ်ဆောင်ချက် ယူ ကို p + ၁ တို့ဖြင့် လုပ်ဆောင်မည်။";
    let funcs = "လုပ်ဆောင်ချက် ယူ သည် o ဟူသော လိပ်စာ တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
o ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် ရှေ့ သည် r ဟူသော ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ လိပ်စာ တစ်ခု ကို \
                 ပြန်ပေးမည်။
r + ၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။\n";
    let module = emit_text(&format!("{funcs}{}", in_start(stmts)));
    let found = ops(main_func(&module));
    let at = found.iter().position(|op| op == "ptr_add 8").expect("a ptr_add 8");
    assert_eq!(found[at + 1], "convert ptr");
    assert_eq!(found[at + 2], "store ptr");
    let steps = found.iter().filter(|op| *op == "ptr_add 8").count();
    assert_eq!(steps, 4);
    let ret = ops(&module.funcs[1]);
    let at = ret.iter().position(|op| op == "ptr_add 8").expect("a ptr_add 8");
    assert_eq!(ret[at + 1], "convert ptr");
}

/// Whether `found` holds the ops of `run` one after another.
fn holds_run(found: &[String], run: &[&str]) -> bool
{
    found.windows(run.len()).any(|w| w.iter().zip(run).all(|(a, b)| a == b))
}

#[test]
fn goldens_without_moves_have_no_take()
{
    let inputs =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/inputs");
    for name in ["folding", "functions", "heap", "lists", "loops", "programs"]
    {
        let text =
            std::fs::read_to_string(inputs.join(format!("{name}.zaka"))).unwrap();
        let module = emit_text(&text);
        let takes = module.funcs.iter().flat_map(|f| &f.code);
        let found: Vec<&Op> = takes.filter(|op| matches!(op, Op::Take(_))).collect();
        assert!(found.is_empty(), "{name}: {found:?}");
    }
}

const FILLS: &str = "လုပ်ဆောင်ချက် ဖြည့် သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု \
                     ကို ပြန်ပေးမည်။\nကိန်းရှင် အဖြေ သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ \
                     တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။\nအဖြေ ၏ အကွက် ၀ ကို ၇ ဟု ထားမည်။\nအဖြေ ကို \
                     ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။\nလုပ်ဆောင်ချက် ဖြည့် ကို လုပ်ဆောင်သော \
                     ရလဒ် ကို ပြသမည်။";

#[test]
fn a_list_result_goes_to_a_slot_under_the_arguments()
{
    let module = emit_mode(FILLS, ResultMode::Duplicate);
    let main = ops(main_func(&module));
    assert_eq!(main[..2], ["local_addr l0", "call f0"]);
    assert_eq!(main_func(&module).locals[0].name, "%ret");
}

#[test]
fn a_returned_variable_lives_in_the_callers_slot_just_in_place()
{
    let built = emit_mode(FILLS, ResultMode::InPlace);
    assert!(holds_run(&ops(&built.funcs[0]), &["result_addr", "zero isize[2]"]));
    assert_eq!(ops(&built.funcs[0]).last().map(String::as_str), Some("ret"));
    let duplicated = emit_mode(FILLS, ResultMode::Duplicate);
    assert!(!duplicated.funcs.iter().any(|f| f.code.contains(&Op::ResultAddr)));
}

#[test]
fn a_returned_owner_struct_is_not_moved_out_first()
{
    let text = "ဘူး ဟူသော ဒေတာအစုအဝေးတွင် ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ပါဝင်သည်။\nလုပ်ဆောင်ချက် ဖန်တီး \
                သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ ဘူး တစ်ခု ကို ပြန်ပေးမည်။\nကိန်းရှင် ခ သည် ဘူး တစ်ခု ဖြစ်၍ \
                တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။\nခ ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    for results in [ResultMode::Duplicate, ResultMode::InPlace]
    {
        let module = emit_mode(text, results);
        let code = &module.funcs[0].code;
        assert!(!code.iter().any(|op| matches!(op, Op::Blit(_))), "{results}");
        assert_eq!(code.last(), Some(&Op::Ret), "{results}");
    }
}

/// The names of the scratch slots of the main function.
fn scratch(text: &str) -> Vec<String>
{
    let module = emit_text(text);
    let locals = &main_func(&module).locals;
    locals.iter().map(|s| s.name.clone()).filter(|n| n.starts_with('%')).collect()
}

#[test]
fn a_literal_set_into_a_slot_is_built_there()
{
    let decl =
        "ကိန်းရှင် l သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။\n";
    assert_eq!(scratch(&in_start(decl)), Vec::<String>::new());
    let shown = format!("{decl}[၄၊ ၅] ကို ပြသမည်။");
    assert_eq!(scratch(&in_start(&shown)), ["%list"]);
}

#[test]
fn a_literal_that_reads_its_own_slot_is_built_apart()
{
    let swap = "ကိန်းရှင် l သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
l ကို [(l ၏ အကွက် ၁)၊ (l ၏ အကွက် ၀)] ဟု ထားမည်။";
    assert_eq!(scratch(swap), ["%list"]);
}

#[test]
fn shared_scratch_slots_serve_each_later_statement()
{
    let text = "ကိန်းရှင် l သည် အကွက် ၅ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၀၊ ၂၀၊ ၃၀၊ ၄၀၊ \
                ၅၀] ဖြစ်ပါစေ။
l ၏ အကွက် ၁ မှစ၍ ၃ ကွက် ကို ပြသမည်။
l ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ကို ပြသမည်။
[၁၊ ၂] ကို ပြသမည်။
[၃၊ ၄] ကို ပြသမည်။";
    let names = |scratch| {
        let options = EmitOptions::builder().scratch(scratch).build();
        let module = emit_mode(text, options);
        let locals = &main_func(&module).locals;
        let names: Vec<String> = locals.iter().map(|s| s.name.clone()).collect();
        names
    };
    // A range is a list of its own: a view, then the list the print drops.
    let apart = ["%list", "%slice", "%list", "%slice", "%list", "%list"];
    assert_eq!(names(ScratchMode::Apart), apart);
    assert_eq!(names(ScratchMode::Shared), ["%list", "%slice", "%list"]);
}

#[test]
fn shared_scratch_slots_remain_apart_within_a_statement()
{
    let text = "လုပ်ဆောင်ချက် ပြ သည်
a ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လည်းကောင်း b ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
a ကို ပြသမည်။
b ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ကိန်းရှင် l သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို l ၏ အကွက် ၀ မှစ၍ ၁ ကွက်၊ l ၏ အကွက် ၁ မှစ၍ ၁ ကွက် တို့ဖြင့် လုပ်ဆောင်မည်။";
    let options = EmitOptions::builder().scratch(ScratchMode::Shared).build();
    let module = emit_mode(text, options);
    let locals = &main_func(&module).locals;
    let names: Vec<&str> = locals.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["%slice", "%slice"]);
}

/// A plain list literal of known cells points at a static list, inner lists
/// first; a literal with a cell known when the program runs moves its cells
/// to the heap.
#[test]
fn a_literal_of_known_cells_points_at_a_static_list()
{
    let text = "ကိန်းရှင် l သည် [[၁၊ ၂]၊ [၃]] ဖြစ်ပါစေ။\nl ကို ပြသမည်။";
    let module = emit_text(&in_start(text));
    let kinds: Vec<String> = module.seqs.iter().map(|s| s.kind.name()).collect();
    assert_eq!(kinds, ["isize[]", "isize[]", "isize[][]"]);
    let main = main_func(&module);
    let set = ["static_seq s2".to_string(), "pop".to_string()];
    assert!(ops(main).windows(2).any(|w| w == set), "{:?}", ops(main));
    assert!(
        main.locals.iter().all(|l| l.name != "%own"),
        "a set needs no slot of its own"
    );
    let built =
        main_ops(&in_start("ကိန်းရှင် n သည် ၁ ဖြစ်ပါစေ။\nကိန်းရှင် l သည် [n၊ ၂] ဖြစ်ပါစေ။"));
    assert!(built.iter().any(|o| o.starts_with("to_heap")), "{built:?}");
    assert!(!built.iter().any(|o| o.starts_with("static_seq")), "{built:?}");
}

/// A counted list literal of known cells fills its slot from a static list
/// with one blit; a literal with a cell known when the program runs stores
/// each cell.
#[test]
fn a_counted_or_struct_literal_of_known_cells_is_one_blit()
{
    let module = emit_text(&in_start(
        "ကိန်းရှင် ခ သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၇၊ ၈] ဖြစ်ပါစေ။",
    ));
    let kinds: Vec<String> = module.seqs.iter().map(|s| s.kind.name()).collect();
    assert_eq!(kinds, ["isize[2]"]);
    let found = ops(main_func(&module));
    let blit = ["seq_addr s0".to_string(), "blit isize[2]".to_string()];
    assert!(found.windows(2).any(|w| w == blit), "{found:?}");
    let point =
        "နေရာ ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု၊ y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။\n";
    let decl = "ကိန်းရှင် p သည် နေရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် x ၁၊ y ၂ ရှိသော နေရာတစ်ခု ဖြစ်ပါစေ။";
    let fields = main_ops(&format!("{point}\n{}", in_start(decl)));
    let blit = ["seq_addr s0".to_string(), "blit နေရာ".to_string()];
    assert!(fields.windows(2).any(|w| w == blit), "{fields:?}");
    let built = main_ops(&in_start(
        "ကိန်းရှင် n သည် ၁ ဖြစ်ပါစေ။\nကိန်းရှင် ခ သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \
         [n၊ ၈] ဖြစ်ပါစေ။",
    ));
    assert!(!built.iter().any(|o| o.starts_with("seq_addr")), "{built:?}");
}

/// A loop that keeps its cell number below the count of a plain list reads
/// each cell with no check; a cell removed in the loop changes the count, so
/// the check remains.
#[test]
fn a_cell_of_a_plain_list_proved_in_range_has_no_check()
{
    let head = "လုပ်ဆောင်ချက် ပေါင်း သည်
ကိန်းများ ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းရှင် န သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ န သည် ကိန်းများ ၏ အကွက်အရေအတွက် အောက် ငယ်နေသရွေ့ \
                အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n";
    let tail = "အဖြေ ကို အဖြေ + (ကိန်းများ ၏ အကွက် န) ဟု ထားမည်။
ပြီးလျှင် န ကို န + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";
    let checks = |text: String| {
        let module = emit_text(&text);
        ops(&module.funcs[0]).iter().filter(|op| op.starts_with("slice_cell")).count()
    };
    assert_eq!(checks(format!("{head}{tail}")), 0);
    let shrinks = "ကိန်းများ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။\n";
    assert_eq!(checks(format!("{head}{shrinks}{tail}")), 1);
}
