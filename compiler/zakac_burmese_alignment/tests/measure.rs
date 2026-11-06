use zakac_burmese_alignment::measure::answer_column;
use zakac_burmese_alignment::measure::drawing_for;
use zakac_burmese_alignment::terminal::Drawing;

#[test]
fn the_column_comes_from_the_last_answer()
{
    assert_eq!(answer_column(b"\x1b[12;8R"), Some(8));
    assert_eq!(answer_column(b"x\x1b[1;1R\x1b[3;15R"), Some(15));
    assert_eq!(answer_column(b"\x1b[12;R"), None);
    assert_eq!(answer_column(b"\x1b[12;8"), None);
    assert_eq!(answer_column(b""), None);
}

#[test]
fn the_test_word_tells_the_count()
{
    assert!(matches!(drawing_for(4), Some(Drawing::Codepoint)));
    assert!(matches!(drawing_for(3), Some(Drawing::Cluster)));
    assert!(drawing_for(5).is_none());
    assert!(drawing_for(-2).is_none());
}
