use zakac_burmese_segmenter::segmenter::SegmenterOptions;
use zakac_burmese_segmenter::segmenter::segment;
use zakac_lexer::words::WordKind;
use zakac_lexer::words::build_words;
use zakac_span::pos::Pos;

/// Words of `text` as `(text, kind, joined)`.
fn words(text: &str) -> Vec<(String, WordKind, bool)>
{
    let pre = segment(text, &SegmenterOptions::default());
    build_words(text, Pos::ZERO, &pre)
        .into_iter()
        .map(|w| (w.text.to_string(), w.kind, w.joined))
        .collect()
}

/// Word texts of `text`.
fn texts(text: &str) -> Vec<String> { words(text).into_iter().map(|w| w.0).collect() }

#[test]
fn blank_input()
{
    assert!(words("").is_empty());
}

#[test]
fn whitespace_separates()
{
    for ws in ["\t", "\n", "\u{0B}", "\u{0C}", "\r", " ", "\u{200B}"]
    {
        let text = format!("အသက်{ws}ကို");
        assert_eq!(texts(&text), ["အသက်", "ကို"], "{ws:?}");
        assert!(!words(&text)[1].2, "{ws:?} should unjoin");
    }
}

#[test]
fn no_break_space_is_not_whitespace()
{
    assert_eq!(texts("a\u{A0}b"), ["a\u{A0}b"]);
}

#[test]
fn self_delimiting_characters()
{
    assert_eq!(texts("ရလဒ်၏"), ["ရလဒ်", "၏"]);
    assert_eq!(texts("ပြသပါ။"), ["ပြသပါ", "။"]);
    assert_eq!(texts("က၊ခ"), ["က", "၊", "ခ"]);
    assert_eq!(texts("a+b"), ["a", "+", "b"]);
    assert_eq!(texts("`a b`"), ["`", "a", "b", "`"]);
    assert_eq!(texts("12.34"), ["12", ".", "34"]);
    assert_eq!(texts("<="), ["<", "="]);
    assert_eq!(texts("a;b"), ["a", ";", "b"]);
    assert_eq!(texts("a_b"), ["a_b"]);
    let kinds: Vec<WordKind> = words("(\"'").into_iter().map(|w| w.1).collect();

    let e = [WordKind::Delim('('), WordKind::Delim('"'), WordKind::Delim('\'')];
    assert_eq!(kinds, e);
}

#[test]
fn script_change_ends_word()
{
    assert_eq!(texts("abcက"), ["abc", "က"]);
    assert_eq!(texts("၁၂34"), ["၁၂", "34"]);
    let joined: Vec<bool> = words("abcက").into_iter().map(|w| w.2).collect();
    assert_eq!(joined, [false, true]);
}

#[test]
fn burmese_digits_and_vannas_join()
{
    assert_eq!(texts("၁၉၉၀"), ["၁၉၉၀"]);
    assert_eq!(texts("က၁"), ["က၁"]);
    assert_eq!(words("၁၉၉၀")[0].1, WordKind::Burmese);
}

#[test]
fn completed_sign_remains_in_word()
{
    assert_eq!(texts("ဖြစ်၍ ရေတွက်၍"), ["ဖြစ်၍", "ရေတွက်၍"]);
}

#[test]
fn spans_cover_text()
{
    let text = "အသက် + ၁၈";
    let pre = segment(text, &SegmenterOptions::default());
    for word in build_words(text, Pos::ZERO, &pre)
    {
        let lo = word.span.lo.as_u32() as usize;
        let hi = word.span.hi.as_u32() as usize;
        assert_eq!(&text[lo..hi], word.text);
    }
}
