mod common;

use common::plain;
use common::run;
use zakac_ast::s_expr::ToSexpr;
use zakac_ir_builder::ir::FoldOptions;
use zakac_ir_builder::ir::Module;

/// `p` an owner of an integer.
const OWNER: &str = "ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။\n";

/// The owner prelude, then `rest`.
fn owner(rest: &str) -> String { format!("{OWNER}{rest}") }

/// The module of `text`, before folding.
fn module(text: &str) -> Module
{
    let lowered = run(text, &FoldOptions::default());
    lowered.plain.unwrap_or_else(|| panic!("{text}: {:?}", lowered.diagnostics))
}

#[test]
fn an_owner_given_to_an_owner_is_taken()
{
    let text = owner(
        "ကိန်းရှင် q သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် p ဖြစ်ပါစေ။
q ကို p ဟု ထားပါ။
q ကို ဗလာ ဟု ထားပါ။",
    );
    let forms = plain(&text);
    assert_eq!(forms[1], "(set g1 (take (addr g0)))");
    assert_eq!(forms[2], "(set g1 (take (addr g0)))");
    assert!(!forms[3].contains("take"), "{}", forms[3]);
    assert!(!module(&text).globals[0].addressed);
}

#[test]
fn a_lend_reads_its_owner()
{
    let text =
        owner("ကိန်းရှင် q သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် p ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။");
    assert_eq!(plain(&text)[1], "(set g1 (lend g0))");
}

#[test]
fn reads_that_do_not_move_remain_reads()
{
    let text = owner(
        "p ကို ပြသပါ။
p + ၁ ၏ တန်ဖိုး ကို ပြသပါ။
p == ဗလာ ကို ပြသပါ။",
    );
    let forms = plain(&text);
    assert_eq!(forms[1], "(print g0)");
    assert_eq!(forms[2], "(print (load (ptr+ g0 1:isize)))");
    assert!(forms[3].starts_with("(print (== g0 "), "{}", forms[3]);
    assert!(!forms[3].contains("take"), "{}", forms[3]);
}

#[test]
fn a_borrow_variable_is_not_taken()
{
    let text = owner(
        "ကိန်းရှင် q သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် p ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
ကိန်းရှင် r သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် q ဖြစ်ပါစေ။",
    );
    assert_eq!(plain(&text)[2], "(set g2 g1)");
}

#[test]
fn an_owner_moves_out_of_a_slot_in_mem()
{
    let text = owner(
        "ကိန်းရှင် pp သည် ငှားထားသော ကိန်းပြည့် ၏ လိပ်စာ ၏ လိပ်စာ p ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် r သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် pp ၏ တန်ဖိုး ဖြစ်ပါစေ။
pp ၏ တန်ဖိုး ၏ တန်ဖိုးအငှား ကို ပြသပါ။
pp ၏ တန်ဖိုး ကို r ဟု ထားပါ။",
    );
    let forms = plain(&text);
    assert_eq!(forms[1], "(set g1 (addr g0))");
    assert_eq!(forms[2], "(set g2 (take g1))");
    assert_eq!(forms[3], "(print (lend (load g1)))");
    assert_eq!(forms[4], "(store g1 (take (addr g2)))");
}

#[test]
fn arguments_and_results_move_owners()
{
    let text = owner(
        "လုပ်ဆောင်ချက် ယူ ကို p တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ငှား ကို p ၏ တန်ဖိုးအငှား တို့ဖြင့် လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် ယူ သည် ကိန်းပြည့် လိပ်စာ o ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
o ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် ငှား သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ o ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
o ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်ချက် ပြန် သည် ကိန်းပြည့် လိပ်စာ o ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ ရလဒ် ကို ပြန်ပေးမည်။
o ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။",
    );
    let forms = plain(&text);
    assert_eq!(forms[1], "(call ယူ (take (addr g0)))");
    assert_eq!(forms[2], "(call ငှား (lend g0))");
    let back = module(&text).funcs[2].to_sexpr();
    assert!(back.contains("(return (take (addr l0)))"), "{back}");
}

#[test]
fn a_list_of_owners_moves_as_a_whole()
{
    let text = "ကိန်းရှင် l သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် m သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း ဖြစ်၍ တန်ဖိုးသည် l ဖြစ်ပါစေ။
ကိန်းရှင် c သည် ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် m ၏ အကွက် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် n သည် ကိန်းပြည့် လိပ်စာ စာရင်း ဖြစ်၍ တန်ဖိုးသည် c၊ ဗလာ တို့ပါသော စာရင်း ဖြစ်ပါစေ။";
    let forms = plain(text);
    assert_eq!(forms[1], "(set g1 (take (addr g0)))");
    assert_eq!(forms[2], "(set g2 (take (cell@ g1 0:usize 2)))");
    assert!(forms[3].starts_with("(set g3 (list (take (addr g2)) 0x0"), "{}", forms[3]);
}

#[test]
fn an_owner_compared_with_its_borrow_needs_no_conversion()
{
    let text = owner(
        "ကိန်းရှင် q သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
p == q ကို ပြသပါ။
q == p ကို ပြသပါ။",
    );
    let forms = plain(&text);
    assert_eq!(forms[2], "(print (== g0 g1))");
    assert_eq!(forms[3], "(print (== g1 g0))");
}
