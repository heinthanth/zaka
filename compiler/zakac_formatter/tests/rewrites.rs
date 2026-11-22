//! The rewrites: the speech form of reserved words, the digits of numbers and
//! the order of the imports.

use zakac_formatter::FmtOptions;
use zakac_formatter::format;
use zakac_formatter::options::Digits;
use zakac_formatter::options::ImportOrder;
use zakac_formatter::options::SpeechMode;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::parse_in_place;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

use crate::common::fmt_with;
use crate::common::repo;

/// The settings with one speech form.
///
/// # Arguments
///
/// * `mode` - The form.
///
/// # Returns
///
/// The settings.
fn speech(mode: SpeechMode) -> FmtOptions
{
    FmtOptions::builder().speech_mode(mode).build()
}

/// The text of a sample.
///
/// # Arguments
///
/// * `name` - The path inside `samples/`.
///
/// # Returns
///
/// The text.
fn sample(name: &str) -> String
{
    std::fs::read_to_string(repo().join("samples").join(name)).unwrap()
}

/// The spoken program of the Docs page on spoken words.
///
/// # Returns
///
/// The program.
fn spoken_program() -> String
{
    let page =
        std::fs::read_to_string(repo().join("docs/manual/language/spoken.md")).unwrap();
    let (_, rest) = page.split_once("```zaka lint-allow=spoken\n").unwrap();
    rest.split_once("```").unwrap().0.to_string()
}

/// The written form turns each spoken word into its written word, and
/// leaves the words of strings unchanged; the spoken form turns them back.
#[test]
fn speech_forms_turn_back_and_forth()
{
    let spoken = spoken_program();
    let written = fmt_with(&spoken, &speech(SpeechMode::Written));
    assert!(written.contains("လုပ်ဆောင်မည်။"), "{written}");
    assert!(!written.contains("ပြသမယ်"), "{written}");
    assert!(written.contains("\"အောင်တယ်\""), "{written}");
    assert_eq!(fmt_with(&written, &speech(SpeechMode::Spoken)), spoken);
}

/// A spoken word that stands for two written words comes back as the one the
/// parser read: `ရှိသည့်` in a member with a value.
#[test]
fn a_spoken_word_comes_back_as_the_word_the_parser_read()
{
    let text = sample("basics/enums.zaka");
    let spoken = fmt_with(&text, &speech(SpeechMode::Spoken));
    assert_ne!(spoken, text);
    assert!(spoken.contains("ရှိတဲ့"), "{spoken}");
    assert_eq!(fmt_with(&spoken, &speech(SpeechMode::Written)), text);
}

/// The digits of number literals take one script, and turn back.
#[test]
fn digits_take_one_script()
{
    let text = sample("basics/hello.zaka");
    let latin = fmt_with(&text, &FmtOptions::builder().digits(Digits::Latin).build());
    assert!(latin.contains("2026 ") && latin.contains("6 * 7"), "{latin}");
    let burmese = FmtOptions::builder().digits(Digits::Burmese).build();
    assert_eq!(fmt_with(&latin, &burmese), text);
}

/// Sorted imports put the stdlib first, and the file parses with the same
/// diagnostics after the sort.
#[test]
fn sorted_imports_put_the_stdlib_first()
{
    let dir = tempfile::tempdir().unwrap();
    let lib = dir.path().join("lib");
    std::fs::create_dir(&lib).unwrap();
    std::fs::copy(repo().join("samples/modules/lib/ops.zaka"), lib.join("ops.zaka"))
        .unwrap();
    let std_line =
        sample("programs/tic_tac_toe.zaka").lines().nth(3).unwrap().to_string();
    let main = sample("modules/fn_table.zaka");
    let (head, rest) = main.split_once('\n').unwrap();
    let (local, tail) = rest.split_once('\n').unwrap();
    let text = format!("{head}\n{local}\n{std_line}\n{tail}");
    let path = dir.path().join("main.zaka");
    let sorted = FmtOptions::builder().import_order(ImportOrder::Sorted).build();
    let out =
        format(&mut SourceMap::new(), FileName::real(path.clone()), &text, &sorted)
            .unwrap();
    assert_eq!(out, format!("{head}\n{std_line}\n{local}\n{tail}"));
    let codes = |text: &str| {
        let mut map = SourceMap::new();
        let file = map.new_source_file(FileName::real(path.clone()), text.to_string());
        let found = parse_in_place(&file, &mut map, &LexerOptions::default()).unwrap();
        found.diagnostics.iter().map(|d| d.code).collect::<Vec<_>>()
    };
    assert_eq!(codes(&out), codes(&text));
}
