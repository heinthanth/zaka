mod common;

use common::codes_as_is;
use zakac_diagnostics::Code;

/// A plain struct, a kind from outside, and C functions that call back a
/// function with each of them, with text and with an optional of text.
const DECLS: &str = "\
စုံ ဟူသော ဒေတာအစုအဝေးတွင် a ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ b ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
ဘူး ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင် n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
လုပ်ဆောင်ချက် with_text သည် \"libcb\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် \
                     အမျိုးအစား) တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် with_plain သည် \"libcb\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (စာသား တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို \
                     လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် with_pair သည် \"libcb\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (စုံ တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံပြီး \
                     ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် with_box သည် \"libcb\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (ဘူး ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် \
                     အမျိုးအစား) တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် box_close သည် \"libcb\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
b ဟူသော ဘူး တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံမည်။
လုပ်ဆောင်ချက် box_open သည် \"libcb\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ ဘူး တစ်ခု ကို ပြန်ပေးမည်။
";

/// A program: `DECLS`, the functions `fns`, then a start block with the
/// statements `stmts`.
fn program(fns: &str, stmts: &str) -> String
{
    format!("{DECLS}{fns}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

/// A function of one parameter of `kind` that gives back 1.
fn one(name: &str, kind: &str) -> String
{
    format!(
        "လုပ်ဆောင်ချက် {name} သည်\nx ဟူသော {kind} တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် \
         ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\n၁ ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n"
    )
}

#[test]
fn callbacks_take_text_structs_and_handles()
{
    let fns = [
        one("က", "စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်"),
        one("ခ", "စာသား"),
        one("ဂ", "စုံ"),
        one("ဃ", "ဘူး ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်"),
    ]
    .concat();
    let stmts = "with_text(က ဟူသော လုပ်ဆောင်ချက်) ကို ပြသမည်။
with_plain(ခ ဟူသော လုပ်ဆောင်ချက်) ကို ပြသမည်။
with_pair(ဂ ဟူသော လုပ်ဆောင်ချက်) ကို ပြသမည်။
with_box(ဃ ဟူသော လုပ်ဆောင်ချက်) ကို ပြသမည်။";
    assert_eq!(codes_as_is(&program(&fns, stmts)), []);
}

#[test]
fn a_callback_of_a_list_is_zk0465()
{
    let decl = "လုပ်ဆောင်ချက် with_list သည် \"libcb\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (ကိန်းပြည့် စာရင်း တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု \
                ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\n";
    assert_eq!(codes_as_is(&program(decl, "")), [Code::KindNotAcrossC]);
}

/// A function that closes the handle it got as a parameter, with the
/// statement `close` around the call.
fn closer(close: &str) -> String
{
    format!(
        "လုပ်ဆောင်ချက် ပိတ် သည်\nb ဟူသော ဘူး တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို \
         လုပ်ဆောင်မည်။\n{close}\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n"
    )
}

#[test]
fn closing_a_handle_parameter_needs_marked_code()
{
    let call = "လုပ်ဆောင်ချက် box_close ကို b တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(codes_as_is(&program(&closer(call), "")), [Code::UnsafeOutside]);
    let marked =
        format!("အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။\n{call}\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။");
    assert_eq!(codes_as_is(&program(&closer(&marked), "")), []);
    // A handle the program made itself closes in safe code.
    let own = "ကိန်းရှင် ဘ သည် box_open() ဖြစ်ပါစေ။\nလုပ်ဆောင်ချက် box_close ကို ဘ တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(codes_as_is(&program("", own)), []);
}
