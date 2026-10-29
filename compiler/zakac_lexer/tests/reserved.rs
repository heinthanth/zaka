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
fn borrow_words_are_whole_words()
{
    assert_eq!(lookup("ငှားထားသော"), Some(Kw::Borrowed));
    assert_eq!(lookup("တန်ဖိုးအငှား"), Some(Kw::Lend));
    assert_eq!(lookup("လိပ်စာအကြမ်း"), Some(Kw::RawAddress));
    assert_eq!(split("တန်ဖိုးအငှား"), None);
    assert_eq!(split("လိပ်စာအကြမ်း"), None);
}

#[test]
fn spec_compounds_split()
{
    let cases: &[(&str, &[Kw])] = &[
        ("တန်ဖိုးကို", &[Kw::Value, Kw::Ko]),
        ("တန်ဖိုးသည်", &[Kw::Value, Kw::Thi]),
        ("ထက်ကြီးသည်", &[Kw::Htet, Kw::Greater, Kw::Thi]),
        ("အောက်ငယ်လျှင်", &[Kw::Auk, Kw::Less, Kw::Then]),
        ("အောက်ပါတို့ကို", &[Kw::Following, Kw::ListKo]),
        ("ရလဒ်အဖြစ်", &[Kw::Result, Kw::As]),
        ("ရွေ့သောရလဒ်", &[Kw::ShiftHead, Kw::Result]),
        ("မငယ်ပါ", &[Kw::NotLess, Kw::Pa]),
        ("မှတ်ချက်ကို", &[Kw::Note, Kw::Ko]),
        ("လိပ်စာကို", &[Kw::Address, Kw::Ko]),
        ("လိပ်စာရှိကို", &[Kw::AtAddress, Kw::Ko]),
        ("လိပ်စာရှိမင်မိုရီကို", &[Kw::AtAddress, Kw::Mem, Kw::Ko]),
        ("လိပ်စာရှိမှတ်ဉာဏ်ကို", &[Kw::AtAddress, Kw::Mem, Kw::Ko]),
        ("တန်ဖိုးအငှားကို", &[Kw::Lend, Kw::Ko]),
        ("လိပ်စာအကြမ်းကို", &[Kw::RawAddress, Kw::Ko]),
    ];
    for (text, expected) in cases
    {
        let kws: Vec<Kw> =
            split(text).expect(text).into_iter().map(|(kw, _)| kw).collect();
        assert_eq!(&kws, expected, "{text}");
    }
}

#[test]
fn spoken_endings_lex_as_the_written_word()
{
    let triples = [
        ("လုပ်ဆောင်မည်", "လုပ်ဆောင်မယ်", Kw::WillDo),
        ("ပြန်ပေးမည်", "ပြန်ပေးမယ်", Kw::WillReturn),
        ("သိမ်းထားမည်", "သိမ်းထားမယ်", Kw::WillKeep),
        ("နေရာအချဲ့အချုံ့လုပ်မည်", "နေရာအချဲ့အချုံ့လုပ်မယ်", Kw::WillResize),
        ("ပြန်လည်စွန့်လွှတ်မည်", "ပြန်လည်စွန့်လွှတ်မယ်", Kw::WillRelease),
    ];
    do_for!((written, spoken, kw) in triples, {
        assert_eq!(lookup(written), Some(kw), "{written}");
        assert_eq!(lookup(spoken), Some(kw), "{spoken}");
    });
    assert_eq!(lookup("မင်မိုရီ"), Some(Kw::Mem));
    assert_eq!(lookup("မှတ်ဉာဏ်"), Some(Kw::Mem));
}

#[test]
fn both_loop_links_lex_as_one_word()
{
    assert_eq!(lookup("ဖြစ်ပြီး"), Some(Kw::Being));
    assert_eq!(lookup("ဖြစ်ကာ"), Some(Kw::Being));
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
    assert_eq!(split("ကိန်းအနေဖြင့်"), None);
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

#[test]
fn list_words_lex_and_split()
{
    assert_eq!(lookup("စာရင်း"), Some(Kw::List));
    assert_eq!(lookup("စာရင်းအလွတ်"), Some(Kw::BlankList));
    assert_eq!(lookup("အကွက်အရေအတွက်"), Some(Kw::CellCount));
    assert_eq!(lookup("ခုရှိသော"), Some(Kw::Having));
    assert_eq!(lookup("ရှိသော"), Some(Kw::Having));
    assert_eq!(lookup("တို့ပါသော"), Some(Kw::ListOf));
    let ordinal = split("ခုမြောက်အကွက်").expect("splits");
    let kws: Vec<Kw> = ordinal.iter().map(|(kw, _)| *kw).collect();
    assert_eq!(kws, [Kw::Ordinal, Kw::Cell]);
}

#[test]
fn names_that_start_with_list_words_remain_names()
{
    assert_eq!(split("စာရင်းအင်း"), None);
    assert_eq!(split("သူတို့"), None);
    assert_eq!(lookup("အရေအတွက်"), None);
}
