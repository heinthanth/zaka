mod common;

use common::check;
use common::codes;
use common::kind_at;
use common::ok;
use zakac_diagnostics::Code;

/// `x` an integer, `p` a borrow of it, `q` an opaque address, `b` an `*i8`.
const PTRS: &str = "ကိန်းရှင် x သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ x ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် q သည် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် b သည် ၈ ဘစ် ကိန်းပြည့် ၏ လိပ်စာ ဖြစ်ပါစေ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

#[test]
fn pointer_kinds_and_null()
{
    let checked = check(PTRS);
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "x ၏ လိပ်စာ"), "&*isize");
    let opaque = "ကိန်းရှင် o သည် ငှားထားသော လိပ်စာ ဖြစ်ပါစေ။";
    ok(&lines(&[PTRS, opaque, "o ကို p ဟု ထားပါ။", "p ကို o ဟု ထားပါ။", "p ကို ဗလာ ဟု ထားပါ။"]));
    assert_eq!(codes(&lines(&[PTRS, "p ကို b ဟု ထားပါ။"])), [Code::KindMismatch]);
    assert_eq!(kind_at(&check(&lines(&[PTRS, "p ကို ဗလာ ဟု ထားပါ။"])), "ဗလာ"), "&*isize");
}

#[test]
fn dereference_and_places()
{
    let checked = check(&lines(&[PTRS, "p ၏ တန်ဖိုး ကို ၆ ဟု ထားပါ။", "*p ကို ပြသပါ။"]));
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "p ၏ တန်ဖိုး"), "isize");
    assert_eq!(codes(&lines(&[PTRS, "x ၏ တန်ဖိုး ကို ပြသပါ။"])), [Code::NotAPointer]);
    assert_eq!(codes(&lines(&[PTRS, "q ၏ တန်ဖိုး ကို ပြသပါ။"])), [Code::OpaquePointer]);
    assert_eq!(codes("၅ ကို ၆ ဟု ထားပါ။"), [Code::NotAPlace]);
    assert_eq!(codes("၅ ၏ လိပ်စာ ကို ပြသပါ။"), [Code::NoAddress]);
    let fixed = "ကိန်းသေ က သည် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။\nက ၏ လိပ်စာ ကို ပြသပါ။";
    assert_eq!(codes(fixed), [Code::FixedAddress]);
    let loop_var = "၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ၏ လိပ်စာ ကို ပြသပါ။";
    assert_eq!(codes(loop_var), [Code::FixedAddress]);
}

#[test]
fn pointer_arithmetic_and_comparison()
{
    let checked = check(&lines(&[
        PTRS,
        "(p + ၁) ကို ပြသပါ။",
        "(p - p) ကို ပြသပါ။",
        "p + ၁ ၏ တန်ဖိုး ကို ပြသပါ။",
    ]));
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "p + ၁"), "&*isize");
    assert_eq!(kind_at(&checked, "p - p"), "isize");
    assert_eq!(codes(&lines(&[PTRS, "(p * ၂) ကို ပြသပါ။"])), [Code::PointerOperator]);
    assert_eq!(codes(&lines(&[PTRS, "(q + ၁) ကို ပြသပါ။"])), [Code::OpaquePointer]);
    assert_eq!(codes(&lines(&[PTRS, "(p - b) ကို ပြသပါ။"])), [Code::PointerKindsDiffer]);
    assert_eq!(codes(&lines(&[PTRS, "(p + b) ကို ပြသပါ။"])), [Code::PointerKindsDiffer]);
    assert_eq!(codes(&lines(&[PTRS, "(p + p) ကို ပြသပါ။"])), [Code::PointerOperator]);
    assert_eq!(codes(&lines(&[PTRS, "(p == x) ကို ပြသပါ။"])), [Code::PointerOperator]);
    ok(&lines(&[PTRS, "အကယ်၍ p သည် ဗလာ နှင့် မညီလျှင် x ကို ပြသပါ။"]));
    ok(&lines(&[PTRS, "(p == q) ကို ပြသပါ။", "(p < p) ကို ပြသပါ။"]));
    assert_eq!(codes(&lines(&[PTRS, "(p < b) ကို ပြသပါ။"])), [Code::PointerKindsDiffer]);
    let cond = lines(&[PTRS, "အကယ်၍ p ဖြစ်လျှင် x ကို ပြသပါ။"]);
    assert_eq!(codes(&cond), [Code::ConditionNotBool]);
}

#[test]
fn casts_and_size_of()
{
    let checked = check(&lines(&[PTRS, "ကိန်းပြည့် ၏ အရွယ်အစား ကို ပြသပါ။"]));
    assert_eq!(kind_at(&checked, "ကိန်းပြည့် ၏ အရွယ်အစား"), "usize");
    ok(&lines(&[PTRS, "p ကို ၈ ဘစ် ကိန်းပြည့် ၏ လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။"]));
    ok(&lines(&[PTRS, "p ကို လက္ခဏာမဲ့ ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။"]));
    ok(&lines(&[PTRS, "၄၀၉၆ ကို ကိန်းပြည့် လိပ်စာအကြမ်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။"]));
    let checked = lines(&[PTRS, "၄၀၉၆ ကို ကိန်းပြည့် ၏ လိပ်စာ အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။"]);
    assert_eq!(codes(&checked), [Code::InvalidCast]);
    let float = lines(&[PTRS, "p ကို ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။"]);
    assert_eq!(codes(&float), [Code::InvalidCast]);
}

#[test]
fn heap_statements_take_pointer_places()
{
    ok(&lines(&[PTRS, "မင်မိုရီတွင် ကိန်းပြည့် ၁၀ ခုစာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။"]));
    ok(&lines(&[PTRS, "မင်မိုရီတွင် ၆၄ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို q တွင် သိမ်းထားမည်။"]));
    ok(&lines(&[PTRS, "လိပ်စာ p ရှိ မင်မိုရီကို ၁၂၈ ဘိုက်စာ ဖြစ်အောင် နေရာအချဲ့အချုံ့လုပ်မည်။"]));
    ok(&lines(&[PTRS, "လိပ်စာ p ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။"]));
    let int = lines(&[PTRS, "မင်မိုရီတွင် ၆၄ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို x တွင် သိမ်းထားမည်။"]);
    assert_eq!(codes(&int), [Code::HeapNeedsPointer]);
    let null = lines(&[PTRS, "မင်မိုရီတွင် ၆၄ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို ဗလာ တွင် သိမ်းထားမည်။"]);
    assert_eq!(codes(&null), [Code::NotAPlace]);
    let float = lines(&[PTRS, "မင်မိုရီတွင် ၁.၅ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။"]);
    assert_eq!(codes(&float), [Code::KindMismatch]);
}
