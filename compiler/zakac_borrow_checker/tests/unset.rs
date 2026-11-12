//! The check that each value is set before it is read: fields of structs
//! inside structs, and earlier cells of a counted loop that fills its list.

mod common;

use common::codes;
use zakac_diagnostics::Code;

/// A point inside a line, so a struct holds a struct.
const KINDS: &str = "အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု၊ y ဟူသော ကိန်းပြည့် တစ်ခု \
                     တို့ပါဝင်သည်။
မျဉ်း ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော ကိန်းပြည့် တစ်ခု၊ ဆုံး ဟူသော အမှတ် တစ်ခု တို့ပါဝင်သည်။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

/// A struct inside a struct is followed one leaf field at a time: the inner
/// struct is set once each of its fields is, and the outer one once each leaf
/// is.
#[test]
fn a_struct_inside_a_struct_is_set_one_leaf_at_a_time()
{
    let decl = "ကိန်းရှင် က သည် မျဉ်း တစ်ခု ဖြစ်ပါစေ။";
    let inner = lines(&[
        KINDS,
        decl,
        "က ၏ ဆုံး ၏ x ကို ၁ ဟု ထားမည်။",
        "က ၏ ဆုံး ၏ x ကို ပြသမည်။",
        "က ၏ ဆုံး ၏ y ကို ၂ ဟု ထားမည်။",
        "က ၏ ဆုံး ကို ပြသမည်။",
    ]);
    assert_eq!(codes(&inner), []);
    let whole = format!("{inner}\nက ကို ပြသမည်။");
    assert_eq!(codes(&whole), [Code::ReadUnset]);
    let all = format!("{inner}\nက ၏ အမည် ကို ၉ ဟု ထားမည်။\nက ကို ပြသမည်။");
    assert_eq!(codes(&all), []);
}

/// Inside a counted loop that writes the cell of its counter on each round, a
/// read of `a[i-k]` is set when cells `s-k` to `s-1` are set before the loop;
/// a cell that nothing sets is ZK0722. The cells the loop covers count as set
/// after it.
#[test]
fn earlier_cells_of_a_filling_loop_are_set()
{
    let fib = lines(&[
        "ကိန်းရှင် ဖ သည် အကွက် ၁၀ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။",
        "ဖ ၏ အကွက် ၀ ကို ၁ ဟု ထားမည်။",
        "ဖ ၏ အကွက် ၁ ကို ၁ ဟု ထားမည်။",
        "၂ မှ ၉ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "ဖ ၏ အကွက် န ကို (ဖ ၏ အကွက် (န - ၁)) + (ဖ ၏ အကွက် (န - ၂)) ဟု ထားမည်။",
        "လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။",
        "ဖ ကို ပြသမည်။",
    ]);
    assert_eq!(codes(&fib), []);
    let gap = lines(&[
        "ကိန်းရှင် ဂ သည် အကွက် ၅ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။",
        "ဂ ၏ အကွက် ၀ ကို ၁ ဟု ထားမည်။",
        "၂ မှ ၄ အထိ န အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။",
        "ဂ ၏ အကွက် န ကို ဂ ၏ အကွက် (န - ၂) ဟု ထားမည်။",
        "လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။",
    ]);
    assert_eq!(codes(&gap), [Code::ReadUnset]);
}

/// A function that divides by 3, filling the quotient and the remainder.
const SPLIT: &str = "လုပ်ဆောင်ချက် ခွဲ သည်
တန် ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လဒ် ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း ကြွင်း ဟူသော \
                     ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လဒ် ကို တန် / ၃ ဟု ထားမည်။
ကြွင်း ကို တန် % ၃ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// A parameter marked `ဒေတာ ထည့်ပေးရန်` needs no value from the caller and is
/// set after the call; one marked `ဒေတာ ပြောင်းလဲရန်` needs its value first.
#[test]
fn a_filled_argument_is_set_by_the_call()
{
    let filled = lines(&[
        SPLIT,
        "ကိန်းရှင် လ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။",
        "ကိန်းရှင် ကျ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။",
        "ခွဲ ဟူသော လုပ်ဆောင်ချက်ကို ၁၀၊ လ၊ ကျ တို့ဖြင့် လုပ်ဆောင်မည်။",
        "\"{လ} {ကျ}\" ဟု ပြသမည်။",
    ]);
    assert_eq!(codes(&filled), []);
    let changed = "လုပ်ဆောင်ချက် တိုး သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို က + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ကိန်းရှင် လ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
တိုး ဟူသော လုပ်ဆောင်ချက်ကို လ တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(codes(changed), [Code::ReadUnset]);
}

/// Inside the function, a parameter marked `ဒေတာ ထည့်ပေးရန်` starts unset: a
/// read before a write is ZK0722, and a path that returns without setting it
/// is ZK0725.
#[test]
fn a_filled_parameter_is_set_on_each_path()
{
    let read_first = "လုပ်ဆောင်ချက် ဖတ်ကြည့် သည်
လဒ် ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လဒ် ကို ပြသမည်။
လဒ် ကို ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";
    assert_eq!(codes(read_first), [Code::ReadUnset]);
    let one_path = "လုပ်ဆောင်ချက် တစ်ဝက် သည်
တန် ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လဒ် ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ တန် သည် ၀ ထက် ကြီးလျှင် လဒ် ကို တန် / ၂ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";
    assert_eq!(codes(one_path), [Code::FillUnsetAtReturn]);
}
