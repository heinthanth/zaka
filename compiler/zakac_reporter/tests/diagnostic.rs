use termcolor::Buffer;
use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Level;
use zakac_reporter::render::Paint;
use zakac_reporter::render::diagnostic_pieces;
use zakac_reporter::render::format_diagnostic;
use zakac_reporter::render::write_diagnostic;
use zakac_reporter::render::write_diagnostics;
use zakac_reporter::width::char_cells;
use zakac_reporter::width::str_cells;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

/// Register `src` and build a diagnostic over the first `needle` in it.
fn setup(src: &str, needle: &str) -> (SourceMap, Diagnostic)
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), src.to_string());
    let lo = file.src.find(needle).expect("needle not in normalized source");
    let span = Span::new(file.start_pos + lo, file.start_pos + lo + needle.len());
    (sm, Diagnostic::new(Code::InvalidCharacter, "it is wrong", span))
}

#[test]
fn ascii_line()
{
    let (sm, diag) = setup("let a = 1\nlet b = ?\n", "?");
    let expected = "\
error[ZK0101]: it is wrong
 --> t.zaka:2:9
  |
2 | let b = ?
  |         ^
";
    assert_eq!(format_diagnostic(&sm, &diag), expected);
}

#[test]
fn burmese_line_uses_cells()
{
    let (sm, diag) = setup("ကိန်းရှင် ကျောင်း ။\n", "ကျောင်း");
    let rendered = format_diagnostic(&sm, &diag);
    let caret_row = rendered.lines().nth(4).unwrap();
    // 5 cells for the first word, 1 for the space, then 6 carets.
    assert_eq!(caret_row, "  |       ^^^^^^");
    assert!(rendered.contains(" --> t.zaka:1:11\n"));
}

#[test]
fn tabs_expand_in_line_and_caret_row()
{
    let (sm, diag) = setup("\tx\n", "x");
    let rendered = format_diagnostic(&sm, &diag);
    assert!(rendered.contains("1 |     x\n"));
    assert!(rendered.contains("  |     ^\n"));
}

#[test]
fn blank_span_at_line_end_gets_one_caret()
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), "\"abc\n".into());
    let end = file.start_pos + 4;
    let diag =
        Diagnostic::new(Code::UnterminatedString, "no close", Span::new(end, end));
    let rendered = format_diagnostic(&sm, &diag);
    assert!(rendered.contains("1 | \"abc\n"));
    assert!(rendered.contains("  |     ^\n"));
}

#[test]
fn multi_line_span_cut_at_first_line_end()
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), "ab\ncd\n".into());
    let span = Span::new(file.start_pos + 1, file.start_pos + 5);
    let diag = Diagnostic::new(Code::InvalidNumber, "m", span);
    let rendered = format_diagnostic(&sm, &diag);
    assert!(rendered.contains("1 | ab\n"));
    assert!(rendered.contains("  |  ^\n"));
}

#[test]
fn placeholder_span_has_no_excerpt()
{
    let sm = SourceMap::new();
    let diag = Diagnostic::new(Code::InvalidNumber, "careful", Span::PLACEHOLDER)
        .with_level(Level::Warning)
        .with_help("look");
    let expected = "warning[ZK0102]: careful\n --> <PLACEHOLDER>\n = help: look\n";
    assert_eq!(format_diagnostic(&sm, &diag), expected);
}

#[test]
fn help_line_after_caret_row()
{
    let (sm, diag) = setup("x\n", "x");
    let rendered = format_diagnostic(&sm, &diag.with_help("remove it"));
    assert!(rendered.ends_with("  | ^\n  = help: remove it\n"));
}

#[test]
fn note_level_tag()
{
    let (sm, diag) = setup("x\n", "x");
    let diag = diag.with_level(Level::Note);
    assert!(format_diagnostic(&sm, &diag).starts_with("note[ZK0101]: it is wrong\n"));
}

#[test]
fn color_output_marks_span()
{
    let (sm, diag) = setup("a b c\n", "b");
    let mut out = Buffer::ansi();
    write_diagnostic(&mut out, &sm, &diag).unwrap();
    let text = String::from_utf8(out.into_inner()).unwrap();
    // underline escape before the marked text
    assert!(text.contains("\u{1b}[4m"));
    assert!(text.contains("b\u{1b}[0m"));
}

#[test]
fn no_color_output_matches_format()
{
    let (sm, diag) = setup("a b c\n", "b");
    let mut out = Buffer::no_color();
    write_diagnostic(&mut out, &sm, &diag).unwrap();
    let text = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(text, format_diagnostic(&sm, &diag));
}

#[test]
fn diagnostics_separated_with_blank_line()
{
    let (sm, diag) = setup("x\n", "x");
    let mut out = Buffer::no_color();
    write_diagnostics(&mut out, &sm, &[diag.clone(), diag.clone()]).unwrap();
    let text = String::from_utf8(out.into_inner()).unwrap();
    let one = format_diagnostic(&sm, &diag);
    assert_eq!(text, format!("{one}\n{one}"));
}

#[test]
fn cell_counts()
{
    assert_eq!(char_cells('\t'), 4);
    assert_eq!(char_cells('a'), 1);
    assert_eq!(char_cells('က'), 1);
    assert_eq!(char_cells('\u{103A}'), 0); // asat
    assert_eq!(char_cells('\u{102D}'), 0); // vowel sign I
    assert_eq!(char_cells('\u{1031}'), 1); // vowel sign E
    assert_eq!(char_cells('春'), 2);
    assert_eq!(str_cells("ကိန်းရှင်"), 5);
    assert_eq!(str_cells("ကျောင်း"), 6);
}

#[test]
fn pieces_join_to_the_plain_text()
{
    let (sm, diag) = setup("ကိန်းရှင် ကျောင်း ။\n", "ကျောင်း");
    let pieces = diagnostic_pieces(&sm, &diag);
    let joined: String = pieces.iter().map(|piece| piece.text.as_str()).collect();
    assert_eq!(joined, format_diagnostic(&sm, &diag));
    assert_eq!(pieces[0].paint, Paint::Tag(Level::Error));
    assert!(pieces.iter().any(|piece| piece.paint == Paint::Marked(Level::Error)));
}
