use bon::Builder;
use zakac_burmese_segmenter::segmenter::SegmenterOptions;
use zakac_burmese_segmenter::segmenter::segment_at;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::lexer as diag;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_while;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::pos::Pos;
use zakac_span::source_file::SourceFile;
use zakac_span::span::Span;

use crate::ident::is_ident;
use crate::number::lex_number;
use crate::number::word_script;
use crate::phrases::join_phrases;
use crate::reserved::NOTE_MARKER;
use crate::reserved::lookup;
use crate::reserved::split_reserved;
use crate::text::Scan;
use crate::text::scan_char;
use crate::text::scan_raw_ident;
use crate::text::scan_str;
use crate::text::scan_to_eol;
use crate::token::Comment;
use crate::token::CommentKind;
use crate::token::Kw;
use crate::token::LexOutput;
use crate::token::Punct;
use crate::token::Token;
use crate::token::TokenKind;
use crate::words::Word;
use crate::words::WordKind;
use crate::words::build_words;

/// Options for the lexer.
#[derive(Debug, Clone, Builder)]
pub struct LexerOptions
{
    /// Maximum look-ahead limit passed to the segmenter.
    #[builder(default = 16)]
    pub max_lookahead_limit: usize,
}

impl Default for LexerOptions
{
    fn default() -> Self { Self::builder().build() }
}

/// Lex a source file.
///
/// # Arguments
///
/// * `source` - The source file; its text is normalized on load.
/// * `options` - The lexer options.
///
/// # Returns
///
/// Tokens (ending with [`TokenKind::Eof`]), comments, strings and diagnostics,
/// with absolute spans.
pub fn lex(source: &SourceFile, options: &LexerOptions) -> LexOutput
{
    lex_at(&source.src, source.start_pos, options)
}

/// Lex `text` that starts at the absolute position `base`.
///
/// # Arguments
///
/// * `text` - The normalized source text.
/// * `base` - Absolute position of `text`.
/// * `options` - The lexer options.
///
/// # Returns
///
/// Tokens (ending with [`TokenKind::Eof`]), comments, strings and diagnostics.
pub fn lex_at(text: &str, base: Pos, options: &LexerOptions) -> LexOutput
{
    let seg_options = SegmenterOptions::builder()
        .max_lookahead_limit(options.max_lookahead_limit)
        .coalesce_non_burmese_chars(false)
        .build();
    let pre = segment_at(text, base, &seg_options);
    let words = build_words(text, base, &pre);
    let mut driver = Driver::builder().text(text).base(base).words(&words).build();
    driver.run();
    driver.finish()
}

/// Punctuation for a single delimiter character.
///
/// # Arguments
///
/// * `ch` - The character.
///
/// # Returns
///
/// The punctuation, or `None` for a character that is no token on its own.
fn single_punct(ch: char) -> Option<Punct>
{
    let punct = match ch
    {
        '၊' => Punct::ListSep,
        '။' => Punct::FullStop,
        '၏' => Punct::Ei,
        '(' => Punct::LParen,
        ')' => Punct::RParen,
        '+' => Punct::Plus,
        '-' => Punct::Minus,
        '*' => Punct::Star,
        '/' => Punct::Slash,
        '%' => Punct::Percent,
        '&' => Punct::Amp,
        '|' => Punct::Pipe,
        '^' => Punct::Caret,
        '~' => Punct::Tilde,
        '!' => Punct::Bang,
        '<' => Punct::Lt,
        '>' => Punct::Gt,
        _ => return None,
    };
    Some(punct)
}

/// Punctuation for two joined delimiter characters.
///
/// # Arguments
///
/// * `first` - The first character.
/// * `second` - The second character.
///
/// # Returns
///
/// The two-character operator, or `None`.
fn pair_punct(first: char, second: char) -> Option<Punct>
{
    let punct = match (first, second)
    {
        ('<', '=') => Punct::Le,
        ('>', '=') => Punct::Ge,
        ('=', '=') => Punct::EqEq,
        ('!', '=') => Punct::Ne,
        ('<', '<') => Punct::Shl,
        ('>', '>') => Punct::Shr,
        ('&', '&') => Punct::AndAnd,
        ('|', '|') => Punct::OrOr,
        _ => return None,
    };
    Some(punct)
}

/// The classifier loop over words.
#[derive(Builder)]
struct Driver<'w, 's>
{
    /// The source text.
    text: &'s str,
    /// Absolute position of `text`.
    base: Pos,
    /// The words of `text`.
    words: &'w [Word<'s>],
    /// Output built so far.
    #[builder(default)]
    out: LexOutput,
}

impl Driver<'_, '_>
{
    /// Run the classifier over all words.
    fn run(&mut self)
    {
        let mut idx = 0;
        do_while!(idx < self.words.len(), idx = self.step(idx));
    }

    /// Join phrases and append the end-of-input token.
    ///
    /// # Returns
    ///
    /// The lexer output.
    fn finish(mut self) -> LexOutput
    {
        self.out.tokens = join_phrases(std::mem::take(&mut self.out.tokens));
        let end = self.base + self.text.len();
        self.push(TokenKind::Eof, Span::new(end, end), false);
        self.out
    }

    /// Lex the token(s) that start at `words[idx]`. The tests run in this
    /// order: comments, quoted literals, punctuation, reserved words, numbers,
    /// identifiers.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the current word.
    ///
    /// # Returns
    ///
    /// Index of the next word to lex.
    fn step(&mut self, idx: usize) -> usize
    {
        ret_if!(let Some(next) = self.lex_comment(idx), next);
        ret_if!(let Some(next) = self.lex_quoted(idx), next);
        ret_if!(let Some(next) = self.lex_punct(idx), next);
        ret_if!(let Some(next) = self.lex_reserved(idx), next);
        ret_if!(let Some(next) = self.lex_digits(idx), next);
        ret_if!(let Some(next) = self.lex_ident(idx), next);
        self.invalid(idx)
    }

    /// Offset of an absolute position in the text.
    ///
    /// # Arguments
    ///
    /// * `pos` - The absolute position.
    ///
    /// # Returns
    ///
    /// The offset.
    fn offset(&self, pos: Pos) -> usize { (pos - self.base) as usize }

    /// Absolute span of the offsets `lo..hi`.
    ///
    /// # Arguments
    ///
    /// * `lo` - Start offset.
    /// * `hi` - End offset.
    ///
    /// # Returns
    ///
    /// The span.
    fn span(&self, lo: usize, hi: usize) -> Span
    {
        Span::new(self.base + lo, self.base + hi)
    }

    /// Index of the first word at or after `idx` that starts at or after the
    /// offset `end`. Used after a scan of the source text.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index to start from.
    /// * `end` - Offset where the scan stopped.
    ///
    /// # Returns
    ///
    /// The word index.
    fn skip_to(&self, idx: usize, end: usize) -> usize
    {
        let limit = self.base + end;
        idx + self.words[idx..].iter().take_while(|w| w.span.lo < limit).count()
    }

    /// Whether `words[idx]` is the delimiter `ch`, joined to the previous word.
    ///
    /// # Arguments
    ///
    /// * `idx` - The word index.
    /// * `ch` - The delimiter.
    ///
    /// # Returns
    ///
    /// `true` if it is.
    fn joined_delim(&self, idx: usize, ch: char) -> bool
    {
        self.words.get(idx).is_some_and(|w| w.joined && w.delim() == Some(ch))
    }

    /// Push a token.
    ///
    /// # Arguments
    ///
    /// * `kind` - The token kind.
    /// * `span` - The token span.
    /// * `joined` - Whether the token is joined to the previous one.
    fn push(&mut self, kind: TokenKind, span: Span, joined: bool)
    {
        self.out
            .tokens
            .push(Token::builder().kind(kind).span(span).joined(joined).build());
    }

    /// Push a reserved word, or absorb it into the previous token when it is
    /// an attached sentence-final particle.
    ///
    /// Attached particles alone are absorbed: in `ညီ သည်` the `သည်` is a
    /// subject marker, as in a chained comparison.
    ///
    /// # Arguments
    ///
    /// * `kw` - The reserved word.
    /// * `span` - Its span.
    /// * `joined` - Whether it is joined to the previous token.
    fn push_kw(&mut self, kw: Kw, span: Span, joined: bool)
    {
        ret_if!(joined && self.absorb(kw, span));
        self.push(TokenKind::Kw(kw), span, joined);
    }

    /// Absorb a particle into the previous token when that token takes one.
    ///
    /// # Arguments
    ///
    /// * `kw` - The particle candidate.
    /// * `span` - Its span.
    ///
    /// # Returns
    ///
    /// `true` when the particle was absorbed.
    fn absorb(&mut self, kw: Kw, span: Span) -> bool
    {
        let Some(last) = self.out.tokens.last_mut()
        else
        {
            return false;
        };
        let TokenKind::Kw(prev) = last.kind
        else
        {
            return false;
        };
        let particle = last.particle.with(kw).filter(|_| prev.absorbs_particle());
        let Some(particle) = particle
        else
        {
            return false;
        };
        last.particle = particle;
        last.span = last.span.to(&span);
        true
    }

    /// Lex a comment: `//`, `///` or the word `မှတ်ချက်`, to the end of the
    /// line.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the current word.
    ///
    /// # Returns
    ///
    /// The next word index, or `None` when the word is not a comment marker.
    fn lex_comment(&mut self, idx: usize) -> Option<usize>
    {
        let kind = self.comment_kind(idx)?;
        let lo = self.offset(self.words[idx].span.lo);
        let end = scan_to_eol(self.text, lo);
        self.out
            .comments
            .push(Comment::builder().kind(kind).span(self.span(lo, end)).build());
        Some(self.skip_to(idx, end))
    }

    /// Comment marker at `words[idx]`.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the current word.
    ///
    /// # Returns
    ///
    /// The marker kind, or `None`.
    fn comment_kind(&self, idx: usize) -> Option<CommentKind>
    {
        let word = &self.words[idx];
        let note = word.kind == WordKind::Burmese && word.text == NOTE_MARKER;
        ret_if!(note, Some(CommentKind::Note));
        ret_if!(word.delim() != Some('/') || !self.joined_delim(idx + 1, '/'), None);
        let doc = self.joined_delim(idx + 2, '/');
        tern!(doc, Some(CommentKind::DocSlash), Some(CommentKind::Slash))
    }

    /// Lex a string, character or raw identifier literal.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the current word.
    ///
    /// # Returns
    ///
    /// The next word index, or `None` when the word is not an opening quote.
    fn lex_quoted(&mut self, idx: usize) -> Option<usize>
    {
        let open = self.offset(self.words[idx].span.lo);
        match self.words[idx].delim()?
        {
            '"' => Some(self.lex_string(idx, open)),
            '\'' => Some(self.lex_char(idx, open)),
            '`' => Some(self.lex_raw_ident(idx, open)),
            _ => None,
        }
    }

    /// Lex a string literal.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the opening quote word.
    /// * `open` - Offset of the opening quote.
    ///
    /// # Returns
    ///
    /// The next word index.
    fn lex_string(&mut self, idx: usize, open: usize) -> usize
    {
        let scan = scan_str(self.text, open, self.base);
        let str_idx = self.out.strings.len() as u32;
        self.out.strings.push(scan.value);
        self.push_scanned(idx, TokenKind::Str(str_idx), open, scan.end, scan.diags)
    }

    /// Lex a character literal.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the opening quote word.
    /// * `open` - Offset of the opening quote.
    ///
    /// # Returns
    ///
    /// The next word index.
    fn lex_char(&mut self, idx: usize, open: usize) -> usize
    {
        let Scan { value, end, diags } = scan_char(self.text, open, self.base);
        self.push_scanned(idx, TokenKind::Char(value), open, end, diags)
    }

    /// Lex a raw identifier.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the opening backtick word.
    /// * `open` - Offset of the opening backtick.
    ///
    /// # Returns
    ///
    /// The next word index.
    fn lex_raw_ident(&mut self, idx: usize, open: usize) -> usize
    {
        let scan = scan_raw_ident(self.text, open, self.base);
        self.push_scanned(idx, TokenKind::RawIdent, open, scan.end, scan.diags)
    }

    /// Push a token found in a text scan and skip the words it covers.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the opening word.
    /// * `kind` - The token kind.
    /// * `open` - Offset of the opening delimiter.
    /// * `end` - Offset where the scan stopped.
    /// * `diags` - Diagnostics from the scan.
    ///
    /// # Returns
    ///
    /// The next word index.
    fn push_scanned(
        &mut self, idx: usize, kind: TokenKind, open: usize, end: usize,
        diags: Vec<Diagnostic>,
    ) -> usize
    {
        self.push(kind, self.span(open, end), self.words[idx].joined);
        self.out.diagnostics.extend(diags);
        self.skip_to(idx, end)
    }

    /// Lex punctuation, merging two-character operators.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the current word.
    ///
    /// # Returns
    ///
    /// The next word index, or `None` when the word is not a delimiter.
    fn lex_punct(&mut self, idx: usize) -> Option<usize>
    {
        let word = self.words[idx];
        let ch = word.delim()?;
        let next = self.words.get(idx + 1).filter(|w| w.joined).and_then(Word::delim);
        let pair = next.and_then(|second| pair_punct(ch, second));
        ret_if!(let Some(punct) = pair, Some(self.push_punct(idx, idx + 2, punct)));
        ret_if!(let Some(punct) = single_punct(ch), Some(self.push_punct(idx, idx + 1, punct)));
        Some(self.invalid(idx))
    }

    /// Push a punctuation token made of `words[idx..end]`.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the first word.
    /// * `end` - Index one past the last word.
    /// * `punct` - The punctuation.
    ///
    /// # Returns
    ///
    /// `end`.
    fn push_punct(&mut self, idx: usize, end: usize, punct: Punct) -> usize
    {
        let span = self.words[idx].span.to(&self.words[end - 1].span);
        self.push(TokenKind::Punct(punct), span, self.words[idx].joined);
        end
    }

    /// Lex a reserved word, or a word made of reserved words.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the current word.
    ///
    /// # Returns
    ///
    /// The next word index, or `None` when the word is not reserved.
    fn lex_reserved(&mut self, idx: usize) -> Option<usize>
    {
        let word = self.words[idx];
        ret_if!(let Some(kw) = lookup(word.text), self.push_kw(kw, word.span, word.joined), Some(idx + 1));
        let pieces = split_reserved(&word)?;
        #[rustfmt::skip]
        do_for!((n, (kw, span)) in pieces.into_iter().enumerate(), self.push_kw(kw, span, n > 0 || word.joined));
        Some(idx + 1)
    }

    /// Lex a word that starts with a digit.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the current word.
    ///
    /// # Returns
    ///
    /// The next word index, or `None` when the word does not start with a
    /// digit.
    fn lex_digits(&mut self, idx: usize) -> Option<usize>
    {
        word_script(self.words[idx].text)?;
        let lexed = lex_number(self.words, idx);
        self.out.tokens.extend(lexed.token);
        self.out.diagnostics.extend(lexed.diags);
        Some(lexed.next)
    }

    /// Lex an identifier.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the current word.
    ///
    /// # Returns
    ///
    /// The next word index, or `None` when the word is not an identifier.
    fn lex_ident(&mut self, idx: usize) -> Option<usize>
    {
        let word = self.words[idx];
        ret_if!(!is_ident(word.text), None);
        self.push(TokenKind::Ident, word.span, word.joined);
        Some(idx + 1)
    }

    /// Report a word that is no token and skip it.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the current word.
    ///
    /// # Returns
    ///
    /// The next word index.
    fn invalid(&mut self, idx: usize) -> usize
    {
        let word = self.words[idx];
        self.out.diagnostics.push(diag::invalid_character(word.span, word.text));
        idx + 1
    }
}
