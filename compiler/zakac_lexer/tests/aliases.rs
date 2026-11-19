//! The alias table: the spellings it gives, and the checks that catch a slip
//! in a row of it.

use std::path::PathBuf;

use zakac_lexer::aliases::ALIASES;
use zakac_lexer::aliases::WRITTEN;
use zakac_lexer::reserved::RESERVED;
use zakac_lexer::token::Kw;
use zakac_rust_sugars::do_for;

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
        let named = row.within.iter().chain(row.except);
        do_for!(w in named, assert!(reserved(w), "{w} in the row of {:?}", row.parts));
    }
}

#[test]
fn each_part_of_a_row_stands_in_a_reserved_word()
{
    for row in ALIASES
    {
        let found = |p: &&str| WRITTEN.iter().any(|w| w.text.contains(*p));
        let first = row.parts.first().copied().unwrap_or_default();
        assert!(found(&first), "{first} of the row of {:?}", row.parts);
    }
}
