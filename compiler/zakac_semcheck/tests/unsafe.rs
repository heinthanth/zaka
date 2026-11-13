use zakac_diagnostics::Code;
use zakac_semcheck::semcheck::SemcheckOptions;

use crate::common::check_as_is;
use crate::common::codes_as_is;

/// A program whose start block holds `steps`, after the declarations `decls`.
fn program(decls: &str, steps: &str) -> String
{
    format!("{decls}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{steps}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n")
}

/// `steps` inside an unsafe block.
fn in_unsafe_block(steps: &str) -> String
{
    format!("အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။\n{steps}\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။")
}

/// A pointer `ပ` to a block of one integer, and an integer `x`.
const BLOCK: &str = "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် x သည် ၇ ဖြစ်ပါစေ။";

/// The pointer work that needs unsafe code, one line each.
const WORK: &[&str] = &[
    "မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။",
    "ပ ၏ တန်ဖိုး ကို ပြသမည်။",
    "ပ ကို x ၏ လိပ်စာ ဟု ထားမည်။",
    "ပ ကို ပ + ၁ ဟု ထားမည်။",
    "၄၀၉၆ ကို ကိန်းပြည့် လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။",
    "လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။",
];

#[test]
fn pointer_work_needs_marked_code()
{
    for line in WORK
    {
        let outside = program("", &format!("{BLOCK}\n{line}"));
        assert_eq!(codes_as_is(&outside), [Code::UnsafeOutside], "{line}");
        let inside = program("", &format!("{BLOCK}\n{}", in_unsafe_block(line)));
        assert_eq!(codes_as_is(&inside), [], "{line}");
    }
}

#[test]
fn a_pointer_value_passes_through_safe_code()
{
    let steps = format!(
        "{BLOCK}\nကိန်းရှင် q သည် ပ ဖြစ်ပါစေ။\nအကယ်၍ q == မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ ဖြစ်လျှင် \
         \"မညွှန်း\" ဟု ပြသမည်။"
    );
    assert_eq!(codes_as_is(&program("", &steps)), []);
}

/// An unsafe function `ဂဏန်း` that makes a block the caller owns.
const MAKER: &str = "စိတ်မချရသော လုပ်ဆောင်ချက် ဂဏန်း သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
ကိန်းရှင် အဖြေ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို အဖြေ တွင် သိမ်းထားမည်။
အဖြေ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";

#[test]
fn a_marked_function_is_called_from_marked_code()
{
    let call = "ကိန်းရှင် ပ သည် ဂဏန်း ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။";
    assert_eq!(codes_as_is(&program(MAKER, call)), [Code::UnsafeOutside]);
    assert_eq!(codes_as_is(&program(MAKER, &in_unsafe_block(call))), []);
    let value = "ကိန်းရှင် ဖ သည် ဂဏန်း ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။";
    assert_eq!(codes_as_is(&program(MAKER, value)), [Code::UnsafeFnValue]);
}

#[test]
fn a_function_inside_marked_code_is_marked_code()
{
    let inner = "လုပ်ဆောင်ချက် အတွင်း သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";
    assert_eq!(codes_as_is(&program("", &in_unsafe_block(inner))), []);
}

#[test]
fn the_owned_mark_stands_on_pointers_alone()
{
    let param = "စိတ်မချရသော လုပ်ဆောင်ချက် ယူ သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";
    assert_eq!(codes_as_is(&program(param, "")), [Code::OwnedNotPtr]);
    let result = "လုပ်ဆောင်ချက် ငါး သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။
၅ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";
    assert_eq!(codes_as_is(&program(result, "")), [Code::OwnedNotPtr]);
    assert_eq!(codes_as_is(&program(MAKER, "")), []);
}

#[test]
fn no_unsafe_refuses_the_marker()
{
    let options = SemcheckOptions::builder().no_unsafe(true).build();
    let found = |text: &str| -> Vec<Code> {
        check_as_is(text, &options).out.diagnostics.iter().map(|d| d.code).collect()
    };
    assert_eq!(found(&program(MAKER, "")), [Code::UnsafeRefused]);
    assert_eq!(found(&program("", &in_unsafe_block(""))), [Code::UnsafeRefused]);
}

/// The block of a conditional or of each loop can open with the unsafe
/// opener, and its statements are unsafe code.
#[test]
fn branches_and_loops_take_marked_blocks()
{
    let open = "အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။";
    let close = "လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";
    let heads = [
        "အကယ်၍ x > ၀ ဖြစ်လျှင်",
        "x သည် ၁၀ အောက် ငယ်နေသရွေ့",
        "၁ မှ ၂ အထိ န အနေဖြင့် ရေတွက်၍",
        "ကိန်းရှင် i သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ i သည် ၂ အောက် ငယ်နေသရွေ့",
        "[၁၊ ၂] ၏ အကွက်တိုင်း ကို c အနေဖြင့် ယူ၍",
    ];
    let work = "ပ ကို x ၏ လိပ်စာ ဟု ထားမည်။";
    for head in heads
    {
        let steps = format!("{head} {open}\n{work}\n{close}");
        assert_eq!(codes_as_is(&program(BLOCK, &steps)), [], "{head}");
        let plain = format!("{head} အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{work}\n{close}");
        assert_eq!(
            codes_as_is(&program(BLOCK, &plain)),
            [Code::UnsafeOutside],
            "{head}"
        );
    }
}
