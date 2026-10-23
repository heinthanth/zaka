use zakac_burmese_segmenter::segmenter::PreLexicalTokenKind;
use zakac_burmese_segmenter::segmenter::SegmenterOptions;
use zakac_burmese_segmenter::segmenter::segment;

fn lexemes(text: &str) -> Vec<&str>
{
    let options = SegmenterOptions::default();
    segment(text, &options).into_iter().map(|t| t.lexeme).collect()
}

fn lexemes_with<'s>(text: &'s str, options: &SegmenterOptions) -> Vec<&'s str>
{
    segment(text, &options).into_iter().map(|t| t.lexeme).collect()
}

fn assert_segments(input: &str, segments: &str)
{
    let expected: Vec<&str> = segments.split('/').collect();
    assert_eq!(lexemes(input), expected);
}

#[test]
fn segment_metta_sutta()
{
    let input = include_str!("../samples/metta_sutta.in");
    let segments = include_str!("metta_sutta.segments");
    assert_segments(input, segments);
}

#[test]
fn segment_mangala_sutta()
{
    let input = include_str!("../samples/mangala_sutta.in");
    let segments = include_str!("mangala_sutta.segments");
    assert_segments(input, segments);
}

#[test]
fn segment_sample_paragraph1()
{
    let input = include_str!("../samples/paragraph1.in");
    let segments = include_str!("paragraph1.segments");
    assert_segments(input, segments);
}

#[test]
fn blank_input()
{
    assert_eq!(lexemes("").len(), 0);
}

#[test]
fn single_consonant()
{
    assert_eq!(lexemes("က"), ["က"]);
}

#[test]
fn basic_vanna_break()
{
    assert_eq!(lexemes("ကာကား"), ["ကာ", "ကား"]);
}

#[test]
fn kinzi()
{
    assert_eq!(lexemes("မင်္ဂလာ"), ["မင်္ဂ", "လာ"]);
    assert_eq!(lexemes("သင်္ဘော"), ["သင်္ဘော"]);
}

#[test]
fn stacked_consonants()
{
    assert_eq!(lexemes("ဝတ္ထု"), ["ဝတ္ထု"]);
    assert_eq!(lexemes("ဥက္ကဋ္ဌ"), ["ဥက္ကဋ္ဌ"]);
}

#[test]
fn asat_and_aforementioned()
{
    assert_eq!(lexemes("ကျွန်ုပ်"), ["ကျွန်ုပ်"]);
    assert_eq!(lexemes("၎င်း"), ["၎င်း"]);
    assert_eq!(lexemes("ဦး"), ["ဦး"]);
}

#[test]
fn standalone_and_digits()
{
    assert_eq!(lexemes("ဤ"), ["ဤ"]);
    assert_eq!(lexemes("၏။"), ["၏", "။"]);
    assert_eq!(lexemes("၁၉၉၀"), ["၁", "၉", "၉", "၀"]);
}

#[test]
fn mixed_script_isolated()
{
    let expected = ["တ", "ရုတ်", "နိုင်", "ငံ", "成", "都", "မြို့"];
    assert_eq!(lexemes("တရုတ်နိုင်ငံ成都မြို့"), expected);
    assert_eq!(lexemes("hello"), ["h", "e", "l", "l", "o"]);

    let expected = ["စု", "စု", "ပေါင်း", " ", "=", " ", "၁", "၀", "၀"];
    assert_eq!(lexemes("စုစုပေါင်း = ၁၀၀"), expected);
}

#[test]
fn coalesce_non_burmese()
{
    let options = SegmenterOptions::builder().coalesce_non_burmese_chars(true).build();
    let expected = ["တ", "ရုတ်", "နိုင်", "ငံ", "成都", "မြို့"];
    assert_eq!(lexemes_with("တရုတ်နိုင်ငံ成都မြို့", &options), expected);
    assert_eq!(lexemes_with("hello", &options), ["hello"]);
}

fn kinds(text: &str) -> Vec<PreLexicalTokenKind>
{
    segment(text, &SegmenterOptions::default()).into_iter().map(|t| t.kind).collect()
}

#[test]
fn token_kinds()
{
    use PreLexicalTokenKind::*;

    assert!(matches!(kinds("ကာ")[..], [BurmeseVanna]));
    assert!(matches!(kinds("၁၀")[..], [BurmeseDigit, BurmeseDigit]));
    assert!(matches!(kinds("ab")[..], [NonBurmeseChar, NonBurmeseChar]));
}

#[test]
fn token_spans_cover_lexemes()
{
    let text = "ကာကား";
    let tokens = segment(text, &SegmenterOptions::default());
    for token in tokens
    {
        let start = token.span.lo.as_u32() as usize;
        let end = token.span.hi.as_u32() as usize;
        assert_eq!(&text[start..end], token.lexeme);
    }
}

#[test]
fn aw_vowel_starts_new_vanna()
{
    // the asat in `ော်` belongs to the vowel, not to the consonant
    assert_eq!(lexemes("သည်ကျော်ပါ"), ["သည်", "ကျော်", "ပါ"]);
    assert_eq!(lexemes("ပြောင်းရွှေ့ကျော်"), ["ပြောင်း", "ရွှေ့", "ကျော်"]);
    // the dot below can still sit between a killed consonant and its asat
    assert_eq!(lexemes("နှင့်"), ["နှင့်"]);
    assert_eq!(lexemes("ကိန့်"), ["ကိန့်"]);
}
