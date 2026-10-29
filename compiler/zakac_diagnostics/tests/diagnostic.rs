use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Level;
use zakac_diagnostics::has_errors;
use zakac_span::pos::Pos;
use zakac_span::span::Span;

#[test]
fn level_comes_from_the_code()
{
    let diag = Diagnostic::new(Code::InvalidNumber, "m", Span::PLACEHOLDER);
    assert_eq!(diag.level, Level::Error);
    assert!(diag.is_error());
    assert!(diag.help.is_none());
    let note = diag.clone().with_level(Level::Note);
    assert!(!note.is_error());
    assert_eq!(note.code, Code::InvalidNumber);
}

#[test]
fn with_help_and_with_span()
{
    let diag =
        Diagnostic::new(Code::InvalidNumber, "m", Span::PLACEHOLDER).with_help("h");
    assert_eq!(diag.help.as_deref(), Some("h"));
    let span = Span::new(Pos::new(3), Pos::new(5));
    let moved = diag.with_span(span);
    assert_eq!(moved.span, span);
    assert_eq!(moved.help.as_deref(), Some("h"));
}

#[test]
fn error_detection()
{
    let warn = Diagnostic::new(Code::InvalidNumber, "m", Span::PLACEHOLDER)
        .with_level(Level::Warning);
    let err = Diagnostic::new(Code::InvalidNumber, "m", Span::PLACEHOLDER);
    assert!(!has_errors(std::slice::from_ref(&warn)));
    assert!(has_errors(&[warn, err]));
}

#[test]
fn serde_round_trip()
{
    let diag = Diagnostic::new(
        Code::UnterminatedString,
        "m",
        Span::new(Pos::new(1), Pos::new(4)),
    )
    .with_help("h");
    let text = serde_saphyr::to_string(&diag).unwrap();
    assert!(text.contains("code: ZK0104"));
    assert!(text.contains("level: error"));
    assert!(!text.contains("labels"));
    let back: Diagnostic = serde_saphyr::from_str(&text).unwrap();
    assert_eq!(back, diag);
}

#[test]
fn with_label_adds_a_label()
{
    let at = Span::new(Pos::new(3), Pos::new(5));
    let diag = Diagnostic::new(Code::InvalidNumber, "m", Span::PLACEHOLDER)
        .with_label(at, "here");
    assert_eq!(diag.labels.len(), 1);
    assert_eq!(diag.labels[0].span, at);
    assert_eq!(diag.labels[0].text, "here");
    let text = serde_saphyr::to_string(&diag).unwrap();
    let back: Diagnostic = serde_saphyr::from_str(&text).unwrap();
    assert_eq!(back, diag);
}

#[test]
fn raw_round_trip_keeps_labels()
{
    let at = Span::new(Pos::new(3), Pos::new(5));
    let plain = Diagnostic::new(Code::InvalidNumber, "m", at).with_help("h");
    let labelled = plain.clone().with_label(at, "here");
    for diag in [plain, labelled]
    {
        let raw = postcard::to_allocvec(&diag).unwrap();
        let back: Diagnostic = postcard::from_bytes(&raw).unwrap();
        assert_eq!(back, diag);
    }
}

#[test]
fn map_spans_moves_the_span_and_each_label()
{
    let at = Span::new(Pos::new(3), Pos::new(5));
    let mut diag = Diagnostic::new(Code::InvalidNumber, "m", at)
        .with_label(at, "a")
        .with_label(at, "b");
    diag.map_spans(&|s: Span| Span::new(s.lo + 10, s.hi + 10));
    let moved = Span::new(Pos::new(13), Pos::new(15));
    assert_eq!(diag.span, moved);
    assert!(diag.labels.iter().all(|l| l.span == moved));
}
