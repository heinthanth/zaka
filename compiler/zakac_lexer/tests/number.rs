mod common;

use common::codes;
use common::kinds;
use common::lexemes;
use zakac_diagnostics::Code;
use zakac_lexer::token::DigitScript;
use zakac_lexer::token::NumberShape;
use zakac_lexer::token::TokenKind;

/// The one number token in `text`, with its lexeme.
fn number(text: &str) -> (NumberShape, String)
{
    let kinds = kinds(text);
    assert_eq!(kinds.len(), 1, "{text}: {kinds:?}");
    let TokenKind::Number(shape) = kinds[0]
    else
    {
        panic!("{text}: not a number: {:?}", kinds[0]);
    };
    (shape, lexemes(text).remove(0))
}

/// A shape with no flags set.
fn plain(script: DigitScript) -> NumberShape
{
    NumberShape::builder()
        .script(script)
        .has_fraction(false)
        .has_exponent(false)
        .has_letters(false)
        .build()
}

#[test]
fn integers()
{
    assert_eq!(number("၁၀"), (plain(DigitScript::Burmese), "၁၀".into()));
    assert_eq!(number("16"), (plain(DigitScript::Western), "16".into()));
}

#[test]
fn fractions_join_across_the_dot()
{
    let (shape, lexeme) = number("၁၂.၃၄");
    assert!(shape.has_fraction && shape.script == DigitScript::Burmese);
    assert_eq!(lexeme, "၁၂.၃၄");
    assert!(number("12.34").0.has_fraction);
}

#[test]
fn exponents()
{
    for text in ["100e+10", "100e-10", "1.234e16", "100e10", "1.5E-3"]
    {
        let (shape, lexeme) = number(text);
        assert!(shape.has_exponent && !shape.has_letters, "{text}");
        assert_eq!(lexeme, text);
    }
}

#[test]
fn letters_for_high_bases()
{
    assert!(number("1F").0.has_letters);
    assert!(number("0x1F").0.has_letters);
    assert_eq!(kinds("FF"), [TokenKind::Ident]);
}

#[test]
fn spaced_dot_does_not_join()
{
    assert_eq!(codes("12 . 34"), [Code::InvalidCharacter]);
}

#[test]
fn mixed_scripts()
{
    for text in ["၁၂.34", "12.၃၄", "၁၂34"]
    {
        assert_eq!(codes(text), [Code::MixedDigitScripts], "{text}");
        assert_eq!(number(text).1, text);
    }
}

#[test]
fn invalid_numbers()
{
    assert_eq!(codes("၁၈ဘစ်"), [Code::InvalidNumber]);
    assert_eq!(codes("1é"), [Code::InvalidNumber]);
    assert_eq!(codes("1_000"), [Code::InvalidNumber]);
    assert!(kinds("၁က").is_empty());
}

#[test]
fn minus_remains_an_operator()
{
    let kinds = kinds("-၅");
    assert_eq!(kinds.len(), 2);
    assert!(matches!(kinds[1], TokenKind::Number(_)));
}
