mod common;

use common::check_as_is;
use common::codes_as_is;
use zakac_ast::stmt::CaptureMode;
use zakac_ast::stmt::FnDecl;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_fn;
use zakac_diagnostics::Code;
use zakac_semcheck::semcheck::SemcheckOptions;

/// A start block around `stmts`.
fn start(stmts: &str) -> String
{
    format!("ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

/// A nested function `ထည့်` that reads the local `ထပ်` around it.
const ADD: &str = "လုပ်ဆောင်ချက် ထည့် သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ထပ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";

/// The kind of a function from an integer to an integer.
const KIND: &str = "(ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား)";

/// The captures the check writes into each function of a program with no
/// diagnostic, under function name, in source order.
///
/// # Arguments
///
/// * `text` - The program.
///
/// # Returns
///
/// Each function with the names it captures.
fn captures(text: &str) -> Vec<(String, Vec<String>)>
{
    let checked = check_as_is(text, &SemcheckOptions::default());
    let msgs: Vec<&str> =
        checked.out.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(msgs.is_empty(), "{msgs:?}");
    let mut walk = Found(Vec::new());
    walk.visit_program(&checked.program);
    walk.0
}

/// The walker that lists each function with its captures; each capture is
/// shared.
struct Found(Vec<(String, Vec<String>)>);

impl Visitor for Found
{
    fn visit_fn(&mut self, n: &FnDecl)
    {
        assert!(n.captures.iter().all(|c| c.mode == CaptureMode::Lend));
        let names = n.captures.iter().map(|c| c.name.name.to_string()).collect();
        self.0.push((n.name.name.to_string(), names));
        walk_fn(self, n);
    }
}

/// The captures of one function of a list.
///
/// # Arguments
///
/// * `found` - The list.
/// * `name` - The function.
///
/// # Returns
///
/// Its captures.
fn of<'a>(found: &'a [(String, Vec<String>)], name: &str) -> &'a [String]
{
    let hit = found.iter().find(|(n, _)| n == name);
    hit.map(|(_, c)| c.as_slice()).unwrap_or_else(|| panic!("no function {name}"))
}

#[test]
fn a_nested_function_captures_the_locals_it_names()
{
    let text = start(&format!("ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{ADD}\nထည့်(၅) ကို ပြသမည်။"));
    let found = captures(&text);
    assert_eq!(of(&found, "ထည့်"), ["ထပ်"]);
    assert!(of(&found, "ပရိုဂရမ်").is_empty());
}

#[test]
fn a_nested_function_that_names_no_local_captures_nothing()
{
    let text = start(
        "လုပ်ဆောင်ချက် နှစ်ဆ သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
နှစ်ဆ(၄) ကို ပြသမည်။",
    );
    assert!(of(&captures(&text), "နှစ်ဆ").is_empty());
}

#[test]
fn a_function_inside_a_nested_one_captures_through_it()
{
    let text = start(
        "ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် အပြင် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် အတွင်း သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ထပ် ကို ထပ် + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် အတွင်း ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။",
    );
    let found = captures(&text);
    assert_eq!(of(&found, "အပြင်"), ["ထပ်"]);
    assert_eq!(of(&found, "အတွင်း"), ["ထပ်"]);
}

#[test]
fn a_closure_another_nested_function_calls_is_captured()
{
    let text = start(&format!(
        "ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{ADD}\nလုပ်ဆောင်ချက် ခေါ် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nထည့်(၂) ကို \
         ပြသမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။"
    ));
    assert_eq!(of(&captures(&text), "ခေါ်"), ["ထည့်"]);
}

#[test]
fn the_address_of_a_closure_can_leave_its_locals()
{
    let back = format!(
        "လုပ်ဆောင်ချက် ပေး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ {KIND} တစ်ခု ကို ပြန်ပေးမည်။\nကိန်းရှင် ထပ် သည် ၁ \
         ဖြစ်ပါစေ။\n{ADD}\nထည့် ဟူသော လုပ်ဆောင်ချက် ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n{}",
        start("")
    );
    assert_eq!(codes_as_is(&back), []);
    let cell = start(&format!(
        "ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{ADD}\nကိန်းရှင် ဖများ သည် [ထည့် ဟူသော လုပ်ဆောင်ချက်] ဖြစ်ပါစေ။"
    ));
    assert_eq!(codes_as_is(&cell), []);
    let outer = start(&format!(
        "ကိန်းရှင် ဂ သည် {KIND} တစ်ခု ဖြစ်ပါစေ။\nအကယ်၍ မှန် ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းရှင် ထပ် \
         သည် ၁ ဖြစ်ပါစေ။\n{ADD}\nဂ ကို ထည့် ဟူသော လုပ်ဆောင်ချက် ဟု ထားမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီ။"
    ));
    assert_eq!(codes_as_is(&outer), []);
}

#[test]
fn c_functions_take_callbacks_and_give_optional_text()
{
    let decls = format!(
        "လုပ်ဆောင်ချက် getenv သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး \
         စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။\nလုပ်ဆောင်ချက် call_with သည် \"libc\" \
         မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ f ဟူသော {KIND} တစ်ခု ကို လက်ခံမည်။\n{}",
        start("")
    );
    assert_eq!(codes_as_is(&decls), []);
    let bare = format!(
        "လုပ်ဆောင်ချက် getenv သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး \
         စာသား တစ်ခု ကို ပြန်ပေးမည်။\n{}",
        start("")
    );
    assert_eq!(codes_as_is(&bare), [Code::KindNotAcrossC]);
    // C passes a list to no callback: a list has no C form.
    let list_param = format!(
        "လုပ်ဆောင်ချက် call_with သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ f ဟူသော (ကိန်းပြည့် စာရင်း \
         တစ်ခု ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံမည်။\n{}",
        start("")
    );
    assert_eq!(codes_as_is(&list_param), [Code::KindNotAcrossC]);
    let closure = format!(
        "လုပ်ဆောင်ချက် call_with သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ f ဟူသော {KIND} တစ်ခု ကို \
         လက်ခံမည်။\n{}",
        start(&format!(
            "ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{ADD}\nလုပ်ဆောင်ချက် call_with ကို ထည့် ဟူသော လုပ်ဆောင်ချက် \
             တို့ဖြင့် လုပ်ဆောင်မည်။"
        ))
    );
    assert_eq!(codes_as_is(&closure), []);
}

#[test]
fn an_own_result_is_text_from_c()
{
    let owned = format!(
        "လုပ်ဆောင်ချက် strdup သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး \
         စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။\n{}",
        start("")
    );
    assert_eq!(codes_as_is(&owned), []);
    let number = format!(
        "လုပ်ဆောင်ချက် abs သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို \
         လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။\n{}",
        start("")
    );
    assert_eq!(codes_as_is(&number), [Code::OwnedNotCText]);
    let zaka = start(
        "လုပ်ဆောင်ချက် ပေး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။\n\"က\" ကို \
         ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။",
    );
    assert_eq!(codes_as_is(&zaka), [Code::OwnedNotPtr]);
}

#[test]
fn a_closure_names_itself_and_an_inner_function_captures_it()
{
    let text = start(
        "ကိန်းရှင် ကြိမ် သည် ၀ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုး သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
ကိန်းရှင် ပွား ထဲသို့ တိုး ဟူသော လုပ်ဆောင်ချက် ကို ကူးထည့်မည်။
လုပ်ဆောင်ချက် အတွင်း သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် တိုး ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ပွား ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် တိုး ကို လုပ်ဆောင်မည်။",
    );
    let found = captures(&text);
    assert_eq!(of(&found, "တိုး"), ["ကြိမ်"]);
    assert_eq!(of(&found, "အတွင်း"), ["တိုး"]);
}

/// A leave for C takes a function value, a closure or not.
#[test]
fn a_leave_for_c_takes_a_function_value()
{
    let leave = "ထည့် ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။\nထည့် ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ \
                 ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်။";
    let closure = start(&format!("ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{ADD}\n{leave}"));
    assert_eq!(codes_as_is(&closure), []);
    let number = start("ကိန်းရှင် n သည် ၁ ဖြစ်ပါစေ။\nn ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။");
    assert_eq!(codes_as_is(&number), [Code::NotAFunction]);
    let inline =
        start("ကိန်းရှင် n သည် ၁ ဖြစ်ပါစေ။\nအကယ်၍ n > ၀ ဖြစ်လျှင် n ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။");
    assert_eq!(codes_as_is(&inline), [Code::NotAFunction]);
}
