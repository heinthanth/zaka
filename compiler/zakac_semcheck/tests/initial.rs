//! `ကနဦးတန်ဖိုး`, the written initial value, and the kinds no program can
//! build.

use zakac_diagnostics::Code;

use crate::common::check;
use crate::common::codes;
use crate::common::kind_at;
use crate::common::ok;

/// An enumeration with a marked member, a struct with an optional, a shared
/// struct, and a struct with a function field.
const KINDS: &str = "အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် တန်ဖိုးအနေဖြင့် ၁ ရှိသည့် နီ၊ ကနဦးတန်ဖိုးအနေဖြင့် \
                     သုံးမည့် တန်ဖိုးအနေဖြင့် ၀ ရှိသည့် စိမ်း၊ တန်ဖိုးအနေဖြင့် ၂ ရှိသည့် ပြာ တို့ပါဝင်သည်။
လူ ဟူသော ဒေတာအစုအဝေးတွင် အမည် ဟူသော စာသား တစ်ခု၊ ကြိုက် ဟူသော အရောင် တစ်ခု၊ ဖုန်း ဟူသော ကိန်းပြည့် \
                     ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။
ဘူး ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
ခလုတ် ဟူသော ဒေတာအစုအဝေးတွင် လုပ် ဟူသော (ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ပါဝင်သည်။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

#[test]
fn the_initial_value_takes_the_kind_of_its_place()
{
    let text = lines(&[
        KINDS,
        "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။",
        "ကိန်းရှင် ခ သည် လူ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။",
        "ကိန်းရှင် ဂ သည် အကွက် ၃ ခုရှိသော အရောင် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။",
        "ကိန်းရှင် ဃ သည် ဘူး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။",
        "က ကို ကနဦးတန်ဖိုး ဟု ထားမည်။",
    ]);
    let checked = check(&text);
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, "ကနဦးတန်ဖိုး"), "isize");
    ok(&lines(&[
        KINDS,
        "ကိန်းရှင် လ သည် လူ(အမည်: \"က\"၊ ကြိုက်: ကနဦးတန်ဖိုး၊ ဖုန်း: ကနဦးတန်ဖိုး) ဖြစ်ပါစေ။",
    ]));
}

#[test]
fn a_kind_with_no_initial_value_is_zk0479()
{
    let no_mark = "ဘက် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဘယ်၊ ညာ တို့ပါဝင်သည်။";
    let cases = [
        "ကိန်းရှင် က သည် ခလုတ် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။",
        "ကိန်းရှင် က သည် (ကိန်းပြည့် တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး \
         ဖြစ်ပါစေ။",
        "ကိန်းရှင် က သည် ဘက် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။",
    ];
    for case in cases
    {
        assert_eq!(
            codes(&lines(&[KINDS, no_mark, case])),
            [Code::NoInitialValue],
            "{case}"
        );
    }
}

#[test]
fn an_initial_value_with_no_kind_is_zk0478()
{
    assert_eq!(codes("ကနဦးတန်ဖိုး ကို ပြသမည်။"), [Code::InitialNeedsKind]);
}

#[test]
fn a_second_initial_member_is_zk0480()
{
    let text = "တံခါး ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် ဖွင့်၊ ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့် \
                ပိတ် တို့ပါဝင်သည်။";
    assert_eq!(codes(text), [Code::TwoInitialMembers]);
}

/// A loop of fields with no optional field and no plain list on it is one
/// ZK0481, whatever the count of kinds it passes; a kind that holds a kind of
/// the loop gets none of its own; an optional field or a plain list ends the
/// chain.
#[test]
fn a_kind_no_program_can_build_is_zk0481()
{
    let shared = "ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင်";
    let selfish =
        format!("အဆစ် {shared} တန် ဟူသော ကိန်းပြည့် တစ်ခု၊ နောက် ဟူသော အဆစ် တစ်ခု တို့ပါဝင်သည်။");
    assert_eq!(codes(&selfish), [Code::KindNeverBuilt]);
    let three = lines(&[
        &format!("က {shared} ဘေး ဟူသော ခ တစ်ခု ပါဝင်သည်။"),
        &format!("ခ {shared} ဘေး ဟူသော ဂ တစ်ခု ပါဝင်သည်။"),
        &format!("ဂ {shared} ဘေး ဟူသော က တစ်ခု ပါဝင်သည်။"),
        &format!("ဃ {shared} ကိုင် ဟူသော က တစ်ခု ပါဝင်သည်။"),
    ]);
    assert_eq!(codes(&three), [Code::KindNeverBuilt]);
    let ended = lines(&[
        &format!(
            "သစ်ပင် {shared} အမည် ဟူသော စာသား တစ်ခု၊ ကလေးများ ဟူသော သစ်ပင် စာရင်း တစ်ခု တို့ပါဝင်သည်။"
        ),
        &format!("ဆ {shared} ဘေး ဟူသော ဆ ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ပါဝင်သည်။"),
    ]);
    ok(&ended);
}
