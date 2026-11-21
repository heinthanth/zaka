//! Which threads can run as light tasks: a thread whose function reaches no
//! call of C, no read of standard input and no call of itself again, through
//! function values too. A thread written `သီးသန့် အပြိုင်` still counts here; the
//! codegen gives it an OS thread of its own.

use std::collections::BTreeSet;

use zakac_borrow_checker::artifact::run_phases;
use zakac_borrow_checker::light_fns;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// The names of the functions whose threads can run light.
fn light(text: &str) -> BTreeSet<String>
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let out = run_phases(
        &file,
        &LexerOptions::default(),
        &SemcheckOptions::default(),
        &FoldOptions::default(),
        &BorrowCheckOptions::default(),
    );
    let errors: Vec<_> = out.diagnostics.iter().filter(|d| d.is_error()).collect();
    assert!(errors.is_empty(), "{errors:?}");
    let module = out.lowered.folded.expect("a folded module");
    let names = light_fns(&module).into_iter();
    names.map(|f| module.funcs[f as usize].name.clone()).collect()
}

/// A C function, a function that calls it, one that calls itself, one that
/// reads standard input, one that calls the C one through a function value,
/// and a plain one.
const FNS: &str = "\"စံကား/အဝင်အထွက်\" မှ စာကြောင်းတစ်ကြောင်းဖတ် နှင့် စံပြုအဝင် တို့ကို ထည့်သွင်းမည်။

strlen ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ s ဟူသော စာသား တစ်ခု ကို \
                   လက်ခံပြီး လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

အရှည် ဟူသော လုပ်ဆောင်ချက် သည် စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
strlen ဟူသော လုပ်ဆောင်ချက်ကို စ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ရေ ဟူသော လုပ်ဆောင်ချက် သည် န ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ န < ၁ ဖြစ်လျှင် ၀ ကို ပြန်ပေးမည်။
ရေ(န - ၁) + ၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ဖတ်ပြ ဟူသော လုပ်ဆောင်ချက် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
စံပြုအဝင် မှ စာကြောင်းတစ်ကြောင်းဖတ် သည့် ရလဒ် ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

တဆင့် ဟူသော လုပ်ဆောင်ချက် သည် စ ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဖ သည် အရှည် ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။
ဖ(စ) ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပေါင်း ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// [`FNS`], then a start block of these lines.
fn program(lines: &str) -> String
{
    format!("{FNS}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{lines}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n")
}

/// The name set of these names.
fn set(names: &[&str]) -> BTreeSet<String>
{
    names.iter().map(|n| n.to_string()).collect()
}

#[test]
fn a_plain_thread_runs_light()
{
    let text = program("ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် အပြိုင် လုပ်ဆောင်မည်။");
    assert_eq!(light(&text), set(&["ပေါင်း"]));
}

#[test]
fn threads_that_reach_c_recursion_or_standard_input_get_os_threads()
{
    let text = program(
        "အရှည် ဟူသော လုပ်ဆောင်ချက်ကို \"က\" ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ရေ ဟူသော လုပ်ဆောင်ချက်ကို ၃ ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ဖတ်ပြ ဟူသော လုပ်ဆောင်ချက်ကို အပြိုင် လုပ်ဆောင်မည်။
ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် အပြိုင် လုပ်ဆောင်မည်။",
    );
    assert_eq!(light(&text), set(&["ပေါင်း"]));
}

#[test]
fn a_thread_that_reaches_c_through_a_function_value_gets_an_os_thread()
{
    let text = program(
        "တဆင့် ဟူသော လုပ်ဆောင်ချက်ကို \"က\" ဖြင့် အပြိုင် လုပ်ဆောင်မည်။
ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် အပြိုင် လုပ်ဆောင်မည်။",
    );
    assert_eq!(light(&text), set(&["ပေါင်း"]));
}

#[test]
fn apart_still_counts_as_light_here_for_the_codegen_reads_the_mark()
{
    let text = program("ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် သီးသန့် အပြိုင် လုပ်ဆောင်မည်။");
    assert_eq!(light(&text), set(&["ပေါင်း"]));
}
