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

#[test]
fn parser_constructors_render_messages()
{
    use zakac_diagnostics::parser as p;
    use zakac_span::span::Span;

    let span = Span::PLACEHOLDER;
    let found = p::expected_token(span, "\"။\"", "\"ကို\"");
    assert!(found.message.contains("\"။\""));
    assert!(found.message.contains("\"ကို\""));
    let hinted = p::with_split_hint(found, "အသက်", "ကို");
    assert!(hinted.help.unwrap().contains("အသက် ကို"));
    assert!(p::expected_stmt_verb(span, "x").help.is_some());
    assert!(p::chained_comparison(span).help.is_some());
    assert!(p::missing_operand(span, "v", 2, 1).message.contains('2'));
    assert!(p::slot_mismatch(span, "v", 3, "ကို", "-").message.contains('3'));
    assert!(p::invalid_digit(span, 'G', 16).message.contains("16"));
    for diag in [
        p::expected_operand(span, "x"),
        p::stmt_verb_in_expr(span, "v"),
        p::double_particle(span, "ကို"),
        p::unclosed_clause(span, "x"),
        p::dangling_operands(span, "v", "x"),
        p::else_without_if(span, "w"),
        p::update_outside_loop(span),
        p::update_not_last(span),
        p::nested_function(span),
        p::nested_single_line_if(span),
        p::invalid_bit_count(span, "7", "8"),
        p::signed_float(span),
        p::invalid_base(span, "40"),
        p::literal_out_of_range(span, "300"),
        p::float_for_int_kind(span),
        p::bad_literal_kind(span),
        p::str_outside_print(span),
        p::assign_target(span),
        p::const_loop(span),
        p::unmatched_block_close(span),
    ]
    {
        assert!(!is_missing(&diag.message), "{}", diag.code);
        assert!(!diag.message.contains("%{"), "{}", diag.code);
        assert!(diag.is_error());
    }
}
