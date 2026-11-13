use zakac_diagnostics::Code;

use crate::common::codes;
use crate::common::ok;

/// A function whose parameters have marks.
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
fn a_marked_call_with_the_declared_marks_checks()
{
    ok(&program("\"က\" ကို \"ခ\" သို့ ရေး မည်။"));
    ok(&program("\"က\" ကို \"ခ\" သို့ ရေး ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။"));
    ok(&program("ရေး ဟူသော လုပ်ဆောင်ချက်ကို \"က\"၊ \"ခ\" တို့ဖြင့် လုပ်ဆောင်မည်။"));
}

#[test]
fn a_mark_other_than_the_parameters_is_zk0331()
{
    let wrong = program("\"က\" ကို \"ခ\" ထဲသို့ ရေး မည်။");
    assert_eq!(codes(&wrong), [Code::WrongMark]);
}
