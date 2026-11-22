//! Trait values on the VM: calls run the function of the held kind, a
//! function that changes its first parameter changes a duplicate of a plain
//! value and the shared object itself, and no block leaks.

use zaka_vm::machine::HostData;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_and_keep;

use crate::common::compile;

/// A counter with a trait whose functions change it, read it and give a
/// doubled one back; the program calls each through trait values.
const COUNTER: &str = r#"တိုးနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
တိုး ဟူသော လုပ်ဆောင်ချက် သည် ပ ဟူသော A တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည် ပ ဟူသော A တစ်ခု ကို လက်ခံပြီး A တစ်ခု ကို ပြန်ပေးမည်။
ဖတ် ဟူသော လုပ်ဆောင်ချက် သည် ပ ဟူသော A တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။

ကောင်တာ ဟူသော ဒေတာအစုအဝေးတွင် အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

တိုး ဟူသော လုပ်ဆောင်ချက် သည် ပ ဟူသော ကောင်တာ တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ပ ၏ အကြိမ် ကို (ပ ၏ အကြိမ်) + ၁ ဟု ထားမည်။
ပ ၏ အကြိမ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

နှစ်ဆ ဟူသော လုပ်ဆောင်ချက် သည် ပ ဟူသော ကောင်တာ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကောင်တာ တစ်ခု ကို ပြန်ပေးမည်။
ကောင်တာ(အကြိမ်: (ပ ၏ အကြိမ်) * ၂) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဖတ် ဟူသော လုပ်ဆောင်ချက် သည် ပ ဟူသော ကောင်တာ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ပ ၏ အကြိမ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပြ ဟူသော လုပ်ဆောင်ချက် သည် ဥ ဟူသော တိုးနိုင်သော အရာ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဥ ၏ ဖတ်() ကို ပြသမည်။
ဥ ၏ တိုး() ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် တ သည် တိုးနိုင်သော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကောင်တာ(အကြိမ်: ၅) ဖြစ်ပါစေ။
ကိန်းရှင် ထ သည် တ ဖြစ်ပါစေ။
တ ၏ တိုး() ကို ပြသမည်။
တ ၏ တိုး() ကို ပြသမည်။
ထ ၏ ဖတ်() ကို ပြသမည်။
ကိန်းရှင် နှ သည် တ ၏ နှစ်ဆ() ဖြစ်ပါစေ။
နှ ၏ ဖတ်() ကို ပြသမည်။
ပြ ဟူသော လုပ်ဆောင်ချက်ကို နှ ဖြင့် လုပ်ဆောင်မည်။
နှ ၏ ဖတ်() ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
"#;

/// Run a program and give its output, after a check that no block leaks.
///
/// # Arguments
///
/// * `text` - The program.
///
/// # Returns
///
/// The output.
fn run_clean(text: &str) -> String
{
    let module = compile(text);
    let mut out = Vec::new();
    let (machine, result) =
        run_and_keep(&module, &mut out, &RunOptions::default(), &HostData::default());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(machine.leaks(), []);
    String::from_utf8(out).unwrap()
}

/// A plain counter goes into a trait value as a duplicate: a change through one
/// trait value leaves a second one, and the caller of a function that
/// changes its parameter, as it was.
#[test]
fn a_change_reaches_the_duplicate_alone()
{
    let out = run_clean(COUNTER);
    assert_eq!(
        out,
        "၆
၇
၅
၁၄
၁၄
၁၅
၁၄
"
    );
}

/// A shared counter goes in as a reference: each trait value and the
/// function's parameter reach the one object.
#[test]
fn a_change_reaches_the_shared_object()
{
    let shared = COUNTER
        .replace("ကောင်တာ ဟူသော ဒေတာအစုအဝေးတွင်", "ကောင်တာ ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင်");
    let out = run_clean(&shared);
    assert_eq!(
        out,
        "၆
၇
၇
၁၄
၁၄
၁၅
၁၅
"
    );
}
