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
    let back: Diagnostic = serde_saphyr::from_str(&text).unwrap();
    assert_eq!(back, diag);
}
