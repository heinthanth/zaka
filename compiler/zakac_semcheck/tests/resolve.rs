mod common;

use common::codes;
use common::codes_as_is;
use common::ok;
use zakac_diagnostics::Code;

/// A function without parameters or result.
const GREET: &str = "လုပ်ဆောင်ချက် နှုတ်ဆက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
\"hi\" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";

/// A function with one integer parameter and an integer result.
const TWICE: &str = "လုပ်ဆောင်ချက် နှစ်ဆ သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

#[test]
fn declared_names_resolve()
{
    ok(&lines(&["ကိန်းရှင် က သည် ၁ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။", "က ကို ပြသမည်။"]));
}

#[test]
fn undeclared_names_are_reported()
{
    assert_eq!(codes("က ကို ပြသမည်။"), [Code::UndeclaredName]);
    assert_eq!(codes("က ကို ၁ ဟု ထားမည်။"), [Code::UndeclaredName]);
    assert_eq!(
        codes("(က + ၁) * က ကို ပြသမည်။"),
        [Code::UndeclaredName, Code::UndeclaredName]
    );
}

#[test]
fn an_initial_value_cannot_see_its_own_name()
{
    assert_eq!(
        codes("ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။"),
        [Code::UndeclaredName]
    );
}

#[test]
fn a_name_is_declared_once_per_scope()
{
    let text =
        lines(&["ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။", "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။"]);
    assert_eq!(codes(&text), [Code::Redeclaration]);
}

#[test]
fn inner_blocks_shadow_outer_names()
{
    ok(&lines(&[
        "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။",
        "အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "ကိန်းရှင် က သည် စာလုံး တစ်လုံး ဖြစ်၍ တန်ဖိုးသည် 'ခ' ဖြစ်ပါစေ။",
        "က ကို ပြသမည်။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]));
}

#[test]
fn block_names_end_with_the_block()
{
    let text = lines(&[
        "အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
        "က ကို ပြသမည်။",
    ]);
    assert_eq!(codes(&text), [Code::UndeclaredName]);
}

#[test]
fn functions_can_be_called_before_their_declaration()
{
    ok(&lines(&["လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်မည်။", GREET]));
    ok(&lines(&["လုပ်ဆောင်ချက် နှစ်ဆ ကို ၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။", TWICE]));
}

#[test]
fn undeclared_functions_are_reported()
{
    assert_eq!(codes("လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်မည်။"), [Code::UndeclaredFunction]);
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
        "လုပ်ဆောင်ချက် စုစု သည်",
        "က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး",
        "အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&text), [Code::DuplicateParam]);
}

#[test]
fn parameters_are_visible_inside_the_function_alone()
{
    assert_eq!(codes(&lines(&[TWICE, "က ကို ပြသမည်။"])), [Code::UndeclaredName]);
}

#[test]
fn constants_cannot_be_assigned()
{
    let text = lines(&["ကိန်းသေ က သည် ၁ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။", "က ကို ၂ ဟု ထားမည်။"]);
    assert_eq!(codes(&text), [Code::AssignToConst]);
}

#[test]
fn constants_need_a_value()
{
    let text = lines(&["ကိန်းသေ က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။", "က ကို ပြသမည်။"]);
    assert_eq!(codes(&text), [Code::ConstWithoutInit]);
}

#[test]
fn counted_loop_variables_cannot_be_assigned()
{
    let text = "၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ၁ ဟု ထားမည်။";
    assert_eq!(codes(text), [Code::AssignToLoopVar]);
}

#[test]
fn loop_variables_end_with_the_loop()
{
    let text = lines(&["၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။", "ကိန်း ကို ပြသမည်။"]);
    assert_eq!(codes(&text), [Code::UndeclaredName]);
}

#[test]
fn generic_loop_update_sees_the_loop_variable()
{
    ok(&lines(&[
        "ကိန်းရှင် ကိန်း သည် ၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပြီး ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို \
         လုပ်ဆောင်မည်။",
        "ကိန်း ကို ပြသမည်။",
        "ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားမည်။",
        "လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီ။",
    ]));
}

#[test]
fn break_and_continue_need_a_loop()
{
    assert_eq!(codes("ကွင်းဆက် ကို ရပ်မည်။"), [Code::BreakOutsideLoop]);
    assert_eq!(codes("နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။"), [Code::ContinueOutsideLoop]);
    ok(&lines(&[
        "ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။",
        "ကိန်း သည် ၅ အောက် ငယ်နေသရွေ့ ကွင်းဆက် ကို ရပ်မည်။",
        "၁ မှ ၁၀ အထိ ခ အနေဖြင့် ရေတွက်၍ နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။",
    ]));
}

#[test]
fn a_function_starts_outside_loops()
{
    let text = lines(&[
        "လုပ်ဆောင်ချက် စမ်း သည်",
        "အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "ကွင်းဆက် ကို ရပ်မည်။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&text), [Code::BreakOutsideLoop]);
}

#[test]
fn return_needs_a_function()
{
    let start = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။";
    let bare = format!("ပြန်ပေးမည်။\n{start}");
    assert_eq!(codes_as_is(&bare), [Code::TopLevelStmt, Code::ReturnOutsideFunction]);
    let value = format!("၁ ကို ပြန်ပေးမည်။\n{start}");
    assert_eq!(codes_as_is(&value), [Code::TopLevelStmt, Code::ReturnOutsideFunction]);
    ok("ပြန်ပေးမည်။");
}

#[test]
fn return_values_match_the_header()
{
    let void_value = lines(&[
        "လုပ်ဆောင်ချက် နှုတ်ဆက် သည်",
        "အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "၁ ကို ပြန်ပေးမည်။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&void_value), [Code::ValueReturnedFromVoid]);
    let bare = lines(&[
        "လုပ်ဆောင်ချက် ဂဏန်း သည်",
        "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။",
        "ပြန်ပေးမည်။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&bare), [Code::MissingReturnValue]);
}

#[test]
fn the_old_result_variable_is_an_undeclared_name()
{
    let valued = lines(&[
        "လုပ်ဆောင်ချက် ဂဏန်း သည်",
        "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။",
        "ရလဒ် ကို ၁ ဟု ထားမည်။",
        "၁ ကို ပြန်ပေးမည်။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&valued), [Code::UndeclaredName]);
    let text = lines(&[
        "လုပ်ဆောင်ချက် ဂဏန်း သည်",
        "အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "ရလဒ် ကို ၁ ဟု ထားမည်။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&text), [Code::UndeclaredName]);
}

#[test]
fn calls_pass_one_argument_per_parameter()
{
    let text = lines(&[TWICE, "လုပ်ဆောင်ချက် နှစ်ဆ ကို လုပ်ဆောင်မည်။"]);
    assert_eq!(codes(&text), [Code::ArgCountMismatch]);
    let text = lines(&[TWICE, "လုပ်ဆောင်ချက် နှစ်ဆ ကို ၁၊ ၂ တို့ဖြင့် လုပ်ဆောင်မည်။"]);
    assert_eq!(codes(&text), [Code::ArgCountMismatch]);
}

#[test]
fn functions_without_a_result_are_no_values()
{
    let text = lines(&[GREET, "လုပ်ဆောင်ချက် နှုတ်ဆက် ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။"]);
    assert_eq!(codes(&text), [Code::ValueFromVoidFunction]);
}
