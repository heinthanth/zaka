mod common;

use common::check;
use common::codes;
use common::kind_at;
use common::ok;
use zakac_diagnostics::Code;
use zakac_semcheck::kind::DataKind;

/// A text and a view of it.
const TEXT: &str = "ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \"က {၁ + ၁} ခ\" ဖြစ်ပါစေ။
ကိန်းရှင် v သည် t ၏ အကွက် ၀ မှစ၍ ၃ ကွက် ဖြစ်ပါစေ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

#[test]
fn text_kinds_read_back_from_their_names()
{
    let text = DataKind::parse("text").expect("text");
    assert_eq!(text, DataKind::TEXT);
    assert_eq!(text.size(64), 24);
    assert_eq!(text.size(32), 12);
    assert!(text.is_aggregate() && !text.is_borrow());
    assert!(text.holds_text() && DataKind::parse("text[2]").unwrap().holds_text());
    assert_eq!(DataKind::parse("text[..]"), None);
}

/// Text has one kind, and a plain list one form: the names the fixed and
/// heap forms had read as nothing.
#[test]
fn text_and_lists_have_one_managed_form()
{
    let gone = ["textf", "text~", "isize[]~", "isize[_]", "text[0]"];
    assert!(gone.iter().all(|n| DataKind::parse(n).is_none()));
    assert_eq!(DataKind::parse("isize[]").map(DataKind::name), Some("isize[]".into()));
}

#[test]
fn literals_joins_and_views_take_their_kinds()
{
    let checked = check(&lines(&[
        TEXT,
        "t + v ကို ပြသမည်။",
        "v ၏ အကွက်အရေအတွက် ကို ပြသမည်။",
        "t ၏ အကွက် ၀ ကို ပြသမည်။",
        "t ၏ အကွက် ၀ မှစ၍ ၁ ကွက် ကို ပြသမည်။",
    ]));
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, "\"က {၁ + ၁} ခ\""), "text");
    assert_eq!(kind_at(&checked, "t + v"), "text");
    assert_eq!(kind_at(&checked, "v ၏ အကွက်အရေအတွက်"), "usize");
    assert_eq!(kind_at(&checked, "t ၏ အကွက် ၀"), "u8");
    assert_eq!(kind_at(&checked, "t ၏ အကွက် ၀ မှစ၍ ၁ ကွက်"), "text");
}

#[test]
fn template_slots_take_each_kind()
{
    ok("ကိန်းရှင် l သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။\n\"{l} \
        {မှန်} {'က'} {၁.၅}\" ဟု ပြသမည်။");
    assert_eq!(codes("\"{မသိ}\" ဟု ပြသမည်။"), [Code::UndeclaredName]);
}

#[test]
fn text_mistakes()
{
    assert_eq!(codes(&lines(&[TEXT, "t ၏ အကွက် ၀ ကို ၁ ဟု ထားမည်။"])), [Code::TextCellWrite]);
    assert_eq!(codes(&lines(&[TEXT, "v ၏ အကွက် ၀ ကို ၁ ဟု ထားမည်။"])), [Code::TextCellWrite]);
    ok(&lines(&[TEXT, "t == t ကို ပြသမည်။", "t != v ကို ပြသမည်။"]));
    assert_eq!(codes(&lines(&[TEXT, "t < t ကို ပြသမည်။"])), [Code::OperandKind]);
    assert_eq!(codes(&lines(&[TEXT, "t + ၁ ကို ပြသမည်။"])), [Code::OperandKind]);
}

/// An each-character loop walks text or a range of text; an each-cell loop
/// walks a list, a range of one, or the list a pointer points at.
#[test]
fn each_loops_walk_lists_slices_and_text()
{
    let walks = [
        "t ၏ စာလုံးတိုင်း ကို c အနေဖြင့် ယူ၍ c ကို ပြသမည်။",
        "v ၏ စာလုံးတိုင်း ကို c အနေဖြင့် ယူ၍ c ကို ပြသမည်။",
        "ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။",
        "l ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို x + ၁ ဟု ထားမည်။",
        "l ၏ အကွက် ၀ မှစ၍ ၁ ကွက် ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသမည်။",
        "ကိန်းရှင် p သည် ကိန်းပြည့် စာရင်း လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် l ၏ လိပ်စာ ဖြစ်ပါစေ။",
        "p ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသမည်။",
    ];
    ok(&lines(&[&[TEXT], &walks[..]].concat()));
    let wrong = [
        "t ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသမည်။",
        "v ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသမည်။",
        "၅ ၏ စာလုံးတိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသမည်။",
    ];
    let found: Vec<_> = wrong.iter().map(|w| codes(&lines(&[TEXT, w]))).collect();
    assert_eq!(found, [[Code::EachSourceWrong]; 3]);
    let owners = "ကိန်းရှင် ps သည် အကွက် ၁ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း တစ်ခု ဖြစ်ပါစေ။";
    let walk = "ps ၏ အကွက်တိုင်း ကို p အနေဖြင့် ယူ၍ p ကို ပြသမည်။";
    ok(&lines(&[owners, walk]));
}

#[test]
fn characters_and_the_utf8_conversion_take_their_kinds()
{
    let checked = check(&lines(&[
        TEXT,
        "t ၏ စာလုံးအရေအတွက် ကို ပြသမည်။",
        "v ၏ စာလုံး ၀ ကို ပြသမည်။",
        "t ၏ ၁ ခုမြောက်စာလုံး ကို ပြသမည်။",
    ]));
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, "t ၏ စာလုံးအရေအတွက်"), "usize");
    assert_eq!(kind_at(&checked, "v ၏ စာလုံး ၀"), "char");
    assert_eq!(kind_at(&checked, "t ၏ ၁ ခုမြောက်စာလုံး"), "char");
    assert_eq!(
        codes("ကိန်းရှင် n သည် ၁ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\nn ၏ စာလုံး ၀ ကို ပြသမည်။"),
        [Code::OperandKind]
    );
}

#[test]
fn a_verbal_join_takes_text_alone()
{
    let checked = check(&lines(&[TEXT, "t၊ v၊ \"!\" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသမည်။"]));
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, "t၊ v၊ \"!\" တို့ကို ဆက်ထားသည့် စာသား"), "text");
    assert_eq!(codes("\"a\"၊ ၁ တို့ကို ဆက်ထားသည့် စာသား ကို ပြသမည်။"), [Code::OperandKind]);
}

/// A character of text is a place, and text takes an add; a view of text
/// takes neither, and text takes no remove.
#[test]
fn text_takes_character_writes_and_adds()
{
    let text = "ကိန်းရှင် t သည် \"ab\" ဖြစ်ပါစေ။";
    ok(&lines(&[text, "t ၏ ၂ ခုမြောက်စာလုံး ကို 'x' ဟု ထားမည်။", "t ၏ စာလုံး ၀ ကို 'y' ဟု ထားမည်။"]));
    ok(&lines(&[text, "t ထဲသို့ \"cd\" ကို ထည့်မည်။", "t ထဲသို့ 'e' ကို ထည့်မည်။"]));
    // A range of text is text of its own, so it can change.
    ok(&lines(&[TEXT, "v ၏ စာလုံး ၀ ကို 'x' ဟု ထားမည်။"]));
    assert_eq!(
        codes(&lines(&[text, "t ၏ စာလုံး ၀ ကို \"x\" ဟု ထားမည်။"])),
        [Code::KindMismatch]
    );
    assert_eq!(codes(&lines(&[text, "t ထဲသို့ ၅ ကို ထည့်မည်။"])), [Code::KindMismatch]);
    assert_eq!(codes(&lines(&[text, "t ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။"])), [Code::ListCannotGrow]);
}
