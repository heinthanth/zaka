use zakac_ast::stmt::Stmt;
use zakac_diagnostics::Code;
use zakac_semcheck::semcheck::SemcheckOptions;

use crate::common::check_as_is;
use crate::common::codes;
use crate::common::codes_as_is;

/// Three functions of one name: an integer marked `ကို`, text marked `ကို`,
/// and text marked `ထဲသို့`.
const SHOW: &str = "\
ပြ ဟူသော လုပ်ဆောင်ချက် သည် န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပြ ဟူသော လုပ်ဆောင်ချက် သည် စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
ပြ ဟူသော လုပ်ဆောင်ချက် သည် စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"ထဲသို့\" ကို သုံး၍ လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// A program: `SHOW`, then a start block that holds `stmts`.
fn program(stmts: &str) -> String
{
    format!("{SHOW}ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

/// A call runs the function its kinds and prepositions fit, and the check
/// writes the inner name of each function into its declaration and its calls.
#[test]
fn a_call_runs_the_function_its_arguments_fit()
{
    let calls = "၅ ကို ပြ မည်။\n\"က\" ကို ပြ မည်။\n\"ခ\" ထဲသို့ ပြ မည်။";
    let checked = check_as_is(&program(calls), &SemcheckOptions::default());
    assert_eq!(checked.out.diagnostics, []);
    let decls = checked.program.stmts.iter().filter_map(|s| match s
    {
        Stmt::Fn(d) if !d.start => Some(d.name.name.to_string()),
        _ => None,
    });
    let decls: Vec<String> = decls.collect();
    assert_eq!(decls, ["ပြ", "ပြ#2", "ပြ#3"]);
    let start = checked.program.stmts.iter().find_map(|s| match s
    {
        Stmt::Fn(d) if d.start => Some(d),
        _ => None,
    });
    let called: Vec<String> = start
        .expect("the program has a start block")
        .block
        .stmts
        .iter()
        .filter_map(|s| match s
        {
            Stmt::Call(n) => Some(n.callee.name.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(called, ["ပြ", "ပြ#2", "ပြ#3"]);
}

#[test]
fn no_fit_is_zk0332_and_two_fits_are_zk0333()
{
    assert_eq!(codes_as_is(&program("မှန် ကို ပြ မည်။")), [Code::NoOverloadFits]);
    let plain = program("ပြ ဟူသော လုပ်ဆောင်ချက်ကို \"x\" ဖြင့် လုပ်ဆောင်မည်။");
    assert_eq!(codes_as_is(&plain), [Code::OverloadUnclear]);
}

#[test]
fn the_same_parameters_and_marks_twice_are_zk0304()
{
    let twice = format!(
        "{SHOW}ပြ ဟူသော လုပ်ဆောင်ချက် သည် ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လက်ခံပြီး \
         အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nပရိုဂရမ်သည် အောက်ပါတို့ကို \
         လုပ်ဆောင်မည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။"
    );
    assert_eq!(codes_as_is(&twice), [Code::DuplicateFunction]);
}

/// Two functions of one name with the same parameter and other results:
/// `ယူ` of a true/false value gives text or an integer.
const TAKE: &str = "ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော မှန်မှားတန်ဖိုး တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က ဖြစ်လျှင် \"ဟုတ်\" ကို ပြန်ပေးမည်။
\"မဟုတ်\" ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ယူ ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော မှန်မှားတန်ဖိုး တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ က ဖြစ်လျှင် ၁ ကို ပြန်ပေးမည်။
၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

";

/// Overloads that differ in the result alone are told apart through the kind
/// the place of the call expects, and a call with no such kind is ZK0333.
#[test]
fn the_place_of_a_call_picks_among_results()
{
    let start = |stmts: &str| {
        format!("{TAKE}ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
    };
    let picked = start(
        "ကိန်းရှင် စ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ယူ(မှန်) ဖြစ်ပါစေ။\nကိန်းရှင် န သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ \
         တန်ဖိုးသည် ယူ(မှန်) ဖြစ်ပါစေ။",
    );
    assert_eq!(codes_as_is(&picked), []);
    assert_eq!(codes_as_is(&start("ယူ(မှန်) ကို ပြသမည်။")), [Code::OverloadUnclear]);
}

/// A function of a file with the name, the kinds and the prepositions of a verb
/// the core gives is ZK0350; one with other kinds adds an overload, quietly.
#[test]
fn a_twin_of_a_core_verb_is_zk0350()
{
    let twin = "ထည့် ဟူသော လုပ်ဆောင်ချက် သည် စ ဟူသော စာသား တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် ဝိဘတ် \"ထဲသို့\" ကို \
                သုံး၍ လည်းကောင်း ပ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လည်းကောင်း \
                လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    assert_eq!(codes(twin), [Code::SameAsCore]);
    let other = "ထည့် ဟူသော လုပ်ဆောင်ချက် သည် စ ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် ဝိဘတ် \"ထဲသို့\" ကို \
                 သုံး၍ လည်းကောင်း ပ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လည်းကောင်း \
                 လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    assert_eq!(codes(other), []);
}
