use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NodeId;
use zakac_ast::stmt::Program;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser as diag;
use zakac_i18n::t;
use zakac_lexer::token::Kw;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::Punct;
use zakac_lexer::token::StrLit;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::pos::Pos;
use zakac_span::source_file::SourceFile;
use zakac_span::span::Span;

use crate::cursor::Cursor;
use crate::describe::kw_text;
use crate::describe::one_of;
use crate::describe::quote;

/// Result of a parse step; the error is the diagnostic to report.
pub(crate) type PResult<T> = Result<T, Diagnostic>;

/// The whole parser output for one source file.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct ParseOutput
{
    /// The program; statements with errors are left out.
    pub program: Program,
    /// Parser diagnostics, in source order.
    #[builder(default)]
    pub diagnostics: Vec<Diagnostic>,
}

/// Parse the tokens of a source file.
///
/// # Arguments
///
/// * `source` - The source file the tokens come from.
/// * `lexed` - The lexer output for it, with absolute spans.
///
/// # Returns
///
/// The program and the parser diagnostics, with absolute spans.
pub fn parse(source: &SourceFile, lexed: &LexOutput) -> ParseOutput
{
    parse_at(&source.src, source.start_pos, lexed)
}

/// Parse the tokens of `text`, which starts at the absolute position `base`.
///
/// # Arguments
///
/// * `text` - The normalized source text.
/// * `base` - Absolute position of `text`.
/// * `lexed` - The lexer output for `text`.
///
/// # Returns
///
/// The program and the parser diagnostics.
pub fn parse_at(text: &str, base: Pos, lexed: &LexOutput) -> ParseOutput
{
    let span = Span::new(base, base + text.len());
    ret_if!(lexed.tokens.is_empty(), blank_output(span));
    let mut parser = Parser::new(text, base, lexed);
    let stmts = parser.parse_program();
    let program = Program::builder().stmts(stmts).span(span).build();
    ParseOutput::builder().program(program).diagnostics(parser.diags).build()
}

/// Parse `text` as one expression that takes all of its tokens.
///
/// # Arguments
///
/// * `text` - The normalized source text.
/// * `base` - Absolute position of `text`.
/// * `lexed` - The lexer output for `text`.
///
/// # Returns
///
/// The expression, or the first diagnostic.
pub fn parse_lone_expr(
    text: &str, base: Pos, lexed: &LexOutput,
) -> Result<Expr, Diagnostic>
{
    ret_if!(
        lexed.tokens.is_empty(),
        Err(diag::expected_operand(Span::new(base, base), ""))
    );
    let mut parser = Parser::new(text, base, lexed);
    let expr = parser.parse_expr()?;
    ret_if!(!parser.at_eof(), Err(parser.err_expected(&t!("parser.eof"))));
    Ok(expr)
}

/// Output for a token list without even an end-of-input token.
///
/// # Arguments
///
/// * `span` - The file span.
///
/// # Returns
///
/// A blank program.
fn blank_output(span: Span) -> ParseOutput
{
    ParseOutput::builder().program(Program::builder().span(span).build()).build()
}

/// The reserved word of a token.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// The reserved word, or `None` for another token.
pub(crate) fn kw_of(tok: &Token) -> Option<Kw>
{
    match tok.kind
    {
        TokenKind::Kw(kw) => Some(kw),
        _ => None,
    }
}

/// Whether a token is a number.
///
/// # Arguments
///
/// * `tok` - The token.
///
/// # Returns
///
/// `true` for a number token.
pub(crate) fn is_number(tok: &Token) -> bool
{
    matches!(tok.kind, TokenKind::Number(_))
}

/// Parser state: the cursor, the source text and the diagnostics so far.
#[derive(Builder)]
pub(crate) struct Parser<'t>
{
    /// Position in the tokens.
    pub(crate) cur: Cursor<'t>,
    /// The normalized source text.
    pub(crate) text: &'t str,
    /// Absolute position of `text`.
    pub(crate) base: Pos,
    /// Decoded string literals, indexed with [`TokenKind::Str`].
    pub(crate) strings: &'t [StrLit],
    /// Diagnostics recorded after recovering from an error.
    #[builder(default)]
    pub(crate) diags: Vec<Diagnostic>,
    /// Number for the next expression node.
    #[builder(default)]
    pub(crate) next_id: u32,
    /// The number of blocks that enclose the current statement.
    #[builder(default)]
    pub(crate) depth: u32,
}

impl<'t> Parser<'t>
{
    /// Create a parser over non-blank lexer output.
    ///
    /// # Arguments
    ///
    /// * `text` - The normalized source text.
    /// * `base` - Absolute position of `text`.
    /// * `lexed` - The lexer output.
    ///
    /// # Returns
    ///
    /// The parser at the first token.
    pub(crate) fn new(text: &'t str, base: Pos, lexed: &'t LexOutput) -> Self
    {
        let cur = Cursor::builder().tokens(&lexed.tokens).build();
        #[rustfmt::skip]
        return Parser::builder().cur(cur).text(text).base(base).strings(&lexed.strings).build();
    }

    /// The current token.
    ///
    /// # Returns
    ///
    /// The token.
    pub(crate) fn peek(&self) -> Token { self.cur.peek_n(0) }

    /// The token `n` places ahead.
    ///
    /// # Arguments
    ///
    /// * `n` - How far to look ahead.
    ///
    /// # Returns
    ///
    /// The token.
    pub(crate) fn peek_n(&self, n: usize) -> Token { self.cur.peek_n(n) }

    /// Move past the current token.
    ///
    /// # Returns
    ///
    /// The token moved past.
    pub(crate) fn bump(&mut self) -> Token { self.cur.bump() }

    /// Move past the current token without returning it.
    pub(crate) fn advance(&mut self) { self.cur.bump(); }

    /// Whether the current token is the reserved word `kw`.
    ///
    /// # Arguments
    ///
    /// * `kw` - The reserved word.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    pub(crate) fn at(&self, kw: Kw) -> bool { self.peek().kind == TokenKind::Kw(kw) }

    /// Whether the current token is the punctuation `punct`.
    ///
    /// # Arguments
    ///
    /// * `punct` - The punctuation.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    pub(crate) fn at_punct(&self, punct: Punct) -> bool
    {
        self.peek().kind == TokenKind::Punct(punct)
    }

    /// Whether the input is used up.
    ///
    /// # Returns
    ///
    /// `true` at the end-of-input token.
    pub(crate) fn at_eof(&self) -> bool { self.peek().kind == TokenKind::Eof }

    /// Move past the reserved word `kw` if it is the current token.
    ///
    /// # Arguments
    ///
    /// * `kw` - The reserved word.
    ///
    /// # Returns
    ///
    /// `true` if it was there.
    pub(crate) fn eat(&mut self, kw: Kw) -> bool
    {
        let found = self.at(kw);
        do_if!(found, self.advance());
        found
    }

    /// Move past the punctuation `punct` if it is the current token.
    ///
    /// # Arguments
    ///
    /// * `punct` - The punctuation.
    ///
    /// # Returns
    ///
    /// `true` if it was there.
    pub(crate) fn eat_punct(&mut self, punct: Punct) -> bool
    {
        let found = self.at_punct(punct);
        do_if!(found, self.advance());
        found
    }

    /// Move past the reserved word `kw`, or fail.
    ///
    /// # Arguments
    ///
    /// * `kw` - The reserved word.
    ///
    /// # Returns
    ///
    /// The token, or an `expected-token` diagnostic.
    pub(crate) fn expect(&mut self, kw: Kw) -> PResult<Token>
    {
        self.expect_one_of(&[kw])
    }

    /// Move past one of the reserved words `kws`, or fail.
    ///
    /// # Arguments
    ///
    /// * `kws` - The accepted reserved words.
    ///
    /// # Returns
    ///
    /// The token, or an `expected-token` diagnostic that lists them.
    pub(crate) fn expect_one_of(&mut self, kws: &[Kw]) -> PResult<Token>
    {
        ret_if!(kws.iter().any(|kw| self.at(*kw)), Ok(self.bump()));
        let names: Vec<String> = kws.iter().map(|kw| quote(&kw_text(*kw))).collect();
        Err(self.err_expected(&one_of(&names)))
    }

    /// Move past the punctuation `punct`, or fail.
    ///
    /// # Arguments
    ///
    /// * `punct` - The punctuation.
    ///
    /// # Returns
    ///
    /// The token, or an `expected-token` diagnostic.
    pub(crate) fn expect_punct(&mut self, punct: Punct) -> PResult<Token>
    {
        ret_if!(self.at_punct(punct), Ok(self.bump()));
        Err(self.err_expected(&quote(punct.as_ref())))
    }

    /// Move past the full stop `။` that ends a statement, or fail.
    ///
    /// # Returns
    ///
    /// The token, or an `expected-token` diagnostic.
    pub(crate) fn expect_stop(&mut self) -> PResult<Token>
    {
        self.expect_punct(Punct::FullStop)
    }

    /// Source text of a span.
    ///
    /// # Arguments
    ///
    /// * `span` - An absolute span inside the text.
    ///
    /// # Returns
    ///
    /// The text.
    pub(crate) fn text_of(&self, span: Span) -> &'t str
    {
        let lo = (span.lo - self.base) as usize;
        let hi = (span.hi - self.base) as usize;
        &self.text[lo..hi]
    }

    /// Description of the current token for a message.
    ///
    /// # Returns
    ///
    /// The quoted token text, or the words for the end of input.
    pub(crate) fn found(&self) -> String
    {
        let tok = self.peek();
        ret_if!(tok.kind == TokenKind::Eof, t!("parser.eof").to_string());
        quote(self.text_of(tok.span))
    }

    /// An `expected-token` diagnostic at the current token.
    ///
    /// # Arguments
    ///
    /// * `expected` - What the grammar allows here.
    ///
    /// # Returns
    ///
    /// The diagnostic.
    pub(crate) fn err_expected(&self, expected: &str) -> Diagnostic
    {
        diag::expected_token(self.peek().span, expected, &self.found())
    }

    /// Number for a new expression node.
    ///
    /// # Returns
    ///
    /// The next unused number.
    pub(crate) fn new_id(&mut self) -> NodeId
    {
        self.next_id += 1;
        NodeId(self.next_id - 1)
    }

    /// Span of the last token moved past.
    ///
    /// # Returns
    ///
    /// The span, or the current token span at the start.
    pub(crate) fn prev_span(&self) -> Span
    {
        self.cur.back(1).map_or(self.peek().span, |tok| tok.span)
    }

    /// Parse a name: an identifier or a raw identifier.
    ///
    /// # Returns
    ///
    /// The name without backticks, or an `expected-token` diagnostic.
    pub(crate) fn parse_ident(&mut self) -> PResult<Ident>
    {
        let tok = self.peek();
        let raw = tok.kind == TokenKind::RawIdent;
        ret_if!(
            tok.kind != TokenKind::Ident && !raw,
            Err(self.err_expected(&t!("parser.name")))
        );
        self.bump();
        let text = self.text_of(tok.span);
        let name = unquote_raw(raw, text);
        Ok(Ident::builder().name(name).span(tok.span).build())
    }
}

/// The name inside a raw identifier, or the text as is.
///
/// # Arguments
///
/// * `raw` - Whether the text is a raw identifier with backticks.
/// * `text` - The token text.
///
/// # Returns
///
/// The name.
pub(crate) fn unquote_raw(raw: bool, text: &str) -> &str
{
    ret_if!(!raw, text);
    text.strip_prefix('`').and_then(|t| t.strip_suffix('`')).unwrap_or(text)
}
