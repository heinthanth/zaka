mod common;

use common::check;
use common::codes;
use common::kind_at;
use common::ok;
use zakac_diagnostics::Code;

/// `l` a list of three integers, `g` a 2 x 3 grid, `p` the address of `l`.
const LISTS: &str = "ကိန်းရှင် l သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် g သည် အကွက် ၂ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် p သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် l ၏ လိပ်စာ ဖြစ်ပါစေ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

#[test]
fn cells_take_the_cell_kind()
{
    let checked = check(&lines(&[LISTS, "l ၏ အကွက် ၀ ကို ပြသပါ။", "g ၏ အကွက် ၁ ကို ပြသပါ။"]));
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "l ၏ အကွက် ၀"), "isize");
    assert_eq!(kind_at(&checked, "g ၏ အကွက် ၁"), "[3]isize");
    assert_eq!(kind_at(&checked, "l ၏ လိပ်စာ"), "*[3]isize");
    ok(&lines(&[LISTS, "p ၏ အကွက် ၂ ကို ၅ ဟု ထားပါ။", "p ၏ အကွက်အရေအတွက် ကို ပြသပါ။"]));
    ok(&lines(&[LISTS, "g ၏ အကွက် ၀ ကို l ဟု ထားပါ။", "l ၏ ၃ ခုမြောက်အကွက် ကို ၁ ဟု ထားပါ။"]));
}

#[test]
fn counts_come_from_the_value()
{
    let text = "ကိန်းရှင် a သည် ၈ ဘစ် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ တို့ပါသော စာရင်း \
                ဖြစ်ပါစေ။\na ကို ပြသပါ။";
    assert_eq!(kind_at(&check(text), "a"), "[2]i8");
    let nested = "ကိန်းရှင် a သည် ကိန်းပြည့် စာရင်း စာရင်း ဖြစ်၍ တန်ဖိုးသည် (၁၊ ၂ တို့ပါသော စာရင်း)၊ (၃၊ \
                  ၄ တို့ပါသော စာရင်း)၊ (၅၊ ၆ တို့ပါသော စာရင်း) တို့ပါသော စာရင်း ဖြစ်ပါစေ။\na ကို \
                  ပြသပါ။";
    assert_eq!(kind_at(&check(nested), "a"), "[3][2]isize");
    let copied =
        lines(&[LISTS, "ကိန်းရှင် b သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် l ဖြစ်ပါစေ။", "b ကို ပြသပါ။"]);
    assert_eq!(kind_at(&check(&copied), "b"), "[3]isize");
}

#[test]
fn list_counts_must_be_known_and_match()
{
    let decl = |rest: &str| codes(&format!("ကိန်းရှင် a သည် {rest} ဖြစ်ပါစေ။"));
    assert_eq!(decl("ကိန်းပြည့် စာရင်း"), [Code::ListCountMissing]);
    assert_eq!(decl("ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ်"), [Code::ListCountMissing]);
    assert_eq!(
        decl("အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ တို့ပါသော စာရင်း"),
        [Code::ListCountMismatch]
    );
    ok("ကိန်းရှင် a သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် စာရင်းအလွတ် ဖြစ်ပါစေ။");
    assert_eq!(codes("စာရင်းအလွတ် ကို ပြသပါ။"), [Code::BlankListKind]);
    let param = "လုပ်ဆောင်ချက် f သည်\nကိန်းပြည့် စာရင်း a ကို လက်ခံပြီး\nအောက်ပါတို့ကို \
                 လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    assert_eq!(codes(param), [Code::ListCountMissing]);
}

#[test]
fn cells_need_a_list_and_an_integer()
{
    let x = "ကိန်းရှင် x သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။";
    assert_eq!(codes(&lines(&[x, "x ၏ အကွက် ၀ ကို ပြသပါ။"])), [Code::NotAList]);
    assert_eq!(codes(&lines(&[x, "x ၏ အကွက်အရေအတွက် ကို ပြသပါ။"])), [Code::NotAList]);
    assert_eq!(
        codes(&lines(&[LISTS, "l ၏ အကွက် မှန် ကို ပြသပါ။"])),
        [Code::CellNumberNotInt]
    );
    assert_eq!(
        codes(&lines(&[LISTS, "l ၏ အကွက် ၁.၅ ကို ပြသပါ။"])),
        [Code::CellNumberNotInt]
    );
}

#[test]
fn constant_lists_and_list_values_are_not_places()
{
    let fixed = "ကိန်းသေ c သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ ၂ တို့ပါသော စာရင်း ဖြစ်ပါစေ။";
    assert_eq!(
        codes(&lines(&[fixed, "c ၏ အကွက် ၀ ကို ၅ ဟု ထားပါ။"])),
        [Code::AssignToConst]
    );
    assert_eq!(
        codes(&lines(&[fixed, "c ၏ အကွက် ၀ ၏ လိပ်စာ ကို ပြသပါ။"])),
        [Code::FixedAddress]
    );
    ok(&lines(&[fixed, "c ၏ အကွက် ၁ ကို ပြသပါ။"]));
    let temp = "(၁၊ ၂ တို့ပါသော စာရင်း) ၏ အကွက် ၀ ကို ၅ ဟု ထားပါ။";
    assert_eq!(codes(temp), [Code::NotAPlace]);
}

#[test]
fn whole_lists_compare_and_duplicate()
{
    let other = "ကိန်းရှင် m သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။";
    let checked = check(&lines(&[LISTS, "(l == l) ကို ပြသပါ။"]));
    assert!(checked.out.diagnostics.is_empty());
    ok(&lines(&[LISTS, "l သည် ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း နှင့် ညီသည် ဟု ပြသပါ။"]));
    ok(&lines(&[LISTS, "l != စာရင်းအလွတ် ကို ပြသပါ။"]));
    assert_eq!(
        codes(&lines(&[LISTS, other, "l == m ကို ပြသပါ။"])),
        [Code::IncompatibleOperands]
    );
    assert_eq!(codes(&lines(&[LISTS, "l + l ကို ပြသပါ။"])), [Code::OperandKind]);
    assert_eq!(codes(&lines(&[LISTS, other, "l ကို m ဟု ထားပါ။"])), [Code::KindMismatch]);
}

#[test]
fn a_function_takes_a_list_but_gives_none_back()
{
    let takes = "လုပ်ဆောင်ချက် f သည်\nအကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း a ကို လက်ခံပြီး\nအောက်ပါတို့ကို \
                 လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    ok(&lines(&[takes, "လုပ်ဆောင်ချက် f ကို ၁၊ ၂ တို့ပါသော စာရင်း တို့ဖြင့် လုပ်ဆောင်ပါ။"]));
    assert_eq!(
        codes(&lines(&[takes, "လုပ်ဆောင်ချက် f ကို ၁၊ ၂၊ ၃ တို့ပါသော စာရင်း တို့ဖြင့် လုပ်ဆောင်ပါ။"])),
        [Code::ListCountMismatch]
    );
    let gives = "လုပ်ဆောင်ချက် f သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း ရလဒ် ကို \
                 ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    assert_eq!(codes(gives), [Code::ListResult]);
}

#[test]
fn lists_do_not_convert()
{
    let cast = "l ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။";
    assert_eq!(codes(&lines(&[LISTS, cast])), [Code::InvalidCast]);
}
