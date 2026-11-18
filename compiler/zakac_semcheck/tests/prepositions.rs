use zakac_diagnostics::Code;
use zakac_rust_sugars::tern;

use crate::common::codes;
use crate::common::ok;

/// A function whose parameters have prepositions.
const WRITE: &str = "\
ရေး ဟူသော လုပ်ဆောင်ချက် သည်
စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"သို့\" ကို သုံး၍ \
                     လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စာ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// A program: `WRITE`, then a start block that holds `stmts`.
fn program(stmts: &str) -> String
{
    format!("{WRITE}ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

#[test]
fn a_preposition_call_with_the_declared_prepositions_checks()
{
    ok(&program("\"က\" ကို \"ခ\" သို့ ရေး မည်။"));
    ok(&program("\"က\" ကို \"ခ\" သို့ ရေး ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။"));
    ok(&program("ရေး ဟူသော လုပ်ဆောင်ချက်ကို \"က\" နှင့် \"ခ\" တို့ဖြင့် လုပ်ဆောင်မည်။"));
}

#[test]
fn a_mark_other_than_the_parameters_is_zk0331()
{
    let wrong = program("\"က\" ကို \"ခ\" ထဲသို့ ရေး မည်။");
    assert_eq!(codes(&wrong), [Code::WrongPreposition]);
}

/// A function kind with two text parameters and no prepositions.
const PLAIN_KIND: &str =
    "စာသား တစ်ခု ကို လည်းကောင်း စာသား တစ်ခု ကို လည်းကောင်း လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား";

/// A declaration of `ဖ` that starts as `ရေး`: with no kind written, it keeps
/// the prepositions of `ရေး`; with `PLAIN_KIND`, it has none.
fn holds_write(plain: bool) -> String
{
    let kind = tern!(plain, format!("{PLAIN_KIND} တစ်ခု ဖြစ်၍ တန်ဖိုးသည် "), String::new());
    format!("ကိန်းရှင် ဖ သည် {kind}ရေး ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။")
}

/// A block that holds `stmts`.
fn block(stmts: &str) -> String
{
    format!("အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။")
}

/// The preposition call through `ဖ`.
const CALL: &str = "\"က\" ကို \"ခ\" သို့ ဖ မည်။";

#[test]
fn a_value_with_no_kind_written_keeps_the_prepositions_of_its_function()
{
    ok(&program(&format!("{}\n{CALL}", holds_write(false))));
}

/// The kind of a value, not the function it holds, gives the prepositions of
/// a call through it, though an outer name of the same spelling reads as a
/// preposition call.
#[test]
fn a_preposition_call_through_a_kind_with_none_is_zk0332()
{
    let inner = block(&format!("{}\n{CALL}", holds_write(true)));
    let shadows = program(&format!("{}\n{inner}", holds_write(false)));
    assert_eq!(codes(&shadows), [Code::NoOverloadFits]);
    let inner = block(&format!("{}\n{CALL}", holds_write(false)));
    let shadowed = program(&format!("{}\n{inner}", holds_write(true)));
    ok(&shadowed);
    let after = program(&format!("{}\n{inner}\n{CALL}", holds_write(true)));
    assert_eq!(codes(&after), [Code::NoOverloadFits]);
}

/// A parameter of a kind with no prepositions takes no preposition call, though
/// a variable of its name elsewhere has prepositions.
#[test]
fn a_parameter_of_a_kind_with_none_is_zk0332()
{
    let send = format!(
        "ပို့ ဟူသော လုပ်ဆောင်ချက် သည်\nဖ ဟူသော ({PLAIN_KIND}) တစ်ခု ကို လက်ခံပြီး\n{}",
        block(CALL)
    );
    let text = format!("{}\n{send}", program(&holds_write(false)));
    assert_eq!(codes(&text), [Code::NoOverloadFits]);
}

/// A function `ဆွဲ` whose parameters take `ကို`, then `with` and `through`
/// in the words of `pair`.
fn draw(pair: (&str, &str)) -> String
{
    let (a, b) = pair;
    format!(
        "ဆွဲ ဟူသော လုပ်ဆောင်ချက် သည်\nစာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လည်းကောင်း ခ ဟူသော \
         စာသား တစ်ခု ကို ဝိဘတ် \"{a}\" ကို သုံး၍ လည်းကောင်း ဂ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"{b}\" ကို သုံး၍ \
         လည်းကောင်း လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\nစာ ကို ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n"
    )
}

#[test]
fn with_fills_through_in_a_function_with_no_with_of_its_own()
{
    let start = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n";
    let end = "\nပရိုဂရမ် ပြီးဆုံးပါပြီး။";
    let through = draw(("သို့", "ဖြင့်"));
    let call = "\"က\" ကို \"ခ\" သို့ \"ဂ\" နှင့် ဆွဲ မည်။";
    ok(&format!("{through}{start}{call}{end}"));
    // With a `နှင့်` of its own, the function needs `ဖြင့်` written.
    let both = draw(("နှင့်", "ဖြင့်"));
    let call = "\"က\" ကို \"ခ\" နှင့် \"ဂ\" နှင့် ဆွဲ မည်။";
    assert!(!codes(&format!("{both}{start}{call}{end}")).is_empty());
}

/// `ဖတ်`, with both phrases and one parameter, and `ရှင်း`, with phrases and
/// no parameters.
const PHRASED: &str = "\
ဖတ် ဟူသော လုပ်ဆောင်ချက် သည်
အစ စကားစု \"ဖိုင်\" နှင့် အဆုံး စကားစု \"စာကို\" တို့ကို သုံး၍
လမ်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"ထဲမှ\" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
လမ်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ရှင်း ဟူသော လုပ်ဆောင်ချက် သည် အစ စကားစု \"ဖန်သားပြင်ကို\" ကို သုံး၍
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// A program: `PHRASED` and `more`, then a start block that holds `stmts`.
fn phrased(more: &str, stmts: &str) -> String
{
    format!("{PHRASED}{more}ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

#[test]
fn a_call_with_the_phrases_of_its_head_checks()
{
    ok(&phrased("", "ဖိုင် \"a\" ထဲမှ စာကို ဖတ် သည့် ရလဒ် ကို ပြသမည်။"));
    ok(&phrased("", "ဖန်သားပြင်ကို ရှင်း မည်။"));
    ok(&phrased("", "ဖတ်(\"a\") ကို ပြသမည်။\nရှင်း မည်။"));
}

#[test]
fn a_sentence_call_without_the_phrases_is_zk0352()
{
    let bare = phrased("", "\"a\" ထဲမှ စာကို ဖတ် သည့် ရလဒ် ကို ပြသမည်။");
    assert_eq!(codes(&bare), [Code::WrongPhrase]);
}

/// A second `ရှင်း` with another starting phrase.
const CLEAR_LIST: &str = "\
ရှင်း ဟူသော လုပ်ဆောင်ချက် သည် အစ စကားစု \"စာရင်းကို\" ကို သုံး၍
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

#[test]
fn phrases_tell_functions_of_one_name_apart()
{
    ok(&phrased(CLEAR_LIST, "ဖန်သားပြင်ကို ရှင်း မည်။\nစာရင်းကို ရှင်း မည်။"));
    assert_eq!(codes(&phrased(CLEAR_LIST, "ရှင်း မည်။")), [Code::OverloadUnclear]);
    let twin = CLEAR_LIST.replace("စာရင်းကို", "ဖန်သားပြင်ကို");
    assert_eq!(codes(&phrased(&twin, "")), [Code::DuplicateFunction]);
}

/// `ဖတ်` as a value: through a variable with no kind written, through a kind
/// with its phrases, and through a kind with none.
#[test]
fn a_function_value_keeps_its_phrases()
{
    let kind = "(အစ စကားစု \"ဖိုင်\" နှင့် အဆုံး စကားစု \"စာကို\" တို့ကို သုံး၍ စာသား တစ်ခု ကို ဝိဘတ် \"ထဲမှ\" ကို \
                သုံး၍ လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား)";
    ok(&phrased(
        "",
        "ကိန်းရှင် ဖ သည် ဖတ် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။\nဖိုင် \"a\" ထဲမှ စာကို ဖ သည့် ရလဒ် ကို ပြသမည်။",
    ));
    let declared = format!(
        "ကိန်းရှင် ဂ သည် {kind} တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဖတ် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။\nဖိုင် \"b\" ထဲမှ \
         စာကို ဂ သည့် ရလဒ် ကို ပြသမည်။"
    );
    ok(&phrased("", &declared));
    let plain = "ကိန်းရှင် ဃ သည် (စာသား တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် \
                 အမျိုးအစား) တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဖတ် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။\nဃ(\"c\") ကို \
                 ပြသမည်။";
    ok(&phrased("", plain));
}

/// A kind with phrases takes no function without them.
#[test]
fn a_kind_with_phrases_wants_them()
{
    let bare = "\
ပြ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";
    let decl = "ကိန်းရှင် ဖ သည် (အစ စကားစု \"ဖန်သားပြင်ကို\" ကို သုံးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု \
                ဖြစ်၍ တန်ဖိုးသည် ပြ ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။";
    assert!(!codes(&phrased(bare, decl)).is_empty());
}

/// `ကူး`, whose first parameter takes `မှ` and whose second takes none.
const MOVE_TEXT: &str = "\
ကူး ဟူသော လုပ်ဆောင်ချက် သည်
ရင်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"မှ\" ကို သုံး၍ လည်းကောင်း ပစ် ဟူသော စာသား တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ရင်း ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// A call gives a preposition after the arguments whose parameters have
/// one; a wrong word there is ZK0331.
#[test]
fn some_parameters_take_no_preposition()
{
    let call = |stmts: &str| phrased(MOVE_TEXT, stmts);
    ok(&call("\"က\" မှ \"ခ\" ကူး မည်။"));
    ok(&call("ကူး ဟူသော လုပ်ဆောင်ချက်ကို \"က\" နှင့် \"ခ\" တို့ဖြင့် လုပ်ဆောင်မည်။"));
    assert_eq!(codes(&call("\"က\" ကို \"ခ\" ကူး မည်။")), [Code::WrongPreposition]);
}
