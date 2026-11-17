use std::path::PathBuf;

use zakac_diagnostics::Code;
use zakac_lexer::reserved::RESERVED;
use zakac_lexer::reserved::lookup;
use zakac_lexer::spoken::SPOKEN;
use zakac_lexer::spoken::spoken;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;

use crate::common::codes;
use crate::common::kinds;
use crate::common::kws;
use crate::common::lex_text;

/// The spoken spellings, one line each: spoken, written, reserved word, and
/// the other written word and reserved word it can stand for.
fn table() -> String
{
    let line = |s: &zakac_lexer::spoken::Spoken| {
        let also = s.also.map(|(kw, w)| format!("\t{w}\t{kw:?}")).unwrap_or_default();
        format!("{}\t{}\t{:?}{also}\n", s.spoken, s.written, s.kw)
    };
    SPOKEN.iter().map(line).collect()
}

#[test]
fn the_spoken_spellings_match_the_reviewed_table()
{
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/expected/spoken.tsv");
    let found = table();
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::write(&path, &found).unwrap();
    }
    let expected = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        found, expected,
        "the spoken spellings differ from tests/expected/spoken.tsv"
    );
}

#[test]
fn no_spoken_spelling_is_a_written_word_or_two_words()
{
    for s in SPOKEN.iter()
    {
        assert!(
            !RESERVED.iter().any(|(w, _)| *w == s.spoken),
            "{} is written",
            s.spoken
        );
        let same: Vec<Kw> =
            SPOKEN.iter().filter(|o| o.spoken == s.spoken).map(|o| o.kw).collect();
        assert_eq!(same, [s.kw], "{} reads as two words", s.spoken);
    }
}

#[test]
fn each_spoken_word_reads_as_its_written_word()
{
    let pairs = [
        ("မှာ", Kw::At),
        ("လို့", Kw::Hu),
        ("ရင်", Kw::Then),
        ("ကနေ", Kw::From),
        ("နဲ့", Kw::Hnint),
        ("မယ့်", Kw::Named),
        ("မယ်", Kw::Will),
        ("တယ်", Kw::Thi),
        ("တဲ့", Kw::That),
        ("ရှိတဲ့", Kw::Having),
        ("ဟူတဲ့", Kw::Called),
        ("ထားမယ်", Kw::Assign),
        ("ပြန်ပေးမယ်", Kw::WillReturn),
        ("ဖြစ်တယ်", Kw::Is),
        ("ထဲမှာ", Kw::Within),
        ("ဖြစ်ရင်", Kw::ThenIs),
        ("ဒီနေရာမှာ", Kw::Here),
        ("မှတ်ယူနိုင်ပါတယ်", Kw::CanRegard),
    ];
    do_for!((text, kw) in pairs, assert_eq!(lookup(text), Some(kw), "{text}"));
    assert_eq!(kinds("ရဲ့"), [TokenKind::Punct(Punct::Ei)]);
    // The `မှ` of `မှတ်` is a vanna of its own, not the particle.
    assert!(spoken("ကနေတ်").is_none());
    assert_eq!(lookup("မည်သည်ကိုမှ"), Some(Kw::Nothing));
    assert!(spoken("မယ်တယ်ကိုကနေ").is_none());
}

#[test]
fn a_spoken_ending_on_a_verb_is_its_particle()
{
    assert_eq!(kws("ညီတယ်"), [Kw::Equal]);
    assert_eq!(kws("ညီပါတယ်"), [Kw::Equal]);
    let (out, _) = lex_text("ညီတယ်");
    assert_eq!(out.tokens[0].particle, zakac_lexer::token::Particle::Thi);
}

#[test]
fn each_spoken_word_is_reported_with_its_fix()
{
    let (out, file) = lex_text("ထားမယ်");
    let d = &out.diagnostics[0];
    assert_eq!(d.code, Code::SpokenForm);
    assert_eq!(d.fixes[0].edits[0].text, "ထားမည်");
    let at =
        (d.span.lo - file.start_pos) as usize..(d.span.hi - file.start_pos) as usize;
    assert_eq!(&file.src[at], "ထားမယ်");
    assert_eq!(codes("ထားမည်"), []);
}

#[test]
fn a_file_with_both_forms_gets_one_mix_warning_at_the_form_it_uses_less()
{
    let (out, file) = lex_text("ထားမယ် ထားမယ် ထားမည်");
    let mix: Vec<_> =
        out.diagnostics.iter().filter(|d| d.code == Code::MixedForms).collect();
    assert_eq!(mix.len(), 1);
    let at = (mix[0].span.lo - file.start_pos) as usize;
    assert_eq!(&file.src[at..], "ထားမည်");
    assert_eq!(mix[0].fixes[0].edits[0].text, "ထားမယ်");
    assert!(!codes("ထားမယ် ထားမယ်").contains(&Code::MixedForms));
}

#[test]
fn a_topic_marker_is_no_written_form_in_the_mix()
{
    let program = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မယ်။\n\"က\" လို့ ပြသမယ်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။";
    assert!(!codes(program).contains(&Code::MixedForms));
    assert!(codes("က သည် ခ နှင့် ညီတယ် ညီသည်").contains(&Code::MixedForms));
}

#[test]
fn a_spelling_of_two_words_reads_as_the_one_in_tho_and_stands_for_the_other()
{
    let having = spoken("ရှိတဲ့").expect("a spoken spelling");
    assert_eq!(having.also, Some((Kw::HavingRel, "ရှိသည့်")));
    let called = spoken("ဟူတဲ့").expect("a spoken spelling");
    assert_eq!(called.also, Some((Kw::CalledRel, "ဟူသည့်")));
}
