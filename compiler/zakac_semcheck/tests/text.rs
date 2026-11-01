mod common;

use common::check;
use common::codes;
use common::kind_at;
use common::ok;
use zakac_diagnostics::Code;
use zakac_semcheck::kind::DataKind;

/// A text and a view of it.
const TEXT: &str = "ကိန်းရှင် t သည် စာသား ဖြစ်၍ တန်ဖိုးသည် \"က {၁ + ၁} ခ\" ဖြစ်ပါစေ။
ကိန်းရှင် v သည် စာသား တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် t ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

#[test]
fn text_kinds_read_back_from_their_names()
{
    let text = DataKind::parse("text").expect("text");
    let view = DataKind::parse("&text").expect("&text");
    assert_eq!(text, DataKind::Text);
    assert_eq!(view, DataKind::text_view());
    assert_eq!(text.size(64), 24);
    assert_eq!(text.size(32), 12);
    assert!(text.is_move_kind() && text.is_aggregate() && !text.is_borrow());
    assert!(text.holds_text() && DataKind::parse("[2]text").unwrap().holds_text());
    assert!(view.is_borrow() && !view.holds_text());
    assert!(view.widens_to(DataKind::slice_of(DataKind::U8), 64));
    assert!(!DataKind::slice_of(DataKind::U8).widens_to(view, 64));
    assert_eq!(DataKind::parse("*&text").map(DataKind::name), Some("*&text".into()));
}

#[test]
fn literals_joins_and_views_take_their_kinds()
{
    let checked = check(&lines(&[
        TEXT,
        "t + v ကို ပြသပါ။",
        "v ၏ အကွက်အရေအတွက် ကို ပြသပါ။",
        "t ၏ အကွက် ၀ ကို ပြသပါ။",
        "t ၏ အကွက် ၀ မှစ၍ ၁ ကွက် ကို ပြသပါ။",
    ]));
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, "\"က {၁ + ၁} ခ\""), "text");
    assert_eq!(kind_at(&checked, "t + v"), "text");
    assert_eq!(kind_at(&checked, "v ၏ အကွက်အရေအတွက်"), "usize");
    assert_eq!(kind_at(&checked, "t ၏ အကွက် ၀"), "u8");
    assert_eq!(kind_at(&checked, "t ၏ အကွက် ၀ မှစ၍ ၁ ကွက်"), "&text");
}

#[test]
fn template_slots_take_each_kind()
{
    ok("ကိန်းရှင် l သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။\n\"{l} {မှန်} \
        {'က'} {၁.၅}\" ဟု ပြသပါ။");
    assert_eq!(codes("\"{မသိ}\" ဟု ပြသပါ။"), [Code::UndeclaredName]);
}

#[test]
fn text_mistakes()
{
    assert_eq!(codes(&lines(&[TEXT, "t ၏ အကွက် ၀ ကို ၁ ဟု ထားပါ။"])), [Code::TextCellWrite]);
    assert_eq!(codes(&lines(&[TEXT, "v ၏ အကွက် ၀ ကို ၁ ဟု ထားပါ။"])), [Code::TextCellWrite]);
    assert_eq!(codes(&lines(&[TEXT, "t == t ကို ပြသပါ။"])), [Code::OperandKind]);
    assert_eq!(codes(&lines(&[TEXT, "t + ၁ ကို ပြသပါ။"])), [Code::OperandKind]);
}

#[test]
fn characters_and_the_utf8_conversion_take_their_kinds()
{
    let checked = check(&lines(&[
        TEXT,
        "t ၏ စာလုံးအရေအတွက် ကို ပြသပါ။",
        "v ၏ စာလုံး ၀ ကို ပြသပါ။",
        "t ၏ ၁ ခုမြောက်စာလုံး ကို ပြသပါ။",
    ]));
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, "t ၏ စာလုံးအရေအတွက်"), "usize");
    assert_eq!(kind_at(&checked, "v ၏ စာလုံး ၀"), "char");
    assert_eq!(kind_at(&checked, "t ၏ ၁ ခုမြောက်စာလုံး"), "char");
    let bytes = DataKind::slice_of(DataKind::U8);
    assert!(bytes.cast_allowed(DataKind::text_view()));
    assert!(!DataKind::slice_of(DataKind::ISIZE).cast_allowed(DataKind::text_view()));
    assert_eq!(
        codes("ကိန်းရှင် n သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။\nn ၏ စာလုံး ၀ ကို ပြသပါ။"),
        [Code::OperandKind]
    );
}

#[test]
fn a_verbal_join_takes_text_alone()
{
    let checked = check(&lines(&[TEXT, "t၊ v၊ \"!\" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသပါ။"]));
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, "t၊ v၊ \"!\" တို့ကို ဆက်ထားသည့် စာသား"), "text");
    assert_eq!(codes("\"a\"၊ ၁ တို့ကို ဆက်ထားသည့် စာသား ကို ပြသပါ။"), [Code::OperandKind]);
}
