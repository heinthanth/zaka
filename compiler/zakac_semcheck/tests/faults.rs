use zakac_diagnostics::Code;

use crate::common::codes;
use crate::common::ok;

/// A function that can fail with text: it fails on 0 and gives back its
/// argument otherwise.
const CHECK: &str = "\
လုပ်ဆောင်ချက် စစ် သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န သည် ၀ နှင့် ညီလျှင် \"သုည\" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။
န ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// A program: `CHECK`, then a function with the result clause `head` and
/// the statements `stmts`, then a start block.
fn program(head: &str, stmts: &str) -> String
{
    format!(
        "{CHECK}လုပ်ဆောင်ချက် သုံး သည်\n{head}\n{stmts}\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nပရိုဂရမ်သည် \
         အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။"
    )
}

const FAILS_TEXT: &str =
    "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။";
const FAILS_ONLY: &str = "အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား \
                          ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။";
const SURE: &str = "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။";

#[test]
fn each_pass_on_form_checks_in_a_function_that_fails_alike()
{
    ok(&program(FAILS_TEXT, "စစ်(၁)? + ၁ ကို ပြန်ပေးမည်။"));
    let word = "စစ် ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ်ကောင်း ကို ပြသမည်။";
    ok(&program(FAILS_ONLY, word));
    let stmt = "စစ် ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။";
    ok(&program(FAILS_ONLY, stmt));
    ok(&program(FAILS_TEXT, "\"မရ\" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။"));
}

#[test]
fn a_start_block_cannot_fail()
{
    let text = format!("{CHECK}ပရိုဂရမ်သည် {FAILS_TEXT}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။");
    assert_eq!(codes(&text), [Code::StartResultKind]);
    let stmts = "စစ်(၁)? ကို ပြသမည်။";
    let text = format!("{CHECK}ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။");
    assert_eq!(codes(&text), [Code::PassOutsideFailing]);
}

#[test]
fn a_call_statement_that_drops_a_failure_is_zk0443()
{
    let stmt = "စစ် ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(codes(&program(FAILS_ONLY, stmt)), [Code::FailureNotPassed]);
}

#[test]
fn a_call_without_a_pass_on_gives_the_whole_result()
{
    let stored = "ကိန်းရှင် ရ သည် စစ် ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ရ ကို စစ်(၂) ဟု ထားမည်။
အကယ်၍ ရ ထဲတွင် အမှားအယွင်း ရှိလျှင် ရ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
ရ ထဲမှ ဒေတာ ကို ပြသမည်။";
    ok(&program(FAILS_ONLY, stored));
}

#[test]
fn a_whole_result_is_no_number()
{
    let text = program(FAILS_TEXT, "စစ်(၁) + ၁ ကို ပြန်ပေးမည်။");
    assert_eq!(codes(&text), [Code::OperandKind]);
}

#[test]
fn a_test_or_a_conversion_of_a_value_that_is_no_result_is_zk0448()
{
    let text = program(FAILS_ONLY, "၅ ထဲမှ ဒေတာ ကို ပြသမည်။");
    assert_eq!(codes(&text), [Code::NotAResult]);
}

#[test]
fn the_good_result_of_a_function_without_one_is_zk0449()
{
    let fails = "\
လုပ်ဆောင်ချက် ရေး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";
    let read = "ရေး() ထဲမှ ဒေတာ ကို ပြသမည်။";
    let text = format!("{fails}{}", program(FAILS_ONLY, read));
    assert_eq!(codes(&text), [Code::NoGoodValue]);
}

#[test]
fn a_pass_on_from_a_sure_function_is_zk0444()
{
    let text = program(SURE, "သုံး()? ကို ပြန်ပေးမည်။");
    assert_eq!(codes(&text), [Code::PassWithoutFailure]);
}

#[test]
fn a_pass_on_in_a_function_that_cannot_fail_is_zk0445()
{
    let text = program(SURE, "စစ်(၁)? ကို ပြန်ပေးမည်။");
    assert_eq!(codes(&text), [Code::PassOutsideFailing]);
}

#[test]
fn a_pass_on_with_another_error_kind_is_zk0446()
{
    let head = "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ကိန်းပြည့် ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု \
                ကို ပြန်ပေးမည်။";
    let text = program(head, "စစ်(၁)? ကို ပြန်ပေးမည်။");
    assert_eq!(codes(&text), [Code::FaultKindDiffers]);
}

#[test]
fn a_failure_in_a_sure_function_is_zk0447()
{
    let text = program(SURE, "လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။");
    assert_eq!(codes(&text), [Code::FaultFromSure]);
}

#[test]
fn an_error_value_from_a_sure_function_is_zk0476()
{
    let text = program(SURE, "၁ ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။");
    assert_eq!(codes(&text), [Code::FaultNeedsResult]);
}

#[test]
fn a_good_result_of_a_function_without_one_is_zk0315()
{
    let fails = "\
လုပ်ဆောင်ချက် ရေး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";
    let text = format!("{fails}{}", program(FAILS_ONLY, "ရေး()? ကို ပြသမည်။"));
    assert_eq!(codes(&text), [Code::ValueFromVoidFunction]);
}

/// Structs whose names end in `ရလဒ်` and `အမှားအယွင်း`, a struct with a field
/// of a result kind, and functions with each result kind.
const KINDS: &str = "\
ဖိုင်ရလဒ် ဟူသော ဒေတာအစုအဝေးတွင် အရွယ် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
ဖိုင်အမှားအယွင်း ဟူသော ဒေတာအစုအဝေးတွင် စာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။
ဖတ်ချက် ဟူသော ဒေတာအစုအဝေးတွင် တွေ့ချက် ဟူသော ဖိုင်ရလဒ် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် \
                     တစ်ခု ပါဝင်သည်။

လုပ်ဆောင်ချက် ဖွင့် သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဖိုင်ရလဒ် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
ဖိုင်ရလဒ်(အရွယ်: ၁) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ရှာ သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ရေး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု သတ်မှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// `KINDS`, then a start block with the statements `stmts`.
fn with_kinds(stmts: &str) -> String
{
    format!("{KINDS}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

#[test]
fn written_result_kinds_and_their_sugar_check()
{
    ok(&with_kinds(
        "ကိန်းရှင် ဖ သည် ဖတ်ချက် တစ်ခု ဖြစ်ပါစေ။
ဖ ၏ တွေ့ချက် ကို ဖွင့်() ဟု ထားမည်။
ကိန်းရှင် ပ သည် ဖိုင်ရလဒ် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဖွင့်() \
         ဖြစ်ပါစေ။
ကိန်းရှင် R သည် ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရှာ() ဖြစ်ပါစေ။
ကိန်းရှင် W သည် အောင်မြင်သည့်သတင်း သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရေး() \
         ဖြစ်ပါစေ။",
    ));
}

#[test]
fn an_error_part_of_a_result_with_none_is_zk0450()
{
    let text = with_kinds("ရှာ() ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။");
    assert_eq!(codes(&text), [Code::NoFaultValue]);
}

#[test]
fn a_failure_with_no_value_and_an_error_value_where_none_fits_are_zk0451_and_zk0476()
{
    let missing = KINDS.replace("လုပ်ဆောင်ချက် အောင်မြင်သည်ဟု", "လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု");
    let text = format!("{missing}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။");
    assert_eq!(codes(&text), [Code::FaultValueMissing]);
    let extra = KINDS
        .replace("လုပ်ဆောင်ချက် မအောင်မြင်ပါဟု သတ်မှတ်မည်။", "\"x\" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။");
    let text = format!("{extra}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။");
    assert_eq!(codes(&text), [Code::FaultNeedsResult]);
}

#[test]
fn lists_and_pointers_of_result_kinds_check()
{
    ok(&with_kinds(
        "ကိန်းရှင် ရများ သည် ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ရများ ထဲသို့ ရှာ() ကို ထည့်မည်။
ကိန်းရှင် နှစ်ခု သည် အကွက် ၂ ခုရှိသော ဖိုင်ရလဒ် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် ဖိုင်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် စာရင်း တစ်ခု \
         ဖြစ်ပါစေ။
နှစ်ခု ၏ အကွက် ၀ ကို ဖွင့်() ဟု ထားမည်။
ကိန်းရှင် R သည် ရှာ() ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် R ၏ လိပ်စာ \
         ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ပြသမည်။",
    ));
}

#[test]
fn a_result_with_no_starting_value_is_a_failure()
{
    ok(&with_kinds("ကိန်းရှင် R သည် ကိန်းပြည့် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ဖြစ်ပါစေ။"));
}

#[test]
fn reading_the_part_a_test_rules_out_is_zk0454()
{
    let wrong = [
        "ကိန်းရှင် W သည် ရေး() ဖြစ်ပါစေ။
အကယ်၍ W ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် W ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။",
        "ကိန်းရှင် R သည် ရှာ() ဖြစ်ပါစေ။
အကယ်၍ R ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် \"ok\" ဟု ပြသမည်။
သို့မဟုတ်ပါက R ထဲမှ ဒေတာ ကို ပြသမည်။",
        "ကိန်းရှင် W သည် ရေး() ဖြစ်ပါစေ။
အကယ်၍ W ထဲတွင် အမှားအယွင်း ရှိလျှင် ပြန်ပေးမည်။
W ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။",
    ];
    for stmts in wrong
    {
        assert_eq!(codes(&with_kinds(stmts)), [Code::WrongPartProven], "{stmts}");
    }
}

#[test]
fn a_change_to_the_result_ends_what_the_test_proved()
{
    ok(&with_kinds(
        "ကိန်းရှင် R သည် ရှာ() ဖြစ်ပါစေ။
အကယ်၍ R ထဲတွင် အမှားအယွင်း ရှိလျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
R ကို ရှာ() ဟု ထားမည်။
R ထဲမှ ဒေတာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
အကယ်၍ R ထဲတွင် အမှားအယွင်း ရှိလျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ရ သည် ၀ ဖြစ်ပါစေ။
ရ သည် ၂ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
R ထဲမှ ဒေတာ ကို ပြသမည်။
R ကို ရှာ() ဟု ထားမည်။
ရ ကို ရ + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။",
    ));
}
