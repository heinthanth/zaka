mod common;

use common::emit_text;
use zakac_codegen::listing::LISTING_HEADER;
use zakac_codegen::listing::listing;
use zakac_codegen::listing::quote;

#[test]
fn listing_names_labels_slots_and_calls()
{
    let text = "ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ၀ ဖြစ်ပါစေ။\nကိန်း သည် ၃ အောက် ငယ်နေသရွေ့ ကိန်း ကို ကိန်း + ၁ ဟု \
                ထားပါ။\n\"ပြီး\" ဟု ပြသပါ။";
    let out = listing(&emit_text(text));
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].starts_with(LISTING_HEADER), "{out}");
    assert!(lines[0].ends_with("platform 64 bits"), "{out}");
    assert!(out.contains("globals\n  g0 ကိန်း isize\n"), "{out}");
    assert!(out.contains("texts\n  t0 \"ပြီး\"\n"), "{out}");
    assert!(out.contains("fn f0 main (params 0)\n"), "{out}");
    assert!(out.contains("L0:\n     3  global_addr g0       ; ကိန်း\n"), "{out}");
    assert!(out.contains("jump_if_false L1"), "{out}");
    assert!(out.contains("jump L0"), "{out}");
    assert!(out.contains("L1:\n"), "{out}");
}

#[test]
fn functions_show_params_result_and_locals()
{
    let text = "လုပ်ဆောင်ချက် နှစ်ဆ ကို ၂ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။\nလုပ်ဆောင်ချက် နှစ်ဆ \
                သည်\nကိန်းပြည့် ဂဏန်း ကို လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို \
                ပြန်ပေးမည်။\nဂဏန်း * ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။\nလုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let out = listing(&emit_text(text));
    assert!(
        out.contains("fn f0 နှစ်ဆ (params 1) -> isize\nlocals\n  l0 ဂဏန်း isize\n"),
        "{out}"
    );
    assert!(out.contains("call f0              ; နှစ်ဆ"), "{out}");
}

#[test]
fn quotes_escape_control_characters()
{
    assert_eq!(quote("a\"b\\c\nd\te\u{7}"), "\"a\\\"b\\\\c\\nd\\te\\u{7}\"");
    assert_eq!(quote("မင်္ဂလာ"), "\"မင်္ဂလာ\"");
}

#[test]
fn heap_ops_list_their_operands()
{
    let out = listing(&emit_text(
        "ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ ဖြစ်ပါစေ။\nမင်မိုရီတွင် ကိန်းပြည့် ၂ ခုစာ နေရာယူ၍ လိပ်စာကို p တွင် \
         သိမ်းထားမည်။\np ၏ တန်ဖိုး ကို ပြသပါ။",
    ));
    assert!(out.contains("globals\n  g0 p *isize\n"), "{out}");
    assert!(out.contains("alloc 8"), "{out}");
    assert!(out.contains("load isize"), "{out}");
}
