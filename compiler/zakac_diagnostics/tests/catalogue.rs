use zakac_diagnostics::Code;

/// Whether a catalogue lookup came back as its own name (a missing text).
fn is_missing(text: &str) -> bool { text.starts_with("codes.") || text.is_empty() }

#[test]
fn all_codes_have_title_and_explanation()
{
    for code in Code::ALL
    {
        let title = code.title();
        let explain = code.explain();
        assert!(!is_missing(&title), "{} has no title", code.id());
        assert!(!is_missing(&explain), "{} has no explanation", code.id());
        assert!(!title.contains("%{"), "{} title has a placeholder", code.id());
        assert!(explain.len() > title.len(), "{} explanation is too short", code.id());
    }
}

#[test]
fn constructors_render_messages()
{
    use zakac_diagnostics::lexer;
    use zakac_span::span::Span;

    let span = Span::PLACEHOLDER;
    let diag = lexer::invalid_character(span, ",");
    assert_eq!(diag.code, Code::InvalidCharacter);
    assert!(diag.message.contains("\",\""));
    assert!(diag.help.is_some());
    assert!(lexer::invalid_character(span, "?").help.is_none());

    assert!(lexer::bad_char_length(span, 3).message.contains('3'));
    assert!(lexer::unknown_escape(span, "{x}").help.is_some());
    assert!(lexer::unknown_escape(span, "\\q").help.is_none());
    for diag in [
        lexer::invalid_number(span, "x"),
        lexer::mixed_digit_scripts(span),
        lexer::unterminated_string(span),
        lexer::unterminated_char(span),
        lexer::unterminated_raw_identifier(span),
        lexer::blank_raw_identifier(span),
    ]
    {
        assert!(!is_missing(&diag.message), "{}", diag.code);
        assert!(diag.is_error());
    }
}
