use zakac_lexer::token::Kw;
use zakac_lexer::token::Particle;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;

use crate::common::kws;
use crate::common::lex_text;

#[rustfmt::skip]
const CASES: &[(&[&str], Kw)] = &[
    (&["သည့် ရလဒ်", "သည့်ရလဒ်", "မည့် ရလဒ်", "မည့်ရလဒ်", "သော ရလဒ်", "သောရလဒ်"],
        Kw::UsedResult),
    (&["ပြောင်းလဲသော ရလဒ်", 
        "ပြောင်းလဲသောရလဒ်"],
        Kw::Convert),
    (&["လုပ်ဆောင်သော ရလဒ်", 
        "လုပ်ဆောင်သောရလဒ်"],
        Kw::CallResult),
    (&["အောက်ပါတို့ကို လုပ်ဆောင်မည်", 
        "အောက်ပါ တို့ကို လုပ်ဆောင်မည်"],
        Kw::BlockOpen),
    (&["အောက်ပါတို့ကို လုပ်ဆောင်ကာ", 
        "အောက်ပါ တို့ကို လုပ်ဆောင်ကာ"],
        Kw::FnOpenRet,),
    (&["လုပ်ဆောင်စရာများ ပြီးပါပြီ",
        "လုပ်ဆောင်စရာများ ပြီးပါပြီး",
        "လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီ",
        "လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး",
        "လုပ်ဆောင်စရာများပြီးပါပြီး",],
        Kw::BlockClose,),
    (&["နေရာယူ၍ လိပ်စာကို",
        "နေရာယူ၍လိပ်စာကို"],
        Kw::AllocTarget),
    (&["မင်မိုရီတွင်",
        "မင်မိုရီ တွင်",
        "မှတ်ဉာဏ်တွင်"],
        Kw::AllocHead),
    (&["ရှိ မင်မိုရီကို",
        "ရှိမင်မိုရီကို",
        "ရှိ မှတ်ဉာဏ်ကို"],
        Kw::HeapHead),
    (&["တို့ ပါဝင်သည်",
        "တို့ပါဝင်သည်"],
        Kw::FieldsClose),
    (&["တို့ ပါဝင်ပြီး",
        "တို့ပါဝင်ပြီး"],
        Kw::FieldsCloseAnd),
    (&["အကုန် ကူးထည့်မည်",
        "အကုန်ကူးထည့်မည်"],
        Kw::WillDupAllIn),
    (&["အခြားတန်ဖိုးများ နှင့်အတူ",
        "အခြားတန်ဖိုးများနှင့်အတူ"],
        Kw::WithOtherValues),
    (&["ခေါ်ဆိုခွင့် ပြုမည်",
        "ခေါ်ဆိုခွင့်ပြုမည်"],
        Kw::AllowCall),
    (&["ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်",
        "ခေါ်ဆိုခွင့်ပြန်လည်ရုပ်သိမ်းမည်"],
        Kw::RevokeCall), ];

#[test]
fn phrases_join_attached_and_spaced()
{
    do_for!((sp, kw) in CASES,  do_for!(t in *sp, assert_eq!(kws(t), [*kw], "{t}")));
}

#[test]
fn phrase_span_covers_all_pieces()
{
    let (out, file) = lex_text("x ကို မအောင်မြင်ပါက အမှားအယွင်း ကို ပြန်ပေးမည်။");
    let tok = out.tokens[2];
    let lo = (tok.span.lo - file.start_pos) as usize;
    let hi = (tok.span.hi - file.start_pos) as usize;
    assert_eq!(&file.src[lo..hi], "မအောင်မြင်ပါက အမှားအယွင်း ကို ပြန်ပေးမည်");
    assert!(!tok.joined);
}

#[test]
fn lone_pieces_remain_pieces()
{
    assert_eq!(kws("မည့်"), [Kw::Named]);
    assert_eq!(kws("ရလဒ် ကို"), [Kw::Result, Kw::Ko]);
}

/// Kind and particle of the one token in `text`.
fn with_particle(text: &str) -> (TokenKind, Particle)
{
    let (out, _) = lex_text(text);
    assert_eq!(out.tokens.len(), 2, "{text}");
    (out.tokens[0].kind, out.tokens[0].particle)
}

#[test]
fn attached_particles_are_absorbed()
{
    let cases = [
        ("ညီ", Kw::Equal, Particle::None),
        ("ညီသည်", Kw::Equal, Particle::Thi),
        ("ညီပါသည်", Kw::Equal, Particle::PaThi),
        ("မငယ်ပါ", Kw::NotLess, Particle::Pa),
        ("မဟုတ်ပါ", Kw::Not, Particle::Pa),
        ("မှန်ပါသည်", Kw::True, Particle::PaThi),
        ("မမှန်သည်", Kw::False, Particle::Thi),
    ];
    for (text, kw, particle) in cases
    {
        assert_eq!(with_particle(text), (TokenKind::Kw(kw), particle), "{text}");
    }
}

#[test]
fn spaced_particles_remain_markers()
{
    // `ညီ သည်` with a space is a verb and a subject marker
    assert_eq!(kws("ညီ သည်"), [Kw::Equal, Kw::Thi]);
    assert_eq!(kws("ပြောင်းလဲသော ရလဒ် သည်"), [Kw::Convert, Kw::Thi]);
}

#[test]
fn compounds_keep_order_and_spans()
{
    assert_eq!(kws("ထက်ကြီးသည်"), [Kw::Htet, Kw::Greater]);
    assert_eq!(kws("အောက်ငယ်လျှင်"), [Kw::Auk, Kw::Less, Kw::Then]);
    let spaced = kws("အောက်ပါ တို့ကို လုပ်ဆောင်မည်");
    let attached = kws("အောက်ပါတို့ကို လုပ်ဆောင်မည်");
    assert_eq!(spaced, attached);
    assert_eq!(kws("တန်ဖိုးကို"), kws("တန်ဖိုး ကို"));
}
