mod common;

use common::check;
use common::check_with;
use common::codes;
use common::kind_at;
use common::ok;
use zakac_diagnostics::Code;
use zakac_semcheck::semcheck::SemcheckOptions;

/// `x` an integer, `p` an owner, `q` a borrow, `r` a raw address and `b` an
/// owner of an `i8`.
const OWN: &str = "ကိန်းရှင် x သည် ကိန်းပြည့် ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် q သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် r သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်ပါစေ။
ကိန်းရှင် b သည် ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
";

/// The prelude, then `rest`.
fn own(rest: &str) -> String { format!("{OWN}{rest}") }

/// The help text of the one diagnostic of `text`.
fn help_of(text: &str) -> String
{
    let out = check(text).out;
    assert_eq!(out.diagnostics.len(), 1, "{text}: {:?}", out.diagnostics);
    out.diagnostics[0].help.clone().unwrap_or_default()
}

/// A function `f` taking one parameter of kind `param`, then `rest`.
fn with_param(param: &str, rest: &str) -> String
{
    format!(
        "{OWN}လုပ်ဆောင်ချက် f သည်
{param} ပ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
{rest}"
    )
}

/// A function `g` with a result of kind `result` that returns `value`.
fn with_result(result: &str, value: &str) -> String
{
    format!(
        "{OWN}လုပ်ဆောင်ချက် g သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ {result} ရလဒ် ကို ပြန်ပေးမည်။
{value} ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။"
    )
}

#[test]
fn borrows_and_raw_addresses_have_names()
{
    let checked = check(&own("x ၏ လိပ်စာ ကို ပြသပါ။
p ၏ လိပ်စာ ကို ပြသပါ။
p ၏ တန်ဖိုးအငှား ကို ပြသပါ။
q ၏ တန်ဖိုးအငှား ကို ပြသပါ။
(p + ၁) ကို ပြသပါ။
(r + ၁) ကို ပြသပါ။
q ၏ တန်ဖိုး ကို ပြသပါ။"));
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, "x ၏ လိပ်စာ"), "&*isize");
    assert_eq!(kind_at(&checked, "p ၏ လိပ်စာ"), "&**isize");
    assert_eq!(kind_at(&checked, "p ၏ တန်ဖိုးအငှား"), "&*isize");
    assert_eq!(kind_at(&checked, "q ၏ တန်ဖိုးအငှား"), "&*isize");
    assert_eq!(kind_at(&checked, "p + ၁"), "&*isize");
    assert_eq!(kind_at(&checked, "r + ၁"), "~isize");
    assert_eq!(kind_at(&checked, "q ၏ တန်ဖိုး"), "isize");
}

#[test]
fn a_borrow_cannot_be_given_to_an_owner()
{
    for line in ["p ကို q ဟု ထားပါ။", "p ကို x ၏ လိပ်စာ ဟု ထားပါ။", "p ကို p + ၁ ဟု ထားပါ။"]
    {
        assert_eq!(codes(&own(line)), [Code::BorrowForOwner], "{line}");
    }
}

#[test]
fn an_owner_given_to_a_borrow_asks_for_a_lend()
{
    let text = own("q ကို p ဟု ထားပါ။");
    assert_eq!(codes(&text), [Code::OwnerForBorrow]);
    assert!(help_of(&text).contains("p ၏ တန်ဖိုးအငှား"));
    ok(&own("q ကို p ၏ တန်ဖိုးအငှား ဟု ထားပါ။"));
    ok(&own("q ကို q ၏ တန်ဖိုးအငှား ဟု ထားပါ။"));
    ok(&own("q ကို q ဟု ထားပါ။"));
}

/// An owner with no variable name, such as a call result, has no lend
/// spelling to offer, so its report gets no help and names no variable.
#[test]
fn an_unnamed_owner_given_to_a_borrow_gets_no_help()
{
    let text = own("လုပ်ဆောင်ချက် g သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် လိပ်စာ ရလဒ် ကို ပြန်ပေးမည်။
p ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
q ကို လုပ်ဆောင်ချက် g ကို လုပ်ဆောင်သော ရလဒ် ဟု ထားပါ။
");
    assert_eq!(codes(&text), [Code::OwnerForBorrow]);
    assert_eq!(help_of(&text), "");
    let message = check(&text).out.diagnostics[0].message.clone();
    assert!(!message.contains('"'), "{message}");
}

#[test]
fn a_lend_needs_a_checked_pointer()
{
    assert_eq!(codes(&own("q ကို r ၏ တန်ဖိုးအငှား ဟု ထားပါ။")), [Code::LendOfRaw]);
    let value = own("x ၏ တန်ဖိုးအငှား ကို ပြသပါ။");
    assert_eq!(codes(&value), [Code::NotAPointer]);
    assert!(help_of(&value).contains("x ၏ လိပ်စာ"));
}

#[test]
fn the_address_of_a_borrow_cannot_be_taken()
{
    assert_eq!(codes(&own("q ၏ လိပ်စာ ကို ပြသပါ။")), [Code::BorrowOfBorrow]);
    let list = "ကိန်းရှင် တန်း သည် ငှားထားသော အကွက် ၂ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း ဖြစ်ပါစေ။\n";
    ok(&own(list));
    let whole = own(&format!("{list}တန်း ၏ လိပ်စာ ကို ပြသပါ။"));
    assert_eq!(codes(&whole), [Code::BorrowOfBorrow]);
    let cell = own(&format!("{list}တန်း ၏ အကွက် ၀ ၏ လိပ်စာ ကို ပြသပါ။"));
    assert_eq!(codes(&cell), [Code::BorrowOfBorrow]);
}

#[test]
fn raw_and_checked_meet_at_a_cast()
{
    assert_eq!(codes(&own("p ကို r ဟု ထားပါ။")), [Code::KindMismatch]);
    assert_eq!(codes(&own("r ကို p ဟု ထားပါ။")), [Code::KindMismatch]);
    ok(&own("r ကို p ကို ကိန်းပြည့် လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။"));
    ok(&own("p ကို r ကို ကိန်းပြည့် လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။"));
    ok(&own("q ကို r ကို ငှားထားသော ကိန်းပြည့် လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။"));
}

#[test]
fn an_integer_becomes_a_checked_address_through_a_raw_one()
{
    let owner = own("p ကို x ကို ကိန်းပြည့် လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။");
    assert_eq!(codes(&owner), [Code::InvalidCast]);
    let borrow = own("q ကို x ကို ငှားထားသော ကိန်းပြည့် လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။");
    assert_eq!(codes(&borrow), [Code::InvalidCast]);
    assert!(help_of(&borrow).contains("~isize"), "{}", help_of(&borrow));
    let opaque = own("(၄၀၉၆ ကို လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ်) ကို ပြသပါ။");
    assert_eq!(codes(&opaque), [Code::InvalidCast]);
    ok(&own("r ကို x ကို ကိန်းပြည့် လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။"));
    ok(&own("q ကို (x ကို ကိန်းပြည့် လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ်) ကို ငှားထားသော ကိန်းပြည့် လိပ်စာ \
             အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။"));
    ok(&own("(p ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ်) ကို ပြသပါ။"));
}

#[test]
fn a_cast_of_a_borrow_keeps_the_borrow()
{
    let cast = "q ကို ၈ ဘစ် ကိန်းပြည့် လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ်";
    let text = own(&format!("b ကို {cast} ဟု ထားပါ။"));
    assert_eq!(codes(&text), [Code::BorrowForOwner]);
    let shown = check(&own(&format!("{cast} ကို ပြသပါ။")));
    assert_eq!(kind_at(&shown, cast), "&*i8");
    ok(&own("ကိန်းရှင် c သည် ငှားထားသော ၈ ဘစ် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
c ကို q ကို ငှားထားသော ၈ ဘစ် ကိန်းပြည့် လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။"));
}

#[test]
fn null_fits_each_flavour()
{
    ok(&own("p ကို ဗလာ ဟု ထားပါ။\nq ကို ဗလာ ဟု ထားပါ။\nr ကို ဗလာ ဟု ထားပါ။"));
    let checked = check(&own("q ကို ဗလာ ဟု ထားပါ။"));
    assert_eq!(kind_at(&checked, "ဗလာ"), "&*isize");
}

#[test]
fn flavours_compare_and_subtract_as_one_shape()
{
    ok(&own("(p == q) ကို ပြသပါ။
((p + ၁) - p) ကို ပြသပါ။
(q < p) ကို ပြသပါ။"));
    assert_eq!(codes(&own("(p - r) ကို ပြသပါ။")), [Code::PointerKindsDiffer]);
}

#[test]
fn borrow_markers_sit_once_before_a_checked_pointer()
{
    for kind in [
        "ငှားထားသော ငှားထားသော ကိန်းပြည့် လိပ်စာ",
        "ငှားထားသော ကိန်းပြည့် လိပ်စာအကြမ်း",
        "ငှားထားသော ကိန်းပြည့်",
    ]
    {
        let text = format!("ကိန်းရှင် m သည် {kind} ဖြစ်ပါစေ။");
        assert_eq!(codes(&text), [Code::BadBorrowMarker], "{kind}");
    }
    let literal = "ကိန်းရှင် m သည် ငှားထားသော ကိန်းပြည့် ၅ ဖြစ်ပါစေ။";
    assert_eq!(codes(literal), [Code::BadBorrowMarker]);
    let text = own("ကိန်းရှင် m သည် ငှားထားသော ကိန်းပြည့် လိပ်စာအကြမ်း ၏ လိပ်စာ r ၏ လိပ်စာ ဖြစ်ပါစေ။");
    let checked = check(&text);
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, "r ၏ လိပ်စာ"), "&*~isize");
}

#[test]
fn no_raw_rejects_raw_kinds()
{
    let decl = "ကိန်းရှင် r သည် ကိန်းပြည့် လိပ်စာအကြမ်း ဖြစ်ပါစေ။";
    let cast = "၄၀၉၆ ကို ကိန်းပြည့် လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။";
    let options = SemcheckOptions::builder().no_raw(true).build();
    for text in [decl, cast]
    {
        let found: Vec<Code> =
            check_with(text, &options).out.diagnostics.iter().map(|d| d.code).collect();
        assert_eq!(found, [Code::RawNotAllowed], "{text}");
        ok(text);
    }
}

#[test]
fn parameters_spell_what_a_call_does()
{
    let borrow = "ငှားထားသော ကိန်းပြည့် လိပ်စာ";
    let call = |arg: &str| format!("လုပ်ဆောင်ချက် f ကို {arg} တို့ဖြင့် လုပ်ဆောင်ပါ။");
    ok(&with_param(borrow, &call("x ၏ လိပ်စာ")));
    assert_eq!(codes(&with_param(borrow, &call("p"))), [Code::OwnerForBorrow]);
    let owner = "ကိန်းပြည့် လိပ်စာ";
    ok(&with_param(owner, &call("p")));
    for arg in ["x ၏ လိပ်စာ", "p ၏ တန်ဖိုးအငှား"]
    {
        assert_eq!(
            codes(&with_param(owner, &call(arg))),
            [Code::BorrowForOwner],
            "{arg}"
        );
    }
}

#[test]
fn a_result_spells_its_flavour()
{
    let global = "x ၏ လိပ်စာ";
    let owner = with_result("ကိန်းပြည့် လိပ်စာ", global);
    assert_eq!(codes(&owner), [Code::BorrowForOwner]);
    ok(&with_result("ငှားထားသော ကိန်းပြည့် လိပ်စာ", global));
}

#[test]
fn a_cast_that_reads_numbers_as_addresses_needs_a_raw_step()
{
    for text in [
        "\
ကိန်းရှင် ဇယား သည် အကွက် ၃ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် i သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို (ပ ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ်) - ၁၀၄၈၆၇၂ ဟု ထားပါ။
ကိန်းရှင် ဒ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ကို ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ \
         အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ဒ ၏ တန်ဖိုး ၏ တန်ဖိုး ကို ၅ ဟု ထားပါ။
ဇယား ၏ အကွက် ၀ ၏ အကွက် i ကို ၉၉ ဟု ထားပါ။
i ကို ပြသပါ။
ဇယား ကို ပြသပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
",
        "\
ကိန်းရှင် ဇယား သည် အကွက် ၃ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် i သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ns သည် အကွက် ၁ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ns ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို (ပ ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ်) - ၃၂ ဟု ထားပါ။
ကိန်းရှင် ဒ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ကို ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ \
         အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
ဒ ၏ တန်ဖိုး ၏ တန်ဖိုး ကို ၅ ဟု ထားပါ။
ဇယား ၏ အကွက် ၀ ၏ အကွက် i ကို ၉၉ ဟု ထားပါ။
i ကို ပြသပါ။
ဇယား ကို ပြသပါ။
",
    ]
    {
        assert_eq!(codes(text), [Code::InvalidCast], "{text}");
        assert!(help_of(text).contains("~*isize"), "{}", help_of(text));
    }
    ok(&own("ကိန်းရှင် ဒ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် (p ကို ကိန်းပြည့် လိပ်စာ ၏ \
             လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ်) ကို ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ အဖြစ် \
             ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။"));
}

/// A grid, an index `i`, and a borrow `ပ` whose cell holds the address of
/// `i` minus the size of the grid, then `rest` and the lines that write
/// through the grid.
fn forged(rest: &str) -> String
{
    format!(
        "\
ကိန်းရှင် ဇယား သည် အကွက် ၃ ခုရှိသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် i သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ns သည် အကွက် ၁ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ns ၏ အကွက် ၀ ၏ လိပ်စာ ဖြစ်ပါစေ။
ပ ၏ တန်ဖိုး ကို (ပ ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ်) - ၃၂ ဟု ထားပါ။
{rest}
ဇယား ၏ အကွက် ၀ ၏ အကွက် i ကို ၉၉ ဟု ထားပါ။
i ကို ပြသပါ။
ဇယား ကို ပြသပါ။
"
    )
}

/// The heap-pointer programs that read a stored number as an owner, or a
/// stored owner as a number, through an opaque address.
const OPAQUE_HEAP_PUNS: [&str; 4] = [
    "\
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် q သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို q တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို q ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။
လိပ်စာ q ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
ကိန်းရှင် အ သည် ငှားထားသော လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
ကိန်းရှင် ဒ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် အ ဖြစ်ပါစေ။
ဒ ၏ တန်ဖိုး ၏ တန်ဖိုး ကို ၅ ဟု ထားပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
",
    "\
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် q သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို q တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို q ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။
ကိန်းရှင် အ သည် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။
ကိန်းရှင် ဒ သည် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် အ ဖြစ်ပါစေ။
လိပ်စာ ဒ ၏ တန်ဖိုး ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ ဒ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ q ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
",
    "\
ကိန်းရှင် i သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ပ ၏ တန်ဖိုး ကို (i ၏ လိပ်စာ) ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။
ကိန်းရှင် အ သည် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။
ကိန်းရှင် ဒ သည် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် အ ဖြစ်ပါစေ။
လိပ်စာ ဒ ၏ တန်ဖိုး ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
လိပ်စာ ဒ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
",
    "\
ကိန်းရှင် i သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
ကိန်းရှင် အ သည် ငှားထားသော လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် အ ဖြစ်ပါစေ။
ဂ ၏ တန်ဖိုး ကို (i ၏ လိပ်စာ) ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ဟု ထားပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
",
];

#[test]
fn an_opaque_address_carries_no_block_of_addresses()
{
    let into_grid = "ဒ ၏ တန်ဖိုး ၏ တန်ဖိုး ကို ၅ ဟု ထားပါ။";
    let take = "ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ";
    let writer = |param: &str, value: &str| {
        format!(
            "လုပ်ဆောင်ချက် ရေး သည်
{param} ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
{value}
{into_grid}
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
"
        )
    };
    let hider = |result: &str, lines: &str| {
        format!(
            "လုပ်ဆောင်ချက် ဖုံး သည်
ငှားထားသော ကိန်းပြည့် လိပ်စာ က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ {result} ရလဒ် ကို ပြန်ပေးမည်။
{lines}
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
"
        )
    };
    let opaque = "ကိန်းရှင် အ သည် ငှားထားသော လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ပ ဖြစ်ပါစေ။";
    let from_opaque = format!("ကိန်းရှင် ဒ သည် {take} ဖြစ်၍ တန်ဖိုးသည် အ ဖြစ်ပါစေ။");
    let mut texts = vec![
        forged(&format!("{opaque}\n{from_opaque}\n{into_grid}")),
        forged(&format!(
            "ကိန်းရှင် ဒ သည် {take} ဖြစ်၍ တန်ဖိုးသည် ပ ကို ငှားထားသော လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ် \
             ဖြစ်ပါစေ။\n{into_grid}"
        )),
        writer("ငှားထားသော လိပ်စာ အ", &from_opaque)
            + &forged("လုပ်ဆောင်ချက် ရေး ကို ပ တို့ဖြင့် လုပ်ဆောင်ပါ။"),
        writer(&format!("{take} ဒ"), "")
            + &forged(&format!("{opaque}\nလုပ်ဆောင်ချက် ရေး ကို အ တို့ဖြင့် လုပ်ဆောင်ပါ။")),
        hider("ငှားထားသော လိပ်စာ", "က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။")
            + &forged(&format!(
                "ကိန်းရှင် ဒ သည် {take} ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် ဖုံး ကို ပ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် \
                 ဖြစ်ပါစေ။\n{into_grid}"
            )),
        hider(
            take,
            "ကိန်းရှင် အ သည် ငှားထားသော လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။\nအ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။",
        ) + &forged("(လုပ်ဆောင်ချက် ဖုံး ကို ပ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်) ၏ တန်ဖိုး ၏ တန်ဖိုး ကို ၅ ဟု ထားပါ။"),
        forged(&format!(
            "ကိန်းရှင် အ သည် ငှားထားသော လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ns ၏ လိပ်စာ \
             ဖြစ်ပါစေ။\n{from_opaque}\n{into_grid}"
        )),
    ];
    texts.extend(OPAQUE_HEAP_PUNS.map(String::from));
    for text in &texts
    {
        assert_eq!(codes(text), [Code::KindMismatch], "{text}");
        let help = help_of(text);
        assert!(help.contains("~*isize") || help.contains("rawptr"), "{help}");
    }
    let raw_step = format!(
        "{opaque}\nကိန်းရှင် ဒ သည် {take} ဖြစ်၍ တန်ဖိုးသည် (အ ကို ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာအကြမ်း အဖြစ် \
         ပြောင်းလဲသော ရလဒ်) ကို {take} အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။"
    );
    ok(&forged(&raw_step));
}

#[test]
fn a_mismatch_between_blocks_of_addresses_names_the_raw_step()
{
    let text = with_param(
        "ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ",
        "လုပ်ဆောင်ချက် f ကို x ၏ လိပ်စာ တို့ဖြင့် လုပ်ဆောင်ပါ။",
    );
    assert_eq!(codes(&text), [Code::KindMismatch]);
    let help = help_of(&text);
    assert!(help.contains("~*isize") && help.contains("&**isize"), "{help}");
    let plain = own("x ကို q ဟု ထားပါ။");
    assert_eq!(codes(&plain), [Code::KindMismatch]);
    assert!(help_of(&plain).contains("ပြောင်းလဲသော ရလဒ်"), "{}", help_of(&plain));
}

#[test]
fn addresses_of_blocks_of_addresses_compare_with_an_opaque_one()
{
    let pair = "ကိန်းရှင် ဒ သည် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် အ သည် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ငှားထားသော လိပ်စာ ဖြစ်ပါစေ။
";
    for line in ["ဒ သည် အ နှင့် ညီ", "အ သည် ဒ နှင့် မညီ", "ဒ သည် ဂ နှင့် မညီ", "(p ၏ လိပ်စာ) == ဂ"]
    {
        ok(&own(&format!("{pair}({line}) ကို ပြသပါ။")));
    }
    let ordered = own(&format!("{pair}(ဒ < အ) ကို ပြသပါ။"));
    assert_eq!(codes(&ordered), [Code::PointerKindsDiffer]);
    let stored = own(&format!("{pair}အ ကို ဒ ဟု ထားပါ။"));
    assert_eq!(codes(&stored), [Code::KindMismatch]);
}

#[test]
fn a_flavour_mistake_is_named_before_the_shape()
{
    let lent =
        "ကိန်းရှင် ဂ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် p ၏ လိပ်စာ ဖြစ်ပါစေ။\n";
    let to_owner = with_param("လိပ်စာ", &format!("{lent}လုပ်ဆောင်ချက် f ကို ဂ တို့ဖြင့် လုပ်ဆောင်ပါ။"));
    assert_eq!(codes(&to_owner), [Code::BorrowForOwner]);
    let owned = "ကိန်းရှင် ဒ သည် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်ပါစေ။\n";
    let to_borrow =
        with_param("ငှားထားသော လိပ်စာ", &format!("{owned}လုပ်ဆောင်ချက် f ကို ဒ တို့ဖြင့် လုပ်ဆောင်ပါ။"));
    assert_eq!(codes(&to_borrow), [Code::OwnerForBorrow]);
    let address_free = with_param("လိပ်စာ", "လုပ်ဆောင်ချက် f ကို q တို့ဖြင့် လုပ်ဆောင်ပါ။");
    assert_eq!(codes(&address_free), [Code::BorrowForOwner]);
}

#[test]
fn a_mismatch_of_flavour_and_shape_offers_no_raw_step()
{
    let lent = "ကိန်းရှင် ဒ သည် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် လိပ်စာ ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ဒ တွင် သိမ်းထားမည်။
ကိန်းရှင် ဂ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ဒ ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
";
    for param in ["ဒဿမကိန်း လိပ်စာ ၏ လိပ်စာ", "ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ၏ လိပ်စာ"]
    {
        let text = with_param(param, &format!("{lent}လုပ်ဆောင်ချက် f ကို ဂ တို့ဖြင့် လုပ်ဆောင်ပါ။"));
        assert_eq!(codes(&text), [Code::KindMismatch], "{text}");
        assert_eq!(help_of(&text), "", "{text}");
    }
    let owned = "ကိန်းရှင် ဒ သည် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်ပါစေ။\n";
    let to_borrow = with_param(
        "ငှားထားသော ဒဿမကိန်း လိပ်စာ ၏ လိပ်စာ",
        &format!("{owned}လုပ်ဆောင်ချက် f ကို ဒ တို့ဖြင့် လုပ်ဆောင်ပါ။"),
    );
    assert_eq!(codes(&to_borrow), [Code::KindMismatch]);
    assert_eq!(help_of(&to_borrow), "");
}
