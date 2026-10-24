use bon::Builder;
use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::lexer as diag;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::ident::is_burmese_digit;
use crate::token::DigitScript;
use crate::token::NumberShape;
use crate::token::Token;
use crate::token::TokenKind;
use crate::words::Word;

/// Result of lexing a number.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct NumberLexed
{
    /// The number token, or `None` when the word is not a valid number.
    pub token: Option<Token>,
    /// Index of the first word after the number.
    pub next: usize,
    /// Diagnostics found.
    pub diags: Vec<Diagnostic>,
}

/// Script of the digit `ch`.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// The digit script, or `None` when `ch` is not a digit.
pub fn digit_script(ch: char) -> Option<DigitScript>
{
    ret_if!(ch.is_ascii_digit(), Some(DigitScript::Western));
    tern!(is_burmese_digit(ch), Some(DigitScript::Burmese), None)
}

/// Script of the first character of `text`, when it is a digit.
///
/// # Arguments
///
/// * `text` - The word text.
///
/// # Returns
///
/// The digit script, or `None`.
pub fn word_script(text: &str) -> Option<DigitScript>
{
    text.chars().next().and_then(digit_script)
}

/// Lex the number that starts at `words[idx]`.
///
/// Re-joins `digits . digits` and `digits[eE] (+|-) digits` across words
/// when the words are joined. The span covers the whole literal; the value is
/// computed in the parser, which knows the base.
///
/// # Arguments
///
/// * `words` - All words.
/// * `idx` - Index of a word that starts with a digit.
///
/// # Returns
///
/// The number token or an `invalid-number` diagnostic, and the next index.
pub fn lex_number(words: &[Word<'_>], idx: usize) -> NumberLexed
{
    let word = &words[idx];
    let script = word_script(word.text).expect("lex_number needs a digit word");
    if !is_number_word(word.text, script)
    {
        let diags = vec![diag::invalid_number(word.span, word.text)];
        return NumberLexed::builder().next(idx + 1).diags(diags).build();
    }

    let mut scan = Scan::builder()
        .words(words)
        .end(idx + 1)
        .span(word.span)
        .text(word.text.to_string())
        .script(script)
        .build();
    scan.fraction();
    scan.exponent();
    scan.mixed_tail();
    scan.finish(word.joined)
}

/// Whether `text` is a run of digits of `script` (Western words can also hold
/// ASCII letters, for bases above 10).
///
/// # Arguments
///
/// * `text` - The word text.
/// * `script` - The script of its first digit.
///
/// # Returns
///
/// `true` if the word is a valid number word.
fn is_number_word(text: &str, script: DigitScript) -> bool
{
    match script
    {
        DigitScript::Burmese => text.chars().all(is_burmese_digit),
        DigitScript::Western => text.chars().all(|ch| ch.is_ascii_alphanumeric()),
    }
}

/// Whether `text` is `digits (. digits)? [eE] [+-]? digits` in Western digits.
///
/// # Arguments
///
/// * `text` - The literal text.
///
/// # Returns
///
/// `true` for exponent notation.
fn is_exponent_form(text: &str) -> bool
{
    let Some((mantissa, power)) = text.split_once(['e', 'E'])
    else
    {
        return false;
    };
    let power = power.strip_prefix(['+', '-']).unwrap_or(power);
    let (whole, frac) = mantissa.split_once('.').unwrap_or((mantissa, "0"));
    [whole, frac, power].iter().all(|part| is_ascii_digits(part))
}

/// Whether `text` is one or more ASCII digits.
///
/// # Arguments
///
/// * `text` - The text to check.
///
/// # Returns
///
/// `true` for a non-blank run of ASCII digits.
fn is_ascii_digits(text: &str) -> bool
{
    !text.is_empty() && text.chars().all(|ch| ch.is_ascii_digit())
}

/// Whether `text` holds an ASCII letter.
///
/// # Arguments
///
/// * `text` - The text to check.
///
/// # Returns
///
/// `true` if an ASCII letter is present.
fn has_ascii_letter(text: &str) -> bool
{
    text.chars().any(|ch| ch.is_ascii_alphabetic())
}

/// State of a number scan over several words.
#[derive(Builder)]
struct Scan<'w, 's>
{
    /// All words.
    words: &'w [Word<'s>],
    /// Index of the first word not consumed.
    end: usize,
    /// Span of the consumed words.
    span: Span,
    /// Text of the consumed words.
    text: String,
    /// Script of the first digit.
    script: DigitScript,
    /// Whether a fraction was consumed.
    #[builder(default)]
    has_fraction: bool,
    /// Diagnostics found.
    #[builder(default)]
    diags: Vec<Diagnostic>,
}

impl<'s> Scan<'_, 's>
{
    /// The word at `idx` when it is joined to the previous one.
    ///
    /// # Arguments
    ///
    /// * `idx` - The word index.
    ///
    /// # Returns
    ///
    /// The word, or `None` when it does not exist or is not joined.
    fn joined_word(&self, idx: usize) -> Option<&'s str>
    {
        self.words.get(idx).filter(|word| word.joined).map(|word| word.text)
    }

    /// Whether the word at `idx` is one of the delimiters in `set`, joined.
    ///
    /// # Arguments
    ///
    /// * `idx` - The word index.
    /// * `set` - The delimiter characters to accept.
    ///
    /// # Returns
    ///
    /// `true` if the word is a joined delimiter from `set`.
    fn joined_delim(&self, idx: usize, set: &[char]) -> bool
    {
        let word = self.words.get(idx);
        word.is_some_and(|w| w.joined && w.delim().is_some_and(|ch| set.contains(&ch)))
    }

    /// Consume `count` words.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of words to consume.
    fn take(&mut self, count: usize)
    {
        let taken = &self.words[self.end..self.end + count];
        taken.iter().for_each(|word| self.text.push_str(word.text));
        self.span = self.span.to(&taken[count - 1].span);
        self.end += count;
    }

    /// Consume `. digits` when it follows.
    fn fraction(&mut self)
    {
        ret_if!(!self.joined_delim(self.end, &['.']));
        let Some(digits) = self.joined_word(self.end + 1)
        else
        {
            return;
        };
        let Some(script) = word_script(digits)
        else
        {
            return;
        };
        ret_if!(!is_number_word(digits, script));
        self.take(2);
        self.has_fraction = true;
        self.check_script(script);
    }

    /// Consume `(+|-) digits` after a Western mantissa that ends in `e`/`E`.
    fn exponent(&mut self)
    {
        let western = self.script == DigitScript::Western;
        ret_if!(!western || !self.text.ends_with(['e', 'E']));
        ret_if!(!self.joined_delim(self.end, &['+', '-']));
        let digits = self.joined_word(self.end + 1);
        ret_if!(!digits.is_some_and(is_ascii_digits));
        self.take(2);
    }

    /// Consume a joined number word of the other script (`၁၂34`).
    fn mixed_tail(&mut self)
    {
        let Some(next) = self.joined_word(self.end)
        else
        {
            return;
        };
        let Some(script) = word_script(next)
        else
        {
            return;
        };
        self.take(1);
        self.check_script(script);
    }

    /// Report `mixed-digit-scripts` when `script` differs from the first one.
    ///
    /// # Arguments
    ///
    /// * `script` - Script of the digits just consumed.
    fn check_script(&mut self, script: DigitScript)
    {
        ret_if!(script == self.script);
        ret_if!(self.diags.iter().any(|d| d.code == Code::MixedDigitScripts));
        self.diags.push(diag::mixed_digit_scripts(self.span));
    }

    /// Shape of the consumed literal.
    ///
    /// # Returns
    ///
    /// The number shape.
    fn shape(&self) -> NumberShape
    {
        let western = self.script == DigitScript::Western;
        let has_exponent = western && is_exponent_form(&self.text);
        let has_letters = western && !has_exponent && has_ascii_letter(&self.text);
        NumberShape::builder()
            .script(self.script)
            .has_fraction(self.has_fraction)
            .has_exponent(has_exponent)
            .has_letters(has_letters)
            .build()
    }

    /// Build the result.
    ///
    /// # Arguments
    ///
    /// * `joined` - Whether the first word is joined to the previous token.
    ///
    /// # Returns
    ///
    /// The number token with its diagnostics.
    fn finish(mut self, joined: bool) -> NumberLexed
    {
        // the mixed-script diagnostic should cover the whole literal
        self.diags.iter_mut().for_each(|d| d.span = self.span);
        let shape = self.shape();
        let kind = TokenKind::Number(shape);
        let token = Token::builder().kind(kind).span(self.span).joined(joined).build();
        NumberLexed::builder().token(token).next(self.end).diags(self.diags).build()
    }
}
