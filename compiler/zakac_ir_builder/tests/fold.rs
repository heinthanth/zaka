mod common;

use common::codes;
use common::codes_with;
use common::folded;
use common::folded_with;
use common::run;
use zakac_ast::s_expr::ToSexpr;
use zakac_diagnostics::Code;
use zakac_ir_builder::ir::FoldOptions;

fn only(exprs: bool, consts: bool, branches: bool) -> FoldOptions
{
    FoldOptions::builder().exprs(exprs).consts(consts).branches(branches).build()
}

#[test]
fn literal_arithmetic_folds()
{
    assert_eq!(folded("(၂ + ၃) * ၄ ကို ပြသပါ။"), ["(print 20:isize)"]);
    assert_eq!(folded("၁ / ၃ ကို ပြသပါ။"), ["(print 0:isize)"]);
    assert_eq!(folded("၁.၀ / ၀.၀ ကို ပြသပါ။"), ["(print inf:f64)"]);
    assert_eq!(folded("၁ < ၂ ကို ပြသပါ။"), ["(print true:bool)"]);
    assert_eq!(folded("၁ << ၄ ကို ပြသပါ။"), ["(print 16:isize)"]);
    assert_eq!(folded("-(၅) ကို ပြသပါ။"), ["(print -5:isize)"]);
    assert_eq!(
        folded("'က' ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။"),
        ["(print 4096:isize)"]
    );
    let text = "ကိန်းရှင် က သည် ၁၆ ဘစ် ဒဿမကိန်း ၁ / ၃ ဖြစ်ပါစေ။";
    assert_eq!(folded(text), ["(set g0 0.33325:f16)"]);
    let text = "ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။\nကိန်းရှင် ခ သည် ဒဿမကိန်း က + ၁ ဖြစ်ပါစေ။";
    assert_eq!(folded(text)[1], "(set g1 (widen f64 (+ g0 1:i8)))");
}

#[test]
fn turning_exprs_off_keeps_the_tree()
{
    let off = only(false, true, true);
    let forms = folded_with("(၂ + ၃) * ၄ ကို ပြသပါ။", &off);
    assert_eq!(forms, ["(print (* (+ 2:isize 3:isize) 4:isize))"]);
}

#[test]
fn errors_are_reported_whatever_the_options()
{
    let text = "ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် (၁၂၇ + ၁) * ၂ ဖြစ်ပါစေ။";
    assert_eq!(codes(text), [Code::ConstantOverflow]);
    assert_eq!(codes_with(text, &FoldOptions::none()), [Code::ConstantOverflow]);
    let text = "ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် (၁၀၀ + ၁) * ၂ ဖြစ်ပါစေ။";
    assert_eq!(codes_with(text, &FoldOptions::none()), [Code::ConstantOverflow]);
    assert_eq!(codes("၁ / ၀ ကို ပြသပါ။"), [Code::ConstantZeroDivisor]);
    assert_eq!(codes("၇ % (၂ - ၂) ကို ပြသပါ။"), [Code::ConstantZeroDivisor]);
    let text = "ကိန်းသေ က သည် ၈ ဘစ် ကိန်းပြည့် -၁၂၈ ဖြစ်ပါစေ။\n-က ကို ပြသပါ။";
    assert_eq!(codes(text), [Code::ConstantOverflow]);
    let lowered =
        run("ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် ၁၂၇ + ၁ ဖြစ်ပါစေ။", &FoldOptions::default());
    assert!(lowered.diagnostics[0].message.contains("i8"), "{:?}", lowered.diagnostics);
    assert!(lowered.folded.is_some());
}

#[test]
fn constants_propagate()
{
    let text = "ကိန်းသေ ဆ သည် ကိန်းပြည့် ၁၀ ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ကိန်းပြည့် ၁၀ ဖြစ်ပါစေ။
ဆ * ၂ ကို ပြသပါ။
က * ၂ ကို ပြသပါ။";
    assert_eq!(folded(text)[2..], ["(print 20:isize)", "(print (* g1 2:isize))"]);
    let off = folded_with(text, &only(true, false, true));
    assert_eq!(off[2], "(print (* g0 2:isize))");
}

#[test]
fn constants_reach_functions_and_chains()
{
    let text = "ကိန်းသေ က သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။
ကိန်းသေ ခ သည် ကိန်းပြည့် က * က ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ပြ သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ခ + ၁ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let lowered = run(text, &FoldOptions::default());
    let module = lowered.folded.expect("folded");
    assert_eq!(common::main_forms(&module)[1], "(set g1 9:isize)");
    let func = module.funcs[0].to_sexpr();
    assert!(func.contains("(print 10:isize)"), "{func}");
}

#[test]
fn propagated_shift_distances_are_checked()
{
    let text = "ကိန်းသေ န သည် လက္ခဏာမဲ့ ကိန်းပြည့် ၇၀ ဖြစ်ပါစေ။
ကိန်းရှင် က သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
က << န ကို ပြသပါ။";
    assert_eq!(codes(text), [Code::ShiftTooFar]);
    assert!(codes_with(text, &only(true, false, true)).is_empty());
}

#[test]
fn constant_branches_are_spliced()
{
    let text = "အကယ်၍ ၁ သည် ၂ အောက် ငယ်လျှင် \"a\" ဟု ပြသပါ။
သို့မဟုတ်ပါက \"b\" ဟု ပြသပါ။
အကယ်၍ ၁ သည် ၂ ထက် ကြီးလျှင် \"c\" ဟု ပြသပါ။";
    assert_eq!(folded(text), ["(print \"a\")"]);
    let kept = folded_with(text, &only(true, true, false));
    assert_eq!(kept[0], "(if true:bool (block (print \"a\")) (block (print \"b\")))");
    assert_eq!(kept.len(), 2);
}

#[test]
fn a_dead_loop_keeps_its_start()
{
    let text = "၁ သည် ၂ ထက် ကြီးနေသရွေ့ \"x\" ဟု ပြသပါ။
ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ၀ ဖြစ်ပြီး ၁ သည် ၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်း ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    assert_eq!(folded(text), ["(set g0 0:isize)"]);
}

#[test]
fn logic_short_circuits_on_a_constant_left_side()
{
    let head = "ကိန်းရှင် က သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။\n";
    let cases = [
        ("(၁ == ၁ && က == ၁)", "(print (== g0 1:isize))"),
        ("(၁ == ၂ && က == ၁)", "(print false:bool)"),
        ("(၁ == ၁ || က == ၁)", "(print true:bool)"),
        ("(၁ == ၂ || က == ၁)", "(print (== g0 1:isize))"),
        ("(က == ၁ && ၁ == ၁)", "(print (and (== g0 1:isize) true:bool))"),
    ];
    for (expr, expected) in cases
    {
        assert_eq!(folded(&format!("{head}{expr} ကို ပြသပါ။"))[1], expected, "{expr}");
    }
}

#[test]
fn a_cell_of_a_list_with_no_cells_is_out_of_range_without_a_panic()
{
    use zakac_ir_builder::ir::ExprNode;
    use zakac_ir_builder::ir::Stmt;

    let text = "ကိန်းရှင် l သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။\nl ၏ အကွက် ၁ ကို ၉ ဟု ထားပါ။";
    let mut module = run(text, &FoldOptions::default()).plain.expect("lowered");
    let Some(Stmt::Store { addr, .. }) = module.main.last_mut()
    else
    {
        panic!("the last statement is a store");
    };
    let ExprNode::CellAddr { len, .. } = &mut addr.node
    else
    {
        panic!("the store writes a cell");
    };
    *len = Some(0);
    let found = zakac_ir_builder::fold::fold(&mut module, &FoldOptions::default(), 64);
    let codes: Vec<Code> = found.iter().map(|d| d.code).collect();
    assert_eq!(codes, [Code::CellOutOfRange]);
}
