use bon::Builder;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::lexer as diag;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::pos::Pos;
use zakac_span::span::Span;

use crate::token::StrLit;
use crate::token::StrPart;

/// Named brace escapes. Names for tab, carriage return and null are not fixed
/// so far; add them here.
pub const NAMED_ESCAPES: &[(&str, char)] = &[("နောက်တစ်ကြောင်း", '\n')];

/// Name of the code point brace escape: `{ယူနီကုဒ် HHHH}`.
pub const CODE_POINT_ESCAPE: &str = "ယူနီကုဒ်";

/// Result of a scan.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Scan<T>
{
    /// The scanned value.
    pub value: T,
    /// Offset after the closing delimiter, or of the line end when the
    /// literal is not closed.
    pub end: usize,
    /// Diagnostics found.
    #[builder(default)]
    pub diags: Vec<Diagnostic>,
}

/// Offset of the end of the line that holds `from`.
///
/// # Arguments
///
/// * `text` - The source text.
/// * `from` - An offset in `text`.
///
/// # Returns
///
/// The offset of the next `\n`, or the text length.
pub fn scan_to_eol(text: &str, from: usize) -> usize
{
    text[from..].find('\n').map_or(text.len(), |off| from + off)
}

/// Scan a string literal.
///
/// # Arguments
///
/// * `text` - The source text.
/// * `open` - Offset of the opening `"`.
/// * `base` - Absolute position of `text`.
///
/// # Returns
///
/// The decoded string, the end offset and the diagnostics.
pub fn scan_str(text: &str, open: usize, base: Pos) -> Scan<StrLit>
{
    let mut decoder = Decoder::new(text, open, base, '"');
    let closed = decoder.run();
    report_if_open(&mut decoder, closed, diag::unterminated_string);
    decoder.finish()
}

/// Scan a character literal.
///
/// # Arguments
///
/// * `text` - The source text.
/// * `open` - Offset of the opening `'`.
/// * `base` - Absolute position of `text`.
///
/// # Returns
///
/// The character (`U+FFFD` when the literal does not hold one character), the
/// end offset and the diagnostics.
pub fn scan_char(text: &str, open: usize, base: Pos) -> Scan<char>
{
    let mut decoder = Decoder::new(text, open, base, '\'');
    let closed = decoder.run();
    report_if_open(&mut decoder, closed, diag::unterminated_char);
    let scan = decoder.finish();

    let count = scan.value.value.chars().count();
    let has_bad = scan.value.parts.iter().any(|p| matches!(p, StrPart::Bad(_)));
    let single = scan.value.value.chars().next().filter(|_| count == 1 && !has_bad);
    let mut diags = scan.diags;
    let span = Span::new(base + open, base + scan.end);
    let report = single.is_none() && closed && !has_bad;
    do_if!(report, diags.push(diag::bad_char_length(span, count)));
    Scan::builder()
        .value(single.unwrap_or('\u{FFFD}'))
        .end(scan.end)
        .diags(diags)
        .build()
}

/// Scan a raw identifier: text up to the next backtick on the same line.
///
/// # Arguments
///
/// * `text` - The source text.
/// * `open` - Offset of the opening backtick.
/// * `base` - Absolute position of `text`.
///
/// # Returns
///
/// The end offset and the diagnostics.
pub fn scan_raw_ident(text: &str, open: usize, base: Pos) -> Scan<()>
{
    let eol = scan_to_eol(text, open);
    let close = text[open + 1..eol].find('`').map(|off| open + 1 + off);
    let Some(close) = close
    else
    {
        let span = Span::new(base + open, base + eol);
        let diags = vec![diag::unterminated_raw_identifier(span)];
        return Scan::builder().value(()).end(eol).diags(diags).build();
    };

    let end = close + 1;
    let span = Span::new(base + open, base + end);
    let diags =
        tern!(close == open + 1, vec![diag::blank_raw_identifier(span)], Vec::new());
    Scan::builder().value(()).end(end).diags(diags).build()
}

/// Report an unterminated literal when the decoder did not find the closing
/// quote.
///
/// # Arguments
///
/// * `decoder` - The decoder after [`Decoder::run`].
/// * `closed` - Whether the closing quote was found.
/// * `make` - Builds the diagnostic from the span.
fn report_if_open(decoder: &mut Decoder<'_>, closed: bool, make: fn(Span) -> Diagnostic)
{
    ret_if!(closed);
    let span = Span::new(decoder.base + decoder.open, decoder.base + decoder.pos);
    decoder.diags.push(make(span));
}

/// Escape decoder for string and character literals.
#[derive(Builder)]
struct Decoder<'t>
{
    /// The source text.
    text: &'t str,
    /// Offset of the opening quote.
    open: usize,
    /// Absolute position of `text`.
    base: Pos,
    /// The quote that closes the literal.
    quote: char,
    /// Current offset.
    pos: usize,
    /// Text not pushed as a part so far.
    #[builder(default)]
    buf: String,
    /// Decoded parts.
    #[builder(default)]
    parts: Vec<StrPart>,
    /// Diagnostics found.
    #[builder(default)]
    diags: Vec<Diagnostic>,
}

impl<'t> Decoder<'t>
{
    /// Create a decoder positioned after the opening quote.
    ///
    /// # Arguments
    ///
    /// * `text` - The source text.
    /// * `open` - Offset of the opening quote.
    /// * `base` - Absolute position of `text`.
    /// * `quote` - The quote character.
    ///
    /// # Returns
    ///
    /// The decoder.
    fn new(text: &'t str, open: usize, base: Pos, quote: char) -> Self
    {
        let pos = open + quote.len_utf8();
        Self::builder().text(text).open(open).base(base).quote(quote).pos(pos).build()
    }

    /// The character at the current offset plus `skip` u8s.
    ///
    /// # Arguments
    ///
    /// * `skip` - Number of u8s to skip.
    ///
    /// # Returns
    ///
    /// The character, or `None` at the end of the text.
    fn peek(&self, skip: usize) -> Option<char>
    {
        self.text[self.pos + skip..].chars().next()
    }

    /// Span from `lo` to `hi` (offsets in the text).
    ///
    /// # Arguments
    ///
    /// * `lo` - Start offset.
    /// * `hi` - End offset.
    ///
    /// # Returns
    ///
    /// The absolute span.
    fn span(&self, lo: usize, hi: usize) -> Span
    {
        Span::new(self.base + lo, self.base + hi)
    }

    /// Decode up to the closing quote or the end of the line.
    ///
    /// # Returns
    ///
    /// `true` when the closing quote was found (and consumed).
    fn run(&mut self) -> bool
    {
        loop
        {
            let Some(ch) = self.peek(0)
            else
            {
                return false;
            };
            ret_if!(ch == '\n', false);
            ret_if!(ch == self.quote, self.pos += ch.len_utf8(), true);
            self.step(ch);
        }
    }

    /// Decode one character or escape.
    ///
    /// # Arguments
    ///
    /// * `ch` - The character at the current offset.
    fn step(&mut self, ch: char)
    {
        match ch
        {
            '\\' => self.backslash(),
            '{' => self.open_brace(),
            '}' => self.close_brace(),
            _ =>
            {
                self.buf.push(ch);
                self.pos += ch.len_utf8();
            }
        }
    }

    /// Push the pending text as a part.
    fn flush_text(&mut self)
    {
        ret_if!(self.buf.is_empty());
        self.parts.push(StrPart::Text(std::mem::take(&mut self.buf)));
    }

    /// Push a decoded escape and move past it.
    ///
    /// # Arguments
    ///
    /// * `ch` - The decoded character.
    /// * `len` - Length of the escape in u8s.
    fn push_escape(&mut self, ch: char, len: usize)
    {
        self.flush_text();
        self.parts.push(StrPart::Escape(ch));
        self.pos += len;
    }

    /// Report an unknown escape, keep its text as written, and move past it.
    ///
    /// # Arguments
    ///
    /// * `len` - Length of the escape in u8s.
    fn push_unknown(&mut self, len: usize)
    {
        let raw = &self.text[self.pos..self.pos + len];
        self.diags.push(diag::unknown_escape(self.span(self.pos, self.pos + len), raw));
        self.flush_text();
        self.parts.push(StrPart::Bad(raw.to_string()));
        self.pos += len;
    }

    /// Keep a `{...}` of a string literal as a template slot and move past it.
    ///
    /// # Arguments
    ///
    /// * `close` - Offset of the closing `}`.
    fn push_template_slot(&mut self, close: usize)
    {
        let text = self.text[self.pos + 1..close].to_string();
        let span = self.span(self.pos + 1, close);
        self.flush_text();
        self.parts.push(StrPart::TemplateSlot { text, span });
        self.pos = close + 1;
    }

    /// Decode a backslash escape.
    fn backslash(&mut self)
    {
        let next = self.peek(1);
        ret_if!(let Some(ch) = next.and_then(simple_escape), self.push_escape(ch, 2));
        ret_if!(next == Some('u'), self.unicode_escape());
        let len = 1 + next.filter(|ch| *ch != '\n').map_or(0, char::len_utf8);
        self.push_unknown(len);
    }

    /// Decode `\u` with 4 or 8 hex digits after it.
    fn unicode_escape(&mut self)
    {
        let digits: String = self.text[self.pos + 2..]
            .chars()
            .take_while(char::is_ascii_hexdigit)
            .take(8)
            .collect();
        let len = 2 + digits.len();
        let decoded = code_point(&digits);
        ret_if!(let Some(ch) = decoded, self.push_escape(ch, len));
        self.push_unknown(len);
    }

    /// Decode `{{`, a `{name}` escape, or a template slot. A template slot ends
    /// at the `}` that matches its `{` on the same line, so it can hold a
    /// string literal with template slots of its own; in a character
    /// literal it is an unknown escape.
    fn open_brace(&mut self)
    {
        ret_if!(self.peek(1) == Some('{'), self.buf.push('{'), self.pos += 2);
        let eol = scan_to_eol(self.text, self.pos);
        let close = template_slot_end(&self.text[..eol], self.pos + 1);
        let Some(close) = close
        else
        {
            self.push_unknown(1);
            return;
        };

        let content = &self.text[self.pos + 1..close];
        let len = close + 1 - self.pos;
        ret_if!(let Some(ch) = named_escape(content), self.push_escape(ch, len));
        ret_if!(self.quote == '"', self.push_template_slot(close));
        self.push_unknown(len);
    }

    /// Decode `}}`; a lone `}` is an unknown escape.
    fn close_brace(&mut self)
    {
        ret_if!(self.peek(1) == Some('}'), self.buf.push('}'), self.pos += 2);
        self.push_unknown(1);
    }

    /// Build the scan result.
    ///
    /// # Returns
    ///
    /// The decoded string with its end offset and diagnostics.
    fn finish(mut self) -> Scan<StrLit>
    {
        self.flush_text();
        let value = self.parts.iter().map(part_text).collect();
        let lit = StrLit::builder().parts(self.parts).value(value).build();
        Scan::builder().value(lit).end(self.pos).diags(self.diags).build()
    }
}

/// The offset of the `}` that closes a template slot: string and character
/// literals inside it are skipped whole, and a nested `{...}` is matched in
/// turn.
///
/// # Arguments
///
/// * `text` - The source text up to the end of the line.
/// * `from` - The offset just after the `{` of the template slot.
///
/// # Returns
///
/// The offset of the closing `}`, or `None` when the line ends first.
fn template_slot_end(text: &str, from: usize) -> Option<usize>
{
    let mut at = from;
    while let Some(ch) = text[at..].chars().next()
    {
        at = match ch
        {
            '}' => return Some(at),
            '{' => template_slot_end(text, at + 1)? + 1,
            '"' | '\'' => quoted_end(text, at + 1, ch)?,
            _ => at + ch.len_utf8(),
        };
    }
    None
}

/// The offset just past the closing quote of a literal inside a template slot:
/// a backslash escape, `{{` and `}}` are skipped, and each `{...}` of the
/// literal is matched with [`template_slot_end`].
///
/// # Arguments
///
/// * `text` - The source text up to the end of the line.
/// * `from` - The offset just after the opening quote.
/// * `quote` - The quote that closes the literal.
///
/// # Returns
///
/// The offset after the closing quote, or `None` when the line ends first.
fn quoted_end(text: &str, from: usize, quote: char) -> Option<usize>
{
    let mut at = from;
    while let Some(ch) = text[at..].chars().next()
    {
        let next = text[at + ch.len_utf8()..].chars().next();
        at = match ch
        {
            _ if ch == quote => return Some(at + 1),
            '\\' => at + 1 + next.map_or(0, char::len_utf8),
            '{' | '}' if next == Some(ch) => at + 2,
            '{' => template_slot_end(text, at + 1)? + 1,
            _ => at + ch.len_utf8(),
        };
    }
    None
}

/// Decode a one-character backslash escape.
///
/// # Arguments
///
/// * `ch` - The character after the backslash.
///
/// # Returns
///
/// The decoded character, or `None`.
fn simple_escape(ch: char) -> Option<char>
{
    match ch
    {
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        '0' => Some('\0'),
        '\\' | '\'' | '"' => Some(ch),
        _ => None,
    }
}

/// Decode 4 or 8 hex digits into a character.
///
/// # Arguments
///
/// * `digits` - The hex digits.
///
/// # Returns
///
/// The character, or `None` for another digit count, a surrogate, or a value
/// above `U+10FFFF`.
fn code_point(digits: &str) -> Option<char>
{
    ret_if!(digits.len() != 4 && digits.len() != 8, None);
    u32::from_str_radix(digits, 16).ok().and_then(char::from_u32)
}

/// Decode the content of a `{...}` escape.
///
/// # Arguments
///
/// * `content` - The text between the braces.
///
/// # Returns
///
/// The character for a known name or a `ယူနီကုဒ် HHHH` code point, else `None`.
fn named_escape(content: &str) -> Option<char>
{
    let named = NAMED_ESCAPES.iter().find(|(name, _)| *name == content);
    ret_if!(let Some((_, ch)) = named, Some(*ch));
    let digits = content.strip_prefix(CODE_POINT_ESCAPE)?.strip_prefix(' ')?;
    code_point(digits)
}

/// Text of a part in the joined value.
///
/// # Arguments
///
/// * `part` - The part.
///
/// # Returns
///
/// The text; a template slot is kept as `{content}`, a bad escape as written.
fn part_text(part: &StrPart) -> String
{
    match part
    {
        StrPart::Text(text) | StrPart::Bad(text) => text.clone(),
        StrPart::Escape(ch) => ch.to_string(),
        StrPart::TemplateSlot { text, .. } => format!("{{{text}}}"),
    }
}
