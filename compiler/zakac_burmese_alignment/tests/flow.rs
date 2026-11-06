use zakac_burmese_alignment::flow::Drawn;
use zakac_burmese_alignment::flow::Grid;
use zakac_burmese_alignment::flow::Part;
use zakac_burmese_alignment::flow::Reach;
use zakac_burmese_alignment::flow::Shaper;
use zakac_burmese_alignment::flow::is_burmese;
use zakac_burmese_alignment::flow::reach;
use zakac_burmese_alignment::flow::starts_unit;

/// A shaper with fixed advances, for a 10 pixel cell: 13 pixels for a
/// Burmese character that takes a cell, 0 for a zero-width sign, 4 for a
/// space (the placement widens it to a cell) and 10 for other text.
struct Fixed;

impl Shaper for Fixed
{
    fn shape(&self, text: &str, _bold: bool) -> Vec<Drawn>
    {
        let advance = |ch: char| match ch
        {
            ' ' => 4.0,
            '\u{102B}'..='\u{103E}'
                if !matches!(
                    ch,
                    '\u{102B}'
                        | '\u{102C}'
                        | '\u{1031}'
                        | '\u{1038}'
                        | '\u{103B}'
                        | '\u{103C}'
                ) =>
            {
                0.0
            }
            _ if is_burmese(ch) => 13.0,
            _ => 10.0,
        };
        text.char_indices()
            .map(|(at, ch)| Drawn::builder().at(at).advance(advance(ch)).build())
            .collect()
    }
}

/// A grid of 10 pixel cells.
fn grid(cols: usize) -> Grid { Grid::builder().cell(10.0).cols(cols).build() }

/// The parts of a row: a plain gutter, the text before, the bold span, the
/// text after.
fn parts<'a>(
    gutter: &'a str, before: &'a str, marked: &'a str, after: &'a str,
) -> [Part<'a>; 4]
{
    [
        Part::builder().text(gutter).build(),
        Part::builder().text(before).build(),
        Part::builder().text(marked).bold(true).build(),
        Part::builder().text(after).build(),
    ]
}

/// A reach.
fn at(start: usize, width: usize) -> Option<Reach>
{
    Some(Reach::builder().start(start).width(width).build())
}

#[test]
fn plain_text_keeps_to_the_grid()
{
    let row = parts("1 | ", "ab ", "cd", " e");
    assert_eq!(reach(&Fixed, grid(80), &row, 2), at(7, 2));
}

#[test]
fn wide_burmese_text_pushes_the_span_right()
{
    // the two units end at 26 px and the widened space at 36, after the
    // 40 px gutter: the span starts at 76 px, not at its grid column 7
    let row = parts("1 | ", "ကက ", "ခ", "");
    assert_eq!(reach(&Fixed, grid(80), &row, 2), at(8, 1));
}

#[test]
fn a_word_starts_at_its_column_when_there_is_room()
{
    // "a" ends at 50 px, before the grid column of the span
    let row = parts("1 | ", "a    ", "ခ", "");
    assert_eq!(reach(&Fixed, grid(80), &row, 2), at(9, 1));
}

#[test]
fn a_row_past_the_edge_is_squeezed()
{
    // unsqueezed, the span runs from 104 px to 117 px in a 100 px window;
    // squeezed from 0, it runs from 87 px to 100 px
    let row = parts("", "ကကကကကကကက", "ခ", "");
    assert_eq!(reach(&Fixed, grid(10), &row, 2), at(9, 1));
}

#[test]
fn a_row_wider_than_the_window_wraps()
{
    let row = parts("1 | ", "abcdef", "g", "");
    assert_eq!(reach(&Fixed, grid(8), &row, 2), None);
}

#[test]
fn a_blank_span_starts_after_the_text_before_it()
{
    // the text before ends at 66 px, past the grid column 6
    let row = parts("1 | ", "ကက", "", "");
    assert_eq!(reach(&Fixed, grid(80), &row, 2), at(7, 1));
}

/// The indexes of `text` where a unit starts.
fn unit_starts(text: &str) -> Vec<usize>
{
    let chars: Vec<char> = text.chars().collect();
    (0..chars.len()).filter(|idx| starts_unit(&chars, *idx)).collect()
}

#[test]
fn units_hold_signs_stacks_and_killed_consonants()
{
    // a consonant with its medial and vowel signs, closed with a killed one
    assert_eq!(unit_starts("ကြောင်"), vec![0]);
    // one unit: a consonant, closed with a killed consonant and a visarga
    assert_eq!(unit_starts("မင်း"), vec![0]);
    assert_eq!(unit_starts("ကခ"), vec![0, 1]);
    // a stacked consonant joins the unit before the virama
    assert_eq!(unit_starts("ကမ္ဘာ"), vec![0, 1]);
    // kinzi starts a unit with the consonant after it
    assert_eq!(unit_starts("သင်္ချိုင်း"), vec![0, 1]);
    // a space and text after it start units of their own
    assert_eq!(unit_starts("က a"), vec![0, 1, 2]);
}
