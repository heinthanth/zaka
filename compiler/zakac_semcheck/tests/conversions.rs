use zakac_diagnostics::Code;

use crate::common::codes_as_is;

/// A struct, a conversion of it to a decimal number, and a function of the
/// old name rule that no longer converts.
const METRE: &str = "\
မီတာ ဟူသော ဒေတာအစုအဝေးတွင် အရှည် ဟူသော ဒဿမကိန်း တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ခု ဖြစ်သည်။
ပြောင်းလဲ ဟူသော လုပ်ဆောင်ချက် သည် မ ဟူသော မီတာ တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဒဿမကိန်း တစ်ခု ကို \
                     ပြန်ပေးမည်။
မ ၏ အရှည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းပြည့်အဖြစ်ပြောင်း ဟူသော လုပ်ဆောင်ချက် သည် မ ဟူသော မီတာ တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် \
                     တစ်ခု ကို ပြန်ပေးမည်။
၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// A program: `METRE`, then a start block that holds `stmts`.
fn program(stmts: &str) -> String
{
    format!("{METRE}ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

/// A conversion goes through the `ပြောင်းလဲ` of its two kinds.
#[test]
fn a_conversion_calls_the_function_of_its_kinds()
{
    let line = "မီတာ(အရှည်: ၁.၀) ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။";
    assert_eq!(codes_as_is(&program(line)), []);
}

/// A conversion that cannot fail passes no failure on: ZK0444.
#[test]
fn a_sure_conversion_with_the_good_result_is_zk0444()
{
    let line = "မီတာ(အရှည်: ၁.၀) ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်ကောင်း ကို ပြသမည်။";
    assert_eq!(codes_as_is(&program(line)), [Code::PassWithoutFailure]);
}

/// A function whose name ends in `အဖြစ်ပြောင်း` is a plain function, and the
/// conversion it once stood for is ZK0408.
#[test]
fn the_old_name_rule_converts_nothing()
{
    let line = "မီတာ(အရှည်: ၁.၀) ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။";
    assert_eq!(codes_as_is(&program(line)), [Code::InvalidCast]);
}

/// Each value converts to the text it prints as.
#[test]
fn a_value_converts_to_its_text()
{
    let line = "(၄၂ ကို စာသား တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်) + \"!\" ကို ပြသမည်။";
    assert_eq!(codes_as_is(&program(line)), []);
}
