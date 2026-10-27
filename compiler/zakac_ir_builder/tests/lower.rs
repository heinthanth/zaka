mod common;

use common::folded;
use common::plain;
use common::run;
use zakac_ast::s_expr::ToSexpr;
use zakac_ir_builder::ir::Access;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::FoldOptions;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::PrintValue;
use zakac_ir_builder::ir::Stmt;
use zakac_semcheck::kind::DataKind;

fn module(text: &str) -> Module
{
    let lowered = run(text, &FoldOptions::default());
    lowered.plain.unwrap_or_else(|| panic!("{text}: {:?}", lowered.diagnostics))
}

fn printed(s: &Stmt) -> &Expr
{
    let Stmt::Print { value: PrintValue::Expr(e), .. } = s
    else
    {
        panic!("not a printed expression: {s:?}");
    };
    e
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

#[test]
fn pointers_lower_to_mem_nodes()
{
    let text = "ကိန်းရှင် x သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ x ၏ လိပ်စာ ဖြစ်ပါစေ။
p ၏ တန်ဖိုး ကို ၆ ဟု ထားပါ။
*p + ၁ ကို ပြသပါ။
p + ၂ ၏ တန်ဖိုး ကို ပြသပါ။
(p - p) ကို ပြသပါ။
p ကို ဗလာ ဟု ထားပါ။";
    let expected = [
        "(set g0 5:isize)",
        "(set g1 (addr g0))",
        "(store g1 6:isize)",
        "(print (+ (load g1) 1:isize))",
        "(print (load (ptr+ g1 2:isize)))",
        "(print (ptr- g1 g1))",
        "(set g1 0x0:ptr)",
    ];
    assert_eq!(plain(text), expected);
    let m = module(text);
    assert!(m.globals[0].addressed);
    assert!(!m.globals[1].addressed);
}

#[test]
fn heap_statements_lower_with_their_unit()
{
    let text = "ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁၀ ခုစာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။
p လိပ်စာရှိကို မင်မိုရီ ၆၄ ဘိုက်စာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။
p လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ကိန်းပြည့် ၏ အရွယ်အစား ကို ပြသပါ။";
    let found = plain(text);
    assert_eq!(found[1], "(alloc g0 10:usize isize)");
    assert_eq!(found[2], "(resize g0 64:usize)");
    assert_eq!(found[3], "(free g0)");
    assert_eq!(found[4], "(print (sizeof isize))");
    assert_eq!(folded(text)[4], "(print 8:usize)");
}

#[test]
fn a_deref_destination_lowers_to_its_address()
{
    let text = "ကိန်းရှင် pp သည် ကိန်းပြည့် ၏ လိပ်စာ ၏ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ၈ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို pp ၏ တန်ဖိုး တွင် သိမ်းထားမည်။
pp ၏ တန်ဖိုး လိပ်စာရှိမင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။";
    assert_eq!(plain(text)[1..], ["(alloc (mem g0) 8:usize)", "(free (mem g0))"]);
}

#[test]
fn a_bare_null_is_an_opaque_pointer()
{
    let text = "ဗလာ ကို ပြသပါ။\nဗလာ == ဗလာ ကို ပြသပါ။";
    let m = module(text);
    assert_eq!(printed(&m.main[0]).kind, DataKind::OPAQUE);
    let ExprNode::Compare { lhs, rhs, .. } = &printed(&m.main[1]).node
    else
    {
        panic!("not a comparison: {:?}", m.main[1]);
    };
    assert_eq!((lhs.kind, rhs.kind), (DataKind::OPAQUE, DataKind::OPAQUE));
    let folded = run(text, &FoldOptions::default()).folded.expect("folded");
    assert_eq!(printed(&folded.main[0]).kind, DataKind::OPAQUE);
    let none = run(text, &FoldOptions::none()).folded.expect("folded");
    assert_eq!(printed(&none.main[0]).kind, DataKind::OPAQUE);
}

const OFFSETS: &str = "ကိန်းရှင် x သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။
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

#[test]
fn an_offset_to_an_opaque_place_keeps_its_kind_under_a_widen()
{
    let found = plain(OFFSETS);
    let expected = [
        "(set g2 (widen ptr (ptr+ g1 1:isize)))",
        "(set g2 (widen ptr (ptr+ g1 1:isize)))",
        "(print (== (widen *isize g2) (ptr+ g1 1:isize)))",
        "(call ယူ (widen ptr (ptr+ g1 1:isize)))",
    ];
    assert_eq!(found[2..], expected);
    let m = module(OFFSETS);
    let Stmt::Set { value, .. } = &m.main[2]
    else
    {
        panic!("not a store: {:?}", m.main[2]);
    };
    let ExprNode::Convert { operand, implicit: true } = &value.node
    else
    {
        panic!("not an implicit conversion: {value:?}");
    };
    assert_eq!(value.kind, DataKind::OPAQUE);
    assert!(operand.kind.is_ptr() && operand.kind != DataKind::OPAQUE);
    let ret = m.funcs[1].to_sexpr();
    assert!(ret.contains("(return (widen ptr (ptr+ l0 1:isize)))"), "{ret}");
}

#[test]
fn a_pointer_default_takes_the_declared_kind()
{
    let text = "ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဗလာပေး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ၏ လိပ်စာ ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let m = module(text);
    let to_isize = DataKind::parse("*isize").expect("a pointer kind");
    let Stmt::Set { value, .. } = &m.main[0]
    else
    {
        panic!("not a store: {:?}", m.main[0]);
    };
    assert_eq!(value.kind, to_isize);
    let Stmt::Set { value, .. } = &m.funcs[0].block[0]
    else
    {
        panic!("not a store: {:?}", m.funcs[0].block[0]);
    };
    assert_eq!(value.kind, to_isize);
}
