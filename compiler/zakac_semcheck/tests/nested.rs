mod common;

use common::check_as_is;
use common::codes_as_is;
use zakac_diagnostics::Code;
use zakac_semcheck::semcheck::SemcheckOptions;

/// A start block around `stmts`.
fn start(stmts: &str) -> String
{
    format!("ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

/// A nested function `ထည့်` that reads the local `ထပ်` around it, with the
/// capture list `list` (`""` for none).
fn add(list: &str) -> String
{
    format!(
        "လုပ်ဆောင်ချက် ထည့် သည် ကိန်းပြည့် က ကို{} {list}လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + ထပ် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။",
        if list.is_empty()
        {
            ""
        }
        else
        {
            "လည်းကောင်း၊"
        }
    )
}

/// `ထည့်` with a duplicate of `ထပ်`.
const DUP: &str = "ထပ် ကိုလည်းကောင်း ";

/// `ထည့်` with a borrow of `ထပ်`.
const LEND: &str = "ထပ် ကို ငှား၍လည်းကောင်း ";

/// The kind of a function from an integer to an integer.
const KIND: &str = "(ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား)";

#[test]
fn a_nested_function_lists_the_locals_it_uses()
{
    for list in [DUP, LEND, "ထပ် ကို အပိုင်အနေဖြင့်လည်းကောင်း "]
    {
        let text =
            start(&format!("ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{}\nထည့်(၅) ကို ပြသပါ။", add(list)));
        let checked = check_as_is(&text, &SemcheckOptions::default());
        let msgs: Vec<&str> =
            checked.out.diagnostics.iter().map(|d| d.message.as_str()).collect();
        assert!(msgs.is_empty(), "{msgs:?}");
    }
}

#[test]
fn a_local_the_list_leaves_out_is_zk0339()
{
    let text = start(&format!("ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{}\nထည့်(၅) ကို ပြသပါ။", add("")));
    assert_eq!(codes_as_is(&text), [Code::CaptureUnlisted]);
}

#[test]
fn a_listed_name_that_is_no_local_is_zk0340()
{
    let global = format!("ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{}", start(&add(DUP)));
    assert_eq!(codes_as_is(&global), [Code::CaptureNotLocal]);
    let top = format!(
        "လုပ်ဆောင်ချက် ထိပ် သည် ထပ် ကိုလည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ \
         ပြီးဆုံးပါပြီး။\n{}",
        start("")
    );
    assert_eq!(codes_as_is(&top), [Code::CaptureNotLocal]);
}

#[test]
fn a_capture_never_used_is_zk0341()
{
    let text = start(&format!(
        "ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\nကိန်းရှင် မသုံး သည် ၂ ဖြစ်ပါစေ။\n{}\nထည့်(၅) ကို ပြသပါ။",
        add("ထပ် ကိုလည်းကောင်း၊ မသုံး ကိုလည်းကောင်း ")
    ));
    assert_eq!(codes_as_is(&text), [Code::CaptureUnused]);
}

#[test]
fn a_function_inside_a_nested_one_lists_through_it()
{
    let inner = |outer: &str| {
        format!(
            "လုပ်ဆောင်ချက် အပြင် သည် {outer}အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် အတွင်း သည် ထပ် ကို ငှား၍လည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ထပ် ကို ထပ် + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် အတွင်း ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။"
        )
    };
    let listed = inner("ထပ် ကို ငှား၍လည်းကောင်း လက်ခံပြီး ");
    let text = start(&format!("ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{listed}"));
    assert_eq!(codes_as_is(&text), []);
    let text = start(&format!("ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{}", inner("")));
    assert_eq!(codes_as_is(&text), [Code::CaptureUnlisted]);
}

#[test]
fn a_closure_another_nested_function_calls_is_listed()
{
    let call = |list: &str| {
        start(&format!(
            "ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{}\nလုပ်ဆောင်ချက် ခေါ် သည် {list}အောက်ပါတို့ကို \
             လုပ်ဆောင်မည်။\nထည့်(၂) ကို ပြသပါ။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။",
            add(DUP)
        ))
    };
    assert_eq!(codes_as_is(&call("ထည့် ကိုလည်းကောင်း လက်ခံပြီး ")), []);
    assert_eq!(codes_as_is(&call("")), [Code::CaptureUnlisted]);
}

#[test]
fn the_address_of_a_borrowing_function_cannot_leave_its_locals()
{
    let back = |list: &str| {
        format!(
            "လုပ်ဆောင်ချက် ပေး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ {KIND} ရလဒ် ကို ပြန်ပေးမည်။\nကိန်းရှင် ထပ် သည် ၁ \
             ဖြစ်ပါစေ။\n{}\nလုပ်ဆောင်ချက် ထည့် ၏ လိပ်စာ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။\nလုပ်ဆောင်စရာများ \
             ပြီးဆုံးပါပြီး။\n{}",
            add(list),
            start("")
        )
    };
    assert_eq!(codes_as_is(&back(LEND)), [Code::ClosureEscapes]);
    assert_eq!(codes_as_is(&back(DUP)), []);
    let cell = start(&format!(
        "ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{}\nကိန်းရှင် ဖများ သည် [လုပ်ဆောင်ချက် ထည့် ၏ လိပ်စာ] ဖြစ်ပါစေ။",
        add(LEND)
    ));
    assert_eq!(codes_as_is(&cell), [Code::ClosureEscapes]);
    let outer = start(&format!(
        "ကိန်းရှင် ဂ သည် {KIND} တစ်ခု ဖြစ်ပါစေ။\nအကယ်၍ မှန် ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်ပါ။\nကိန်းရှင် ထပ် \
         သည် ၁ ဖြစ်ပါစေ။\n{}\nဂ ကို လုပ်ဆောင်ချက် ထည့် ၏ လိပ်စာ ဟု ထားပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီ။",
        add(LEND)
    ));
    assert_eq!(codes_as_is(&outer), [Code::ClosureEscapes]);
    let same = start(&format!(
        "ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\nကိန်းရှင် ဂ သည် {KIND} တစ်ခု ဖြစ်ပါစေ။\n{}\nဂ ကို လုပ်ဆောင်ချက် ထည့် \
         ၏ လိပ်စာ ဟု ထားပါ။\nဂ(၂) ကို ပြသပါ။",
        add(LEND)
    ));
    assert_eq!(codes_as_is(&same), []);
}

#[test]
fn c_functions_take_callbacks_and_give_text_or_a_failure()
{
    let decls = format!(
        "လုပ်ဆောင်ချက် getenv သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ စာသား n ကို လက်ခံပြီး စာသား ရလဒ် \
         သို့မဟုတ် မအောင်မြင်သည့်သတင်း ကို ပြန်ပေးမည်။\nလုပ်ဆောင်ချက် call_with သည် \"libc\" မှလာသော \
         လုပ်ဆောင်ချက်ဖြစ်၍ {KIND} f ကို လက်ခံမည်။\n{}",
        start("")
    );
    assert_eq!(codes_as_is(&decls), []);
    let bare = format!(
        "လုပ်ဆောင်ချက် getenv သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ စာသား n ကို လက်ခံပြီး စာသား ရလဒ် \
         ကို ပြန်ပေးမည်။\n{}",
        start("")
    );
    assert_eq!(codes_as_is(&bare), [Code::KindNotAcrossC]);
    let text_param = format!(
        "လုပ်ဆောင်ချက် call_with သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ (စာသား ကို လက်ခံသော \
         လုပ်ဆောင်ချက် အမျိုးအစား) f ကို လက်ခံမည်။\n{}",
        start("")
    );
    assert_eq!(codes_as_is(&text_param), [Code::KindNotAcrossC]);
    let closure = format!(
        "လုပ်ဆောင်ချက် call_with သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ {KIND} f ကို လက်ခံမည်။\n{}",
        start(&format!(
            "ကိန်းရှင် ထပ် သည် ၁ ဖြစ်ပါစေ။\n{}\nလုပ်ဆောင်ချက် call_with ကို လုပ်ဆောင်ချက် ထည့် ၏ လိပ်စာ \
             တို့ဖြင့် လုပ်ဆောင်ပါ။",
            add(DUP)
        ))
    );
    assert_eq!(codes_as_is(&closure), [Code::ClosureToC]);
}

#[test]
fn an_own_result_is_text_from_c()
{
    let owned = format!(
        "လုပ်ဆောင်ချက် strdup သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ စာသား s ကို လက်ခံပြီး စာသား ရလဒ် \
         သို့မဟုတ် မအောင်မြင်သည့်သတင်း ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။\n{}",
        start("")
    );
    assert_eq!(codes_as_is(&owned), []);
    let number = format!(
        "လုပ်ဆောင်ချက် abs သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ ၃၂ ဘစ် ကိန်းပြည့် n ကို လက်ခံပြီး ၃၂ ဘစ် \
         ကိန်းပြည့် ရလဒ် ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။\n{}",
        start("")
    );
    assert_eq!(codes_as_is(&number), [Code::OwnedNotCText]);
    let zaka = start(
        "လုပ်ဆောင်ချက် ပေး သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား ရလဒ် ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။\n\"က\" ကို \
         ရလဒ်အဖြစ် ပြန်ပေးပါ။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။",
    );
    assert_eq!(codes_as_is(&zaka), [Code::OwnedNotCText]);
}

#[test]
fn a_closure_names_itself_and_an_inner_function_lists_it()
{
    let text = start(
        "ကိန်းရှင် ကြိမ် သည် ၀ ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် (လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် တိုး သည် ကြိမ် ကိုလည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားပါ။
ကိန်းရှင် ပွား ထဲသို့ လုပ်ဆောင်ချက် တိုး ၏ လိပ်စာ ကို ကူးထည့်မည်။
လုပ်ဆောင်ချက် အတွင်း သည် တိုး ကိုလည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် တိုး ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် ပွား ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
လုပ်ဆောင်ချက် တိုး ကို လုပ်ဆောင်ပါ။",
    );
    assert_eq!(codes_as_is(&text), []);
}
