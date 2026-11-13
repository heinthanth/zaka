use zakac_diagnostics::Code;

use crate::common::codes;
use crate::common::stmts;

/// A program that must parse with no diagnostics.
fn parses(text: &str) { stmts(text); }

/// A list in words joins its last item with `နှင့်`, in each place a list
/// stands: cells, rows, pieces of a join, arguments, imported names, fields,
/// members and switch cases.
#[test]
fn lists_join_their_last_item_with_hnint()
{
    parses("ကိန်းရှင် က သည် ၇ ပါသော စာရင်း ဖြစ်ပါစေ။");
    parses("ကိန်းရှင် က သည် ၁ နှင့် ၂ တို့ပါသော စာရင်း ဖြစ်ပါစေ။");
    parses("ကိန်းရှင် က သည် ၁၊ ၂ နှင့် ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။");
    parses("ကိန်းရှင် ဇ သည် \"က\" အနေဖြင့် ၁ ပါသော ဇယား ဖြစ်ပါစေ။");
    parses("ကိန်းရှင် ဇ သည် \"က\" အနေဖြင့် ၁ နှင့် \"ခ\" အနေဖြင့် ၂ တို့ပါသော ဇယား ဖြစ်ပါစေ။");
    parses("\"က\"၊ \"ခ\" နှင့် \"ဂ\" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသမည်။");
    parses("၁ ကို သုံး၍ ပြ မည်။");
    parses("၁ နှင့် ၂ တို့ကို သုံး၍ ပြ မည်။");
    parses("ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။");
    parses("ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁၊ ၂ နှင့် ၃ တို့ဖြင့် လုပ်ဆောင်မည်။");
    parses("\"စံကား/အဝင်အထွက်\" မှ ရေး ကို ထည့်သွင်းမည်။");
    parses("\"စံကား/အဝင်အထွက်\" မှ ရေး နှင့် စာကြောင်းတစ်ကြောင်းဖတ် တို့ကို ထည့်သွင်းမည်။");
    parses("မီး ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ နှင့် စိမ်း တို့ပါဝင်သည်။");
    parses("အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု နှင့် y ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။");
    parses("ရက် သည် ၀ နှင့် ၆ တို့ ဖြစ်လျှင် ၁ ကို ပြသမည်။\n၁၊ ၂ နှင့် ၃ တို့ ဖြစ်လျှင် ၂ ကို ပြသမည်။");
}

/// Two or more items without `နှင့်` before the last, or with a `၊` after
/// it, are ZK0245.
#[test]
fn a_list_without_hnint_is_zk0245()
{
    let needs = [Code::ListNeedsAnd];
    assert_eq!(codes("ကိန်းရှင် က သည် ၁၊ ၂ တို့ပါသော စာရင်း ဖြစ်ပါစေ။"), needs);
    assert_eq!(codes("ကိန်းရှင် က သည် ၁ နှင့် ၂၊ ၃ တို့ပါသော စာရင်း ဖြစ်ပါစေ။"), needs);
    assert_eq!(codes("ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁၊ ၂ တို့ဖြင့် လုပ်ဆောင်မည်။"), needs);
    assert_eq!(codes("မီး ဟူသော ဒေတာအမျိုးအစားကွဲတွင် နီ၊ ဝါ တို့ပါဝင်သည်။"), needs);
}

/// One item takes the word with no `တို့`, two or more the word with it:
/// ZK0246 for the other.
#[test]
fn the_list_word_fits_the_count_zk0246()
{
    let count = [Code::ListWordCount];
    assert_eq!(codes("ကိန်းရှင် က သည် ၇ တို့ပါသော စာရင်း ဖြစ်ပါစေ။"), count);
    assert_eq!(codes("ကိန်းရှင် က သည် ၁ နှင့် ၂ ပါသော စာရင်း ဖြစ်ပါစေ။"), count);
    assert_eq!(codes("ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁ တို့ဖြင့် လုပ်ဆောင်မည်။"), count);
    assert_eq!(codes("ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁ နှင့် ၂ ဖြင့် လုပ်ဆောင်မည်။"), count);
    assert_eq!(codes("၁ နှင့် ၂ ကို သုံး၍ ပြ မည်။"), count);
    assert_eq!(codes("\"စံကား/အဝင်အထွက်\" မှ ရေး တို့ကို ထည့်သွင်းမည်။"), count);
}

/// An item with a `နှင့်` of its own stands in parentheses, so the `နှင့်` of
/// the list is never read as part of it.
#[test]
fn an_item_with_its_own_hnint_stands_in_parentheses()
{
    parses("ပြ ဟူသော လုပ်ဆောင်ချက်ကို (၁ နှင့် ၂ ကို ပေါင်းသည့်ရလဒ်) ဖြင့် လုပ်ဆောင်မည်။");
    parses("ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၃ နှင့် (၁ နှင့် ၂ ကို ပေါင်းသည့်ရလဒ်) တို့ဖြင့် လုပ်ဆောင်မည်။");
    let bare = "ပြ ဟူသော လုပ်ဆောင်ချက်ကို ၁ နှင့် ၂ ကို ပေါင်းသည့်ရလဒ် ဖြင့် လုပ်ဆောင်မည်။";
    assert!(!codes(bare).is_empty(), "{bare}");
}

/// A struct literal in words names each field with `အနေဖြင့်`; the bare
/// field and value of the old form do not parse.
#[test]
fn struct_values_name_each_field_with_as()
{
    let decl =
        "လူ ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု နှင့် အသက် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။\n";
    parses(&format!("{decl}နာမည် အနေဖြင့် \"မမ\" ရှိသော လူတစ်ခု ကို ပြသမည်။"));
    parses(&format!("{decl}နာမည် အနေဖြင့် \"မမ\" နှင့် အသက် အနေဖြင့် ၂၀ ရှိသော လူတစ်ခု ကို ပြသမည်။"));
    let old = format!("{decl}နာမည် \"မမ\" ရှိသော လူတစ်ခု ကို ပြသမည်။");
    assert!(!codes(&old).is_empty(), "{old}");
}
