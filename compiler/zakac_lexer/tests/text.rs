mod common;

use common::codes;
use common::kinds;
use common::lex_text;
use zakac_diagnostics::Code;
use zakac_lexer::text::scan_char;
use zakac_lexer::text::scan_raw_ident;
use zakac_lexer::text::scan_str;
use zakac_lexer::token::StrPart;
use zakac_lexer::token::TokenKind;
use zakac_span::pos::Pos;

/// Decoded value and diagnostic codes of the string literal at the start of
/// `text`.
fn decode(text: &str) -> (String, Vec<Code>)
{
    let scan = scan_str(text, 0, Pos::ZERO);
    (scan.value.value, scan.diags.into_iter().map(|d| d.code).collect())
}

#[test]
fn backslash_escapes()
{
    let cases = [
        (r#""\n""#, "\n"),
        (r#""\r""#, "\r"),
        (r#""\t""#, "\t"),
        (r#""\0""#, "\0"),
        (r#""\\""#, "\\"),
        (r#""\'""#, "'"),
        (r#""\"""#, "\""),
        (r#""\u1000""#, "က"),
        (r#""\u0001F525""#, "🔥"),
    ];
    for (text, value) in cases
    {
        assert_eq!(decode(text), (value.to_string(), vec![]), "{text}");
    }
}

#[test]
fn brace_escapes()
{
    assert_eq!(decode(r#""{{x}}""#).0, "{x}");
    assert_eq!(decode("\"a{နောက်တစ်ကြောင်း}b\"").0, "a\nb");
    assert_eq!(decode("\"{ယူနီကုဒ် 1000}\"").0, "က");
    assert_eq!(decode("\"{ယူနီကုဒ် 0001F525}\"").0, "🔥");
}

#[test]
fn unknown_escapes_keep_a_hole()
{
    let scan = scan_str(r#""အသက် {အသက်} နှစ်""#, 0, Pos::ZERO);
    assert!(scan.value.parts.contains(&StrPart::Hole("အသက်".into())));
    assert_eq!(scan.value.value, "အသက် {အသက်} နှစ်");
    assert_eq!(scan.diags.len(), 1);
    assert!(scan.diags[0].help.is_some());

    for text in
        [r#""\q""#, r#""\u12""#, r#""\u123456""#, r#""\uD800""#, r#""a}b""#, r#""a{b""#]
    {
        assert_eq!(decode(text).1, [Code::UnknownEscape], "{text}");
    }
}

#[test]
fn unterminated_literals()
{
    let scan = scan_str("\"abc\nnext", 0, Pos::ZERO);
    assert_eq!(scan.end, 4);
    assert_eq!(scan.value.value, "abc");
    assert_eq!(scan.diags[0].code, Code::UnterminatedString);

    assert_eq!(decode("\"abc").1, [Code::UnterminatedString]);
    let scan = scan_char("'a", 0, Pos::ZERO);
    assert_eq!(scan.diags[0].code, Code::UnterminatedChar);
}

#[test]
fn char_literals()
{
    for (text, ch) in [
        ("'က'", 'က'),
        ("'A'", 'A'),
        ("'春'", '春'),
        ("'🔥'", '🔥'),
        (r"'\n'", '\n'),
        ("'\\''", '\''),
    ]
    {
        let scan = scan_char(text, 0, Pos::ZERO);
        assert_eq!((scan.value, scan.diags.len()), (ch, 0), "{text}");
    }
    for text in ["''", "'ab'", "'ကာ'"]
    {
        let scan = scan_char(text, 0, Pos::ZERO);
        assert_eq!(scan.value, '\u{FFFD}', "{text}");
        assert_eq!(scan.diags[0].code, Code::BadCharLength, "{text}");
    }
}

#[test]
fn raw_identifiers()
{
    let scan = scan_raw_ident("`ကို ပြသပါ + 1` x", 0, Pos::ZERO);
    assert_eq!((scan.end, scan.diags.len()), ("`ကို ပြသပါ + 1`".len(), 0));
    assert_eq!(
        scan_raw_ident("``", 0, Pos::ZERO).diags[0].code,
        Code::BlankRawIdentifier
    );
    let scan = scan_raw_ident("`abc\n`", 0, Pos::ZERO);
    assert_eq!((scan.end, scan.diags[0].code), (4, Code::UnterminatedRawIdentifier));
}

#[test]
fn literals_in_the_token_stream()
{
    let kinds = kinds("\"၁၈ နှစ်အောက် ဝင်ခွင့်မပြုပါ\" ဟု ပြသပါ။");
    assert_eq!(kinds[0], TokenKind::Str(0));
    assert_eq!(kinds.len(), 4);
    assert_eq!(super_kind("`a b`"), TokenKind::RawIdent);
    assert_eq!(super_kind("'က'"), TokenKind::Char('က'));
}

/// Kind of the first token of `text`.
fn super_kind(text: &str) -> TokenKind { kinds(text)[0] }

#[test]
fn lexing_resumes_after_unterminated_string()
{
    let (out, _) = lex_text("\"abc\nအသက် ကို ပြသပါ။");
    assert_eq!(out.diagnostics.len(), 1);
    assert_eq!(out.tokens.len(), 6);
    assert_eq!(codes("'ab' ကို ပြသပါ။"), [Code::BadCharLength]);
}

#[test]
fn quotes_inside_strings_do_not_open_literals()
{
    let kinds = kinds(r#""it's `x`" ဟု ပြသပါ။"#);
    assert_eq!(kinds.len(), 4);
}
