//! The alias table: the spellings it gives, and the checks that catch a slip
//! in a row of it.

use std::path::PathBuf;

use zakac_lexer::aliases::ALIASES;
use zakac_lexer::aliases::WRITTEN;
use zakac_lexer::reserved::RESERVED;
use zakac_lexer::reserved::lookup;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;

use crate::common::kinds;
use crate::common::lexemes;

/// The written spellings the aliases add, one line each: the spelling, the
/// first spelling of its word, and the reserved word.
fn table() -> String
{
    let added = WRITTEN.iter().filter(|w| w.text != w.first);
    added.map(|w| format!("{}\t{}\t{:?}\n", w.text, w.first, w.kw)).collect()
}

#[test]
fn the_written_spellings_match_the_reviewed_table()
{
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/expected/aliases.tsv");
    let found = table();
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::write(&path, &found).unwrap();
    }
    let expected = std::fs::read_to_string(&path).unwrap();
    assert_eq!(found, expected, "the spellings differ from tests/expected/aliases.tsv");
}

#[test]
fn no_written_spelling_reads_as_two_words()
{
    for w in WRITTEN.iter()
    {
        let kws: Vec<Kw> =
            WRITTEN.iter().filter(|o| o.text == w.text).map(|o| o.kw).collect();
        assert_eq!(kws, [w.kw], "{} reads as {kws:?}", w.text);
    }
}

#[test]
fn each_word_a_row_names_is_a_reserved_word()
{
    let reserved = |w: &str| RESERVED.iter().any(|(r, _)| *r == w);
    for row in ALIASES
    {
        let named = row.swaps_within.iter().chain(row.except);
        do_for!(w in named, assert!(reserved(w), "{w} in the row of {}", row.base));
    }
}

#[test]
fn each_part_of_a_row_stands_in_a_reserved_word()
{
    for row in ALIASES
    {
        let found = |p: &&str| WRITTEN.iter().any(|w| w.text.contains(*p));
        assert!(found(&row.base), "the row of {}", row.base);
    }
}

#[test]
fn spoken_parts_of_several_vannas_read_as_their_word()
{
    let pairs = [
        ("ဒါမှမဟုတ်", Kw::Or),
        ("ဒါမှမဟုတ်ဘဲ", Kw::ElseIf),
        ("ဒါမှမဟုတ်ရင်", Kw::Else),
        ("စိတ်မချရပေမယ့်", Kw::UnsafeThough),
        ("ရှိစေအောင်", Kw::ToHave),
        ("ရှိစေဖို့", Kw::ToHave),
        ("ပြောင်းလဲဖို့", Kw::ToChange),
        ("ဖြစ်၏", Kw::Is),
    ];
    do_for!((w, kw) in pairs, assert_eq!(lookup(w), Some(kw), "{w}"));
    // `မဟုတ်လျှင်` is "if not", `၏` alone is "of", and `အောင်` is no spoken
    // form of each `ရန်`.
    do_for!(w in ["မဟုတ်လျှင်", "၏", "ပြောင်းလဲအောင်"], assert_eq!(lookup(w), None, "{w}"));
}

#[test]
fn formal_ends_lex_whole_in_a_file()
{
    // The segmenter makes `၏` a word of its own, so these go through the word
    // builder and the lexer, not the table alone.
    let formal = WRITTEN.iter().filter(|w| w.text.ends_with('၏'));
    for w in formal
    {
        let text = format!("ကျော် {}။", w.text);
        assert_eq!(lexemes(&text)[1], w.text, "{text}");
        assert_eq!(kinds(&text)[1], TokenKind::Kw(w.kw), "{text}");
    }
    let glued = lexemes("ကျော် သည် ကိန်းပြည့်ဖြစ်၏။");
    assert_eq!(glued[2..], ["ကိန်းပြည့်", "ဖြစ်၏", "။"]);
}

#[test]
fn the_mark_of_stands_alone_in_a_file()
{
    let ei = TokenKind::Punct(Punct::Ei);
    let texts = ["ကျော်၏ အသက်", "ကျော် ၏ အသက်", "က နှင့် ခ ၏ ပေါင်းလဒ်", "စ ၏ ဖြစ်"];
    do_for!(t in texts, assert!(kinds(t).contains(&ei), "{t}"));
    assert_eq!(lexemes("ကျော်၏ အသက်"), ["ကျော်", "၏", "အသက်"]);
}
