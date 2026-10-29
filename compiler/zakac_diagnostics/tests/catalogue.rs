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
        p::const_loop(span),
        p::unmatched_block_close(span),
        p::bad_list_count(span, "၀"),
    ]
    {
        assert!(!is_missing(&diag.message), "{}", diag.code);
        assert!(!diag.message.contains("%{"), "{}", diag.code);
        assert!(diag.is_error());
    }
}

#[test]
fn resolve_constructors_render_messages()
{
    use zakac_diagnostics::resolve as r;
    use zakac_span::span::Span;

    let span = Span::PLACEHOLDER;
    assert!(r::redeclaration(span, "က").help.is_some());
    assert!(r::missing_return_value(span, "f", "i8").message.contains("i8"));
    assert!(r::arg_count_mismatch(span, "f", 2, 1).message.contains('2'));
    for diag in [
        r::undeclared_name(span, "က"),
        r::redeclaration(span, "က"),
        r::undeclared_function(span, "f"),
        r::duplicate_function(span, "f"),
        r::duplicate_param(span, "က"),
        r::assign_to_const(span, "က"),
        r::assign_to_loop_var(span, "က"),
        r::const_without_init(span, "က"),
        r::break_outside_loop(span),
        r::continue_outside_loop(span),
        r::return_outside_function(span),
        r::value_returned_from_void(span, "f"),
        r::missing_return_value(span, "f", "i8"),
        r::arg_count_mismatch(span, "f", 2, 1),
        r::value_from_void_function(span, "f"),
    ]
    {
        assert!(!is_missing(&diag.message), "{}", diag.code);
        assert!(!diag.message.contains("%{"), "{}", diag.code);
        assert!(diag.is_error());
        assert!(diag.code.id().starts_with("ZK03"), "{}", diag.code);
    }
}

#[test]
fn kinds_constructors_render_messages()
{
    use zakac_diagnostics::kinds as k;
    use zakac_span::span::Span;

    let span = Span::PLACEHOLDER;
    assert!(k::kind_mismatch(span, "i8", "i16").help.is_some());
    let unbridged = k::kind_mismatch_unbridged(span, "[2]isize", "isize");
    assert!(unbridged.help.is_none() && unbridged.message.contains("[2]isize"));
    let pun = k::address_block_mismatch(span, "&**isize", "&ptr", "~*isize");
    assert!(pun.help.is_some_and(|h| h.contains("~*isize") && h.contains("&**isize")));
    assert!(k::shift_too_far(span, "9", 8).message.contains('8'));
    for diag in [
        k::kind_mismatch(span, "i8", "i16"),
        k::condition_not_bool(span, "i8"),
        k::operand_kind(span, "+", "a number", "bool"),
        k::incompatible_operands(span, "i8", "u8"),
        k::loop_bound_not_int(span, "f64"),
        k::literal_does_not_fit(span, "300", "u8"),
        k::shift_too_far(span, "9", 8),
        k::invalid_cast(span, "bool", "f64"),
        k::step_not_positive(span),
    ]
    {
        assert!(!is_missing(&diag.message), "{}", diag.code);
        assert!(!diag.message.contains("%{"), "{}", diag.code);
        assert!(diag.is_error());
        assert!(diag.code.id().starts_with("ZK04"), "{}", diag.code);
    }
}

#[test]
fn fold_constructors_render_messages()
{
    use zakac_diagnostics::fold as f;
    use zakac_span::span::Span;

    let span = Span::PLACEHOLDER;
    assert!(f::constant_overflow(span, "+", "i8").message.contains("i8"));
    assert!(f::constant_zero_divisor(span, "/").message.contains('/'));
    for diag in
        [f::constant_overflow(span, "+", "i8"), f::constant_zero_divisor(span, "%")]
    {
        assert!(!is_missing(&diag.message), "{}", diag.code);
        assert!(!diag.message.contains("%{"), "{}", diag.code);
        assert!(diag.is_error());
        assert!(diag.code.id().starts_with("ZK06"), "{}", diag.code);
    }
}

#[test]
fn pointer_constructors_render_messages()
{
    use zakac_diagnostics::kinds as k;
    use zakac_span::span::Span;

    let span = Span::PLACEHOLDER;
    assert_eq!(k::not_a_place(span).code, Code::NotAPlace);
    assert!(k::no_address(span, "a literal").message.contains("a literal"));
    assert!(k::fixed_address(span, "က").message.contains("က"));
    assert!(k::not_a_pointer(span, "isize").message.contains("isize"));
    assert!(k::opaque_pointer(span).help.is_some());
    assert!(k::pointer_operator(span, "*").message.contains("\"*\""));
    assert!(k::pointer_kinds_differ(span, "*i8", "*isize").message.contains("*i8"));
    assert!(k::heap_needs_pointer(span, "bool").message.contains("bool"));
    assert!(k::condition_not_bool(span, "*isize").help.is_some());
    assert!(k::condition_not_bool(span, "isize").help.is_none());
}

#[test]
fn list_constructors_render_messages()
{
    use zakac_diagnostics::fold as f;
    use zakac_diagnostics::kinds as k;
    use zakac_span::span::Span;

    let span = Span::PLACEHOLDER;
    assert!(k::list_count_mismatch(span, 3, 2).message.contains('3'));
    assert!(k::blank_list_kind(span).help.is_some());
    assert!(k::list_count_missing(span).help.is_some());
    assert!(k::not_a_list(span, "isize").message.contains("isize"));
    assert!(k::cell_number_not_int(span, "f64").message.contains("f64"));
    assert!(k::list_result(span).help.is_some());
    assert!(k::list_too_large(span, "[9]u8").message.contains("[9]u8"));
    let out = f::cell_out_of_range(span, "3", 0, 2);
    assert_eq!(out.code, Code::CellOutOfRange);
    assert!(out.message.contains('3') && out.message.contains('2'));
}

#[test]
fn ownership_constructors_render_messages()
{
    use zakac_diagnostics::kinds as k;
    use zakac_diagnostics::ownership as o;
    use zakac_diagnostics::ownership::Invalidation;
    use zakac_span::span::Span;

    let span = Span::PLACEHOLDER;
    let moved = o::moved_out(span, "ပ");
    assert_eq!(moved.code, Code::MovedOut);
    assert!(moved.message.contains("\"ပ\""));
    assert_eq!(o::freed_borrow(span).code, Code::FreedBorrow);
    assert_eq!(o::borrow_stored(span).code, Code::BorrowStored);
    let loan = o::invalidated_loan(span, "ပ", &Invalidation::Freed.text(), "မိတ္တူ", span);
    assert_eq!(loan.code, Code::InvalidatedLoan);
    assert!(loan.message.contains("freed"));
    assert!(loan.message.contains("\"မိတ္တူ\""));
    assert_eq!(loan.labels.len(), 1);
    assert!(loan.labels[0].text.contains("မိတ္တူ"));
    assert_eq!(Invalidation::Resized.text(), "resized");
    assert_eq!(Invalidation::Moved.text(), "moved");
    assert_eq!(o::borrow_of_borrow(span).code, Code::BorrowOfBorrow);
    let local = o::local_borrow_returned(span, "ပ");
    assert_eq!(local.code, Code::LocalBorrowReturned);
    assert!(local.message.contains("\"ပ\""));
    let alias = o::aliased_argument(span, "ရှင်း", 1, 2);
    assert_eq!(alias.code, Code::AliasedArgument);
    assert!(alias.message.contains("argument 2"));
    assert_eq!(o::block_leaked(span, "ပ").code, Code::BlockLeaked);
    assert_eq!(o::null_deref(span, "ပ").code, Code::NullDeref);
    let past = o::offset_past_end(span, 4, 4);
    assert_eq!(past.code, Code::OffsetPastEnd);
    assert!(past.message.contains("offset 4"));
    let possible = o::null_deref_possible(span, "ပ");
    assert_eq!(possible.code, Code::NullDerefPossible);
    assert!(!possible.is_error());
    assert_eq!(o::mixed_blocks(span).code, Code::MixedBlocks);
    assert_eq!(o::bad_borrow_marker(span).code, Code::BadBorrowMarker);
    assert_eq!(o::borrow_for_owner(span).code, Code::BorrowForOwner);
    let owner = o::owner_for_borrow(span, Some("ပ"));
    assert_eq!(owner.code, Code::OwnerForBorrow);
    assert!(owner.help.unwrap().contains("ပ ၏ တန်ဖိုးအငှား"));
    let nameless = o::owner_for_borrow(span, None);
    assert_eq!(nameless.code, Code::OwnerForBorrow);
    assert!(nameless.help.is_none());
    assert_eq!(o::lend_of_raw(span).code, Code::LendOfRaw);
    assert_eq!(o::raw_not_allowed(span).code, Code::RawNotAllowed);

    let lend = k::lend_of_value(span, "isize", "x");
    assert_eq!(lend.code, Code::NotAPointer);
    assert!(lend.help.unwrap().contains("x ၏ လိပ်စာ"));
}
