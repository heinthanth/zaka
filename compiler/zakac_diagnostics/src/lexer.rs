use zakac_i18n::t;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::code::Code;
use crate::diagnostic::Diagnostic;

/// A word holds a character that cannot start or continue a token.
///
/// # Arguments
///
/// * `span` - The word.
/// * `word` - The word text.
///
/// # Returns
///
/// The `ZK0101` diagnostic; a lone `,` gets a hint about `၊`.
pub fn invalid_character(span: Span, word: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0101.message", w = word);
    let diag = Diagnostic::new(Code::InvalidCharacter, msg, span);
    tern!(word == ",", diag.with_help(t!("codes.ZK0101.help_comma")), diag)
}

/// A word starts with a digit but is not a number.
///
/// # Arguments
///
/// * `span` - The word.
/// * `word` - The word text.
///
/// # Returns
///
/// The `ZK0102` diagnostic.
pub fn invalid_number(span: Span, word: &str) -> Diagnostic
{
    Diagnostic::new(Code::InvalidNumber, t!("codes.ZK0102.message", w = word), span)
}

/// A number mixes Burmese and Western digits.
///
/// # Arguments
///
/// * `span` - The number.
///
/// # Returns
///
/// The `ZK0103` diagnostic.
pub fn mixed_digit_scripts(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::MixedDigitScripts, t!("codes.ZK0103.message"), span)
}

/// A string literal reaches the end of the line.
///
/// # Arguments
///
/// * `span` - From the opening quote to the end of the line.
///
/// # Returns
///
/// The `ZK0104` diagnostic.
pub fn unterminated_string(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::UnterminatedString, t!("codes.ZK0104.message"), span)
}

/// A character literal reaches the end of the line.
///
/// # Arguments
///
/// * `span` - From the opening quote to the end of the line.
///
/// # Returns
///
/// The `ZK0105` diagnostic.
pub fn unterminated_char(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::UnterminatedChar, t!("codes.ZK0105.message"), span)
}

/// An escape sequence is not known.
///
/// # Arguments
///
/// * `span` - The escape sequence.
/// * `raw` - The escape sequence text.
///
/// # Returns
///
/// The `ZK0106` diagnostic, with a hint for brace escapes.
pub fn unknown_escape(span: Span, raw: &str) -> Diagnostic
{
    let diag =
        Diagnostic::new(Code::UnknownEscape, t!("codes.ZK0106.message", e = raw), span);
    let brace = raw.starts_with(['{', '}']);
    tern!(brace, diag.with_help(t!("codes.ZK0106.help_brace")), diag)
}

/// A character literal does not hold one character.
///
/// # Arguments
///
/// * `span` - The literal.
/// * `count` - Number of characters found.
///
/// # Returns
///
/// The `ZK0107` diagnostic.
pub fn bad_char_length(span: Span, count: usize) -> Diagnostic
{
    let msg = t!("codes.ZK0107.message", n = count.to_string());
    Diagnostic::new(Code::BadCharLength, msg, span)
}

/// A raw identifier reaches the end of the line.
///
/// # Arguments
///
/// * `span` - From the opening backtick to the end of the line.
///
/// # Returns
///
/// The `ZK0108` diagnostic.
pub fn unterminated_raw_identifier(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::UnterminatedRawIdentifier, t!("codes.ZK0108.message"), span)
}

/// A raw identifier has nothing between its backticks.
///
/// # Arguments
///
/// * `span` - The two backticks.
///
/// # Returns
///
/// The `ZK0109` diagnostic.
pub fn blank_raw_identifier(span: Span) -> Diagnostic
{
    Diagnostic::new(Code::BlankRawIdentifier, t!("codes.ZK0109.message"), span)
}
