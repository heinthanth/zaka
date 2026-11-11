mod common;

use common::codes;
use common::ok;
use zakac_diagnostics::Code;

/// A function that can change its integer parameter, and one that cannot.
const FNS: &str = "လုပ်ဆောင်ချက် တိုး သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို န + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် နှစ်ခု သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း မ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို မ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် ပြ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ကိန်းရှင် က သည် ၅ ဖြစ်ပါစေ။
";

/// The codes of [`FNS`] and one more line.
fn with(line: &str) -> Vec<Code> { codes(&format!("{FNS}{line}")) }

#[test]
fn a_place_is_changed()
{
    ok(&format!("{FNS}လုပ်ဆောင်ချက် တိုး ကို က တို့ဖြင့် လုပ်ဆောင်မည်။"));
}

#[test]
fn a_call_writes_no_mark()
{
    ok(&format!("{FNS}လုပ်ဆောင်ချက် ပြ ကို က တို့ဖြင့် လုပ်ဆောင်မည်။"));
}

#[test]
fn a_changed_argument_is_a_writable_place_of_the_same_kind()
{
    let value = "လုပ်ဆောင်ချက် တိုး ကို က + ၁ တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(with(value), [Code::NotAPlace]);
    let constant = "ကိန်းသေ ဂ သည် ၅ ဖြစ်ပါစေ။\nလုပ်ဆောင်ချက် တိုး ကို ဂ တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(with(constant), [Code::AssignToConst]);
    let narrow =
        "ကိန်းရှင် ဃ သည် ၅ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\nလုပ်ဆောင်ချက် တိုး ကို ဃ တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(with(narrow), [Code::KindMismatch]);
}

#[test]
fn a_changed_place_appears_in_no_other_argument()
{
    let twice = "လုပ်ဆောင်ချက် နှစ်ခု ကို က၊ က + ၁ တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(with(twice), [Code::EditShared]);
}
