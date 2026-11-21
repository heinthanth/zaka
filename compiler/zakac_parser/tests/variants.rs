//! Members of an enumeration that hold data: the declaration, the two forms
//! of a value, the test, the read, and the arms of a switch.

use zakac_diagnostics::Code;

use crate::common::codes;
use crate::common::expr;
use crate::common::stmt;
use crate::common::stmts;

/// An enumeration whose first member holds an integer.
const SHAPE: &str =
    "ပုံ ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် ကိန်းပြည့် တစ်ခု ရှိသည့် စက်ဝိုင်း နှင့် အမှတ် တို့ပါဝင်သည်။";

#[test]
fn a_member_takes_the_kind_of_its_data()
{
    let tree = stmt(SHAPE);
    assert!(tree.starts_with("(enum ပုံ [စက်ဝိုင်း:"), "{tree}");
    assert!(tree.ends_with(", အမှတ်])"), "{tree}");
}

#[test]
fn a_value_gives_its_data_in_parentheses_or_in_words()
{
    let notation = expr("ပုံများထဲမှ စက်ဝိုင်း(၅)");
    assert!(notation.starts_with("(member ပုံများ စက်ဝိုင်း "), "{notation}");
    let words = expr("ဒေတာအနေဖြင့် ၅ ရှိသော ပုံများထဲမှ စက်ဝိုင်း");
    assert_eq!(words, notation);
    assert_eq!(expr("ပုံများထဲမှ အမှတ်"), "(member ပုံများ အမှတ်)");
}

#[test]
fn the_words_form_holds_one_pair_of_data()
{
    let two = format!(
        "{SHAPE}\nကိန်းရှင် က သည် ဒေတာအနေဖြင့် ၅ နှင့် x အနေဖြင့် ၁ ရှိသော ပုံများထဲမှ စက်ဝိုင်း ဖြစ်ပါစေ။"
    );
    assert_eq!(codes(&two), [Code::ExpectedToken]);
}

#[test]
fn a_test_names_the_member_and_can_name_its_data()
{
    let plain = expr("ပ သည် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်သည်");
    assert!(plain.contains("(variant စက်ဝိုင်း)"), "{plain}");
    let named = expr("ပ သည် ရ ဟူသည့် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်သည်");
    assert!(named.contains("(variant ရ စက်ဝိုင်း)"), "{named}");
    let not = expr("ပ သည် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်သည် မဟုတ်");
    assert!(not.starts_with("(not "), "{not}");
}

#[test]
fn a_read_converts_to_the_member()
{
    let read = expr("ပ ကို စက်ဝိုင်း အဖြစ် ပြောင်းလဲသော ရလဒ်");
    assert!(read.contains("variant_data"), "{read}");
    assert!(read.contains("(variant စက်ဝိုင်း)"), "{read}");
}

#[test]
fn a_switch_mixes_test_arms_and_plain_arms()
{
    let text = format!(
        "{SHAPE}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းရှင် ပ သည် ပုံများထဲမှ အမှတ် ဖြစ်ပါစေ။\nပ သည် ရ \
         ဟူသည့် စက်ဝိုင်း အဖြစ်သို့ ပြောင်းလဲနိုင်လျှင် ရ ကို ပြသမည်။\nအမှတ် ဖြစ်လျှင် \"အမှတ်\" ဟု ပြသမည်။\nပရိုဂရမ် \
         ပြီးဆုံးပါပြီး။"
    );
    let all = stmts(&text);
    let start =
        all.last().map(zakac_ast::s_expr::ToSexpr::to_sexpr).unwrap_or_default();
    assert!(start.contains("(switch ပ (variant ရ စက်ဝိုင်း)"), "{start}");
    assert!(start.contains("အမှတ် (block"), "{start}");
}

/// A generic enumeration in the word form.
const MAYBE: &str = "ဖြစ်နိုင် ဟူသော ဒေတာအမျိုးအစားကွဲသည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ \
                     ၎င်းတွင် ဒေတာအနေဖြင့် T တစ်ခု ရှိသည့် တွေ့ နှင့် မတွေ့ တို့ပါဝင်သည်။";

#[test]
fn a_generic_enumeration_takes_kind_parameters_in_both_forms()
{
    let words = stmt(MAYBE);
    assert!(words.starts_with("(enum ဖြစ်နိုင်<T> [တွေ့:"), "{words}");
    let sign =
        stmt("ဖြစ်နိုင်<T> ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ဒေတာအနေဖြင့် T တစ်ခု ရှိသည့် တွေ့ နှင့် မတွေ့ တို့ပါဝင်သည်။");
    assert_eq!(sign, words);
}

#[test]
fn a_member_writes_its_kinds_in_both_forms()
{
    let sign = format!(
        "{MAYBE}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းရှင် က သည် ဖြစ်နိုင်<စာသား>များထဲမှ မတွေ့ \
         ဖြစ်ပါစေ။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။"
    );
    let words = sign.replace("ဖြစ်နိုင်<စာသား>များထဲမှ", "စာသား ကို အစားထိုးအသုံးပြုသည့် ဖြစ်နိုင်များထဲမှ");
    let tree = |t: &str| {
        stmts(t).last().map(zakac_ast::s_expr::ToSexpr::to_sexpr).unwrap_or_default()
    };
    let (a, b) = (tree(&sign), tree(&words));
    assert!(a.contains("(member ဖြစ်နိုင်များ<"), "{a}");
    assert_eq!(a, b);
}
