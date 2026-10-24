mod common;

use common::plain;
use common::run;
use zakac_ast::s_expr::ToSexpr;
use zakac_ir_builder::ir::Access;
use zakac_ir_builder::ir::FoldOptions;
use zakac_ir_builder::ir::Module;

fn module(text: &str) -> Module
{
    let lowered = run(text, &FoldOptions::default());
    lowered.plain.unwrap_or_else(|| panic!("{text}: {:?}", lowered.diagnostics))
}

#[test]
fn declarations_store_values_and_defaults()
{
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၈ ဘစ် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ၃၂ ဘစ် ဒဿမကိန်း ဖြစ်ပါစေ။
ကိန်းရှင် ဃ သည် စာလုံး ဖြစ်ပါစေ။
ကိန်းရှင် င သည် မှန်မှားတန်ဖိုး ဖြစ်ပါစေ။
ကိန်းသေ စ သည် ဒဿမကိန်း ၂.၅ ဖြစ်ပါစေ။";
    let expected = [
        "(set g0 5:isize)",
        "(set g1 0:i8)",
        "(set g2 0.0:f32)",
        "(set g3 '\\0':char)",
        "(set g4 false:bool)",
        "(set g5 2.5:f64)",
    ];
    assert_eq!(plain(text), expected);
    let m = module(text);
    assert_eq!(m.globals[5].access, Access::Const);
    assert_eq!(m.globals[0].name, "က");
}

#[test]
fn negated_literals_are_constants()
{
    let text = "ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် -၁၂၈ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် အနုတ် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ၁၆ ဘစ် ဒဿမကိန်း -၃ ဖြစ်ပါစေ။
ကိန်းရှင် ဃ သည် ကိန်းပြည့် -(၅) ဖြစ်ပါစေ။
-က ကို ပြသပါ။";
    let expected = [
        "(set g0 -128:i8)",
        "(set g1 -5:isize)",
        "(set g2 -3.0:f16)",
        "(set g3 (neg 5:isize))",
        "(print (neg g0))",
    ];
    assert_eq!(plain(text), expected);
}

#[test]
fn widening_becomes_explicit()
{
    let text = "ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၁၆ ဘစ် ကိန်းပြည့် က ဖြစ်ပါစေ။
ခ ကို က + ခ ဟု ထားပါ။
က < ခ ကို ပြသပါ။
ကိန်းရှင် ဂ သည် ဒဿမကိန်း က ဖြစ်ပါစေ။
ကိန်းရှင် ဃ သည် ၃၂ ဘစ် ဒဿမကိန်း ၁ ဖြစ်ပါစေ။";
    let expected = [
        "(set g0 5:i8)",
        "(set g1 (widen i16 g0))",
        "(set g1 (+ (widen i16 g0) g1))",
        "(print (< (widen i16 g0) g1))",
        "(set g2 (widen f64 g0))",
        "(set g3 1.0:f32)",
    ];
    assert_eq!(plain(text), expected);
}

#[test]
fn verbs_and_notation_lower_alike()
{
    let head = "ကိန်းရှင် က သည် ကိန်းပြည့် ၆ ဖြစ်ပါစေ။\nကိန်းရှင် ခ သည် ကိန်းပြည့် ၄ ဖြစ်ပါစေ။\n";
    let pairs = [
        ("က နှင့် ခ ၏ ပေါင်းလဒ်", "က + ခ"),
        ("က ကို ခ ဖြင့် စားသော စားကြွင်း", "က % ခ"),
        ("က ကို ဘယ်သို့ ၂ ဘစ် ရွေ့သောရလဒ်", "က << ၂"),
        ("က ကို ညာသို့ ၂ ဘစ် ရွေ့သောရလဒ်", "က >> ၂"),
        ("(က သည် ခ ထက် ကြီးသည်)", "က > ခ"),
        ("(က သည် ခ အောက် မငယ်သည်)", "က >= ခ"),
        ("က ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်", "~က"),
    ];
    for (verbal, notation) in pairs
    {
        let a = plain(&format!("{head}{verbal} ကို ပြသပါ။"));
        let b = plain(&format!("{head}{notation} ကို ပြသပါ။"));
        assert_eq!(a, b, "{verbal}");
    }
    let shift = plain(&format!("{head}က << ၂ ကို ပြသပါ။"));
    assert_eq!(shift[2], "(print (<< g0 2:usize))");
}

#[test]
fn conversions_and_logic()
{
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် ၆ ဖြစ်ပါစေ။
က ကို ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။
(က > ၁ && က < ၉) ကို ပြသပါ။
(က == ၁ မဟုတ်) ကို ပြသပါ။
\"မင်္ဂလာပါ\" ဟု ပြသပါ။";
    let expected = [
        "(set g0 6:isize)",
        "(print (cast f64 g0))",
        "(print (and (> g0 1:isize) (< g0 9:isize)))",
        "(print (! (== g0 1:isize)))",
        "(print \"မင်္ဂလာပါ\")",
    ];
    assert_eq!(plain(text), expected);
}

#[test]
fn blocks_flatten_and_shadowing_gets_a_new_slot()
{
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် က သည် ကိန်းပြည့် ၂ ဖြစ်ပါစေ။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
က ကို ပြသပါ။";
    let expected = ["(set g0 1:isize)", "(set g1 2:isize)", "(print g1)", "(print g0)"];
    assert_eq!(plain(text), expected);
}

#[test]
fn branches_nest_in_the_else_list()
{
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
အကယ်၍ က သည် ၀ အောက် ငယ်လျှင် \"a\" ဟု ပြသပါ။
ထိုသို့မဟုတ်ဘဲ က သည် ၀ နှင့် ညီလျှင် \"b\" ဟု ပြသပါ။
သို့မဟုတ်ပါက \"c\" ဟု ပြသပါ။";
    let expected = "(if (< g0 0:isize) (block (print \"a\")) (block (if (== g0 \
                    0:isize) (block (print \"b\")) (block (print \"c\")))))";
    assert_eq!(plain(text)[1], expected);
}

#[test]
fn loops()
{
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။
က သည် ၀ ထက် ကြီးနေသရွေ့ က ကို က - ၁ ဟု ထားပါ။
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ၀ ဖြစ်ပြီး ကိန်း သည် ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်း ကို ပြသပါ။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
၁ မှ ၁၀ အထိ ၂ စီ န အနေဖြင့် တိုး၍ န ကို ပြသပါ။
ကွင်းဆက် ကို ရပ်ပါ။";
    let forms = plain(&text.replace("ကွင်းဆက် ကို ရပ်ပါ။", ""));
    let expected = [
        "(set g0 5:isize)",
        "(loop (> g0 0:isize) (block (set g0 (- g0 1:isize))))",
        "(loop (init (set g1 0:isize)) (< g1 3:isize) (update (set g1 (+ g1 \
         1:isize))) (block (print g1)))",
        "(up g2 1:isize 10:isize 2:isize (block (print g2)))",
    ];
    assert_eq!(forms, expected);
    assert_eq!(
        module(&text.replace("ကွင်းဆက် ကို ရပ်ပါ။", "")).globals[2].access,
        Access::LoopVar
    );
}

#[test]
fn counted_loops_take_the_common_kind_of_their_bounds()
{
    let text = "ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၁၆ ဘစ် ကိန်းပြည့် ၉ ဖြစ်ပါစေ။
က မှ ခ အထိ န အနေဖြင့် ရေတွက်၍ န ကို ပြသပါ။";
    let expected = "(count g2 (widen i16 g0) g1 (block (print g2)))";
    assert_eq!(plain(text)[2], expected);
}

#[test]
fn functions_have_their_own_slots()
{
    let text = "ကိန်းရှင် ဆ သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ ကို ၂ တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ပြ သည် ကိန်းပြည့် က ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ခ သည် ကိန်းပြည့် က * ဆ ဖြစ်ပါစေ။
ခ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် နှစ်ဆ သည် ကိန်းပြည့် က ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ရလဒ် ကို က * ၂ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် နှစ်ဆ ကို ၄ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။";
    let m = module(text);
    let funcs: Vec<String> = m.funcs.iter().map(ToSexpr::to_sexpr).collect();
    let expected = [
        "(fn ပြ (params (l0 က isize)) (result -) (locals (l1 ခ isize)) (block (set l1 \
         (* l0 g0)) (print l1)))",
        "(fn နှစ်ဆ (params (l0 က isize)) (result l1) (locals) (block (set l1 0:isize) \
         (set l1 (* l0 2:isize)) (return l1)))",
    ];
    assert_eq!(funcs, expected);
    let main = common::main_forms(&m);
    assert_eq!(
        main,
        ["(set g0 3:isize)", "(call ပြ 2:isize)", "(print (call နှစ်ဆ 4:isize))"]
    );
}

#[test]
fn an_explicit_return_is_not_doubled()
{
    let text = "လုပ်ဆောင်ချက် ခုနစ် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
၇ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let expected = "(fn ခုနစ် (params) (result l0) (locals) (block (set l0 0:isize) \
                    (return 7:isize)))";
    assert_eq!(module(text).funcs[0].to_sexpr(), expected);
}

#[test]
fn nothing_is_lowered_after_a_check_error()
{
    let lowered = run("က ကို ပြသပါ။", &FoldOptions::default());
    assert!(lowered.plain.is_none() && lowered.folded.is_none());
    assert!(!lowered.diagnostics.is_empty());
}
