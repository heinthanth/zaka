mod common;

use zakac_burmese_segmenter::segmenter::SegmenterOptions;
use zakac_burmese_segmenter::segmenter::segment;
use zakac_lexer::ident::is_ident;
use zakac_lexer::reserved::RESERVED;
use zakac_lexer::reserved::lookup;
use zakac_lexer::reserved::split_reserved;
use zakac_lexer::token::Kw;
use zakac_lexer::words::build_words;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_span::filename::FileName;
use zakac_span::pos::Pos;
use zakac_span::source_map::SourceMap;

/// Split `text`, which must be one word, and return the piece texts.
fn split(text: &str) -> Option<Vec<(Kw, String)>>
{
    let pre = segment(text, &SegmenterOptions::default());
    let words = build_words(text, Pos::ZERO, &pre);
    assert_eq!(words.len(), 1, "not one word: {text}");
    let pieces = split_reserved(&words[0])?;
    let piece_text = |(kw, span): (Kw, zakac_span::span::Span)| {
        (kw, text[span.lo.as_u32() as usize..span.hi.as_u32() as usize].to_string())
    };
    Some(pieces.into_iter().map(piece_text).collect())
}

#[test]
fn identifier_classes()
{
    assert!(is_ident("အသက်"));
    assert!(is_ident("၎င်း"));
    assert!(is_ident("age_1"));
    assert!(is_ident("_x"));
    assert!(is_ident("🔥"));
    assert!(is_ident("👨\u{200D}👩"));
    assert!(is_ident("春"));
    assert!(!is_ident("1a"));
    assert!(!is_ident("၁က"));
    assert!(!is_ident("#"));
    assert!(!is_ident("*"));
    assert!(!is_ident(""));
}

#[test]
fn table_spellings_are_normalized()
{
    let mut sm = SourceMap::new();
    for (text, kw) in RESERVED
    {
        let file = sm.new_source_file(FileName::Custom("t".into()), text.to_string());
        assert_eq!(&file.src, text, "{kw:?} is not in normalized spelling");
    }
}

#[test]
fn table_rows_look_up()
{
    do_for!((t, kw) in RESERVED, assert_eq!(lookup(t), Some(*kw), "{t}"));
    assert_eq!(lookup("အသက်"), None);
}

#[test]
fn table_rows_are_one_word_each()
{
    for (text, _) in RESERVED
    {
        let pre = segment(text, &SegmenterOptions::default());
        assert_eq!(build_words(text, Pos::ZERO, &pre).len(), 1, "{text}");
    }
}

#[test]
fn spec_compounds_split()
{
    let cases: &[(&str, &[Kw])] = &[
        ("တန်ဖိုးကို", &[Kw::Value, Kw::Ko]),
        ("ထက်ကြီးသည်", &[Kw::Htet, Kw::Greater, Kw::Thi]),
        ("အောက်ငယ်လျှင်", &[Kw::Auk, Kw::Less, Kw::Then]),
        ("အောက်ပါတို့ကို", &[Kw::Following, Kw::ListKo]),
        ("ရလဒ်အဖြစ်", &[Kw::Result, Kw::As]),
        ("ရွေ့သောရလဒ်", &[Kw::ShiftHead, Kw::Result]),
        ("မငယ်ပါ", &[Kw::NotLess, Kw::Pa]),
        ("မှတ်ချက်ကို", &[Kw::Note, Kw::Ko]),
    ];
    for (text, expected) in cases
    {
        let kws: Vec<Kw> =
            split(text).expect(text).into_iter().map(|(kw, _)| kw).collect();
        assert_eq!(&kws, expected, "{text}");
    }
}

#[test]
fn particles_alone_do_not_split()
{
    assert_eq!(split("ကိုကို"), None);
    assert_eq!(split("ပါသည်"), None);
}

#[test]
fn non_reserved_words_do_not_split()
{
    assert_eq!(split("အသက်သည်"), None);
    assert_eq!(split("ကိန်းအနေဖြင့်"), None);
    assert_eq!(split("သည်"), None);
}

/// Check that a split of `a + b` (or `a + b + c`) is a valid split.
fn check_join(parts: &[&str])
{
    let text: String = parts.concat();
    let Some(pieces) = split(&text)
    else
    {
        let all_particles = parts.iter().all(|p| lookup(p).unwrap().is_particle());
        assert!(all_particles, "{text} did not split");
        return;
    };
    let joined: String = pieces.iter().map(|(_, t)| t.as_str()).collect();
    assert_eq!(joined, text);
    assert!(pieces.len() <= parts.len(), "{text}: more pieces than parts");
    for (kw, piece) in &pieces
    {
        assert_eq!(lookup(piece), Some(*kw));
    }
}

fn split_back_pair(a: &str, b: &str)
{
    let text = format!("{a}{b}");
    do_if!(lookup(&text).is_some(), return);
    check_join(&[a, b]);
}

#[test]
fn all_pairs_split_back()
{
    do_for!((a, _) in RESERVED, do_for!((b, _) in RESERVED, split_back_pair(a, b)));
}

#[test]
fn sampled_triples_split_back()
{
    let mut seed: u64 = 0x5eed_2026;
    let mut next = |n: usize| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as usize % n
    };
    for _ in 0..3000
    {
        let parts = [
            RESERVED[next(RESERVED.len())].0,
            RESERVED[next(RESERVED.len())].0,
            RESERVED[next(RESERVED.len())].0,
        ];
        do_if!(lookup(&parts.concat()).is_some(), continue);
        check_join(&parts);
    }
}
