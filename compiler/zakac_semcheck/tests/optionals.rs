mod common;

use common::codes;
use common::ok;
use zakac_diagnostics::Code;

/// The kind of an optional number.
const MAYBE: &str = "ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်";

/// The kind of a number or a text error.
const RESULT: &str = "ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်";

/// A start block with the statements `stmts`.
fn start(stmts: &str) -> String
{
    format!("ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

/// A variable `name` of `kind` that starts at `value`.
fn var(name: &str, kind: &str, value: &str) -> String
{
    format!("ကိန်းရှင် {name} သည် {kind} တစ်ခု ဖြစ်၍ တန်ဖိုးသည် {value} ဖြစ်ပါစေ။\n")
}

#[test]
fn optionals_and_results_test_and_read_their_parts()
{
    let stmts = format!(
        "{}{}{}အကယ်၍ ခ ထဲတွင် ဒေတာ ရှိလျှင် ခ ထဲမှ ဒေတာ + ၁ ကို ပြသမည်။
အကယ်၍ ဂ ထဲတွင် အချက်အလက် မရှိနေလျှင် \"မရှိ\" ဟု ပြသမည်။
ဂ ကို ၅ ဟု ထားမည်။
အကယ်၍ ရ ထဲတွင် အမှားအယွင်း ရှိလျှင် ရ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။
အကယ်၍ ရ ထဲတွင် ရလဒ်ကောင်း ရှိနေလျှင် ရ ထဲမှ ဒေတာ ကို ပြသမည်။
ရ ကို ၇ ဟု ထားမည်။",
        var("ခ", MAYBE, "၃"),
        var("ဂ", MAYBE, "အလွတ်"),
        var("ရ", RESULT, "\"မရ\" ဟူသော အမှားအယွင်း")
    );
    ok(&start(&stmts));
}

#[test]
fn a_test_word_of_the_other_kind_is_zk0475()
{
    let result_test =
        format!("{}အကယ်၍ ခ ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် \"ရှိ\" ဟု ပြသမည်။", var("ခ", MAYBE, "၃"));
    assert_eq!(codes(&start(&result_test)), [Code::TestWordMismatch]);
    let data_test =
        format!("{}အကယ်၍ ရ ထဲတွင် ဒေတာ ရှိလျှင် \"ရှိ\" ဟု ပြသမည်။", var("ရ", RESULT, "၃"));
    assert_eq!(codes(&start(&data_test)), [Code::TestWordMismatch]);
    let fault_read = format!("{}ခ ထဲမှ အမှားအယွင်း ဒေတာ ကို ပြသမည်။", var("ခ", MAYBE, "၃"));
    assert_eq!(codes(&start(&fault_read)), [Code::TestWordMismatch]);
}

#[test]
fn an_error_value_with_no_result_kind_is_zk0476()
{
    let plain = var("က", "ကိန်းပြည့်", "၃ ဟူသော အမှားအယွင်း");
    assert_eq!(codes(&start(&plain)), [Code::FaultNeedsResult]);
    let optional = var("က", MAYBE, "၃ ဟူသော အမှားအယွင်း");
    assert_eq!(codes(&start(&optional)), [Code::FaultNeedsResult]);
}

#[test]
fn an_empty_optional_with_no_optional_kind_is_zk0477()
{
    let plain = var("က", "ကိန်းပြည့်", "အလွတ်");
    assert_eq!(codes(&start(&plain)), [Code::NoDataNeedsOptional]);
    let result = var("က", RESULT, "အလွတ်");
    assert_eq!(codes(&start(&result)), [Code::NoDataNeedsOptional]);
}

#[test]
fn reading_an_optional_a_test_proves_empty_is_zk0454()
{
    let stmts =
        format!("{}အကယ်၍ ခ ထဲတွင် ဒေတာ မရှိလျှင် ခ ထဲမှ ဒေတာ ကို ပြသမည်။", var("ခ", MAYBE, "အလွတ်"));
    assert_eq!(codes(&start(&stmts)), [Code::WrongPartProven]);
}

/// A function `စစ်` with the header line `head` and the statements `stmts`,
/// then a start block.
fn function(head: &str, stmts: &str) -> String
{
    format!(
        "လုပ်ဆောင်ချက် စစ် သည်\nန ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး\n{head}\n{stmts}\nလုပ်ဆောင်စရာများ \
         ပြီးဆုံးပါပြီး။\n{}",
        start("")
    )
}

/// A loop whose test is the constant true and that no break leaves never
/// ends, so a function can end in one; a break that leaves it makes the end
/// reachable, and a break in a nested loop leaves that loop alone.
#[test]
fn a_loop_that_never_ends_closes_each_path()
{
    let head = "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။";
    let forever = |inner: &str| {
        format!(
            "မှန် နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{inner}\nအကယ်၍ န သည် ၀ နှင့် ညီလျှင် ၁ ကို \
             ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။"
        )
    };
    ok(&function(head, &forever("န ကို န - ၁ ဟု ထားမည်။")));
    let leaves = forever("အကယ်၍ န သည် ၅ နှင့် ညီလျှင် ကွင်းဆက် ကို ရပ်မည်။");
    assert_eq!(codes(&function(head, &leaves)), [Code::EndWithoutReturn]);
    let nested = forever("န သည် ၅ ထက် ကြီးနေသရွေ့ ကွင်းဆက် ကို ရပ်မည်။");
    ok(&function(head, &nested));
}

#[test]
fn a_function_that_can_end_with_no_result_is_zk0339()
{
    let head = "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။";
    let open = "အကယ်၍ န သည် ၀ နှင့် ညီလျှင် ၁ ကို ပြန်ပေးမည်။";
    assert_eq!(codes(&function(head, open)), [Code::EndWithoutReturn]);
    let closed = format!("{open}\nသို့မဟုတ်ပါက ၂ ကို ပြန်ပေးမည်။");
    ok(&function(head, &closed));
    let head = format!("အောက်ပါတို့ကို လုပ်ဆောင်ကာ {MAYBE} တစ်ခု ကို ပြန်ပေးမည်။");
    let maybe = "အကယ်၍ န သည် ၀ နှင့် ညီလျှင် န ကို ပြန်ပေးမည်။\nအလွတ်ကို ပြန်ပေးမည်။";
    ok(&function(&head, maybe));
    let head = format!("အောက်ပါတို့ကို လုပ်ဆောင်ကာ {RESULT} တစ်ခု ကို ပြန်ပေးမည်။");
    let fails = "အကယ်၍ န သည် ၀ နှင့် ညီလျှင် \"သုည\" ဟူသော အမှားအယွင်း ကို ပြန်ပေးမည်။\nန ကို ပြန်ပေးမည်။";
    ok(&function(&head, fails));
}

/// A count written before an optional goes to the list inside it first, as
/// the innermost list of nested lists takes the nearest count; a list after
/// `ဒေတာဆဲလ်` takes the counts left. A list literal fills the data.
#[test]
fn a_count_goes_to_the_list_inside_an_optional()
{
    let inside = "အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်";
    ok(&start(&var("က", inside, "[၁၊ ၂၊ ၃]")));
    ok(&start(&var("က", inside, "အလွတ်")));
    assert_eq!(codes(&start(&var("က", inside, "[၁၊ ၂]"))), [Code::ListCountMismatch]);
    let outside = "အကွက် ၂ ခုရှိသော ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် စာရင်း";
    ok(&start(&var("ခ", outside, "[၄၊ အလွတ်]")));
    let both = "အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် စာရင်း";
    ok(&start(&var("ဂ", both, "ကနဦးတန်ဖိုး")));
}

/// A literal takes the kind of the data of an optional, or of the good value
/// of a result: a `၈ ဘစ်` optional takes `၁` as it is, and a value outside
/// the kind inside is ZK0406.
#[test]
fn a_literal_takes_the_kind_inside()
{
    let narrow = "၈ ဘစ် ကိန်းပြည့် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်";
    let half = "၃၂ ဘစ် ဒဿမကိန်း ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်";
    let small = "၈ ဘစ် ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်";
    let stmts = format!(
        "{}{}{}",
        var("ခ", narrow, "၁"),
        var("ဖ", half, "၁.၅"),
        var("ရ", small, "၁၀၀")
    );
    ok(&start(&stmts));
    let wide = var("ခ", narrow, "၃၀၀");
    assert_eq!(codes(&start(&wide)), vec![Code::LiteralDoesNotFit]);
}
