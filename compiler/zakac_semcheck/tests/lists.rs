use zakac_diagnostics::Code;
use zakac_rust_sugars::do_for;

use crate::common::check;
use crate::common::codes;
use crate::common::kind_at;
use crate::common::ok;

/// `l` a list of three integers, `g` a 2 x 3 grid, `p` a borrow of `l`.
const LISTS: &str = "ကိန်းရှင် l သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် g သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် p သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် l ၏ လိပ်စာ ဖြစ်ပါစေ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

#[test]
fn cells_take_the_cell_kind()
{
    let checked = check(&lines(&[LISTS, "l ၏ အကွက် ၀ ကို ပြသမည်။", "g ၏ အကွက် ၁ ကို ပြသမည်။"]));
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "l ၏ အကွက် ၀"), "isize");
    assert_eq!(kind_at(&checked, "g ၏ အကွက် ၁"), "isize[3]");
    assert_eq!(kind_at(&checked, "l ၏ လိပ်စာ"), "isize[3]*");
    ok(&lines(&[LISTS, "p ၏ အကွက် ၂ ကို ၅ ဟု ထားမည်။", "p ၏ အကွက်အရေအတွက် ကို ပြသမည်။"]));
    ok(&lines(&[LISTS, "g ၏ အကွက် ၀ ကို l ဟု ထားမည်။", "l ၏ ၃ ခုမြောက်အကွက် ကို ၁ ဟု ထားမည်။"]));
}

#[test]
fn list_counts_must_match()
{
    let decl = |rest: &str| codes(&format!("ကိန်းရှင် a သည် {rest} ဖြစ်ပါစေ။"));
    assert_eq!(
        decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ တို့ပါသော စာရင်း"),
        [Code::ListCountMismatch]
    );
    assert_eq!(
        codes("ကိန်းရှင် a သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ် ဖြစ်ပါစေ။"),
        [Code::BracketsOnCountedList]
    );
    ok("ကိန်းရှင် a သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။");
    assert_eq!(codes("စာရင်းအလွတ် ကို ပြသမည်။"), [Code::BlankListKind]);
}

#[test]
fn cells_need_a_list_and_an_integer()
{
    let x = "ကိန်းရှင် x သည် ၅ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။";
    assert_eq!(codes(&lines(&[x, "x ၏ အကွက် ၀ ကို ပြသမည်။"])), [Code::NotAList]);
    assert_eq!(codes(&lines(&[x, "x ၏ အကွက်အရေအတွက် ကို ပြသမည်။"])), [Code::NotAList]);
    assert_eq!(
        codes(&lines(&[LISTS, "l ၏ အကွက် မှန် ကို ပြသမည်။"])),
        [Code::CellNumberNotInt]
    );
    assert_eq!(
        codes(&lines(&[LISTS, "l ၏ အကွက် ၁.၅ ကို ပြသမည်။"])),
        [Code::CellNumberNotInt]
    );
}

#[test]
fn constant_lists_and_list_values_are_not_places()
{
    let fixed = "ကိန်းသေ c သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ တို့ပါသော စာရင်း ဖြစ်ပါစေ။";
    assert_eq!(
        codes(&lines(&[fixed, "c ၏ အကွက် ၀ ကို ၅ ဟု ထားမည်။"])),
        [Code::AssignToConst]
    );
    assert_eq!(
        codes(&lines(&[fixed, "c ၏ အကွက် ၀ ၏ လိပ်စာ ကို ပြသမည်။"])),
        [Code::FixedAddress]
    );
    ok(&lines(&[fixed, "c ၏ အကွက် ၁ ကို ပြသမည်။"]));
    let temp = "(၁၊ ၂ တို့ပါသော စာရင်း) ၏ အကွက် ၀ ကို ၅ ဟု ထားမည်။";
    assert_eq!(codes(temp), [Code::NotAPlace]);
}

#[test]
fn whole_lists_compare_and_duplicate()
{
    let other = "ကိန်းရှင် m သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။";
    let checked = check(&lines(&[LISTS, "(l == l) ကို ပြသမည်။"]));
    assert!(checked.out.diagnostics.is_empty());
    ok(&lines(&[LISTS, "l သည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း နှင့် ညီသည် ဟု ပြသမည်။"]));
    assert_eq!(
        codes(&lines(&[LISTS, "l != စာရင်းအလွတ် ကို ပြသမည်။"])),
        [Code::BracketsOnCountedList]
    );
    assert_eq!(
        codes(&lines(&[LISTS, other, "l == m ကို ပြသမည်။"])),
        [Code::IncompatibleOperands]
    );
    assert_eq!(codes(&lines(&[LISTS, "l + l ကို ပြသမည်။"])), [Code::OperandKind]);
    assert_eq!(codes(&lines(&[LISTS, other, "l ကို m ဟု ထားမည်။"])), [Code::KindMismatch]);
}

#[test]
fn a_function_takes_and_gives_back_a_list()
{
    let takes = "လုပ်ဆောင်ချက် f သည်\na ဟူသော အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ကို \
                 လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    ok(&lines(&[takes, "f ဟူသော လုပ်ဆောင်ချက်ကို ၁၊ ၂ တို့ပါသော စာရင်း တို့ဖြင့် လုပ်ဆောင်မည်။"]));
    assert_eq!(
        codes(&lines(&[
            takes,
            "f ဟူသော လုပ်ဆောင်ချက်ကို ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း တို့ဖြင့် လုပ်ဆောင်မည်။"
        ])),
        [Code::ListCountMismatch]
    );
    let gives = "လုပ်ဆောင်ချက် f သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ကို \
                 ပြန်ပေးမည်။\nကိန်းရှင် အဖြေ သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။\nအဖြေ \
                 ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    ok(gives);
}

#[test]
fn lists_do_not_convert()
{
    let cast = "l ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။";
    assert_eq!(codes(&lines(&[LISTS, cast])), [Code::InvalidCast]);
}

#[test]
fn an_open_count_keeps_the_cell_kind_of_its_value()
{
    let ints = "ကိန်းရှင် ns သည် အကွက် ၁ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။";
    let owners = "ကိန်းရှင် ps သည် အကွက် ၁ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း တစ်ခု ဖြစ်ပါစေ။";
    let floats = "ကိန်းရှင် fs သည် အကွက် ၂ ခုရှိသော ဒဿမကိန်း စာရင်း တစ်ခု ဖြစ်ပါစေ။";
    let refused = [
        (ints, "ကိန်းပြည့် လိပ်စာ စာရင်း", "ns"),
        (ints, "ကိန်းပြည့် လိပ်စာ စာရင်း စာရင်း", "ns တို့ပါသော စာရင်း"),
        (owners, "ကိန်းပြည့် စာရင်း", "ps"),
        (floats, "မှန်မှားတန်ဖိုး စာရင်း", "fs"),
        (ints, "၈ ဘစ် ကိန်းပြည့် စာရင်း", "ns"),
    ];
    for (first, kind, value) in refused
    {
        let text =
            format!("{first}\nကိန်းရှင် b သည် {kind} တစ်ခု ဖြစ်၍ တန်ဖိုးသည် {value} ဖြစ်ပါစေ။");
        assert_eq!(codes(&text), [Code::KindMismatch], "{text}");
    }
    let callee = "လုပ်ဆောင်ချက် လွှတ် သည်
က ဟူသော အကွက် ၁ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ps သည် ကိန်းပြည့် လိပ်စာ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုး ဖြစ်ပါစေ။
လိပ်စာ ps ၏ အကွက် ၀ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    assert_eq!(codes(callee), [Code::KindMismatch]);
    let same = [
        (ints, "ကိန်းပြည့် စာရင်း", "ns"),
        (owners, "ကိန်းပြည့် လိပ်စာ စာရင်း", "ps"),
        (ints, "ကိန်းပြည့် စာရင်း စာရင်း", "ns တို့ပါသော စာရင်း"),
    ];
    for (first, kind, value) in same
    {
        ok(&format!("{first}\nကိန်းရှင် b သည် {kind} တစ်ခု ဖြစ်၍ တန်ဖိုးသည် {value} ဖြစ်ပါစေ။"));
    }
}

/// The help texts of the diagnostics of `text`, each with its code.
fn helps(text: &str) -> Vec<(Code, Option<String>)>
{
    let out = check(text).out;
    out.diagnostics.iter().map(|d| (d.code, d.help.clone())).collect()
}

#[test]
fn a_list_mismatch_does_not_offer_a_conversion()
{
    let ints = "ကိန်းရှင် ns သည် အကွက် ၁ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။";
    let refused = [
        "ကိန်းရှင် b သည် ကိန်းပြည့် လိပ်စာ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ns ဖြစ်ပါစေ။",
        "ကိန်းရှင် b သည် အကွက် ၁ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ns ဖြစ်ပါစေ။",
        "ကိန်းရှင် b သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ns ဖြစ်ပါစေ။",
        "ns ကို ၅ ဟု ထားမည်။",
        "ကိန်းရှင် b သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။",
    ];
    for line in refused
    {
        let text = lines(&[ints, line]);
        assert_eq!(helps(&text), [(Code::KindMismatch, None)], "{text}");
    }
}

/// `s` a slice of `l`, and `h` a heap block of four integers.
#[test]
fn ranges_make_lists_and_text()
{
    let checked = check(&lines(&[
        LISTS,
        "ကိန်းရှင် t သည် \"abc\" ဖြစ်ပါစေ။",
        "l ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ကို ပြသမည်။",
        "p ၏ အကွက် ၀ မှစ၍ ၃ ကွက် ကို ပြသမည်။",
        "t ၏ အကွက် ၀ မှစ၍ ၁ ကွက် ကို ပြသမည်။",
    ]));
    let msgs: Vec<&str> =
        checked.out.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(msgs.is_empty(), "{msgs:?}");
    assert_eq!(kind_at(&checked, "l ၏ အကွက် ၁ မှစ၍ ၂ ကွက်"), "isize[]");
    assert_eq!(kind_at(&checked, "p ၏ အကွက် ၀ မှစ၍ ၃ ကွက်"), "isize[]");
    assert_eq!(kind_at(&checked, "t ၏ အကွက် ၀ မှစ၍ ၁ ကွက်"), "text");
}

#[test]
fn range_mistakes()
{
    let at = |line: &str| codes(&lines(&[LISTS, line]));
    assert_eq!(at("၅ ၏ အကွက် ၀ မှစ၍ ၁ ကွက် ကို ပြသမည်။"), [Code::NotAList]);
    assert_eq!(at("l ၏ အကွက် ၀ မှစ၍ ၁.၅ ကွက် ကို ပြသမည်။"), [Code::CellNumberNotInt]);
    assert_eq!(at("(l) ၏ အကွက် ၀ မှစ၍ ၁ ကွက် ကို ပြသမည်။").len(), 0);
    // A range copies its cells, so a literal can be its source.
    assert_eq!(at("[၁၊ ၂] ၏ အကွက် ၀ မှစ၍ ၁ ကွက် ကို ပြသမည်။").len(), 0);
}

/// A counted list converts to a plain list on its own; nothing converts to a
/// counted list.
#[test]
fn list_forms_convert()
{
    let lists = "ကိန်းရှင် f သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် p သည် [၁၊ ၂] ဖြစ်ပါစေ။
p ကို ပြသမည်။";
    let checked = check(lists);
    assert_eq!(kind_at(&checked, "p"), "isize[]");
    ok(&lines(&[lists, "p ကို f ဟု ထားမည်။"]));
    assert_eq!(codes(&lines(&[lists, "f ကို p ဟု ထားမည်။"])), [Code::KindMismatch]);
    assert_eq!(
        codes(&lines(&[lists, "f == p ကို ပြသမည်။"])),
        [Code::IncompatibleOperands]
    );
}

/// `[]` has no cells: a counted list takes it in no place, and neither does it
/// take `စာရင်းအလွတ်`; `ကနဦးတန်ဖိုး` fills it. A plain list takes `[]`.
#[test]
fn brackets_never_fit_a_counted_list()
{
    let counted = "ကိန်းရှင် f သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။";
    let filled =
        "ကိန်းရှင် f သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။";
    ok(&lines(&[filled, "f ကို ကနဦးတန်ဖိုး ဟု ထားမည်။"]));
    let declared = counted.replace("ဖြစ်ပါစေ", "ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ");
    assert_eq!(codes(&declared), [Code::BracketsOnCountedList]);
    let blank = counted.replace("ဖြစ်ပါစေ", "ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ် ဖြစ်ပါစေ");
    assert_eq!(codes(&blank), [Code::BracketsOnCountedList]);
    assert_eq!(
        codes(&lines(&[counted, "f ကို [] ဟု ထားမည်။"])),
        [Code::BracketsOnCountedList]
    );
    assert_eq!(
        codes(&lines(&[counted, "f ကို စာရင်းအလွတ် ဟု ထားမည်။"])),
        [Code::BracketsOnCountedList]
    );
    ok("ကိန်းရှင် p သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဖြစ်ပါစေ။\np ကို [] ဟု ထားမည်။");
}

/// A constant keeps the kind its declaration gives: a plain list remains
/// plain, and a written count makes a counted list.
#[test]
fn a_constant_list_keeps_its_kind()
{
    let text = "ကိန်းသေ c သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။\nc ကို ပြသမည်။";
    assert_eq!(kind_at(&check(text), "c"), "isize[]");
    let written = "ကိန်းသေ c သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။\nc ကို ပြသမည်။";
    assert_eq!(kind_at(&check(written), "c"), "isize[]");
    let counted = written.replace("သည် ကိန်း", "သည် အကွက် ၂ ခုရှိသော ကိန်း");
    assert_eq!(kind_at(&check(&counted), "c"), "isize[2]");
}

/// Add and remove statements change a plain list; a counted list and text
/// keep their cells, and the new cell takes the cell kind.
#[test]
fn plain_lists_grow_and_shrink()
{
    ok("ကိန်းရှင် l သည် [၁] ဖြစ်ပါစေ။\nl ထဲသို့ ၂ ကို ထည့်မည်။\nl ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။");
    let counted = "ကိန်းရှင် f သည် အကွက် ၁ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁] ဖြစ်ပါစေ။";
    assert_eq!(codes(&lines(&[counted, "f ထဲသို့ ၂ ကို ထည့်မည်။"])), [Code::ListCannotGrow]);
    let text = "ကိန်းရှင် t သည် \"abc\" ဖြစ်ပါစေ။\nt ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။";
    assert_eq!(codes(text), [Code::ListCannotGrow]);
    let wrong = "ကိန်းရှင် l သည် [၁] ဖြစ်ပါစေ။\nl ထဲသို့ \"x\" ကို ထည့်မည်။";
    assert_eq!(codes(wrong), [Code::KindMismatch]);
}

#[test]
fn adds_in_front_of_a_cell_or_a_character_check_their_place()
{
    let head = "ကိန်းရှင် l သည် [၁] ဖြစ်ပါစေ။
ကိန်းရှင် c သည် အကွက် ၁ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁] ဖြစ်ပါစေ။
ကိန်းရှင် t သည် \"ab\" ဖြစ်ပါစေ။
ကိန်းရှင် n သည် ၁ ဖြစ်ပါစေ။";
    let check = |line: &str| codes(&lines(&[head, line]));
    let fine = [
        "l ၏ အကွက် ၀ ၏ အရှေ့တွင် ၂ ကို ထည့်မည်။",
        "l ၏ ၁ ခုမြောက်အကွက် ၏ အရှေ့တွင် ၂ ကို ထည့်မည်။",
        "t ၏ စာလုံး ၀ ၏ အရှေ့တွင် \"x\" ကို ထည့်မည်။",
        "t ၏ စာလုံး ၀ ၏ အရှေ့တွင် 'x' ကို ထည့်မည်။",
        "t ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။",
    ];
    do_for!(line in fine, assert_eq!(check(line), [], "{line}"));
    let wrong = [
        ("c ၏ အကွက် ၀ ၏ အရှေ့တွင် ၂ ကို ထည့်မည်။", Code::ListCannotGrow),
        ("n ၏ အရှေ့တွင် ၂ ကို ထည့်မည်။", Code::InsertNotAtCell),
        ("l ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။", Code::OperandKind),
        ("l ၏ အကွက် ၀ ၏ အရှေ့တွင် \"x\" ကို ထည့်မည်။", Code::KindMismatch),
        ("l ၏ အကွက် \"a\" ၏ အရှေ့တွင် ၂ ကို ထည့်မည်။", Code::CellNumberNotInt),
        ("t ၏ စာလုံး ၀ ၏ အရှေ့တွင် ၅ ကို ထည့်မည်။", Code::KindMismatch),
    ];
    do_for!((line, code) in wrong, assert_eq!(check(line), [code], "{line}"));
}

#[test]
fn replaces_of_a_range_check_their_text_and_value()
{
    let head = "ကိန်းရှင် l သည် [၁] ဖြစ်ပါစေ။
ကိန်းရှင် t သည် \"ab\" ဖြစ်ပါစေ။
ကိန်းရှင် n သည် ၁ ဖြစ်ပါစေ။";
    let check = |line: &str| codes(&lines(&[head, line]));
    let fine = [
        "t ၏ အကွက် ၀ မှ ၁ ထိကို \"x\" ဖြင့် အစားထိုးမည်။",
        "t ၏ အကွက် n မှ n ထိကို 'x' ဖြင့် အစားထိုးမည်။",
        "t ၏ ၁ လုံးမြောက်စာလုံးမှ ၂ လုံးမြောက်စာလုံးထိကို t ဖြင့် အစားထိုးမည်။",
        "t ၏ ၁ ခုမြောက်စာလုံးမှ n ခုမြောက်စာလုံးထိကို \"{n}\" ဖြင့် အစားထိုးမည်။",
    ];
    do_for!(line in fine, assert_eq!(check(line), [], "{line}"));
    let wrong = [
        ("l ၏ အကွက် ၀ မှ ၀ ထိကို \"x\" ဖြင့် အစားထိုးမည်။", Code::OperandKind),
        ("t ၏ အကွက် ၀ မှ ၀ ထိကို ၅ ဖြင့် အစားထိုးမည်။", Code::KindMismatch),
        ("t ၏ အကွက် \"a\" မှ ၀ ထိကို \"x\" ဖြင့် အစားထိုးမည်။", Code::CellNumberNotInt),
        ("t ၏ အကွက် ၀ မှ မှန် ထိကို \"x\" ဖြင့် အစားထိုးမည်။", Code::CellNumberNotInt),
    ];
    do_for!((line, code) in wrong, assert_eq!(check(line), [code], "{line}"));
}

/// A blank list takes the list kind written after it, in place of the one its
/// place gives; on a counted list kind it is ZK0436, as anywhere else.
#[test]
fn a_blank_list_takes_a_written_kind()
{
    let blank = "[] ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု";
    let full = "[၁] ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု";
    let checked = check(&format!("{blank} ကို ပြသမည်။\n{full} ကို ပြသမည်။"));
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, blank), kind_at(&checked, full));
    ok("စာရင်းအလွတ် ဟူသော စာသား စာရင်း တစ်ခု ကို ပြသမည်။");
    ok("ကိန်းရှင် a သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။");
    assert_eq!(
        codes("[] ဟူသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြသမည်။"),
        [Code::BracketsOnCountedList]
    );
    let other =
        "ကိန်းရှင် a သည် စာသား စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [] ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။";
    assert!(!codes(other).is_empty());
}
