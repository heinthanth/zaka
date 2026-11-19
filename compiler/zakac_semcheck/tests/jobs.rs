//! The mark of objects jobs can use, and the built-in trait of the kinds
//! jobs can use, as a bound of a generic.

use zakac_diagnostics::Code;

use crate::common::check_as_is;

/// A generic function that gives back its argument, for kinds jobs can use.
const PASS_ON: &str = "ပြန် ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
တန် ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
T တွင် အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော ဂုဏ်သတ္တိ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
တန် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// The codes of a program of a struct head, the generic, and a start block
/// that passes a number and an object of the struct to it.
///
/// # Arguments
///
/// * `marks` - The marks of the struct `အကောင့်`, between `ဟူသော` and
///   `ဒေတာအစုအဝေး`.
///
/// # Returns
///
/// The codes.
fn codes(marks: &str) -> Vec<Code>
{
    let text = format!(
        "အကောင့် ဟူသော {marks} ဒေတာအစုအဝေးတွင် လက်ကျန် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

{PASS_ON}
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြန်(၇) ကို ပြသမည်။
ကိန်းရှင် က သည် အကောင့်(လက်ကျန်: ၅) ဖြစ်ပါစေ။
ပြန်(က) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
"
    );
    let checked = check_as_is(&text, &Default::default());
    checked.out.diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn an_object_with_the_mark_meets_the_bound()
{
    assert!(codes("အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော မျှဝေနိုင်သော").is_empty());
}

#[test]
fn the_words_of_the_mark_read_glued()
{
    assert!(codes("အပြိုင်လုပ်ငန်းများတွင်အသုံးပြုနိုင်သော မျှဝေနိုင်သော").is_empty());
}

#[test]
fn a_plain_object_does_not_meet_the_bound()
{
    assert_eq!(codes("မျှဝေနိုင်သော"), [Code::BoundNotMet]);
}

#[test]
fn a_value_meets_the_bound_with_no_mark()
{
    assert!(codes("").is_empty());
}

#[test]
fn the_mark_on_a_value_is_zk0493()
{
    assert_eq!(codes("အပြိုင်လုပ်ငန်းများတွင် အသုံးပြုနိုင်သော"), [Code::JobsMarkOnValue]);
}

#[test]
fn the_words_of_the_mark_remain_free_names()
{
    let text = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် အသုံးပြုနိုင်သော သည် ၁ ဖြစ်ပါစေ။
အသုံးပြုနိုင်သော ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";
    let checked = check_as_is(text, &Default::default());
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
}
