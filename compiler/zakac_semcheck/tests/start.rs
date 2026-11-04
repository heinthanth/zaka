mod common;

use common::codes;
use common::codes_as_is;
use common::ok;
use zakac_diagnostics::Code;
use zakac_rust_sugars::tern;

/// A start block with `params` after its header word and `ret` as its result
/// clause, or no result when `ret` is blank.
fn start(params: &str, ret: &str) -> String
{
    let open = tern!(ret.is_empty(), "အောက်ပါတို့ကို လုပ်ဆောင်မည်။", ret);
    format!("ပရိုဂရမ်သည် {params} {open}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

const TWO: &str = "စာသား စာရင်း က နှင့် စာသား စာရင်း ခ တို့ကို လက်ခံပြီး";
const INT_RET: &str = "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။";

#[test]
fn a_start_block_takes_up_to_two_text_lists_and_an_integer_result()
{
    ok(&start("", ""));
    ok(&start("စာသား စာရင်း က ကို လက်ခံပြီး", ""));
    ok(&start(TWO, INT_RET));
    let small = "အောက်ပါတို့ကို လုပ်ဆောင်ကာ လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။";
    ok(&start("", small));
}

#[test]
fn declarations_and_functions_stand_beside_a_start_block()
{
    let text = format!(
        "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ က ကို ပြသပါ။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။
{}",
        start("", "")
    );
    ok(&text);
}

#[test]
fn a_statement_at_the_top_level_is_zk0326()
{
    let text = format!("၁ ကို ပြသပါ။\n{}", start("", ""));
    assert_eq!(codes(&text), [Code::TopLevelStmt]);
}

#[test]
fn a_program_with_no_start_block_is_zk0327()
{
    assert_eq!(codes_as_is("ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။"), [Code::NoStart]);
    assert_eq!(codes_as_is(""), [Code::NoStart]);
    assert_eq!(codes_as_is("၁ ကို ပြသပါ။"), [Code::TopLevelStmt, Code::NoStart]);
}

#[test]
fn a_second_start_block_is_zk0325()
{
    let text = format!("{}\n{}", start("", ""), start("", ""));
    assert_eq!(codes(&text), [Code::SecondStart]);
}

#[test]
fn a_start_parameter_that_is_no_text_list_is_zk0440()
{
    let int = start("ကိန်းပြည့် က ကို လက်ခံပြီး", "");
    assert_eq!(codes(&int), [Code::StartParamKind]);
    let text = start("စာသား က ကို လက်ခံပြီး", "");
    assert_eq!(codes(&text), [Code::StartParamKind]);
    let three = start("စာသား စာရင်း က၊ စာသား စာရင်း ခ နှင့် စာသား စာရင်း ဂ တို့ကို လက်ခံပြီး", "");
    assert_eq!(codes(&three), [Code::StartParamKind]);
    let edit = start("လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော စာသား စာရင်း က ကို လက်ခံပြီး", "");
    assert_eq!(codes(&edit), [Code::StartParamKind]);
}

#[test]
fn a_start_result_that_is_no_integer_is_zk0441()
{
    let text = start("", "အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရလဒ် ကို ပြန်ပေးမည်။");
    assert_eq!(codes(&text), [Code::StartResultKind]);
}

#[test]
fn no_call_names_the_start_block()
{
    let text = format!("{}\nလုပ်ဆောင်ချက် `ပရိုဂရမ်` ကို လုပ်ဆောင်ပါ။", start("", ""));
    assert_eq!(codes(&text), [Code::TopLevelStmt, Code::UndeclaredFunction]);
}
