use crate::common::codes;

/// The kind of a number or a text error.
const RESULT: &str = "ကိန်းပြည့် သို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာသား ရှိနိုင်သော ဒေတာဆဲလ်";

/// A start block with the statements `stmts`.
fn start(stmts: &str) -> String
{
    format!("ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

/// A variable `name` of `kind` that starts at `value`.
fn var(name: &str, kind: &str, value: &str) -> String
{
    format!("ကိန်းရှင် {name} သည် {kind} တစ်ခု ဖြစ်၍ တန်ဖိုးသည် {value} ဖြစ်ပါစေ။\n")
}

/// A function `စစ်` with the header line `head` and the statements `stmts`,
/// then a start block.
fn function(head: &str, stmts: &str) -> String
{
    format!(
        "လုပ်ဆောင်ချက် စစ် သည်\nန ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး\n{head}\n{stmts}\nလုပ်ဆောင်စရာများ \
         ပြီးဆုံးပါပြီး။\n{}",
        start("")
    )
}

/// Whether the parser rejects `text` with parse errors and nothing else.
fn rejected(text: &str) -> bool
{
    let found = codes(text);
    !found.is_empty() && found.iter().all(|c| c.to_string().starts_with("ZK02"))
}

#[test]
fn each_old_result_spelling_is_a_parse_error()
{
    let head = "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။";
    assert!(rejected(&function(head, "န ကို ပြန်ပေးမည်။")));
    let head = "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။";
    assert!(rejected(&function(head, "န ကို ရလဒ်အဖြစ် ပြန်ပေးမည်။")));
    let head = format!("အောက်ပါတို့ကို လုပ်ဆောင်ကာ {RESULT} တစ်ခု ကို ပြန်ပေးမည်။");
    assert!(rejected(&function(&head, "န ကို ရလဒ်ကောင်းအဖြစ် ပြန်ပေးမည်။")));
    assert!(rejected(&function(&head, "\"မရ\" ကို အမှားအယွင်းအဖြစ် ပြန်ပေးမည်။")));
    let head = "အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် သို့မဟုတ် စာသား အမှားအယွင်း ကို ပြန်ပေးမည်။";
    assert!(rejected(&function(head, "န ကို ပြန်ပေးမည်။")));
    let test =
        format!("{}အကယ်၍ ရ သည် ရလဒ်ကောင်း ဖြစ်လျှင် \"ရ\" ဟု ပြသမည်။", var("ရ", RESULT, "၃"));
    assert!(rejected(&start(&test)));
    let read =
        format!("{}ရ ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။", var("ရ", RESULT, "၃"));
    assert!(rejected(&start(&read)));
}
