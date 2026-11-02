mod common;

use common::codes;
use common::ok;
use zakac_diagnostics::Code;

/// A function that can change its integer parameter, and one that cannot.
const FNS: &str = "လုပ်ဆောင်ချက် တိုး သည်
လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို န + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် နှစ်ခု သည်
လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော ကိန်းပြည့် န၊ ကိန်းပြည့် မ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို မ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ပြ သည်
ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။
";

/// The codes of [`FNS`] and one more line.
fn with(line: &str) -> Vec<Code> { codes(&format!("{FNS}{line}")) }

#[test]
fn a_marked_place_is_changed()
{
    ok(&format!("{FNS}လုပ်ဆောင်ချက် တိုး ကို က (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။"));
}

#[test]
fn the_mark_goes_on_both_ends()
{
    assert_eq!(with("လုပ်ဆောင်ချက် တိုး ကို က တို့ဖြင့် လုပ်ဆောင်ပါ။"), [Code::EditMarkMissing]);
    let stray = "လုပ်ဆောင်ချက် ပြ ကို က (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။";
    assert_eq!(with(stray), [Code::EditMarkExtra]);
}

#[test]
fn a_marked_argument_is_a_writable_place_of_the_same_kind()
{
    let value = "လုပ်ဆောင်ချက် တိုး ကို က + ၁ (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။";
    assert_eq!(with(value), [Code::NotAPlace]);
    let constant = "ကိန်းသေ ဂ သည် ၅ ဖြစ်ပါစေ။\nလုပ်ဆောင်ချက် တိုး ကို ဂ (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်) \
                    တို့ဖြင့် လုပ်ဆောင်ပါ။";
    assert_eq!(with(constant), [Code::AssignToConst]);
    let narrow = "ကိန်းရှင် ဃ သည် ၈ ဘစ် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။\nလုပ်ဆောင်ချက် တိုး ကို ဃ (လုပ်ဆောင်ချက်မှ \
                  ပြောင်းလဲနိုင်သည်) တို့ဖြင့် လုပ်ဆောင်ပါ။";
    assert_eq!(with(narrow), [Code::KindMismatch]);
}

#[test]
fn a_changed_place_appears_in_no_other_argument()
{
    let twice = "လုပ်ဆောင်ချက် နှစ်ခု ကို က (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)၊ က + ၁ တို့ဖြင့် လုပ်ဆောင်ပါ။";
    assert_eq!(with(twice), [Code::EditShared]);
}
