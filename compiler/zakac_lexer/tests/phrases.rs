mod common;

use common::kws;
use common::lex_text;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Particle;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;

#[rustfmt::skip]
const CASES: &[(&[&str], Kw)] = &[
    (&["စားသော စားလဒ်", 
        "စားသောစားလဒ်"], 
        Kw::Divide),
    (&["စားသော စားကြွင်း", 
        "စားသောစားကြွင်း"], 
        Kw::Remainder),
    (&["ဘစ်အလိုက် ဆုံလဒ်", 
        "ဘစ်အလိုက်ဆုံလဒ်"],
        Kw::BitAnd),
    (&["ဘစ်အလိုက် ပေါင်းစုလဒ်", 
        "ဘစ်အလိုက်ပေါင်းစုလဒ်"],
        Kw::BitOr),
    (&["ဘစ်အလိုက် ခြားနားလဒ်", 
        "ဘစ်အလိုက်ခြားနားလဒ်"],
        Kw::BitXor),
    (&["ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်", 
    "ဘစ်အလိုက်ပြောင်းပြန်ပြောင်းလဒ်"],
        Kw::BitNot),
    (&["ရွေ့သော ရလဒ်", 
        "ရွေ့သောရလဒ်"],
        Kw::Shift),
    (&["ပြောင်းလဲသော ရလဒ်", 
        "ပြောင်းလဲသောရလဒ်"],
        Kw::Convert),
    (&["လုပ်ဆောင်သော ရလဒ်", 
        "လုပ်ဆောင်သောရလဒ်"],
        Kw::CallResult),
    (&["ရလဒ်အဖြစ် ပြန်ပေးပါ", 
        "ရလဒ် အဖြစ် ပြန်ပေးပါ", "ရလဒ်အဖြစ်ပြန်ပေးပါ"],
        Kw::ReturnValue),
    (&["အောက်ပါတို့ကို လုပ်ဆောင်ပါ", 
        "အောက်ပါ တို့ကို လုပ်ဆောင်ပါ"],
        Kw::BlockOpen),
    (&["အောက်ပါတို့ကို လုပ်ဆောင်မည်", 
        "အောက်ပါ တို့ကို လုပ်ဆောင်မည်"],
        Kw::FnOpen,),
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
    (&["လိပ်စာရှိကို",
        "လိပ်စာရှိ ကို"],
        Kw::ResizeHead),
    (&["လိပ်စာရှိ မင်မိုရီကို",
        "လိပ်စာရှိမင်မိုရီကို",
        "လိပ်စာရှိ မှတ်ဉာဏ်ကို"],
        Kw::FreeHead),
    (&["အောက်ပါတို့ကို လုပ်ဆောင်မယ်"],
        Kw::FnOpen), ];

#[test]
fn phrases_join_attached_and_spaced()
{
    do_for!((sp, kw) in CASES,  do_for!(t in *sp, assert_eq!(kws(t), [*kw], "{t}")));
}

#[test]
fn phrase_span_covers_all_pieces()
{
    let (out, file) = lex_text("x ကို ရလဒ် အဖြစ် ပြန်ပေးပါ။");
    let tok = out.tokens[2];
    let lo = (tok.span.lo - file.start_pos) as usize;
    let hi = (tok.span.hi - file.start_pos) as usize;
    assert_eq!(&file.src[lo..hi], "ရလဒ် အဖြစ် ပြန်ပေးပါ");
    assert!(!tok.joined);
}

#[test]
fn lone_pieces_remain_pieces()
{
    assert_eq!(kws("ဘစ်အလိုက်"), [Kw::BitwiseHead]);
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
    assert_eq!(kws("ပေါင်းလဒ် သည်"), [Kw::Sum, Kw::Thi]);
    // verb-nouns never absorb, even attached
    assert_eq!(kws("ပေါင်းလဒ်သည်"), [Kw::Sum, Kw::Thi]);
}

#[test]
fn compounds_keep_order_and_spans()
{
    assert_eq!(kws("ထက်ကြီးသည်"), [Kw::Htet, Kw::Greater]);
    assert_eq!(kws("အောက်ငယ်လျှင်"), [Kw::Auk, Kw::Less, Kw::Then]);
    let spaced = kws("အောက်ပါ တို့ကို လုပ်ဆောင်ပါ");
    let attached = kws("အောက်ပါတို့ကို လုပ်ဆောင်ပါ");
    assert_eq!(spaced, attached);
    assert_eq!(kws("တန်ဖိုးကို"), kws("တန်ဖိုး ကို"));
}
