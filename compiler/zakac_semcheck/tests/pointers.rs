mod common;

use common::check;
use common::codes;
use common::kind_at;
use common::ok;
use zakac_diagnostics::Code;

/// `x` an integer, `p` a borrow of it, `q` an opaque address, `b` an `i8*`.
const PTRS: &str = "ကိန်းရှင် x သည် ၅ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် x ၏ လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် q သည် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် b သည် ၈ ဘစ် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

#[test]
fn pointer_kinds_and_null()
{
    let checked = check(PTRS);
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "x ၏ လိပ်စာ"), "isize*");
    let opaque = "ကိန်းရှင် o သည် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။";
    ok(&lines(&[
        PTRS,
        opaque,
        "o ကို p ဟု ထားမည်။",
        "p ကို o ဟု ထားမည်။",
        "p ကို မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ ဟု ထားမည်။",
    ]));
    assert_eq!(codes(&lines(&[PTRS, "p ကို b ဟု ထားမည်။"])), [Code::KindMismatch]);
    assert_eq!(
        kind_at(
            &check(&lines(&[PTRS, "p ကို မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ ဟု ထားမည်။"])),
            "မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ"
        ),
        "isize*"
    );
}

#[test]
fn dereference_and_places()
{
    let checked = check(&lines(&[PTRS, "p ၏ တန်ဖိုး ကို ၆ ဟု ထားမည်။", "*p ကို ပြသမည်။"]));
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "p ၏ တန်ဖိုး"), "isize");
    assert_eq!(codes(&lines(&[PTRS, "x ၏ တန်ဖိုး ကို ပြသမည်။"])), [Code::NotAPointer]);
    assert_eq!(codes(&lines(&[PTRS, "q ၏ တန်ဖိုး ကို ပြသမည်။"])), [Code::OpaquePointer]);
    assert_eq!(codes("၅ ကို ၆ ဟု ထားမည်။"), [Code::NotAPlace]);
    assert_eq!(codes("၅ ၏ လိပ်စာ ကို ပြသမည်။"), [Code::NoAddress]);
    let fixed = "ကိန်းသေ က သည် ၁ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။\nက ၏ လိပ်စာ ကို ပြသမည်။";
    assert_eq!(codes(fixed), [Code::FixedAddress]);
    let loop_var = "၁ မှ ၃ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ၏ လိပ်စာ ကို ပြသမည်။";
    assert_eq!(codes(loop_var), [Code::FixedAddress]);
}

#[test]
fn pointer_arithmetic_and_comparison()
{
    let checked = check(&lines(&[
        PTRS,
        "(p + ၁) ကို ပြသမည်။",
        "(p - p) ကို ပြသမည်။",
        "p + ၁ ၏ တန်ဖိုး ကို ပြသမည်။",
    ]));
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "p + ၁"), "isize*");
    assert_eq!(kind_at(&checked, "p - p"), "isize");
    assert_eq!(codes(&lines(&[PTRS, "(p * ၂) ကို ပြသမည်။"])), [Code::PointerOperator]);
    assert_eq!(codes(&lines(&[PTRS, "(q + ၁) ကို ပြသမည်။"])), [Code::OpaquePointer]);
    assert_eq!(codes(&lines(&[PTRS, "(p - b) ကို ပြသမည်။"])), [Code::PointerKindsDiffer]);
    assert_eq!(codes(&lines(&[PTRS, "(p + b) ကို ပြသမည်။"])), [Code::PointerKindsDiffer]);
    assert_eq!(codes(&lines(&[PTRS, "(p + p) ကို ပြသမည်။"])), [Code::PointerOperator]);
    assert_eq!(codes(&lines(&[PTRS, "(p == x) ကို ပြသမည်။"])), [Code::PointerOperator]);
    ok(&lines(&[PTRS, "အကယ်၍ p သည် မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ နှင့် မညီလျှင် x ကို ပြသမည်။"]));
    ok(&lines(&[PTRS, "(p == q) ကို ပြသမည်။", "(p < p) ကို ပြသမည်။"]));
    assert_eq!(codes(&lines(&[PTRS, "(p < b) ကို ပြသမည်။"])), [Code::PointerKindsDiffer]);
    let cond = lines(&[PTRS, "အကယ်၍ p ဖြစ်လျှင် x ကို ပြသမည်။"]);
    assert_eq!(codes(&cond), [Code::ConditionNotBool]);
}

#[test]
fn casts_and_size_of()
{
    let checked = check(&lines(&[PTRS, "ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား ကို ပြသမည်။"]));
    assert_eq!(kind_at(&checked, "ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား"), "usize");
    ok(&lines(&[PTRS, "p ကို ၈ ဘစ် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။"]));
    ok(&lines(&[PTRS, "p ကို လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။"]));
    ok(&lines(&[PTRS, "၄၀၉၆ ကို ကိန်းပြည့် လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။"]));
    let float = lines(&[PTRS, "p ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။"]);
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
    let null =
        lines(&[PTRS, "မင်မိုရီတွင် ၆၄ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ တွင် သိမ်းထားမည်။"]);
    assert_eq!(codes(&null), [Code::NotAPlace]);
    let float = lines(&[PTRS, "မင်မိုရီတွင် ၁.၅ ဘိုက်စာ နေရာယူ၍ လိပ်စာကို p တွင် သိမ်းထားမည်။"]);
    assert_eq!(codes(&float), [Code::KindMismatch]);
}

#[test]
fn copies_and_fills_check_addresses_values_and_cells()
{
    let owner = "ကိန်းရှင် o သည် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။";
    ok(&lines(&[PTRS, "လိပ်စာ p ရှိ ကိန်းပြည့် ၁ ခုစာ မင်မိုရီကို လိပ်စာ b ဆီသို့ ပြောင်းထည့်မည်။"]));
    ok(&lines(&[PTRS, "လိပ်စာ q ရှိ မင်မိုရီကို ၀ ဖြင့် ၈ ဘိုက်စာ ဖြည့်မည်။"]));
    let int = lines(&[PTRS, "လိပ်စာ x ရှိ ၈ ဘိုက်စာ မင်မိုရီကို လိပ်စာ p ဆီသို့ ကူးယူမည်။"]);
    assert_eq!(codes(&int), [Code::HeapNeedsPointer]);
    let truth = lines(&[PTRS, "လိပ်စာ p ရှိ မင်မိုရီကို မှန် ဖြင့် ကိန်းပြည့် ၁ ခုစာ ဖြည့်မည်။"]);
    assert_eq!(codes(&truth), [Code::KindMismatch]);
    // Pointer cells are plain addresses, so a raw duplicate or fill of them
    // is allowed; text cells hold counted blocks, so it is not.
    ok(&lines(&[PTRS, owner, "လိပ်စာ o ရှိ ၈ ဘိုက်စာ မင်မိုရီကို လိပ်စာ q ဆီသို့ ကူးယူမည်။"]));
    ok(&lines(&[
        PTRS,
        "လိပ်စာ q ရှိ မင်မိုရီကို မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ ဖြင့် ကိန်းပြည့် လိပ်စာ ၁ ခုစာ ဖြည့်မည်။",
    ]));
    let texts = "ကိန်းရှင် ts သည် စာသား လိပ်စာ တစ်ခု ဖြစ်ပါစေ။";
    let cells = lines(&[PTRS, texts, "လိပ်စာ ts ရှိ ၈ ဘိုက်စာ မင်မိုရီကို လိပ်စာ ts ဆီသို့ ကူးယူမည်။"]);
    assert_eq!(codes(&cells), [Code::BlitOfHeldCells]);
}
