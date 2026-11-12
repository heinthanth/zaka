mod common;

use common::codes_as_is;
use zakac_diagnostics::Code;

/// The error enumeration and an input stream, as the stdlib
/// declares them.
const KINDS: &str = "\
အချက်အလက်အဝင်အထွက်အမှားအယွင်း ဟူသော ဒေတာအမျိုးအစားကွဲတွင် မတွေ့ခြင်း၊ တခြားအကြောင်းအရင်း တို့ပါဝင်သည်။
အချက်အလက်အဝင်စီးကြောင်း ဟူသော ဒေတာအစုအဝေးတွင် လက်ကိုင် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။
";

/// A program: `KINDS`, the declarations `decls`, then a blank start block.
fn program(decls: &str) -> String
{
    format!("{KINDS}{decls}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။")
}

#[test]
fn an_external_function_the_host_serves_checks()
{
    let read = "လုပ်ဆောင်ချက် ဖိုင်ကိုဖတ် သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ လမ်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \
                \"၌ရှိသော\" ကို သုံး၍ လက်ခံပြီး စာသား သို့မဟုတ် အမှားအယွင်းအနေဖြင့် \
                အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(read)), []);
    let line = "လုပ်ဆောင်ချက် စာကြောင်းတစ်ကြောင်းဖတ် သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ ရင်း ဟူသော \
                အချက်အလက်အဝင်စီးကြောင်း တစ်ခု ကို ဝိဘတ် \"မှ\" ကို သုံး၍ လက်ခံပြီး စာသား သို့မဟုတ် \
                အမှားအယွင်းအနေဖြင့် အချက်အလက်အဝင်အထွက်အမှားအယွင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(line)), []);
}

#[test]
fn an_external_function_the_host_does_not_serve_is_zk0334()
{
    let unknown = "လုပ်ဆောင်ချက် ဟိုဟို သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်သည်။";
    assert_eq!(codes_as_is(&program(unknown)), [Code::UnknownHostFn]);
    let wrong_result = "လုပ်ဆောင်ချက် ဖိုင်ကိုဖတ် သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ လမ်း ဟူသော စာသား \
                        တစ်ခု ကို လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(wrong_result)), [Code::UnknownHostFn]);
}

#[test]
fn c_functions_take_kinds_c_passes_whole()
{
    let abs = "လုပ်ဆောင်ချက် abs သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု \
               ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(abs)), []);
    let frexp = "စိတ်မချရသော လုပ်ဆောင်ချက် frexp သည် \"libm\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ x ဟူသော \
                 ဒဿမကိန်း တစ်ခု ကို လည်းကောင်း e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ကို လည်းကောင်း \
                 လက်ခံပြီး ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(frexp)), []);
    let pair = "စုံ ဟူသော ဒေတာအစုအဝေးတွင် က ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ ခ ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု \
                တို့ပါဝင်သည်။\nလုပ်ဆောင်ချက် div သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ a ဟူသော ၃၂ ဘစ် \
                ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း b ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး စုံ တစ်ခု \
                ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(pair)), []);
}

/// A C function that passes a pointer needs the mark, and one that gives back
/// text that can be NULL does not: its result is an optional. A list of numbers
/// passes as the address of its first cell, and C can change one marked
/// `ဒေတာ ပြောင်းလဲရန်`.
#[test]
fn c_functions_with_pointers_need_the_mark_and_lists_of_numbers_do_not()
{
    let frexp = "လုပ်ဆောင်ချက် frexp သည် \"libm\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ x ဟူသော ဒဿမကိန်း တစ်ခု ကို \
                 လည်းကောင်း e ဟူသော ၃၂ ဘစ် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ဒဿမကိန်း တစ်ခု \
                 ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(frexp)), [Code::ForeignNeedsMark]);
    let getenv = "လုပ်ဆောင်ချက် getenv သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော စာသား တစ်ခု \
                  ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(getenv)), []);
    // The old form, a text or a failure, is refused: C gives NULL, which is no
    // failure.
    let old = "လုပ်ဆောင်ချက် getenv သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော စာသား တစ်ခု ကို \
               လက်ခံပြီး စာသား သို့မဟုတ် မအောင်မြင်သည့်သတင်း ရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(old)), [Code::KindNotAcrossC]);
    let lists = "လုပ်ဆောင်ချက် memset သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ b ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် \
                 ကိန်းပြည့် စာရင်း တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လည်းကောင်း c ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို \
                 လည်းကောင်း n ဟူသော အကွက် ၂ ခုရှိသော ဒဿမကိန်း စာရင်း တစ်ခု ကို လည်းကောင်း လက်ခံမည်။";
    assert_eq!(codes_as_is(&program(lists)), []);
    let edit = "လုပ်ဆောင်ချက် abs သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် \
                တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံမည်။";
    assert_eq!(codes_as_is(&program(edit)), [Code::EditAcrossC]);
    let texts =
        "လုပ်ဆောင်ချက် f သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ s ဟူသော စာသား စာရင်း တစ်ခု ကို လက်ခံမည်။";
    assert_eq!(codes_as_is(&program(texts)), [Code::KindNotAcrossC]);
}

#[test]
fn c_function_rules_have_their_codes()
{
    let burmese = "လုပ်ဆောင်ချက် ရေး သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်သည်။";
    assert_eq!(codes_as_is(&program(burmese)), [Code::NotACName]);
    let text = "လုပ်ဆောင်ချက် getenv သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော စာသား တစ်ခု ကို \
                လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(text)), [Code::KindNotAcrossC]);
    let edit = "လုပ်ဆောင်ချက် inc သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် \
                တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံမည်။";
    assert_eq!(codes_as_is(&program(edit)), [Code::EditAcrossC]);
    let twice = "လုပ်ဆောင်ချက် abs သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် \
                 တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\nလုပ်ဆောင်ချက် abs သည် \"libc\" \
                 မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော ၆၄ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ၆၄ ဘစ် ကိန်းပြည့် \
                 တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(codes_as_is(&program(twice)), [Code::CNameTwice]);
}

/// `printf` from libc, which takes other values.
const PRINTF: &str = "လုပ်ဆောင်ချက် printf သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ f ဟူသော စာသား \
                      တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို \
                      ပြန်ပေးမည်။";

/// A program with `PRINTF` and a start block that runs `stmts`.
fn printing(decls: &str, stmts: &str) -> String
{
    format!(
        "{KINDS}{PRINTF}\n{decls}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{stmts}\nပရိုဂရမ် \
         ပြီးဆုံးပါပြီး။"
    )
}

#[test]
fn c_functions_take_text_and_other_values()
{
    let puts =
        "လုပ်ဆောင်ချက် puts သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ s ဟူသော စာသား တစ်ခု ကို လက်ခံမည်။";
    assert_eq!(codes_as_is(&program(puts)), []);
    let call = "လုပ်ဆောင်ချက် printf ကို \"%s %ld\\n\"၊ \"က\"၊ ၄၂ တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(codes_as_is(&printing("", call)), []);
    let bare = "လုပ်ဆောင်ချက် printf ကို \"က\\n\" တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(codes_as_is(&printing("", bare)), []);
}

#[test]
fn other_values_have_their_rules()
{
    let none = "လုပ်ဆောင်ချက် printf ကို လုပ်ဆောင်မည်။";
    assert_eq!(codes_as_is(&printing("", none)), [Code::ArgCountMismatch]);
    let pair = "စုံ ဟူသော ဒေတာအစုအဝေးတွင် က ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။";
    let call = "ကိန်းရှင် တ သည် စုံ တစ်ခု ဖြစ်ပါစေ။\nလုပ်ဆောင်ချက် printf ကို \"%d\"၊ တ တို့ဖြင့် လုပ်ဆောင်မည်။";
    // C reads a struct through `...` as it reads a struct parameter.
    assert_eq!(codes_as_is(&printing(pair, call)), []);
    let host = "လုပ်ဆောင်ချက် ရေး သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ စ ဟူသော စာသား တစ်ခု ကို လည်းကောင်း \
                အခြားတန်ဖိုးများနှင့်အတူ လက်ခံမည်။";
    assert_eq!(codes_as_is(&program(host)), [Code::UnknownHostFn]);
}

/// A kind from outside, with two C fields, and C functions that give,
/// take and close a handle of it.
const HANDLES: &str = "\
စက် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင် n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ name ဟူသော စာသား တစ်ခု တို့ပါဝင်သည်။
လုပ်ဆောင်ချက် open_it သည် \"libx\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
k ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး စက် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် close_it သည် \"libx\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ c ဟူသော စက် တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံမည်။";

/// The start block of a program with `HANDLES` and a handle `h`, then
/// `steps`.
fn with_handle(steps: &str) -> String
{
    format!(
        "{HANDLES}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းရှင် h သည် လုပ်ဆောင်ချက် open_it ကို ၁ တို့ဖြင့် \
         လုပ်ဆောင်သော ရလဒ် ထဲမှ ဒေတာ ဖြစ်ပါစေ။\n{steps}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။"
    )
}

#[test]
fn a_handle_passes_reads_and_closes_from_safe_code()
{
    let steps = "ကိန်းရှင် m သည် h ၏ n ဖြစ်ပါစေ။\nကိန်းရှင် s သည် h ၏ name ဖြစ်ပါစေ။\nလုပ်ဆောင်ချက် \
                 close_it ကို h တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(codes_as_is(&with_handle(steps)), []);
}

#[test]
fn a_handle_is_not_made_copied_printed_or_written()
{
    let made = with_handle("ကိန်းရှင် x သည် စက်() ဖြစ်ပါစေ။");
    assert_eq!(codes_as_is(&made), [Code::HandleInZaka]);
    let copied = with_handle("ကိန်းရှင် x သည် h ၏ မိတ္တူ ဖြစ်ပါစေ။");
    assert_eq!(codes_as_is(&copied), [Code::HandleInZaka]);
    assert_eq!(codes_as_is(&with_handle("h ကို ပြသမည်။")), [Code::HandleInZaka]);
    let written = with_handle("h ၏ n ကို ၅ ဟု ထားမည်။");
    assert_eq!(codes_as_is(&written), [Code::CFieldWrite]);
}

#[test]
fn a_c_field_of_a_kind_c_cannot_hold_is_refused()
{
    let decl = "စက် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင် xs ဟူသော ကိန်းပြည့် စာရင်း တစ်ခု ပါဝင်သည်။";
    assert_eq!(codes_as_is(&program(decl)), [Code::KindNotAcrossC]);
}

/// Marked code writes a number field of a handle; a text field is a pointer
/// C owns, so no code writes it.
#[test]
fn marked_code_writes_a_number_field_of_a_handle()
{
    let marked = |line: &str| {
        format!("အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။\n{line}\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။")
    };
    assert_eq!(codes_as_is(&with_handle(&marked("h ၏ n ကို ၅ ဟု ထားမည်။"))), []);
    let text = with_handle(&marked("h ၏ name ကို \"x\" ဟု ထားမည်။"));
    assert_eq!(codes_as_is(&text), [Code::CFieldWrite]);
}

/// A program that reads the field `field` of a kind from outside whose C
/// fields are `fields`.
fn reads_c_field(fields: &str, field: &str) -> String
{
    format!(
        "ရေတွက်စက် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင် {fields} ပါဝင်သည်။
လုပ်ဆောင်ချက် open_it သည် \"libx\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ ရေတွက်စက် တစ်ခု ကို ပြန်ပေးမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စက် သည် open_it() ဖြစ်ပါစေ။
စက် ၏ {field} ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။"
    )
}

/// Two programs that give one name to two C structs make two kinds: the
/// second reads its own fields, not those of the first.
#[test]
fn one_name_of_two_c_structs_makes_two_kinds()
{
    let one = reads_c_field("count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု", "count");
    assert_eq!(codes_as_is(&one), []);
    let fields = "count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု တို့";
    assert_eq!(codes_as_is(&reads_c_field(fields, "step")), []);
}
