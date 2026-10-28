mod common;

use common::check;
use common::check_with;
use common::codes;
use common::kind_at;
use common::ok;
use zakac_diagnostics::Code;
use zakac_semcheck::semcheck::SemcheckOptions;

/// A function with one integer parameter and an integer result.
const TWICE: &str = "လုပ်ဆောင်ချက် နှစ်ဆ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";

/// `က` as an `i8` with value 3, `ခ` as a `u8` with value 4.
const I8_U8: &str = "ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် ၄ ဖြစ်ပါစေ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

#[test]
fn unprefixed_literals_take_the_expected_kind()
{
    let checked = check("ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။");
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "၅"), "i8");
    let checked = check("ကိန်းရှင် က သည် ၃၂ ဘစ် ဒဿမကိန်း ၃ ဖြစ်ပါစေ။");
    assert_eq!(kind_at(&checked, "၃"), "f32");
    assert_eq!(kind_at(&check("၅ ကို ပြသပါ။"), "၅"), "isize");
    assert_eq!(kind_at(&check("၁.၅ ကို ပြသပါ။"), "၁.၅"), "f64");
}

#[test]
fn prefixed_literals_keep_their_kind()
{
    let checked = check("ကိန်းရှင် က သည် ကိန်းပြည့် ၈ ဘစ် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။");
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "၈ ဘစ် ကိန်းပြည့် ၅"), "i8");
}

#[test]
fn literals_must_fit_the_expected_kind()
{
    assert_eq!(codes("ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် ၂၀၀ ဖြစ်ပါစေ။"), [Code::LiteralDoesNotFit]);
    assert_eq!(codes("ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် ၁၂၈ ဖြစ်ပါစေ။"), [Code::LiteralDoesNotFit]);
    ok("ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် -၁၂၈ ဖြစ်ပါစေ။");
    assert_eq!(
        codes("ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် -၁ ဖြစ်ပါစေ။"),
        [Code::LiteralDoesNotFit]
    );
}

#[test]
fn a_literal_operand_takes_the_other_operand_kind()
{
    let text = lines(&[I8_U8, "ကိန်းရှင် ဂ သည် လက္ခဏာမဲ့ ၁၆ ဘစ် ကိန်းပြည့် ခ + ၁ ဖြစ်ပါစေ။"]);
    let checked = check(&text);
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "ခ + ၁"), "u8");
    assert_eq!(kind_at(&checked, "၁"), "u8");
    let verbal = check(&lines(&[I8_U8, "ခ နှင့် ၁ ၏ ပေါင်းလဒ် ကို ပြသပါ။"]));
    assert_eq!(kind_at(&verbal, "ခ နှင့် ၁ ၏ ပေါင်းလဒ်"), "u8");
}

#[test]
fn int_and_float_literals_meet_as_a_float()
{
    let checked = check("၁.၅ + ၂ ကို ပြသပါ။");
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "၁.၅ + ၂"), "f64");
}

#[test]
fn values_widen_to_their_place()
{
    ok(&lines(&[I8_U8, "ကိန်းရှင် ဂ သည် ၁၆ ဘစ် ကိန်းပြည့် က ဖြစ်ပါစေ။"]));
    ok(&lines(&[I8_U8, "ကိန်းရှင် ဂ သည် ၁၆ ဘစ် ကိန်းပြည့် ခ ဖြစ်ပါစေ။"]));
    ok(&lines(&[I8_U8, "ကိန်းရှင် ဂ သည် ဒဿမကိန်း က ဖြစ်ပါစေ။"]));
    let narrowing = lines(&[I8_U8, "ကိန်းရှင် ဂ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် က ဖြစ်ပါစေ။"]);
    assert_eq!(codes(&narrowing), [Code::KindMismatch]);
    let char_to_int = lines(&[I8_U8, "က ကို 'က' ဟု ထားပါ။"]);
    assert_eq!(codes(&char_to_int), [Code::KindMismatch]);
}

#[test]
fn platform_width_comes_from_the_options()
{
    let text = "ကိန်းရှင် က သည် ၃၂ ဘစ် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် က ဖြစ်ပါစေ။";
    ok(text);
    let narrow = SemcheckOptions::builder().platform_bits(32).build();
    let found: Vec<Code> =
        check_with(text, &narrow).out.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(found, [Code::KindMismatch]);
}

#[test]
fn conditions_are_booleans()
{
    assert_eq!(codes("အကယ်၍ ၁ ဖြစ်လျှင် \"x\" ဟု ပြသပါ။"), [Code::ConditionNotBool]);
    ok("အကယ်၍ ၁ သည် ၂ အောက် ငယ်လျှင် \"x\" ဟု ပြသပါ။");
    ok("အကယ်၍ မှန် မဟုတ်လျှင် \"x\" ဟု ပြသပါ။");
}

#[test]
fn operators_take_their_own_kinds()
{
    assert_eq!(codes("မှန် + ၁ ကို ပြသပါ။"), [Code::OperandKind]);
    assert_eq!(codes("!၁ ကို ပြသပါ။"), [Code::OperandKind]);
    assert_eq!(codes("'က' & ၁ ကို ပြသပါ။"), [Code::OperandKind]);
    assert_eq!(codes("(၁ ဖြစ်၍ မှန်) ကို ပြသပါ။"), [Code::OperandKind]);
    assert_eq!(codes("(မှန် < မမှန်) ကို ပြသပါ။"), [Code::OperandKind]);
    assert_eq!(codes(&lines(&[I8_U8, "-ခ ကို ပြသပါ။"])), [Code::OperandKind]);
    ok("(၁ > ၀ ဖြစ်၍ မှန်) ကို ပြသပါ။");
}

#[test]
fn comparisons_give_booleans()
{
    let checked = check("'က' < 'ခ' ကို ပြသပါ။");
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "'က' < 'ခ'"), "bool");
    ok("မှန် == မမှန် ကို ပြသပါ။");
    assert_eq!(codes("'က' == ၁ ကို ပြသပါ။"), [Code::IncompatibleOperands]);
}

#[test]
fn operands_need_a_common_kind()
{
    assert_eq!(codes(&lines(&[I8_U8, "က + ခ ကို ပြသပါ။"])), [Code::IncompatibleOperands]);
    let checked =
        check(&lines(&[I8_U8, "ကိန်းရှင် ဂ သည် ၁၆ ဘစ် ကိန်းပြည့် ၁ ဖြစ်ပါစေ။", "ခ + ဂ ကို ပြသပါ။"]));
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "ခ + ဂ"), "i16");
}

#[test]
fn counted_loops_count_with_integers()
{
    let text = "၁.၅ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။";
    assert_eq!(codes(text), [Code::LoopBoundNotInt]);
    let text = "၁ မှ ၁၀ အထိ ၀ စီ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသပါ။";
    assert_eq!(codes(text), [Code::StepNotPositive]);
    let text = lines(&[I8_U8, "၁ မှ က အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။"]);
    let checked = check(&text);
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "ကိန်း"), "i8");
}

#[test]
fn shifts_take_an_unsigned_distance()
{
    let checked = check("၁ << ၃ ကို ပြသပါ။");
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "၃"), "usize");
    assert_eq!(kind_at(&checked, "၁ << ၃"), "isize");
    assert_eq!(codes("၁ << ၆၄ ကို ပြသပါ။"), [Code::ShiftTooFar]);
    assert_eq!(codes(&lines(&[I8_U8, "၁ << က ကို ပြသပါ။"])), [Code::OperandKind]);
    ok("၄ ကို ညာသို့ ၂ ဘစ် ရွေ့သောရလဒ် ကို ပြသပါ။");
}

#[test]
fn conversions_follow_the_table()
{
    let checked = check("၁၁.၁၁ ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။");
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "၁၁.၁၁ ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ်"), "isize");
    ok("'က' ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။");
    assert_eq!(codes("မှန် ကို ဒဿမကိန်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။"), [Code::InvalidCast]);
    assert_eq!(
        codes("၁ ကို မှန်မှားတန်ဖိုး အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသပါ။"),
        [Code::InvalidCast]
    );
}

#[test]
fn calls_check_argument_and_return_kinds()
{
    let checked =
        check(&lines(&[TWICE, "လုပ်ဆောင်ချက် နှစ်ဆ ကို ၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။"]));
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "လုပ်ဆောင်ချက် နှစ်ဆ ကို ၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်"), "isize");
    let bad_arg = lines(&[TWICE, "လုပ်ဆောင်ချက် နှစ်ဆ ကို 'က' တို့ဖြင့် လုပ်ဆောင်ပါ။"]);
    assert_eq!(codes(&bad_arg), [Code::KindMismatch]);
    let bad_return = lines(&[
        "လုပ်ဆောင်ချက် ဂဏန်း သည်",
        "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။",
        "၁.၅ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&bad_return), [Code::KindMismatch]);
}

#[test]
fn one_mistake_gives_one_diagnostic()
{
    assert_eq!(codes("(က + ၁) * ၂ ကို ပြသပါ။"), [Code::UndeclaredName]);
    assert_eq!(codes("ကိန်းရှင် ခ သည် ကိန်းပြည့် (က + ၁) * ၂ ဖြစ်ပါစေ။"), [Code::UndeclaredName]);
}

#[test]
fn integer_operators_ignore_a_float_context()
{
    let checked = check("ကိန်းရှင် က သည် ၁၂၈ ဘစ် ဒဿမကိန်း ၁ << ၂ ဖြစ်ပါစေ။");
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, "၁ << ၂"), "isize");
    ok("ကိန်းရှင် က သည် ၁၂၈ ဘစ် ဒဿမကိန်း ၆ & ၃ ဖြစ်ပါစေ။");
    ok("ကိန်းရှင် က သည် ၁၂၈ ဘစ် ဒဿမကိန်း ~၆ ဖြစ်ပါစေ။");
    ok("ကိန်းရှင် က သည် ၁၂၈ ဘစ် ဒဿမကိန်း ၆ ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ် ဖြစ်ပါစေ။");
}

#[test]
fn unprefixed_literals_reach_the_full_range_of_their_kind()
{
    ok("ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၆၄ ဘစ် ကိန်းပြည့် 18446744073709551615 ဖြစ်ပါစေ။");
    ok("ကိန်းရှင် က သည် ၁၂၈ ဘစ် ကိန်းပြည့် 1180591620717411303424 ဖြစ်ပါစေ။");
    assert_eq!(codes("18446744073709551615 ကို ပြသပါ။"), [Code::LiteralDoesNotFit]);
}
