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
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";

/// `က` as an `i8` with value 3, `ခ` as a `u8` with value 4.
const I8_U8: &str = "ကိန်းရှင် က သည် ၃ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ၄ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

#[test]
fn unprefixed_literals_take_the_expected_kind()
{
    let checked = check("ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။");
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "၅"), "i8");
    let checked = check("ကိန်းရှင် က သည် ၃၂ ဘစ် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။");
    assert_eq!(kind_at(&checked, "၃"), "f32");
    assert_eq!(kind_at(&check("၅ ကို ပြသမည်။"), "၅"), "isize");
    assert_eq!(kind_at(&check("၁.၅ ကို ပြသမည်။"), "၁.၅"), "f64");
}

#[test]
fn prefixed_literals_keep_their_kind()
{
    let checked =
        check("ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။");
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "၅ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု"), "i8");
}

#[test]
fn literals_must_fit_the_expected_kind()
{
    assert_eq!(
        codes("ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂၀၀ ဖြစ်ပါစေ။"),
        [Code::LiteralDoesNotFit]
    );
    assert_eq!(
        codes("ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၂၈ ဖြစ်ပါစေ။"),
        [Code::LiteralDoesNotFit]
    );
    ok("ကိန်းရှင် က သည် ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် -၁၂၈ ဖြစ်ပါစေ။");
    assert_eq!(
        codes("ကိန်းရှင် က သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် -၁ ဖြစ်ပါစေ။"),
        [Code::LiteralDoesNotFit]
    );
}

#[test]
fn a_literal_operand_takes_the_other_operand_kind()
{
    let text = lines(&[
        I8_U8,
        "ကိန်းရှင် ဂ သည် လက္ခဏာမဲ့ ၁၆ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခ + ၁ ဖြစ်ပါစေ။",
    ]);
    let checked = check(&text);
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "ခ + ၁"), "u8");
    assert_eq!(kind_at(&checked, "၁"), "u8");
    let verbal = check(&lines(&[I8_U8, "ခ နှင့် ၁ ၏ ပေါင်းလဒ် ကို ပြသမည်။"]));
    assert_eq!(kind_at(&verbal, "ခ နှင့် ၁ ၏ ပေါင်းလဒ်"), "u8");
}

#[test]
fn int_and_float_literals_meet_as_a_float()
{
    let checked = check("၁.၅ + ၂ ကို ပြသမည်။");
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "၁.၅ + ၂"), "f64");
}

#[test]
fn values_widen_to_their_place()
{
    ok(&lines(&[I8_U8, "ကိန်းရှင် ဂ သည် ၁၆ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။"]));
    ok(&lines(&[I8_U8, "ကိန်းရှင် ဂ သည် ၁၆ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခ ဖြစ်ပါစေ။"]));
    ok(&lines(&[I8_U8, "ကိန်းရှင် ဂ သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။"]));
    let narrowing =
        lines(&[I8_U8, "ကိန်းရှင် ဂ သည် လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။"]);
    assert_eq!(codes(&narrowing), [Code::KindMismatch]);
    let char_to_int = lines(&[I8_U8, "က ကို 'က' ဟု ထားမည်။"]);
    assert_eq!(codes(&char_to_int), [Code::KindMismatch]);
}

#[test]
fn platform_width_comes_from_the_options()
{
    let text = "ကိန်းရှင် က သည် ၁ ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။";
    ok(text);
    let narrow = SemcheckOptions::builder().platform_bits(32).build();
    let found: Vec<Code> =
        check_with(text, &narrow).out.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(found, [Code::KindMismatch]);
}

#[test]
fn conditions_are_booleans()
{
    assert_eq!(codes("အကယ်၍ ၁ ဖြစ်လျှင် \"x\" ဟု ပြသမည်။"), [Code::ConditionNotBool]);
    ok("အကယ်၍ ၁ သည် ၂ အောက် ငယ်လျှင် \"x\" ဟု ပြသမည်။");
    ok("အကယ်၍ မှန် မဟုတ်လျှင် \"x\" ဟု ပြသမည်။");
}

#[test]
fn operators_take_their_own_kinds()
{
    assert_eq!(codes("မှန် + ၁ ကို ပြသမည်။"), [Code::OperandKind]);
    assert_eq!(codes("!၁ ကို ပြသမည်။"), [Code::OperandKind]);
    assert_eq!(codes("'က' & ၁ ကို ပြသမည်။"), [Code::OperandKind]);
    assert_eq!(codes("(၁ ဖြစ်၍ မှန်) ကို ပြသမည်။"), [Code::OperandKind]);
    assert_eq!(codes("(မှန် < မမှန်) ကို ပြသမည်။"), [Code::OperandKind]);
    assert_eq!(codes(&lines(&[I8_U8, "-ခ ကို ပြသမည်။"])), [Code::OperandKind]);
    ok("(၁ > ၀ ဖြစ်၍ မှန်) ကို ပြသမည်။");
}

#[test]
fn comparisons_give_booleans()
{
    let checked = check("'က' < 'ခ' ကို ပြသမည်။");
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "'က' < 'ခ'"), "bool");
    ok("မှန် == မမှန် ကို ပြသမည်။");
    assert_eq!(codes("'က' == ၁ ကို ပြသမည်။"), [Code::IncompatibleOperands]);
}

#[test]
fn operands_need_a_common_kind()
{
    assert_eq!(codes(&lines(&[I8_U8, "က + ခ ကို ပြသမည်။"])), [Code::IncompatibleOperands]);
    let checked = check(&lines(&[
        I8_U8,
        "ကိန်းရှင် ဂ သည် ၁ ဟူသော ၁၆ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။",
        "ခ + ဂ ကို ပြသမည်။",
    ]));
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "ခ + ဂ"), "i16");
}

#[test]
fn counted_loops_count_with_integers()
{
    let text = "၁.၅ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။";
    assert_eq!(codes(text), [Code::LoopBoundNotInt]);
    let text = "၁ မှ ၁၀ အထိ ၀ စီ ကိန်း အနေဖြင့် တိုး၍ ကိန်း ကို ပြသမည်။";
    assert_eq!(codes(text), [Code::StepNotPositive]);
    let text = lines(&[I8_U8, "၁ မှ က အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။"]);
    let checked = check(&text);
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "ကိန်း"), "i8");
}

#[test]
fn shifts_take_an_unsigned_distance()
{
    let checked = check("၁ << ၃ ကို ပြသမည်။");
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "၃"), "usize");
    assert_eq!(kind_at(&checked, "၁ << ၃"), "isize");
    assert_eq!(codes("၁ << ၆၄ ကို ပြသမည်။"), [Code::ShiftTooFar]);
    assert_eq!(codes(&lines(&[I8_U8, "၁ << က ကို ပြသမည်။"])), [Code::OperandKind]);
    ok("၄ ကို ညာသို့ ၂ ဘစ် ရွေ့သောရလဒ် ကို ပြသမည်။");
}

#[test]
fn conversions_follow_the_table()
{
    let checked = check("၁၁.၁၁ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။");
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "၁၁.၁၁ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်"), "isize");
    ok("'က' ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။");
    assert_eq!(
        codes("မှန် ကို ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။"),
        [Code::InvalidCast]
    );
    assert_eq!(
        codes("၁ ကို မှန်မှားတန်ဖိုး တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။"),
        [Code::InvalidCast]
    );
}

#[test]
fn calls_check_argument_and_return_kinds()
{
    let checked =
        check(&lines(&[TWICE, "နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို ၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။"]));
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို ၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်"), "isize");
    let bad_arg = lines(&[TWICE, "နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို 'က' တို့ဖြင့် လုပ်ဆောင်မည်။"]);
    assert_eq!(codes(&bad_arg), [Code::KindMismatch]);
    let bad_return = lines(&[
        "လုပ်ဆောင်ချက် ဂဏန်း သည်",
        "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။",
        "၁.၅ ကို ပြန်ပေးမည်။",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    ]);
    assert_eq!(codes(&bad_return), [Code::KindMismatch]);
}

#[test]
fn one_mistake_gives_one_diagnostic()
{
    assert_eq!(codes("(က + ၁) * ၂ ကို ပြသမည်။"), [Code::UndeclaredName]);
    assert_eq!(
        codes("ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (က + ၁) * ၂ ဖြစ်ပါစေ။"),
        [Code::UndeclaredName]
    );
}

#[test]
fn integer_operators_ignore_a_float_context()
{
    // An integer operator keeps its integer kind in a float context, and
    // no float kind holds each value of `isize`.
    let checked = check("ကိန်းရှင် က သည် ၆၄ ဘစ် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁ << ၂ ဖြစ်ပါစေ။");
    let found: Vec<Code> = checked.out.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(found, [Code::KindMismatch]);
    assert_eq!(kind_at(&checked, "၁ << ၂"), "isize");
    assert_eq!(
        codes("ကိန်းရှင် က သည် ၆၄ ဘစ် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၆ & ၃ ဖြစ်ပါစေ။"),
        [Code::KindMismatch]
    );
    assert_eq!(
        codes("ကိန်းရှင် က သည် ၆၄ ဘစ် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ~၆ ဖြစ်ပါစေ။"),
        [Code::KindMismatch]
    );
    assert_eq!(
        codes(
            "ကိန်းရှင် က သည် ၆၄ ဘစ် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၆ ၏ ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ် \
             ဖြစ်ပါစေ။"
        ),
        [Code::KindMismatch]
    );
}

#[test]
fn unprefixed_literals_reach_the_full_range_of_their_kind()
{
    ok("ကိန်းရှင် က သည် 18446744073709551615 ဟူသော လက္ခဏာမဲ့ ၆၄ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။");
    assert_eq!(codes("18446744073709551615 ကို ပြသမည်။"), [Code::LiteralDoesNotFit]);
}

#[test]
fn a_mismatch_names_the_conversion_verb_when_it_can_convert()
{
    let out = common::check("ကိန်းရှင် အ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။").out;
    let found: Vec<(Code, Option<String>)> =
        out.diagnostics.iter().map(|d| (d.code, d.help.clone())).collect();
    assert!(found[0].1.is_some(), "{found:?}");
    let out = common::check("ကိန်းရှင် အ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၂.၅ ဖြစ်ပါစေ။").out;
    assert!(out.diagnostics[0].help.as_ref().is_some_and(|h| h.contains("ပြောင်းလဲသော")));
}
