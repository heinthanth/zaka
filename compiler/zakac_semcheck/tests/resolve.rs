mod common;

use common::codes;
use common::ok;
use zakac_diagnostics::Code;

/// A function without parameters or result.
const GREET: &str = "လုပ်ဆောင်ချက် နှုတ်ဆက် ဆိုသည်မှာ
ထိုလုပ်ဆောင်ချက်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
\"hi\" ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";

/// A function with one integer parameter and an integer result.
const TWICE: &str = "လုပ်ဆောင်ချက် နှစ်ဆ ဆိုသည်မှာ
ကိန်းပြည့် က ကို လက်ခံပြီး
ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးပြီး
ထိုလုပ်ဆောင်ချက်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က * ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

#[test]
fn declared_names_resolve()
{
    ok(&lines(&["ကိန်းရှင် က သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။", "က ကို ပြသပါ။"]));
}

#[test]
fn undeclared_names_are_reported()
{
    assert_eq!(codes("က ကို ပြသပါ။"), [Code::UndeclaredName]);
    assert_eq!(codes("က ကို ၁ ဟု ထားပါ။"), [Code::UndeclaredName]);
    assert_eq!(
        codes("(က + ၁) * က ကို ပြသပါ။"),
        [Code::UndeclaredName, Code::UndeclaredName]
    );
}

#[test]
fn an_initial_value_cannot_see_its_own_name()
{
    assert_eq!(codes("ကိန်းရှင် က သည် ကိန်းပြည့် က ဖြစ်ပါစေ။"), [Code::UndeclaredName]);
}

#[test]
fn a_name_is_declared_once_per_scope()
{
    let text = lines(&["ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။", "ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။"]);
    assert_eq!(codes(&text), [Code::Redeclaration]);
}

#[test]
fn inner_blocks_shadow_outer_names()
{
    ok(&lines(&[
        "ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။",
        "အောက်ပါတို့ကို လုပ်ဆောင်ပါ။",
        "ကိန်းရှင် က သည် စာလုံး 'ခ' ဖြစ်ပါစေ။",
        "က ကို ပြသပါ။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]));
}

#[test]
fn block_names_end_with_the_block()
{
    let text = lines(&[
        "အောက်ပါတို့ကို လုပ်ဆောင်ပါ။",
        "ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
        "က ကို ပြသပါ။",
    ]);
    assert_eq!(codes(&text), [Code::UndeclaredName]);
}

#[test]
fn functions_can_be_called_before_their_declaration()
{
    ok(&lines(&["လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်ပါ။", GREET]));
    ok(&lines(&["လုပ်ဆောင်ချက် နှစ်ဆ ကို ၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။", TWICE]));
}

#[test]
fn undeclared_functions_are_reported()
{
    assert_eq!(codes("လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်ပါ။"), [Code::UndeclaredFunction]);
}

#[test]
fn a_function_name_is_declared_once()
{
    assert_eq!(codes(&lines(&[GREET, GREET])), [Code::DuplicateFunction]);
}

#[test]
fn parameter_names_are_distinct()
{
    let text = lines(&[
        "လုပ်ဆောင်ချက် စုစု ဆိုသည်မှာ",
        "ကိန်းပြည့် က၊ ကိန်းပြည့် က တို့ကို လက်ခံပြီး",
        "ထိုလုပ်ဆောင်ချက်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&text), [Code::DuplicateParam]);
}

#[test]
fn parameters_are_visible_inside_the_function_alone()
{
    assert_eq!(codes(&lines(&[TWICE, "က ကို ပြသပါ။"])), [Code::UndeclaredName]);
}

#[test]
fn constants_cannot_be_assigned()
{
    let text = lines(&["ကိန်းသေ က သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။", "က ကို ၂ ဟု ထားပါ။"]);
    assert_eq!(codes(&text), [Code::AssignToConst]);
}

#[test]
fn constants_need_a_value()
{
    let text = lines(&["ကိန်းသေ က သည် ကိန်းပြည့် ဖြစ်ပါစေ။", "က ကို ပြသပါ။"]);
    assert_eq!(codes(&text), [Code::ConstWithoutInit]);
}

#[test]
fn counted_loop_variables_cannot_be_assigned()
{
    let text = "၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ၁ ဟု ထားပါ။";
    assert_eq!(codes(text), [Code::AssignToLoopVar]);
}

#[test]
fn loop_variables_end_with_the_loop()
{
    let text = lines(&["၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။", "ကိန်း ကို ပြသပါ။"]);
    assert_eq!(codes(&text), [Code::UndeclaredName]);
}

#[test]
fn generic_loop_update_sees_the_loop_variable()
{
    ok(&lines(&[
        "ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ၀ ဖြစ်ပြီး ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။",
        "ကိန်း ကို ပြသပါ။",
        "ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားပါ။",
        "လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီ။",
    ]));
}

#[test]
fn break_and_continue_need_a_loop()
{
    assert_eq!(codes("ကွင်းဆက် ကို ရပ်ပါ။"), [Code::BreakOutsideLoop]);
    assert_eq!(codes("နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။"), [Code::ContinueOutsideLoop]);
    ok(&lines(&[
        "ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ဖြစ်ပါစေ။",
        "ကိန်း သည် ၅ အောက် ငယ်နေသရွေ့ ကွင်းဆက် ကို ရပ်ပါ။",
        "၁ မှ ၁၀ အထိ ခ အနေဖြင့် ရေတွက်၍ နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။",
    ]));
}

#[test]
fn a_function_starts_outside_loops()
{
    let text = lines(&[
        "လုပ်ဆောင်ချက် စမ်း ဆိုသည်မှာ",
        "ထိုလုပ်ဆောင်ချက်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "ကွင်းဆက် ကို ရပ်ပါ။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&text), [Code::BreakOutsideLoop]);
}

#[test]
fn return_needs_a_function()
{
    assert_eq!(codes("ပြန်ပေးပါ။"), [Code::ReturnOutsideFunction]);
    assert_eq!(codes("၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။"), [Code::ReturnOutsideFunction]);
}

#[test]
fn return_values_match_the_header()
{
    let void_value = lines(&[
        "လုပ်ဆောင်ချက် နှုတ်ဆက် ဆိုသည်မှာ",
        "ထိုလုပ်ဆောင်ချက်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&void_value), [Code::ValueReturnedFromVoid]);
    let bare = lines(&[
        "လုပ်ဆောင်ချက် ဂဏန်း ဆိုသည်မှာ",
        "ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးပြီး",
        "ထိုလုပ်ဆောင်ချက်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "ပြန်ပေးပါ။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&bare), [Code::MissingReturnValue]);
}

#[test]
fn the_result_variable_exists_in_valued_functions()
{
    ok(&lines(&[
        "လုပ်ဆောင်ချက် ဂဏန်း ဆိုသည်မှာ",
        "ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးပြီး",
        "ထိုလုပ်ဆောင်ချက်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "ရလဒ် ကို ၁ ဟု ထားပါ။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]));
    let text = lines(&[
        "လုပ်ဆောင်ချက် ဂဏန်း ဆိုသည်မှာ",
        "ထိုလုပ်ဆောင်ချက်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "ရလဒ် ကို ၁ ဟု ထားပါ။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&text), [Code::UndeclaredName]);
}

#[test]
fn calls_pass_one_argument_per_parameter()
{
    let text = lines(&[TWICE, "လုပ်ဆောင်ချက် နှစ်ဆ ကို လုပ်ဆောင်ပါ။"]);
    assert_eq!(codes(&text), [Code::ArgCountMismatch]);
    let text = lines(&[TWICE, "လုပ်ဆောင်ချက် နှစ်ဆ ကို ၁၊ ၂ တို့ဖြင့် လုပ်ဆောင်ပါ။"]);
    assert_eq!(codes(&text), [Code::ArgCountMismatch]);
}

#[test]
fn functions_without_a_result_are_no_values()
{
    let text = lines(&[GREET, "လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။"]);
    assert_eq!(codes(&text), [Code::ValueFromVoidFunction]);
}
