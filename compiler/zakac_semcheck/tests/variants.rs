//! Members of an enumeration that hold data: the kinds, the tests, the reads
//! and the names a test gives the data.

use zakac_diagnostics::Code;
use zakac_rust_sugars::do_for;

use crate::common::codes;
use crate::common::ok;

/// An enumeration whose first member holds an integer, and a value of it.
const SHAPE: &str = "ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့် အမှတ် \
                     တို့ပါဝင်သည်။
ကိန်းရှင် ပ သည် ပုံများထဲမှ စက်ဝိုင်း(၅) ဖြစ်ပါစေ။";

/// The program of the shape with more lines.
fn with(line: &str) -> String { format!("{SHAPE}\n{line}") }

#[test]
fn values_tests_reads_and_names_check()
{
    ok(&with("ကိန်းရှင် ခ သည် ဒေတာအနေဖြင့် ၃ ရှိသော ပုံများထဲမှ စက်ဝိုင်း ဖြစ်ပါစေ။"));
    ok(&with("ပ သည် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်သည် ဟု ပြသမည်။"));
    ok(&with("ပ ကို စက်ဝိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။"));
    ok(&with("အကယ်၍ ပ သည် ရ ဟူသည့် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် ရ + ၁ ကို ပြသမည်။"));
    ok(&with(
        "ပ သည် ရ ဟူသည့် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် ရ ကို ပြသမည်။\nအမှတ် ဖြစ်လျှင် \"အမှတ်\" ဟု ပြသမည်။",
    ));
    ok(&with("(ပ == ပုံများထဲမှ စက်ဝိုင်း(၅)) ကို ပြသမည်။"));
}

#[test]
fn values_and_data_do_not_mix_zk0353()
{
    let text = "ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့် တန်ဖိုးအနေဖြင့် \
                ၁ ရှိသည့် အမှတ် တို့ပါဝင်သည်။";
    assert_eq!(codes(text), [Code::ValueAndData]);
}

#[test]
fn data_goes_where_a_member_holds_some()
{
    let none = with("ကိန်းရှင် ခ သည် ပုံများထဲမှ စက်ဝိုင်း ဖြစ်ပါစေ။");
    assert_eq!(codes(&none), [Code::VariantNeedsData]);
    let given = with("ကိန်းရှင် ခ သည် ပုံများထဲမှ အမှတ်(၁) ဖြစ်ပါစေ။");
    assert_eq!(codes(&given), [Code::VariantHasNoData]);
    let read = with("ပ ကို အမှတ် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။");
    assert_eq!(codes(&read), [Code::VariantHasNoData]);
    let named = with("အကယ်၍ ပ သည် ရ ဟူသည့် အမှတ် အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် \"x\" ဟု ပြသမည်။");
    assert_eq!(codes(&named), [Code::VariantHasNoData]);
}

#[test]
fn a_test_looks_at_an_enumeration_zk0356()
{
    let text = "ကိန်းရှင် ခ သည် ၅ ဖြစ်ပါစေ။\nခ သည် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်သည် ဟု ပြသမည်။";
    assert_eq!(codes(text), [Code::NotAnEnumValue]);
    assert_eq!(
        codes(&with("ပ ကို တြိဂံ အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။")),
        [Code::UnknownMember]
    );
}

#[test]
fn a_name_for_data_stands_in_a_whole_condition_zk0357()
{
    let value = with("ကိန်းရှင် ခ သည် ပ သည် ရ ဟူသည့် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်သည် ဖြစ်ပါစေ။");
    assert_eq!(codes(&value), [Code::NameOutsideTest]);
}

#[test]
fn a_read_a_test_ruled_out_is_zk0358()
{
    let text = with(
        "ပ သည် အမှတ် ဖြစ်လျှင် ပ ကို စက်ဝိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။\nတခြား ပုံ တစ်ခု ဖြစ်လျှင် \
         \"x\" ဟု ပြသမည်။",
    );
    assert_eq!(codes(&text), [Code::WrongVariantProven]);
    ok(&with(
        "အကယ်၍ ပ သည် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် ပ ကို စက်ဝိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။",
    ));
}

#[test]
fn a_switch_names_members_with_no_data()
{
    let data = with(
        "ပ သည် ပုံများထဲမှ စက်ဝိုင်း(၅) ဖြစ်လျှင် \"x\" ဟု ပြသမည်။\nတခြား ပုံ တစ်ခု ဖြစ်လျှင် \"z\" ဟု \
         ပြသမည်။",
    );
    assert_eq!(codes(&data), [Code::CaseWithData]);
    let missing = with("ပ သည် စက်ဝိုင်း ဖြစ်လျှင် \"x\" ဟု ပြသမည်။");
    assert_eq!(codes(&missing), [Code::SwitchMissing]);
}

#[test]
fn a_kind_builds_through_one_member()
{
    let tree = "ဖော်ပြချက် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် ကိန်း နှင့် \
                ဒေတာအနေဖြင့် အပေါင်း တစ်ခု ရှိသည့် ပေါင်း တို့ပါဝင်သည်။
အပေါင်း ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် ဘယ် ဟူသော ဖော်ပြချက် တစ်ခု နှင့် ညာ ဟူသော ဖော်ပြချက် တစ်ခု \
                တို့ပါဝင်သည်။
ကိန်းရှင် ဖ သည် ဖော်ပြချက်များထဲမှ ကိန်း(၁) ဖြစ်ပါစေ။";
    ok(tree);
    let itself = "ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ပုံ တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့် အမှတ် တို့ပါဝင်သည်။";
    assert_eq!(codes(itself), [Code::StructHoldsItself]);
}

#[test]
fn the_initial_member_needs_a_mark()
{
    let text = with("ကိန်းရှင် ခ သည် ပုံ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။");
    assert_eq!(codes(&text), [Code::NoInitialValue]);
}

#[test]
fn a_field_names_the_enumeration_through_its_stem()
{
    let text = "အမှားများ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် စာသား တစ်ခု ရှိသည့် မတွေ့ နှင့် ပိတ်ထား \
                တို့ပါဝင်သည်။
မှတ်တမ်း ဟူသော ဒေတာအစုအဝေးတွင် ပြဿနာ ဟူသော အမှား တစ်ခု ပါဝင်သည်။
ကိန်းရှင် မ သည် မှတ်တမ်း(ပြဿနာ: အမှားများထဲမှ မတွေ့(\"a\")) ဖြစ်ပါစေ။";
    ok(text);
}

/// A generic enumeration.
const MAYBE: &str = "ဖြစ်နိုင် ဟူသော ဒေတာအမျိုးအစားကွဲသည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ \
                     ၎င်းတွင် ဒေတာအနေဖြင့် T တစ်ခု ရှိသည့် တွေ့ နှင့် မတွေ့ တို့ပါဝင်သည်။";

/// The generic enumeration with more lines.
fn maybe(line: &str) -> String { format!("{MAYBE}\n{line}") }

#[test]
fn a_generic_member_takes_its_kinds_from_data_place_or_words()
{
    ok(&maybe("ကိန်းရှင် က သည် ဖြစ်နိုင်များထဲမှ တွေ့(၅) ဖြစ်ပါစေ။"));
    ok(&maybe("ကိန်းရှင် က သည် ဖြစ်နိုင်<စာသား> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဖြစ်နိုင်များထဲမှ မတွေ့ ဖြစ်ပါစေ။"));
    ok(&maybe("ကိန်းရှင် က သည် ဖြစ်နိုင်<စာသား>များထဲမှ မတွေ့ ဖြစ်ပါစေ။"));
    ok(&maybe("ကိန်းရှင် က သည် စာသား ကို အစားထိုးအသုံးပြုသည့် ဖြစ်နိုင်များထဲမှ မတွေ့ ဖြစ်ပါစေ။"));
    ok(&maybe(
        "ကိန်းရှင် က သည် ဖြစ်နိုင်များထဲမှ တွေ့(၅) ဖြစ်ပါစေ။\nက သည် တွေ့ ဖြစ်လျှင် \"x\" ဟု ပြသမည်။\nတခြား \
         ဖြစ်နိုင် တစ်ခု ဖြစ်လျှင် \"z\" ဟု ပြသမည်။",
    ));
}

#[test]
fn generic_member_kinds_are_checked()
{
    assert_eq!(codes(&maybe("ဖြစ်နိုင်များထဲမှ မတွေ့ ကို ပြသမည်။")), [Code::KindsUnknown]);
    let count = maybe("ကိန်းရှင် က သည် ဖြစ်နိုင်<ကိန်းပြည့်၊ စာသား>များထဲမှ မတွေ့ ဖြစ်ပါစေ။");
    assert_eq!(codes(&count), [Code::KindArgCount]);
    let data = maybe(
        "ကိန်းရှင် က သည် ဖြစ်နိုင်<ကိန်းပြည့်> တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဖြစ်နိုင်များထဲမှ တွေ့(\"က\") ဖြစ်ပါစေ။",
    );
    assert_eq!(codes(&data), [Code::KindMismatch]);
}

#[test]
fn a_case_names_a_generic_member_with_or_with_no_kinds()
{
    let head = "ကိန်းရှင် က သည် ဖြစ်နိုင်များထဲမှ တွေ့(၅) ဖြစ်ပါစေ။\nက သည် ";
    let tail = " ဖြစ်လျှင် \"x\" ဟု ပြသမည်။\nတခြား ဖြစ်နိုင် တစ်ခု ဖြစ်လျှင် \"z\" ဟု ပြသမည်။";
    let cases = [
        "တွေ့",
        "ဖြစ်နိုင်များထဲမှ တွေ့",
        "ဖြစ်နိုင်<ကိန်းပြည့်>များထဲမှ တွေ့",
        "ကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် ဖြစ်နိုင်များထဲမှ တွေ့",
    ];
    do_for!(case in cases, ok(&maybe(&format!("{head}{case}{tail}"))));
}
