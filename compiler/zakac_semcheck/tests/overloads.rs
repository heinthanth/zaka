mod common;

use common::check_as_is;
use common::codes_as_is;
use zakac_ast::stmt::Stmt;
use zakac_diagnostics::Code;
use zakac_semcheck::semcheck::SemcheckOptions;

/// Three functions of one name: an integer marked `ကို`, text marked `ကို`,
/// and text marked `ထဲသို့`.
const SHOW: &str = "\
လုပ်ဆောင်ချက် ပြ သည် ကိန်းပြည့် န(ကို) လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ပြ သည် စာသား စ(ကို) လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ပြ သည် စာသား စ(ထဲသို့) လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// A program: `SHOW`, then a start block that holds `stmts`.
fn program(stmts: &str) -> String
{
    format!("{SHOW}ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

/// A call runs the function its kinds and marks fit, and the check writes
/// the inner name of each function into its declaration and its calls.
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
    let plain = program("လုပ်ဆောင်ချက် ပြ ကို \"x\" တို့ဖြင့် လုပ်ဆောင်ပါ။");
    assert_eq!(codes_as_is(&plain), [Code::OverloadUnclear]);
}

#[test]
fn the_same_parameters_and_marks_twice_are_zk0304()
{
    let twice = format!(
        "{SHOW}လုပ်ဆောင်ချက် ပြ သည် ကိန်းပြည့် ခ(ကို) လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ \
         ပြီးဆုံးပါပြီး။\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။"
    );
    assert_eq!(codes_as_is(&twice), [Code::DuplicateFunction]);
}
