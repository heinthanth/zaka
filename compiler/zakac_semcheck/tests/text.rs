mod common;

use common::check;
use common::codes;
use common::kind_at;
use common::ok;
use zakac_diagnostics::Code;
use zakac_semcheck::kind::DataKind;

/// A text and a view of it.
const TEXT: &str = "ကိန်းရှင် t သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် \"က {၁ + ၁} ခ\" ဖြစ်ပါစေ။
ကိန်းရှင် v သည် စာသား တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် t ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

#[test]
fn text_kinds_read_back_from_their_names()
{
    let text = DataKind::parse("text").expect("text");
    let view = DataKind::parse("text[..]").expect("text[..]");
    assert_eq!(text, DataKind::TEXT);
    assert_eq!(view, DataKind::text_view());
    assert_eq!(text.size(64), 24);
    assert_eq!(text.size(32), 12);
    assert!(!text.is_move_kind() && text.is_aggregate() && !text.is_borrow());
    assert!(text.holds_text() && DataKind::parse("text[2]").unwrap().holds_text());
    assert!(view.is_borrow() && !view.holds_text());
    assert!(!view.widens_to(DataKind::slice_of(DataKind::U8), 64));
    assert!(!DataKind::slice_of(DataKind::U8).widens_to(view, 64));
    assert_eq!(
        DataKind::parse("text[..]*").map(DataKind::name),
        Some("text[..]*".into())
    );
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
    assert_eq!(kind_at(&checked, "t ၏ အကွက် ၀ မှစ၍ ၁ ကွက်"), "text[..]");
}

#[test]
fn template_slots_take_each_kind()
{
    ok("ကိန်းရှင် l သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။\n\"{l} \
        {မှန်} {'က'} {၁.၅}\" ဟု ပြသပါ။");
    assert_eq!(codes("\"{မသိ}\" ဟု ပြသပါ။"), [Code::UndeclaredName]);
}

#[test]
fn text_mistakes()
{
    assert_eq!(codes(&lines(&[TEXT, "t ၏ အကွက် ၀ ကို ၁ ဟု ထားပါ။"])), [Code::TextCellWrite]);
    assert_eq!(codes(&lines(&[TEXT, "v ၏ အကွက် ၀ ကို ၁ ဟု ထားပါ။"])), [Code::TextCellWrite]);
    ok(&lines(&[TEXT, "t == t ကို ပြသပါ။", "t != v ကို ပြသပါ။"]));
    assert_eq!(codes(&lines(&[TEXT, "t < t ကို ပြသပါ။"])), [Code::OperandKind]);
    assert_eq!(codes(&lines(&[TEXT, "t + ၁ ကို ပြသပါ။"])), [Code::OperandKind]);
}

/// An each-character loop walks text or a view of text; an each-cell loop
/// walks a list or a slice whose cells hold no owner pointer.
#[test]
fn each_loops_walk_lists_slices_and_text()
{
    let walks = [
        "t ၏ စာလုံးတိုင်း ကို c အနေဖြင့် ယူ၍ c ကို ပြသပါ။",
        "v ၏ စာလုံးတိုင်း ကို c အနေဖြင့် ယူ၍ c ကို ပြသပါ။",
        "ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။",
        "l ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို x + ၁ ဟု ထားပါ။",
        "l ၏ အကွက် ၀ မှစ၍ ၁ ကွက် ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသပါ။",
    ];
    ok(&lines(&[&[TEXT], &walks[..]].concat()));
    let wrong = [
        "t ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသပါ။",
        "v ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသပါ။",
        "၅ ၏ စာလုံးတိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသပါ။",
    ];
    let found: Vec<_> = wrong.iter().map(|w| codes(&lines(&[TEXT, w]))).collect();
    assert_eq!(found, [[Code::EachSourceWrong]; 3]);
    let owners = "ကိန်းရှင် ps သည် အကွက် ၁ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း တစ်ခု ဖြစ်ပါစေ။";
    let walk = "ps ၏ အကွက်တိုင်း ကို p အနေဖြင့် ယူ၍ p ကို ပြသပါ။";
    assert_eq!(codes(&lines(&[owners, walk])), [Code::EachSourceWrong]);
}

/// A view of text never becomes a slice of u8s, whose cells can be written,
/// so a write through it cannot break the UTF-8 of the text.
#[test]
fn a_text_view_is_no_slice_of_u8s()
{
    let u8s = "လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း";
    let held = format!("ကိန်းရှင် b သည် {u8s} တစ်ခု ဖြစ်၍ တန်ဖိုးသည် v ဖြစ်ပါစေ။");
    assert_eq!(codes(&lines(&[TEXT, &held])), [Code::KindMismatch]);
    let given = format!(
        "လုပ်ဆောင်ချက် ရေး သည်\n{u8s} b ကို လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\nb ၏ အကွက် ၀ ကို ၂၅၅ ဟု \
         ထားပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီ။\nလုပ်ဆောင်ချက် ရေး ကို v တို့ဖြင့် လုပ်ဆောင်ပါ။"
    );
    assert_eq!(codes(&lines(&[TEXT, &given])), [Code::KindMismatch]);
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

/// A character of text is a place, and text takes an add; fixed text and a
/// view of text take neither, and text takes no remove.
#[test]
fn text_takes_character_writes_and_adds()
{
    let text = "ကိန်းရှင် t သည် \"ab\" ဖြစ်ပါစေ။";
    ok(&lines(&[text, "t ၏ ၂ ခုမြောက်စာလုံး ကို 'x' ဟု ထားပါ။", "t ၏ စာလုံး ၀ ကို 'y' ဟု ထားပါ။"]));
    ok(&lines(&[text, "t ထဲသို့ \"cd\" ကို ထည့်ပါ။", "t ထဲသို့ 'e' ကို ထည့်ပါ။"]));
    assert_eq!(
        codes(&lines(&[TEXT, "v ၏ စာလုံး ၀ ကို 'x' ဟု ထားပါ။"])),
        [Code::TextCannotChange]
    );
    assert_eq!(codes(&lines(&[TEXT, "v ထဲသို့ \"c\" ကို ထည့်ပါ။"])), [Code::TextCannotChange]);
    assert_eq!(
        codes(&lines(&[text, "t ၏ စာလုံး ၀ ကို \"x\" ဟု ထားပါ။"])),
        [Code::KindMismatch]
    );
    assert_eq!(codes(&lines(&[text, "t ထဲသို့ ၅ ကို ထည့်ပါ။"])), [Code::KindMismatch]);
    assert_eq!(codes(&lines(&[text, "t ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။"])), [Code::ListCannotGrow]);
}
