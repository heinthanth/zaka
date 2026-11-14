use zakac_diagnostics::Code;
use zakac_rust_sugars::tern;

use crate::common::codes;
use crate::common::ok;

/// A function whose parameters have prepositions.
const WRITE: &str = "\
လုပ်ဆောင်ချက် ရေး သည်
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
    let send =
        format!("လုပ်ဆောင်ချက် ပို့ သည်\nဖ ဟူသော ({PLAIN_KIND}) တစ်ခု ကို လက်ခံပြီး\n{}", block(CALL));
    let text = format!("{}\n{send}", program(&holds_write(false)));
    assert_eq!(codes(&text), [Code::NoOverloadFits]);
}
