mod common;

use std::io::Cursor;
use std::path::PathBuf;

use common::codes;
use common::kinds;
use common::kws;
use common::lex_text;
use zakac_diagnostics::Code;
use zakac_lexer::artifact::LexArtifact;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::token::CommentKind;
use zakac_lexer::token::Kw;
use zakac_lexer::token::Punct;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_for;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::PhaseArtifact;
use zakac_session::config::Input;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// Inputs that must lex without diagnostics.
const CLEAN_INPUTS: &[&str] =
    &["comments", "compounds", "functions", "literals", "operators", "statements"];

/// Compare the text artifact of `tests/inputs/{name}.zaka` with
/// `tests/expected/{name}.zktokt`, and check that the expected file decodes
/// back.
///
/// With `ZAKA_BLESS=1` the expected file is rewritten instead.
fn check_expected(name: &str) -> usize
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(root.join(format!("tests/inputs/{name}.zaka")))
        .unwrap();
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom(format!("{name}.zaka")), text);
    let artifact = LexArtifact::from_source(&file, &LexerOptions::default());
    let encoded =
        String::from_utf8(artifact.encode(ArtifactFormat::Text).unwrap()).unwrap();

    let expected_path = root.join(format!("tests/expected/{name}.zktokt"));
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::create_dir_all(expected_path.parent().unwrap()).unwrap();
        std::fs::write(&expected_path, &encoded).unwrap();
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap();
    assert_eq!(encoded, expected, "output differs from the expected file for {name}");

    let input = Input::Data { name: name.to_string(), content: expected.into_bytes() };
    let mut stdin = Cursor::new(Vec::<u8>::new());
    let decoded = LexArtifact::decode(
        ArtifactFormat::Text,
        &input,
        &mut stdin,
        &mut SourceMap::new(),
    );
    assert_eq!(
        decoded.unwrap(),
        artifact,
        "expected file does not decode back for {name}"
    );
    artifact.diagnostics.len()
}

#[test]
fn clean_inputs_match_expected()
{
    do_for!(n in CLEAN_INPUTS, assert_eq!(check_expected(n), 0, "{n} has diagnostics"));
}

#[test]
fn invalid_input_matches_expected()
{
    assert!(check_expected("invalid") > 0);
}

#[test]
fn blank_input_has_eof_alone()
{
    let (out, _) = lex_text("");
    assert_eq!(out.tokens.len(), 1);
    assert_eq!(out.tokens[0].kind, TokenKind::Eof);
}

#[test]
fn comments_go_to_the_side_table()
{
    let (out, _) = lex_text("// a\n/// b\nမှတ်ချက် c\nx");
    let comment_kinds: Vec<CommentKind> = out.comments.iter().map(|c| c.kind).collect();
    assert_eq!(
        comment_kinds,
        [CommentKind::Slash, CommentKind::DocSlash, CommentKind::Note]
    );
    assert_eq!(kinds("// a\nx"), [TokenKind::Ident]);
}

#[test]
fn spaced_slashes_are_operators()
{
    assert_eq!(
        kinds("/ /"),
        [TokenKind::Punct(Punct::Slash), TokenKind::Punct(Punct::Slash)]
    );
}

#[test]
fn comment_marker_inside_a_word_is_not_a_comment()
{
    assert_eq!(kws("မှတ်ချက်ကို"), [Kw::Note, Kw::Ko]);
    assert_eq!(kinds("မှတ်ချက်များ"), [TokenKind::Ident]);
}

#[test]
fn brackets_and_colon_are_punctuation()
{
    assert_eq!(
        kinds("[က:]"),
        [
            TokenKind::Punct(Punct::LBracket),
            TokenKind::Ident,
            TokenKind::Punct(Punct::Colon),
            TokenKind::Punct(Punct::RBracket),
        ]
    );
}

#[test]
fn two_character_operators()
{
    let cases = [
        ("<=", Punct::Le),
        (">=", Punct::Ge),
        ("==", Punct::EqEq),
        ("!=", Punct::Ne),
        ("<<", Punct::Shl),
        (">>", Punct::Shr),
        ("&&", Punct::AndAnd),
        ("||", Punct::OrOr),
    ];
    for (text, punct) in cases
    {
        assert_eq!(kinds(text), [TokenKind::Punct(punct)], "{text}");
    }
    assert_eq!(kinds("< ="), [TokenKind::Punct(Punct::Lt)]);
    assert_eq!(codes("< ="), [Code::InvalidCharacter]);
}

#[test]
fn burmese_punctuation()
{
    let expected = [
        TokenKind::Ident,
        TokenKind::Punct(Punct::Ei),
        TokenKind::Ident,
        TokenKind::Punct(Punct::ListSep),
        TokenKind::Ident,
        TokenKind::Punct(Punct::FullStop),
    ];
    assert_eq!(kinds("က၏ခ၊ဂ။"), expected);
}

#[test]
fn joined_flags()
{
    let (out, _) = lex_text("\"abc\"ကို x");
    let joined: Vec<bool> = out.tokens.iter().map(|t| t.joined).collect();
    assert_eq!(joined, [false, true, false, false]);
}

#[test]
fn comma_gets_a_hint()
{
    let (out, _) = lex_text("က, ခ");
    assert_eq!(out.diagnostics[0].code, Code::InvalidCharacter);
    assert!(out.diagnostics[0].help.is_some());
}

#[test]
fn invalid_words_are_skipped()
{
    assert_eq!(kinds("က ; ခ"), [TokenKind::Ident, TokenKind::Ident]);
    assert_eq!(codes("က ; ခ ?"), [Code::InvalidCharacter, Code::InvalidCharacter]);
}

#[test]
fn identifiers_of_several_scripts()
{
    for text in ["အသက်", "age", "_x1", "春天", "🔥", "၎င်း"]
    {
        assert_eq!(kinds(text), [TokenKind::Ident], "{text}");
    }
}

#[test]
fn a_word_in_two_scripts_is_one_identifier()
{
    for text in ["ဖြစ်ရပ်_၁", "က_ခ", "abcက", "x_ကို", "အသက်2", "🔥က", "ရလဒ်_x"]
    {
        assert_eq!(kinds(text), [TokenKind::Ident], "{text}");
        assert_eq!(common::lexemes(text), [text], "{text}");
    }
}

#[test]
fn a_two_script_identifier_keeps_its_neighbours()
{
    let text = "ဖြစ်ရပ်_၁ သို့မဟုတ် ဖြစ်ရပ်_၂၏ ပေါင်းလဒ်";
    let want = ["ဖြစ်ရပ်_၁", "သို့မဟုတ်", "ဖြစ်ရပ်_၂", "၏", "ပေါင်းလဒ်"];
    assert_eq!(common::lexemes(text), want);
    assert!(codes(text).is_empty());
}

#[test]
fn a_run_that_is_no_identifier_is_lexed_per_word()
{
    let text = "က«ခ";
    assert_eq!(codes(text), [Code::InvalidCharacter]);
    assert_eq!(common::lexemes(text), ["က", "ခ"]);
}
