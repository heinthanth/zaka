mod common;

use common::codes_as_is;
use zakac_diagnostics::Code;

/// The error enumeration and an input stream, as the stdlib
/// declares them.
const KINDS: &str = "\
ဒေတာအမျိုးအစား အချက်အလက်အဝင်အထွက်အမှားအယွင်း ဆိုသည်မှာ မတွေ့ခြင်း၊ တခြားအကြောင်းအရင်း တို့ ဖြစ်သည်။
ဒေတာအမျိုးအစား အချက်အလက်အဝင်စီးကြောင်း တွင် ကိန်းပြည့် လက်ကိုင် တို့ ပါဝင်သည်။
";

/// A program: `KINDS`, the declarations `decls`, then a blank start block.
fn program(decls: &str) -> String
{
    format!("{KINDS}{decls}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

#[test]
fn an_external_function_the_host_serves_checks()
{
    let read = "လုပ်ဆောင်ချက် ဖိုင်ကိုဖတ် သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ စာသား လမ်း(၌ရှိသော) လက်ခံပြီး \
                စာသား ရလဒ် သို့မဟုတ် အချက်အလက်အဝင်အထွက်အမှားအယွင်း အမှားအယွင်း ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(read)), []);
    let line = "လုပ်ဆောင်ချက် စာကြောင်းတစ်ကြောင်းဖတ် သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ \
                အချက်အလက်အဝင်စီးကြောင်း ရင်း(မှ) လက်ခံပြီး စာသား ရလဒ် သို့မဟုတ် အချက်အလက်အဝင်အထွက်အမှားအယွင်း \
                အမှားအယွင်း ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(line)), []);
}

#[test]
fn an_external_function_the_host_does_not_serve_is_zk0334()
{
    let unknown = "လုပ်ဆောင်ချက် ဟိုဟို သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်သည်။";
    assert_eq!(codes_as_is(&program(unknown)), [Code::UnknownHostFn]);
    let wrong_result = "လုပ်ဆောင်ချက် ဖိုင်ကိုဖတ် သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ စာသား လမ်း ကို \
                        လက်ခံပြီး စာသား ရလဒ် ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(wrong_result)), [Code::UnknownHostFn]);
}

#[test]
fn c_functions_take_kinds_c_passes_whole()
{
    let abs = "လုပ်ဆောင်ချက် abs သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ ၃၂ ဘစ် ကိန်းပြည့် n ကို လက်ခံပြီး \
               ၃၂ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(abs)), []);
    let frexp = "လုပ်ဆောင်ချက် frexp သည် \"libm\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ ဒဿမကိန်း x၊ ငှားထားသော \
                 ၃၂ ဘစ် ကိန်းပြည့် ၏ လိပ်စာ e တို့ကို လက်ခံပြီး ဒဿမကိန်း ရလဒ် ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(frexp)), []);
    let pair = "ဒေတာအမျိုးအစား စုံ တွင် ၃၂ ဘစ် ကိန်းပြည့် က၊ ၃၂ ဘစ် ကိန်းပြည့် ခ တို့ ပါဝင်သည်။\nလုပ်ဆောင်ချက် \
                div သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ ၃၂ ဘစ် ကိန်းပြည့် a၊ ၃၂ ဘစ် ကိန်းပြည့် b \
                တို့ကို လက်ခံပြီး စုံ ရလဒ် ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(pair)), []);
}

#[test]
fn c_function_rules_have_their_codes()
{
    let burmese = "လုပ်ဆောင်ချက် ရေး သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်သည်။";
    assert_eq!(codes_as_is(&program(burmese)), [Code::NotACName]);
    let text = "လုပ်ဆောင်ချက် getenv သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ စာသား n ကို လက်ခံပြီး \
                စာသား ရလဒ် ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(text)), [Code::KindNotAcrossC]);
    let owner = "လုပ်ဆောင်ချက် malloc သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ ကိန်းပြည့် n ကို လက်ခံပြီး \
                 လိပ်စာ ရလဒ် ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(owner)), [Code::KindNotAcrossC]);
    let edit = "လုပ်ဆောင်ချက် inc သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ လုပ်ဆောင်ချက်မှ \
                ပြောင်းလဲနိုင်သော ၃၂ ဘစ် ကိန်းပြည့် n ကို လက်ခံမည်။";
    assert_eq!(codes_as_is(&program(edit)), [Code::EditAcrossC]);
    let twice = "လုပ်ဆောင်ချက် abs သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ ၃၂ ဘစ် ကိန်းပြည့် n ကို \
                 လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။\nလုပ်ဆောင်ချက် abs သည် \"libc\" \
                 မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ ၆၄ ဘစ် ကိန်းပြည့် n ကို လက်ခံပြီး ၆၄ ဘစ် ကိန်းပြည့် ရလဒ် ကို \
                 ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(twice)), [Code::CNameTwice]);
}

/// `printf` from libc, which takes other values.
const PRINTF: &str = "လုပ်ဆောင်ချက် printf သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ စာသား f နှင့် \
                      အခြားတန်ဖိုးများ ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။";

/// A program with `PRINTF` and a start block that runs `stmts`.
fn printing(decls: &str, stmts: &str) -> String
{
    format!(
        "{KINDS}{PRINTF}\n{decls}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် \
         ပြီးဆုံးပါပြီး။"
    )
}

#[test]
fn c_functions_take_text_and_other_values()
{
    let puts = "လုပ်ဆောင်ချက် puts သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ စာသား s ကို လက်ခံမည်။";
    assert_eq!(codes_as_is(&program(puts)), []);
    let call = "လုပ်ဆောင်ချက် printf ကို \"%s %ld\\n\"၊ \"က\"၊ ၄၂ တို့ဖြင့် လုပ်ဆောင်ပါ။";
    assert_eq!(codes_as_is(&printing("", call)), []);
    let bare = "လုပ်ဆောင်ချက် printf ကို \"က\\n\" တို့ဖြင့် လုပ်ဆောင်ပါ။";
    assert_eq!(codes_as_is(&printing("", bare)), []);
}

#[test]
fn other_values_have_their_rules()
{
    let none = "လုပ်ဆောင်ချက် printf ကို လုပ်ဆောင်ပါ။";
    assert_eq!(codes_as_is(&printing("", none)), [Code::ArgCountMismatch]);
    let pair = "ဒေတာအမျိုးအစား စုံ တွင် ကိန်းပြည့် က တို့ ပါဝင်သည်။";
    let call = "ကိန်းရှင် တ သည် စုံ တစ်ခု ဖြစ်ပါစေ။\nလုပ်ဆောင်ချက် printf ကို \"%d\"၊ တ တို့ဖြင့် လုပ်ဆောင်ပါ။";
    assert_eq!(codes_as_is(&printing(pair, call)), [Code::KindNotAcrossC]);
    let host =
        "လုပ်ဆောင်ချက် ရေး သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ စာသား စ နှင့် အခြားတန်ဖိုးများ ကို လက်ခံမည်။";
    assert_eq!(codes_as_is(&program(host)), [Code::UnknownHostFn]);
}
