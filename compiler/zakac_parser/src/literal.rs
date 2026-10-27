use zakac_ast::expr::Expr;
use zakac_ast::expr::FloatLit;
use zakac_ast::expr::IntLit;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Kind;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser as diag;
use zakac_i18n::t;
use zakac_lexer::token::DigitScript;
use zakac_lexer::token::Kw;
use zakac_lexer::token::NumberShape;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::parser::PResult;
use crate::parser::Parser;

/// Value of one digit: Burmese and Western digits, then ASCII letters for
/// 10 to 35.
///
/// # Arguments
///
/// * `ch` - The character.
///
/// # Returns
///
/// The value, or `None` for a character that is no digit.
fn digit_value(ch: char) -> Option<u32>
{
    ret_if!(('၀'..='၉').contains(&ch), Some(ch as u32 - '၀' as u32));
    ch.to_digit(36)
}

/// Value of a run of decimal digits in either script.
///
/// # Arguments
///
/// * `text` - The digits.
///
/// # Returns
///
/// The value, or `None` for a character that is no decimal digit or a value
/// too large for `u128`.
pub(crate) fn decimal_value(text: &str) -> Option<u128>
{
    text.chars().try_fold(0u128, |acc, ch| {
        let digit = digit_value(ch).filter(|d| *d < 10)?;
        acc.checked_mul(10)?.checked_add(u128::from(digit))
    })
}

/// Value of the digits of an integer literal.
///
/// # Arguments
///
/// * `text` - The digits.
/// * `base` - The base, from 2 to 36.
/// * `span` - The digits span, for diagnostics.
///
/// # Returns
///
/// The value, an `invalid-digit` or a `literal-out-of-range` diagnostic.
fn int_value(text: &str, base: u32, span: Span) -> Result<u128, Diagnostic>
{
    text.chars().try_fold(0u128, |acc, ch| {
        let digit = digit_value(ch).filter(|d| *d < base);
        let digit = digit.ok_or_else(|| diag::invalid_digit(span, ch, base))?;
        let next = acc.checked_mul(u128::from(base));
        let next = next.and_then(|v| v.checked_add(u128::from(digit)));
        next.ok_or_else(|| diag::literal_out_of_range(span, text))
    })
}

/// Whether an integer value fits its kind.
///
/// Unsigned kinds and literals with a base prefix (bit patterns) take the full
/// width. Signed decimal literals take one more than the largest positive
/// value, so that a negated minimum such as `-128` folds later. A literal
/// without a kind prefix takes the kind of its place, which the semantic
/// check finds, so it is not limited here.
///
/// # Arguments
///
/// * `value` - The value.
/// * `kind` - The kind prefix.
/// * `based` - Whether a base prefix is written.
///
/// # Returns
///
/// `true` if it fits.
fn fits(value: u128, kind: Option<Kind>, based: bool) -> bool
{
    let Some(kind) = kind
    else
    {
        return true;
    };
    let bits = kind.bits.map_or(64, u32::from);
    let signed = kind.signed != Some(false);
    ret_if!(signed && !based, value <= 1u128 << (bits - 1));
    bits >= 128 || value < 1u128 << bits
}

/// The text of a float literal with Burmese digits made Western.
///
/// # Arguments
///
/// * `text` - The literal text.
///
/// # Returns
///
/// The text Rust can parse.
fn western(text: &str) -> String
{
    let map = |ch: char| {
        digit_value(ch).filter(|_| !ch.is_ascii()).map(|d| (b'0' + d as u8) as char)
    };
    text.chars().map(|ch| map(ch).unwrap_or(ch)).collect()
}

/// The shape of a number token.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// The shape, or `None` for letters read as digits.
fn shape_of(tok: &Token) -> Option<NumberShape>
{
    match tok.kind
    {
        TokenKind::Number(shape) => Some(shape),
        _ => None,
    }
}

/// Whether the kind is the base kind `base`.
///
/// # Arguments
///
/// * `kind` - The kind prefix, when written.
/// * `base` - The base kind.
///
/// # Returns
///
/// `true` if it is.
fn is_base(kind: Option<Kind>, base: BaseKind) -> bool
{
    kind.is_some_and(|k| k.base == base)
}

impl Parser<'_>
{
    /// Parse a number literal with its optional base and kind prefixes.
    ///
    /// # Returns
    ///
    /// An integer or floating-point literal, or a diagnostic.
    pub(crate) fn parse_literal(&mut self) -> PResult<Expr>
    {
        let start = self.peek().span;
        let base = self.parse_base()?;
        let kind = self.parse_literal_kind()?;
        let tok = self.literal_digits(base)?;
        let span = start.to(&tok.span);
        let text = self.text_of(tok.span);
        let shape = shape_of(&tok);
        let fraction = shape.is_some_and(|s| s.has_fraction || s.has_exponent);
        let float = is_base(kind, BaseKind::Float);
        ret_if!(float && base.is_some(), self.based_float(text, base, kind, span));
        let is_float = base.is_none() && (float || fraction);
        ret_if!(is_float, self.float_lit(text, kind, span));
        let value = int_value(text, base.unwrap_or(10), tok.span)?;
        let out_of_range = !fits(value, kind, base.is_some());
        ret_if!(out_of_range, Err(diag::literal_out_of_range(span, text)));
        let script = shape.map_or(DigitScript::Western, |s| s.script);
        let base = base.unwrap_or(10) as u8;
        let id = self.new_id();
        #[rustfmt::skip]
        let lit = IntLit::builder().id(id).value(value).base(base).maybe_kind(kind)
            .script(script).span(span).build();
        Ok(Expr::Int(lit))
    }

    /// Parse `အခြေ N ဖြင့်` when it comes next.
    ///
    /// # Returns
    ///
    /// The base, `None` without a prefix, or an `invalid-base` diagnostic.
    fn parse_base(&mut self) -> PResult<Option<u32>>
    {
        ret_if!(!self.eat(Kw::Base), Ok(None));
        let tok = self.peek();
        ret_if!(shape_of(&tok).is_none(), Err(self.err_expected(&t!("parser.digits"))));
        self.bump();
        let text = self.text_of(tok.span);
        let base = decimal_value(text).filter(|b| (2..=36).contains(b));
        let base = base.ok_or_else(|| diag::invalid_base(tok.span, text))?;
        self.expect(Kw::Hpjint)?;
        Ok(Some(base as u32))
    }

    /// Parse the kind prefix of a literal when one comes next.
    ///
    /// # Returns
    ///
    /// The kind, or a `bad-literal-kind` diagnostic for a character,
    /// boolean or address kind.
    fn parse_literal_kind(&mut self) -> PResult<Option<Kind>>
    {
        ret_if!(!self.starts_kind(), Ok(None));
        let kind = self.parse_kind()?;
        let numeric = matches!(kind.base, BaseKind::Int | BaseKind::Float);
        let numeric = numeric && kind.depth == 0;
        ret_if!(!numeric, Err(diag::bad_literal_kind(kind.span)));
        Ok(Some(kind))
    }

    /// Move past the digits of a literal: a number token, or after a base
    /// prefix above 10 a word of ASCII letters and digits.
    ///
    /// # Arguments
    ///
    /// * `base` - The base prefix, when written.
    ///
    /// # Returns
    ///
    /// The digits token, or an `expected-token` diagnostic.
    fn literal_digits(&mut self, base: Option<u32>) -> PResult<Token>
    {
        let tok = self.peek();
        ret_if!(shape_of(&tok).is_some(), Ok(self.bump()));
        let text = self.text_of(tok.span);
        let letters = tok.kind == TokenKind::Ident
            && text.chars().all(|c| c.is_ascii_alphanumeric());
        ret_if!(letters && base.is_some_and(|b| b > 10), Ok(self.bump()));
        Err(self.err_expected(&t!("parser.digits")))
    }

    /// Build a floating-point literal from decimal text.
    ///
    /// # Arguments
    ///
    /// * `text` - The digits, with an optional fraction and exponent.
    /// * `kind` - The kind prefix, when written.
    /// * `span` - The whole literal.
    ///
    /// # Returns
    ///
    /// The literal, or a `float-for-int-kind` diagnostic.
    fn float_lit(&mut self, text: &str, kind: Option<Kind>, span: Span)
    -> PResult<Expr>
    {
        ret_if!(is_base(kind, BaseKind::Int), Err(diag::float_for_int_kind(span)));
        let text = western(text);
        let value = text.parse::<f64>();
        let value = value.map_err(|_| diag::invalid_digit(span, '.', 10))?;
        Ok(self.float_expr(value, text, kind, span))
    }

    /// Build a floating-point literal from integer digits in another base.
    ///
    /// # Arguments
    ///
    /// * `text` - The digits.
    /// * `base` - The base prefix.
    /// * `kind` - The float kind prefix.
    /// * `span` - The whole literal.
    ///
    /// # Returns
    ///
    /// The literal, or an `invalid-digit` diagnostic.
    fn based_float(
        &mut self, text: &str, base: Option<u32>, kind: Option<Kind>, span: Span,
    ) -> PResult<Expr>
    {
        let value = int_value(text, base.unwrap_or(10), span)?;
        Ok(self.float_expr(value as f64, value.to_string(), kind, span))
    }

    /// Wrap a float value in a literal node.
    ///
    /// # Arguments
    ///
    /// * `value` - The value.
    /// * `text` - The decimal text in Western digits.
    /// * `kind` - The kind prefix, when written.
    /// * `span` - The whole literal.
    ///
    /// # Returns
    ///
    /// The expression.
    fn float_expr(
        &mut self, value: f64, text: String, kind: Option<Kind>, span: Span,
    ) -> Expr
    {
        let id = self.new_id();
        #[rustfmt::skip]
        let node = FloatLit::builder().id(id).value(value).text(text).maybe_kind(kind)
            .span(span).build();
        Expr::Float(node)
    }
}
