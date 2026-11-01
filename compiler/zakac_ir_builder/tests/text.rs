mod common;

use common::codes;
use common::codes_with;
use common::folded;
use common::folded_with;
use common::plain;
use common::run;
use zakac_ast::s_expr::ToSexpr;
use zakac_diagnostics::Code;
use zakac_ir_builder::ir::FoldOptions;

/// The statements of the first function of `text`, folded.
fn func_forms(text: &str) -> Vec<String>
{
    let lowered = run(text, &FoldOptions::default());
    let module = lowered.folded.expect("folded");
    module.funcs[0].block.iter().map(ToSexpr::to_sexpr).collect()
}

#[test]
fn literals_lower_to_static_text_or_a_format()
{
    let text = "ကိန်းရှင် က သည် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။\n\"က = {က}\" ဟု ပြသပါ။";
    assert_eq!(plain(text)[1], "(print (format \"က = \" g0))");
    let text = "ကိန်းရှင် t သည် စာသား ဖြစ်၍ တန်ဖိုးသည် \"ab\" ဖြစ်ပါစေ။";
    assert_eq!(folded(text)[0], "(set g0 \"ab\")");
}

#[test]
fn constant_template_slots_and_joins_fold_to_static_text()
{
    assert_eq!(folded("\"{၁ + ၁} {မှန်} {'က'}\" ဟု ပြသပါ။"), ["(print \"၂ မှန် က\")"]);
    assert_eq!(folded("\"a\" + \"b\" + \"{၃}\" ဟု ပြသပါ။"), ["(print \"ab၃\")"]);
    let text = "ကိန်းသေ က သည် ကိန်းပြည့် ၇ ဖြစ်ပါစေ။\n\"{က}\" ဟု ပြသပါ။";
    assert_eq!(folded(text)[1], "(print \"၇\")");
    assert_eq!(
        folded("\"a {\"b {\"c {၁ + ၁}\"}\"} d\" ဟု ပြသပါ။"),
        ["(print \"a b c ၂ d\")"]
    );
    let off = FoldOptions::none();
    assert_eq!(
        folded_with("\"a\" + \"{၃}\" ဟု ပြသပါ။", &off),
        ["(print (format \"a\" (format 3:isize)))"]
    );
}

#[test]
fn places_that_hold_text_are_dropped_as_their_scope_ends()
{
    let text = "လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် t သည် စာသား ဖြစ်၍ တန်ဖိုးသည် \"ab\" ဖြစ်ပါစေ။
t ကို \"cd\" ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(
        func_forms(text),
        [
            "(set l0 \"ab\")",
            "(set l1 \"cd\")",
            "(drop l0)",
            "(set l0 (take (addr l1)))",
            "(drop l0)",
        ]
    );
}

#[test]
fn a_return_keeps_its_value_before_the_drops()
{
    let text = "လုပ်ဆောင်ချက် f သည် စာသား t ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
t ၏ အကွက်အရေအတွက် ကို ကိန်းပြည့် အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    let forms = func_forms(text);
    let tail = &forms[forms.len() - 3..];
    assert!(tail[0].starts_with("(set l1 "), "{forms:?}");
    assert_eq!(tail[1..], ["(drop l0)", "(return l1)"]);
}

#[test]
fn a_break_drops_the_text_of_the_loop_block()
{
    let text = "ကိန်းရှင် i သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။
i သည် ၃ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်းရှင် t သည် စာသား ဖြစ်၍ တန်ဖိုးသည် \"{i}\" ဖြစ်ပါစေ။
ကွင်းဆက် ကို ရပ်ပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    let forms = folded(text);
    assert!(forms[1].contains("(drop g1) (break)"), "{forms:?}");
}

/// A text constant and a template that reads it.
const NAMED: &str = "ကိန်းသေ အမည် သည် စာသား ဖြစ်၍ တန်ဖိုးသည် \"မမ\" ဖြစ်ပါစေ။";

#[test]
fn text_constants_fold_into_templates_and_chains()
{
    let text = format!("{NAMED}\n\"ဟေး {{အမည်}}\" ဟု ပြသပါ။");
    assert_eq!(folded(&text)[1], "(print \"ဟေး မမ\")");
    let chain = format!(
        "{NAMED}\nကိန်းသေ ခ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် \"{{အမည်}}!\" ဖြစ်ပါစေ။\nကိန်းသေ ဂ သည် \
         စာသား ဖြစ်၍ တန်ဖိုးသည် ခ + ခ ဖြစ်ပါစေ။"
    );
    assert_eq!(folded(&chain)[2], "(set g2 \"မမ!မမ!\")");
}

#[test]
fn a_text_constant_holds_static_text_whatever_the_options()
{
    let text = format!(
        "{NAMED}\nကိန်းသေ ခ သည် စာသား ဖြစ်၍ တန်ဖိုးသည် \"{{အမည်}}\" + \"!\" ဖြစ်ပါစေ။\nကိန်းရှင် x \
         သည် စာသား ဖြစ်၍ တန်ဖိုးသည် အမည် ဖြစ်ပါစေ။"
    );
    let forms = folded_with(&text, &FoldOptions::none());
    assert_eq!(forms[1], "(set g1 \"မမ!\")");
    assert_eq!(forms[2], "(set g2 g0)");
}

#[test]
fn a_text_constant_that_needs_the_program_to_run_is_zk0606()
{
    let var =
        "ကိန်းရှင် n သည် ကိန်းပြည့် ၃ ဖြစ်ပါစေ။\nကိန်းသေ က သည် စာသား ဖြစ်၍ တန်ဖိုးသည် \"x {n}\" ဖြစ်ပါစေ။";
    assert_eq!(codes(var), [Code::TextConstNotKnown]);
    assert_eq!(codes_with(var, &FoldOptions::none()), [Code::TextConstNotKnown]);
    let call = "လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရလဒ် ကို ပြန်ပေးမည်။\n\"z\" ကို \
                ရလဒ်အဖြစ် ပြန်ပေးပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီ။\nကိန်းသေ ခ သည် စာသား ဖြစ်၍ \
                တန်ဖိုးသည် လုပ်ဆောင်ချက် f ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။";
    assert_eq!(codes(call), [Code::TextConstNotKnown]);
}

#[test]
fn a_range_of_known_text_that_cuts_a_character_is_zk0607()
{
    let text = "ကိန်းသေ t သည် စာသား ဖြစ်၍ တန်ဖိုးသည် \"ကမ္ဘာ\" ဖြစ်ပါစေ။";
    assert_eq!(
        codes(&format!("{text}\nt ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ကို ပြသပါ။")),
        [Code::RangeCutsChar]
    );
    assert!(codes(&format!("{text}\nt ၏ အကွက် ၀ မှစ၍ ၃ ကွက် ကို ပြသပါ။")).is_empty());
}

#[test]
fn characters_lower_to_char_reads()
{
    let text = "ကိန်းရှင် t သည် စာသား ဖြစ်၍ တန်ဖိုးသည် \"ab\" ဖြစ်ပါစေ။\nt ၏ စာလုံးအရေအတွက် ကို \
                ပြသပါ။\nt ၏ စာလုံး ၀ ကို ပြသပါ။\nt ၏ ၁ ခုမြောက်စာလုံး ကို ပြသပါ။";
    let forms = plain(text);
    assert_eq!(
        forms[1..4],
        [
            "(print (chars g0))",
            "(print (char@ g0 0:usize))",
            "(print (char1@ g0 1:usize))"
        ]
    );
}

#[test]
fn a_verbal_join_folds_as_a_join_does()
{
    let text = "\"a\"၊ \"{၁}\"၊ \"c\" တို့ကို ဆက်ထားသည့် စာသား ကို ပြသပါ။";
    assert_eq!(folded(text), ["(print \"a၁c\")"]);
    assert_eq!(
        folded_with(text, &FoldOptions::none()),
        ["(print (format \"a\" (format 1:isize) \"c\"))"]
    );
}
