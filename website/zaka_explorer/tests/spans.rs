use zaka_explorer::compile::spans::OffsetTable;
use zaka_explorer::compile::spans::Range;
use zaka_explorer::compile::spans::find_range;
use zaka_explorer::compile::spans::snap;
use zaka_explorer::compile::spans::to_original;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

#[test]
fn table_round_trips_ascii_burmese_and_emoji()
{
    // 'a' = 1 u8 / 1 unit, 'က' = 3 u8s / 1 unit, '🔥' = 4 u8s / 2 units.
    let text = "aက🔥b";
    let table = OffsetTable::new(text);
    let pairs = [(0u32, 0u32), (1, 1), (4, 2), (8, 4), (9, 5)];
    for (utf8, utf16) in pairs
    {
        assert_eq!(table.to_utf16(utf8), utf16, "utf8 {utf8}");
        assert_eq!(table.to_utf8(utf16), utf8, "utf16 {utf16}");
    }
}

#[test]
fn table_maps_inner_u8s_to_the_char_start()
{
    let table = OffsetTable::new("aကb");
    assert_eq!(table.to_utf16(2), 1);
    assert_eq!(table.to_utf16(3), 1);
    assert_eq!(table.to_utf16(99), 3);
    assert_eq!(table.to_utf8(99), 5);
}

#[test]
fn snap_moves_to_the_char_start()
{
    assert_eq!(snap("aကb", 2), 1);
    assert_eq!(snap("aကb", 4), 4);
    assert_eq!(snap("aကb", 40), 5);
}

#[test]
fn find_range_counts_the_end_as_inside()
{
    let ranges = [Range::new(0, 3), Range::new(3, 5), Range::new(7, 9)];
    assert_eq!(find_range(&ranges, 0), Some(0));
    assert_eq!(find_range(&ranges, 3), Some(1));
    assert_eq!(find_range(&ranges, 5), Some(1));
    assert_eq!(find_range(&ranges, 6), None);
    assert_eq!(find_range(&ranges, 9), Some(2));
    assert_eq!(find_range(&ranges, 10), None);
    assert_eq!(find_range(&[], 0), None);
}

#[test]
fn to_original_is_identit_on_normalized_text()
{
    let text = "ကိန်းရှင် က";
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t".into()), text.into());
    let span = Span::new(file.start_pos + 3, file.start_pos + 9);
    assert_eq!(to_original(&file, text, &span), Range::new(3, 9));
}

#[test]
fn to_original_follows_normalization_shrinks()
{
    // CRLF becomes LF and each two-char ဦ becomes one char; see the span
    // crate test `maps_positions_through_shrinks`.
    let text = "a\r\n\u{1025}\u{102e}\u{1025}\u{102e}b";
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t".into()), text.into());
    assert_eq!(file.src, "a\n\u{1026}\u{1026}b");
    let last = Span::new(file.start_pos + 8, file.start_pos + 9);
    assert_eq!(to_original(&file, text, &last), Range::new(15, 16));
    assert_eq!(to_original(&file, text, &Span::PLACEHOLDER), Range::default());
}
