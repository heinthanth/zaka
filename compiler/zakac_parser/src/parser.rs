use std::collections::HashMap;
use std::collections::HashSet;

use bon::Builder;
use zakac_ast::expr::Expr;
use zakac_ast::expr::Ident;
use zakac_ast::expr::NodeId;
use zakac_ast::stmt::Program;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::parser as diag;
use zakac_i18n::t;
use zakac_lexer::reserved::PLURAL;
use zakac_lexer::spelling::canonical;
use zakac_lexer::spelling::spellings;
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
    /// The number the next expression node would take, so the parse of
    /// another file can go on from it.
    #[builder(default)]
    pub next_id: u32,
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

/// Parse the tokens of one file of a program read with its imports.
///
/// # Arguments
///
/// * `source` - The source file the tokens come from.
/// * `lexed` - The lexer output for it, with absolute spans.
/// * `first_id` - The number for the first expression node, so the nodes of
///   each file of the program get different numbers.
/// * `structs` - The struct names the file imports, so each of them can start a
///   kind as the names it declares do.
/// * `marked` - The functions the file imports that declare marks, with the
///   number of marks of each, so a marked call of each can be read.
///
/// # Returns
///
/// The program and the parser diagnostics, with absolute spans.
pub fn parse_with(
    source: &SourceFile, lexed: &LexOutput, first_id: u32, structs: &HashSet<String>,
    marked: &HashMap<String, Vec<usize>>,
) -> ParseOutput
{
    let (text, base) = (source.src.as_str(), source.start_pos);
    let span = Span::new(base, base + text.len());
    ret_if!(lexed.tokens.is_empty(), blank_output(span));
    let mut parser = Parser::new(text, base, lexed);
    parser.next_id = first_id;
    parser.structs.extend(structs.iter().flat_map(|s| spellings(s)));
    let marked = marked.iter().flat_map(|(name, counts)| {
        spellings(name).into_iter().map(move |n| (n, counts.clone()))
    });
    marked.for_each(|(name, counts)| {
        parser.marked.entry(name).or_default().extend(counts)
    });
    finish(parser, span)
}

/// Parse the program of a parser and collect its output.
///
/// # Arguments
///
/// * `parser` - The parser at the first token.
/// * `span` - The file span.
///
/// # Returns
///
/// The program, the parser diagnostics and the next node number.
fn finish(mut parser: Parser<'_>, span: Span) -> ParseOutput
{
    let stmts = parser.parse_program();
    let program = Program::builder().stmts(stmts).span(span).build();
    #[rustfmt::skip]
    return ParseOutput::builder().program(program).diagnostics(parser.diags)
        .next_id(parser.next_id).build();
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
    finish(Parser::new(text, base, lexed), span)
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
    let at_base = Span::new(base, base);
    ret_if!(lexed.tokens.is_empty(), Err(diag::expected_operand(at_base, "")));
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
    /// The struct names the program declares or imports, each in both
    /// spellings of `ဒေတာ`, so a name can start a kind however it is written.
    #[builder(default)]
    pub(crate) structs: HashSet<String>,
    /// The classifier each struct of this file declares, under the name in
    /// the spelling the compiler keeps, for the help of a declaration that
    /// leaves it out.
    #[builder(default)]
    pub(crate) classifiers: HashMap<String, &'t str>,
    /// The functions the program declares or imports that declare marks,
    /// each name in both spellings of `ဒေတာ`, with the number of marks of
    /// each function of that name, so those names alone end a marked call.
    #[builder(default)]
    pub(crate) marked: HashMap<String, Vec<usize>>,
    /// Whether a list count is being parsed: its `ရှိသော` closes the count,
    /// so no struct literal starts inside it.
    #[builder(default)]
    pub(crate) in_count: bool,
    /// Whether the value of a fill is being parsed: a `ဖြင့်` after a lone
    /// operand closes the value, and the size follows it.
    #[builder(default)]
    pub(crate) in_fill: bool,
    /// Whether a list of parameters or fields is open: a function kind with
    /// parameters then stands in parentheses.
    #[builder(default)]
    pub(crate) in_list: bool,
    /// Whether the place that holds the function of a long call is being
    /// parsed: a `ကို` after a lone operand closes the place.
    #[builder(default)]
    pub(crate) in_callee: bool,
    /// Whether a statement other than an import has been parsed at the top
    /// level, so a later import is out of place.
    #[builder(default)]
    pub(crate) imports_closed: bool,
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
        let mut parser = Parser::builder().cur(cur).text(text).base(base).strings(&lexed.strings).build();
        let declared = |pair: &[Token]| {
            let opener = pair[0].kind == TokenKind::Kw(Kw::DataKindOpen);
            let named = opener && pair[1].kind == TokenKind::Ident;
            named.then(|| parser.text_of(pair[1].span))
        };
        let names: Vec<&str> = lexed.tokens.windows(2).filter_map(declared).collect();
        // the stem of an enumeration names it too: `အရောင်` for `အရောင်များ`
        let stems = names.iter().filter_map(|name| name.strip_suffix(PLURAL));
        parser.structs =
            names.iter().copied().chain(stems).flat_map(spellings).collect();
        let starts = lexed.tokens.iter().enumerate();
        let starts = starts.filter(|(_, t)| t.kind == TokenKind::Kw(Kw::DataKindOpen));
        let found = starts
            .filter_map(|(at, _)| parser.struct_classifier(&lexed.tokens, at))
            .map(|(name, word)| (canonical(name).into_owned(), word))
            .collect();
        parser.classifiers = found;
        let heads = lexed.tokens.iter().enumerate();
        let heads = heads.filter(|(_, t)| t.kind == TokenKind::Kw(Kw::Function));
        let heads: Vec<(&str, usize)> = heads
            .filter_map(|(at, _)| parser.marked_header(&lexed.tokens, at))
            .collect();
        for (name, count) in heads
        {
            let names = spellings(name).into_iter();
            names.for_each(|n| parser.marked.entry(n).or_default().push(count));
        }
        parser
    }

    /// The name and the number of marks of the function declaration whose
    /// `လုပ်ဆောင်ချက်` sits at token `at`, when its parameters have marks:
    /// a name with `(` straight after it, before the full stop of the header.
    ///
    /// # Arguments
    ///
    /// * `tokens` - The tokens of the file.
    /// * `at` - The index of `လုပ်ဆောင်ချက်` in them.
    ///
    /// # Returns
    ///
    /// The pair, or `None` for a call, or a header with no marks.
    fn marked_header(&self, tokens: &[Token], at: usize) -> Option<(&'t str, usize)>
    {
        let tokens = tokens.get(at..)?;
        let name = tokens.get(1).filter(|t| t.kind == TokenKind::Ident)?;
        ret_if!(tokens.get(2)?.kind != TokenKind::Kw(Kw::Thi), None);
        let stop = TokenKind::Punct(Punct::FullStop);
        let header: Vec<&Token> =
            tokens.iter().take_while(|t| t.kind != stop).collect();
        // A mark at depth 0 marks the header; one inside the parentheses of a
        // function kind marks a parameter of that kind.
        let mut depth = 0usize;
        let mut count = 0;
        for pair in header.windows(2)
        {
            let paren =
                pair[1].kind == TokenKind::Punct(Punct::LParen) && pair[1].joined;
            count +=
                usize::from(depth == 0 && pair[0].kind == TokenKind::Ident && paren);
            match pair[0].kind
            {
                TokenKind::Punct(Punct::LParen) => depth += 1,
                TokenKind::Punct(Punct::RParen) => depth = depth.saturating_sub(1),
                _ => (),
            }
        }
        (count > 0).then(|| (self.text_of(name.span), count))
    }

    /// The name and the classifier of the struct declaration whose
    /// `ဒေတာအမျိုးအစား` sits at token `at`.
    ///
    /// # Arguments
    ///
    /// * `tokens` - The tokens of the file.
    /// * `at` - The index of `ဒေတာအမျိုးအစား` in them.
    ///
    /// # Returns
    ///
    /// The pair, or `None` for a declaration with no classifier clause.
    fn struct_classifier(
        &self, tokens: &[Token], at: usize,
    ) -> Option<(&'t str, &'t str)>
    {
        let tokens = tokens.get(at..)?;
        let name = tokens.get(1).filter(|t| t.kind == TokenKind::Ident)?;
        let stop = TokenKind::Punct(Punct::FullStop);
        let clause = tokens.iter().take_while(|t| t.kind != stop);
        let is =
            clause.clone().position(|t| t.kind == TokenKind::Kw(Kw::ClassifierIs))?;
        let word = tokens.get(is + 1)?;
        Some((self.text_of(name.span), self.text_of(word.span)))
    }

    /// Whether the token `n` places ahead is a declared struct or
    /// enumeration name that starts a kind. A name before `ထဲမှ` names a
    /// member instead: `အမှားများထဲမှ မတွေ့`.
    ///
    /// # Arguments
    ///
    /// * `n` - How far to look ahead.
    ///
    /// # Returns
    ///
    /// `true` for such a name.
    pub(crate) fn is_struct_at(&self, n: usize) -> bool
    {
        let tok = self.peek_n(n);
        let named = tok.kind == TokenKind::Ident
            && self.structs.contains(self.text_of(tok.span));
        named && self.peek_n(n + 1).kind != TokenKind::Kw(Kw::Among)
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
        let is_name = tok.kind == TokenKind::Ident || raw;
        ret_if!(!is_name, Err(self.err_expected(&t!("parser.name"))));
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
