mod common;

use zakac_burmese_segmenter::segmenter::SegmenterOptions;
use zakac_burmese_segmenter::segmenter::segment;
use zakac_diagnostics::Code;
use zakac_lexer::ident::is_ident;
use zakac_lexer::reserved::OLD_ENDINGS;
use zakac_lexer::reserved::RESERVED;
use zakac_lexer::reserved::lookup;
use zakac_lexer::reserved::split_reserved;
use zakac_lexer::spelling::canonical;
use zakac_lexer::spelling::spellings;
use zakac_lexer::token::Kw;
use zakac_lexer::token::TokenKind;
use zakac_lexer::words::build_words;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;
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
    for gone in ["ငှားထားသော", "တန်ဖိုးအငှား", "လိပ်စာအကြမ်း"]
    {
        assert_eq!(lookup(gone), None, "{gone}");
    }
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
        ("ရှိမင်မိုရီကို", &[Kw::Located, Kw::Mem, Kw::Ko]),
        ("ရှိမှတ်ဉာဏ်ကို", &[Kw::Located, Kw::Mem, Kw::Ko]),
        ("ပရိုဂရမ်သည်", &[Kw::Program, Kw::Thi]),
    ];
    for (text, expected) in cases
    {
        let kws: Vec<Kw> =
            split(text).expect(text).into_iter().map(|(kw, _)| kw).collect();
        assert_eq!(&kws, expected, "{text}");
    }
}

#[test]
fn spoken_endings_are_not_reserved()
{
    let triples = [
        ("လုပ်ဆောင်မည်", "လုပ်ဆောင်မယ်", Kw::WillDo),
        ("ပြန်ပေးမည်", "ပြန်ပေးမယ်", Kw::WillReturn),
        ("သိမ်းထားမည်", "သိမ်းထားမယ်", Kw::WillKeep),
        ("နေရာအချဲ့အချုံ့လုပ်မည်", "နေရာအချဲ့အချုံ့လုပ်မယ်", Kw::WillResize),
        ("ပြန်လည်စွန့်လွှတ်မည်", "ပြန်လည်စွန့်လွှတ်မယ်", Kw::WillRelease),
        ("ကူးယူမည်", "ကူးယူမယ်", Kw::WillBlit),
        ("ပြောင်းထည့်မည်", "ပြောင်းထည့်မယ်", Kw::WillShift),
        ("ဖြည့်မည်", "ဖြည့်မယ်", Kw::WillFill),
    ];
    do_for!((written, spoken, kw) in triples, {
        assert_eq!(lookup(written), Some(kw), "{written}");
        assert_eq!(lookup(spoken), None, "{spoken}");
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
        // A word made of particles alone is kept whole, and two neighbouring
        // parts can join into one particle, such as `တို့` and `ကို`.
        let particle = |s: &str| lookup(s).is_some_and(Kw::is_particle);
        let all_particles = parts.iter().all(|p| particle(p));
        let joined_pair = |i: usize| {
            let (left, right) = parts.split_at(i);
            particle(&left.concat()) && particle(&right.concat())
        };
        let paired = parts.len() == 3 && (joined_pair(1) || joined_pair(2));
        assert!(all_particles || paired, "{text} did not split");
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
fn table_words_lex_and_split()
{
    assert_eq!(lookup("ဇယား"), Some(Kw::Table));
    assert_eq!(lookup("ဇယားအလွတ်"), Some(Kw::BlankTable));
    assert_eq!(lookup("ညွှန်းသော"), Some(Kw::Indexed));
    assert_eq!(lookup("အညွှန်း"), Some(Kw::IndexWord));
    assert_eq!(lookup("အညွှန်းအရေအတွက်"), Some(Kw::IndexCount));
    assert_eq!(lookup("အညွှန်းတိုင်း"), Some(Kw::EachIndex));
    assert_eq!(lookup("အတန်းတိုင်း"), Some(Kw::EachRow));
    // The null pointer word keeps its own token.
    assert_eq!(lookup("မညွှန်းသော"), Some(Kw::PointsAtNo));
    let kws = |w: &str| -> Vec<Kw> {
        split(w).expect("splits").iter().map(|(kw, _)| *kw).collect()
    };
    assert_eq!(kws("ဖြင့်ညွှန်းသော"), [Kw::Hpjint, Kw::Indexed]);
    assert_eq!(kws("အညွှန်းတိုင်းကို"), [Kw::EachIndex, Kw::Ko]);
    assert_eq!(kws("အတန်းတိုင်းကို"), [Kw::EachRow, Kw::Ko]);
    // A name that ends in a reserved word remains a name.
    assert_eq!(split("ဖိုင်အညွှန်း"), None);
}

#[test]
fn replace_words_lex_and_split()
{
    assert_eq!(lookup("အစားထိုးမည်"), Some(Kw::Replace));
    assert_eq!(lookup("လုံးမြောက်"), Some(Kw::Ordinal));
    assert_eq!(lookup("ထိ"), Some(Kw::Upto));
    let kws = |word| {
        split(word).expect("splits").iter().map(|(kw, _)| *kw).collect::<Vec<Kw>>()
    };
    assert_eq!(kws("လုံးမြောက်စာလုံးမှ"), [Kw::Ordinal, Kw::CharKind, Kw::From]);
    assert_eq!(kws("လုံးမြောက်စာလုံးထိကို"), [Kw::Ordinal, Kw::CharKind, Kw::Upto, Kw::Ko]);
    assert_eq!(kws("ထိကို"), [Kw::Upto, Kw::Ko]);
}

#[test]
fn slice_words_lex_whole()
{
    assert_eq!(lookup("တစ်စိတ်တစ်ပိုင်း"), None);
    assert_eq!(lookup("မှစ၍"), Some(Kw::StartingAt));
    assert_eq!(lookup("ကွက်"), Some(Kw::CellWord));
    assert_eq!(split("တစ်ကွက်ကျော်"), None);
}

#[test]
fn names_that_start_with_list_words_remain_names()
{
    assert_eq!(split("စာရင်းအင်း"), None);
    assert_eq!(split("သူတို့"), None);
    assert_eq!(split("ပရိုဂရမ်ပိတ်"), None);
    assert_eq!(lookup("အရေအတွက်"), None);
}

/// `ဒေတာ` and `အချက်အလက်` are one word in reserved words and names.
#[test]
fn both_spellings_of_data_are_one_word()
{
    assert_eq!(lookup("ဒေတာအမျိုးအစား"), Some(Kw::DataKindOpen));
    assert_eq!(lookup("အချက်အလက်အမျိုးအစား"), Some(Kw::DataKindOpen));
    assert_eq!(canonical("ဒေတာစာရင်း"), "အချက်အလက်စာရင်း");
    assert_eq!(canonical("အသက်"), "အသက်");
    assert_eq!(spellings("ဒေတာစာရင်း"), ["ဒေတာစာရင်း", "အချက်အလက်စာရင်း"]);
    assert_eq!(spellings("အချက်အလက်"), ["အချက်အလက်", "ဒေတာ"]);
    assert_eq!(spellings("အသက်"), ["အသက်"]);
}

/// The reserved words of `text`, its names and literals left out.
fn kw_alone(text: &str) -> Vec<Kw>
{
    let kws = common::kinds(text).into_iter();
    kws.filter_map(|k| tern!(let TokenKind::Kw(kw) = k, Some(kw), None)).collect()
}

/// The words of the declarations of structs, enumerations and aliases lex
/// whole, and the longest one wins over the words inside it, joined to
/// `တွင်` or apart from it.
#[test]
fn declaration_words_lex_joined_and_apart()
{
    let cases: &[(&str, &[Kw])] = &[
        ("ဒေတာအစုအဝေးတွင်", &[Kw::StructWord, Kw::At]),
        ("ဒေတာအစုအဝေး တွင်", &[Kw::StructWord, Kw::At]),
        ("အချက်အလက်အစုအဝေးတွင်", &[Kw::StructWord, Kw::At]),
        ("ဒေတာအမျိုးအစားကွဲတွင်", &[Kw::EnumWord, Kw::At]),
        ("ဒေတာအမျိုးအစားကွဲ တွင်", &[Kw::EnumWord, Kw::At]),
        ("အချက်အလက်အမျိုးအစားကွဲတွင်", &[Kw::EnumWord, Kw::At]),
        ("ဒေတာအမျိုးအစား", &[Kw::DataKindOpen]),
        ("တို့ပါဝင်သည်", &[Kw::FieldsClose]),
        ("တို့ ပါဝင်သည်", &[Kw::FieldsClose]),
        ("တို့ပါဝင်ပြီး", &[Kw::FieldsCloseAnd]),
        ("ပါဝင်သည်", &[Kw::Contains]),
        ("ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်", &[Kw::InitialMark]),
        ("ကနဦးတန်ဖိုး အနေဖြင့် သုံးမည့်", &[Kw::InitialMark]),
        ("တန်ဖိုးအနေဖြင့်", &[Kw::ValueAs]),
        ("တန်ဖိုး အနေဖြင့်", &[Kw::ValueAs]),
        ("ရှိသည့်", &[Kw::HavingRel]),
        ("ဟုလည်း မှတ်ယူနိုင်ပါသည်", &[Kw::AliasClose]),
        ("ဟုလည်း မှတ်ယူနိုင်ပြီး", &[Kw::AliasCloseAnd]),
        ("ဝိဘတ် \"x\" ကို သုံး၍", &[Kw::MarkWord, Kw::Ko, Kw::Using]),
        ("လုံးစာ", &[Kw::Items]),
    ];
    do_for!((text, want) in cases, assert_eq!(kw_alone(text), *want, "{text}"));
}

/// `ဟူသော လုပ်ဆောင်ချက် အမျိုးအစား` keeps `ဟူသော` apart from the function kind
/// after it, and `ဟူသော လုပ်ဆောင်ချက်` alone is the function value, as is
/// `မည့် လုပ်ဆောင်ချက်`.
#[test]
fn called_before_a_function_kind_keeps_apart()
{
    let kind = kw_alone("ဖ ဟူသော လုပ်ဆောင်ချက် အမျိုးအစား တစ်ခု");
    assert_eq!(kind, [Kw::Called, Kw::FnKind]);
    assert_eq!(kw_alone("ဖ ဟူသော လုပ်ဆောင်ချက်"), [Kw::FnValue]);
    assert_eq!(kw_alone("ဖ မည့် လုပ်ဆောင်ချက်ကို"), [Kw::FnValue, Kw::Ko]);
}

#[test]
fn an_old_ending_is_reported_and_read_as_the_new_verb()
{
    for (old, new, kw) in OLD_ENDINGS
    {
        let (out, _) = common::lex_text(old);
        assert_eq!(out.diagnostics.len(), 1, "{old}");
        assert_eq!(out.diagnostics[0].code, Code::OldEnding, "{old}");
        assert_eq!(common::kws(old), [*kw], "{old}");
        let (out, _) = common::lex_text(new);
        assert!(out.diagnostics.is_empty(), "{new}");
        assert_eq!(common::kws(new), [*kw], "{new}");
    }
}
