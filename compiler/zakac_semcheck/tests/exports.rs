//! Functions that code outside can call,
//! `"<C name>" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော လုပ်ဆောင်ချက်ဖြစ်၍ ...`.

use zakac_diagnostics::Code;

use crate::common::codes;
use crate::common::ok;

/// A function `name` with the C name `c_name`, the parameters `params`, the
/// result kind `ret`, and the statements `stmts`.
fn export(name: &str, c_name: &str, params: &str, ret: &str, stmts: &str) -> String
{
    format!(
        "{name} ဟူသော လုပ်ဆောင်ချက် သည် \"{c_name}\" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော \
         လုပ်ဆောင်ချက်ဖြစ်၍\n{params} လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ {ret} တစ်ခု ကို \
         ပြန်ပေးမည်။\n{stmts}\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n"
    )
}

/// One parameter `က` of the kind `kind`.
fn one(kind: &str) -> String
{
    format!("က ဟူသော {kind} တစ်ခု ကို")
}

#[test]
fn a_function_c_can_call_checks()
{
    let int = "၃၂ ဘစ် ကိန်းပြည့်";
    ok(&export("ပေါင်း", "add_numbers", &one(int), int, "က ကို ပြန်ပေးမည်။"));
    // C gives text as a `char *`, as it does to a function value it calls back.
    ok(&export("အရှည်", "text_len", &one("စာသား"), int, "၀ ကို ပြန်ပေးမည်။"));
}

#[test]
fn a_kind_c_cannot_take_is_zk0465()
{
    let text = export("ပဲ့တင်", "echo_text", &one("စာသား"), "စာသား", "က ကို ပြန်ပေးမည်။");
    assert_eq!(codes(&text), [Code::KindNotAcrossC]);
}

#[test]
fn c_names_are_checked()
{
    let int = "ကိန်းပြည့်";
    let stmts = "က ကို ပြန်ပေးမည်။";
    assert_eq!(codes(&export("က၁", "2x", &one(int), int, stmts)), [Code::NotACName]);
    assert_eq!(
        codes(&export("က၂", "zk_add", &one(int), int, stmts)),
        [Code::CNameTaken]
    );
    assert_eq!(codes(&export("က၃", "main", &one(int), int, stmts)), [Code::CNameTaken]);
    let twice = export("က၄", "twice", &one(int), int, stmts)
        + &export("က၅", "twice", &one(int), int, stmts);
    assert_eq!(codes(&twice), [Code::CNameTaken]);
}

#[test]
fn a_nested_function_c_cannot_call_is_zk0490()
{
    let inner = "အတွင်း ဟူသော လုပ်ဆောင်ချက် သည် \"inner_fn\" အမည်ဖြင့် ပြင်ပမှ ခေါ်ဆိုနိုင်သော \
                 လုပ်ဆောင်ချက်ဖြစ်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nက ကို \
                 ပြန်ပေးမည်။";
    let int = "ကိန်းပြည့်";
    let outer = export("အပြင်", "outer_fn", &one(int), int, inner);
    assert_eq!(codes(&outer), [Code::ExportNotPlain]);
}

/// A struct `name` of two 32-bit fields, with the C name `c_name` when one
/// is given.
fn point(name: &str, c_name: Option<&str>) -> String
{
    let mark = c_name.map_or("တွင်".to_string(), |c| {
        format!("သည် \"{c}\" အမည်ဖြင့် ပြင်ပမှ ယူသုံးနိုင်ပြီး ၎င်းတွင်")
    });
    format!(
        "{name} ဟူသော ဒေတာအစုအဝေး{mark} x ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ၃၂ ဘစ် ကိန်းပြည့် \
         တစ်ခု တို့ပါဝင်သည်။\n"
    )
}

#[test]
fn a_struct_c_takes_needs_a_c_name()
{
    let int = "၃၂ ဘစ် ကိန်းပြည့်";
    let call = export("အကွာ", "manhattan", &one("pt"), int, "(က ၏ x) ကို ပြန်ပေးမည်။");
    ok(&(point("pt", Some("point")) + &call));
    assert_eq!(codes(&(point("pt", None) + &call)), [Code::StructNeedsCName]);
}

#[test]
fn struct_c_names_are_checked()
{
    assert_eq!(codes(&point("pt", Some("2d"))), [Code::NotACName]);
    assert_eq!(codes(&point("pt", Some("zk_point"))), [Code::CNameTaken]);
    let twice = point("pt", Some("point")) + &point("pt2", Some("point"));
    assert_eq!(codes(&twice), [Code::CNameTaken]);
    let int = "ကိန်းပြည့်";
    let clash = point("pt", Some("twice"))
        + &export("က", "twice", &one(int), int, "က ကို ပြန်ပေးမည်။");
    assert_eq!(codes(&clash), [Code::CNameTaken]);
}
