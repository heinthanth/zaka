//! Trait values, `<trait> အရာ`: which traits can be a data kind, and what a
//! trait value cannot do.

use zakac_ast::expr::CallExpr;
use zakac_ast::visit::Visitor;
use zakac_ast::visit::walk_call_expr;
use zakac_diagnostics::Code;
use zakac_semcheck::semcheck::SemcheckOptions;

use crate::common::check_as_is;
use crate::common::codes_as_is;

/// A trait whose function takes its kind first.
const GREET: &str = "နှုတ်ဆက်နိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။
";

/// A trait whose function takes its kind twice.
const ORDER: &str = "စီနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
ငယ်သလား ဟူသော လုပ်ဆောင်ချက် သည်
က ဟူသော A တစ်ခု ကို လည်းကောင်း
ခ ဟူသော A တစ်ခု ကို လည်းကောင်း လက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။
";

/// The codes of a program with these declarations and a function that
/// takes `param` and runs `body`.
///
/// # Arguments
///
/// * `decls` - The declarations.
/// * `param` - The kind phrase of the parameter `တ`.
/// * `body` - The statements of the function.
///
/// # Returns
///
/// The codes.
fn codes_of(decls: &str, param: &str, body: &str) -> Vec<Code>
{
    let text = format!(
        "{decls}
စမ်း ဟူသော လုပ်ဆောင်ချက် သည် တ ဟူသော {param} တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
{body}
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
"
    );
    codes_as_is(&text)
}

/// A trait whose functions take its kind first is a data kind; one that
/// takes it twice is ZK0360, an unknown trait ZK0344, and a trait the
/// compiler gives ZK0360.
#[test]
fn which_traits_can_be_a_data_kind()
{
    assert!(codes_of(GREET, "နှုတ်ဆက်နိုင်သော အရာ", "").is_empty());
    assert_eq!(codes_of(ORDER, "စီနိုင်သော အရာ", ""), [Code::TraitNotAKind]);
    assert_eq!(codes_of("", "ပျံသန်းနိုင်သော အရာ", ""), [Code::UnknownTrait]);
    assert_eq!(codes_of("", "နံပါတ်နှင့်တူသော အရာ", ""), [Code::TraitNotAKind]);
}

/// A trait value is not compared, printed or given an initial value: ZK0361.
#[test]
fn what_a_trait_value_cannot_do()
{
    let param = "နှုတ်ဆက်နိုင်သော အရာ";
    let compare = codes_of(GREET, param, "အကယ်၍ တ == တ ဖြစ်လျှင် ၁ ကို ပြသမည်။");
    assert!(compare.contains(&Code::TraitValueUse), "{compare:?}");
    assert_eq!(codes_of(GREET, param, "တ ကို ပြသမည်။"), [Code::TraitValueUse]);
    let initial = "ကိန်းရှင် ဥ သည် နှုတ်ဆက်နိုင်သော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။";
    assert!(codes_of(GREET, param, initial).contains(&Code::TraitValueUse));
}

/// A struct with the greeting trait, through a plain function.
const PERSON: &str = "လူ ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ \
                      ယောက် ဖြစ်သည်။
နှုတ်ဆက် ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော လူ တစ်ယောက် ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
က ၏ နာမည် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";

/// A trait whose function changes its first parameter.
const COUNT: &str = "တိုးနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍
၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။
တိုး ဟူသော လုပ်ဆောင်ချက် သည် က ဟူသော A တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။
";

/// A counter with the trait of `COUNT`, as a plain struct or a shared object.
///
/// # Arguments
///
/// * `shared` - Whether the counter is a shared object.
///
/// # Returns
///
/// The declarations.
fn counter(shared: bool) -> String
{
    let mark = if shared
    {
        "မျှဝေနိုင်သော "
    }
    else
    {
        ""
    };
    format!(
        "ကောင်တာ ဟူသော {mark}ဒေတာအစုအဝေးတွင် အကြိမ် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
တိုး ဟူသော လုပ်ဆောင်ချက် သည် ပ ဟူသော ကောင်တာ တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ပ ၏ အကြိမ် ကို (ပ ၏ အကြိမ်) + ၁ ဟု ထားမည်။
ပ ၏ အကြိမ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
"
    )
}

/// The codes of a program with these declarations and these statements in
/// its start block.
///
/// # Arguments
///
/// * `decls` - The declarations.
/// * `stmts` - The statements.
///
/// # Returns
///
/// The codes.
fn run_codes(decls: &str, stmts: &str) -> Vec<Code>
{
    let text = format!(
        "{decls}
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
{stmts}
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
"
    );
    codes_as_is(&text)
}

/// A value whose kind has the trait goes into a trait value, and a call on
/// the trait value takes the result kind of the trait function; a kind
/// without the trait is ZK0343.
#[test]
fn values_go_in_and_calls_run_through()
{
    let decls = format!("{GREET}{PERSON}");
    let stmts = "ကိန်းရှင် တ သည် နှုတ်ဆက်နိုင်သော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လူ(နာမည်: \"မောင်\") ဖြစ်ပါစေ။
ကိန်းရှင် စ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် တ ၏ နှုတ်ဆက်() ဖြစ်ပါစေ။
ကိန်းရှင် ဆ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် နှုတ်ဆက်(တ) ဖြစ်ပါစေ။";
    let found = run_codes(&decls, stmts);
    assert!(found.is_empty(), "{found:?}");
    let lacks = "ကိန်းရှင် တ သည် နှုတ်ဆက်နိုင်သော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၃ ဖြစ်ပါစေ။";
    assert_eq!(run_codes(&decls, lacks), [Code::BoundNotMet]);
}

/// A list of trait values takes values of each kind with the trait.
#[test]
fn a_list_holds_trait_values()
{
    let cat = PERSON.replace("လူ", "ကြောင်").replace("ယောက်", "ကောင်");
    let decls = format!("{GREET}{PERSON}{cat}");
    let stmts = "ကိန်းရှင် များ သည် နှုတ်ဆက်နိုင်သော အရာ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [လူ(နာမည်: \"က\")၊ \
                 ကြောင်(နာမည်: \"ခ\")] ဖြစ်ပါစေ။
များ ၏ အကွက်တိုင်း ကို ဝ အနေဖြင့် ယူ၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဝ ၏ နှုတ်ဆက်() ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";
    let found = run_codes(&decls, stmts);
    assert!(found.iter().all(|c| *c == Code::PrintSentenceWithKo), "{found:?}");
}

/// A plain value that goes into a trait value whose trait has a function
/// that changes its first parameter is ZK0362; a shared object is not.
#[test]
fn a_plain_value_goes_in_as_a_clone()
{
    let stmts = "ကိန်းရှင် တ သည် တိုးနိုင်သော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကောင်တာ(အကြိမ်: ၀) ဖြစ်ပါစေ။
ကိန်းရှင် ရ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် တ ၏ တိုး() ဖြစ်ပါစေ။";
    let plain = format!("{COUNT}{}", counter(false));
    assert_eq!(run_codes(&plain, stmts), [Code::TraitValueClones]);
    let shared = format!("{COUNT}{}", counter(true));
    assert!(run_codes(&shared, stmts).is_empty());
}

/// A trait value goes to a generic function that needs its trait.
#[test]
fn a_trait_value_meets_its_own_trait()
{
    let generic = "ခေါ် ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်
က ဟူသော T တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။
ဤနေရာတွင် သတိပြုရန်မှာ
T တွင် နှုတ်ဆက်နိုင်သော ဂုဏ်သတ္တိ ရှိသည်။
သတိပြုစရာများ ပြီးဆုံးပါပြီး။
နှုတ်ဆက်(က) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
";
    let decls = format!("{GREET}{PERSON}{generic}");
    let stmts = "ကိန်းရှင် တ သည် နှုတ်ဆက်နိုင်သော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လူ(နာမည်: \"မောင်\") ဖြစ်ပါစေ။
ကိန်းရှင် စ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခေါ်(တ) ဖြစ်ပါစေ။";
    let found = run_codes(&decls, stmts);
    assert!(found.is_empty(), "{found:?}");
}

/// Collects the name and the argument count of each call.
struct Calls(Vec<(String, usize)>);

impl Visitor for Calls
{
    fn visit_call_expr(&mut self, n: &CallExpr)
    {
        self.0.push((n.callee.name.to_string(), n.args.len()));
        walk_call_expr(self, n);
    }
}

/// The check writes the conversion as a call that takes the value and the
/// function of its kind, and the call through the trait value under the
/// number of the trait function.
#[test]
fn the_program_names_the_conversion_and_the_call()
{
    let text = format!(
        "{GREET}{PERSON}
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် တ သည် နှုတ်ဆက်နိုင်သော အရာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် လူ(နာမည်: \"မောင်\") ဖြစ်ပါစေ။
ကိန်းရှင် စ သည် စာသား တစ်ခု ဖြစ်၍ တန်ဖိုးသည် တ ၏ နှုတ်ဆက်() ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
"
    );
    let checked = check_as_is(&text, &SemcheckOptions::default());
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    let mut calls = Calls(Vec::new());
    calls.visit_program(&checked.program);
    assert!(calls.0.contains(&("%trait-value".to_string(), 2)), "{:?}", calls.0);
    assert!(calls.0.contains(&("%trait-call:0".to_string(), 1)), "{:?}", calls.0);
}
