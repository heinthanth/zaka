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
