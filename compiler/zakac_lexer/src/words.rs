use bon::Builder;
use zakac_burmese_segmenter::segmenter::PreLexicalToken;
use zakac_burmese_segmenter::segmenter::PreLexicalTokenKind;
use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::pos::Pos;
use zakac_span::span::Span;

use crate::ident::is_self_delimiting;
use crate::ident::is_whitespace;
use crate::reserved::lookup;
use crate::reserved::split_reserved;

/// Script class of a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordKind
{
    /// Burmese vannas and digits.
    Burmese,
    /// Other characters.
    Foreign,
    /// A self-delimiting character, which is a word on its own.
    Delim(char),
}

/// A maximal run of pre-tokens with no whitespace, no self-delimiting
/// character and no change of script inside; the `၏` that ends a reserved
/// word, as in `ဖြစ်၏`, remains in the word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct Word<'s>
{
    /// The pre-tokens of the word; their spans give the vanna boundaries.
    pub pre: &'s [PreLexicalToken<'s>],
    /// The word text.
    pub text: &'s str,
    /// Source region of the word.
    pub span: Span,
    /// Script class.
    pub kind: WordKind,
    /// `true` when no whitespace separates this word from the previous one.
    pub joined: bool,
}

impl Word<'_>
{
    /// The delimiter character of a [`WordKind::Delim`] word.
    ///
    /// # Returns
    ///
    /// The character, or `None` for other words.
    pub fn delim(&self) -> Option<char>
    {
        tern!(let WordKind::Delim(ch) = self.kind, Some(ch), None)
    }
}

/// Class of one pre-token for word building.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreClass
{
    /// Whitespace; ends the current word.
    Space,
    /// Part of a word of this kind.
    Word(WordKind),
}

/// Group pre-tokens into words.
///
/// # Arguments
///
/// * `text` - The source text the pre-tokens were segmented from.
/// * `base` - Absolute position of `text`.
/// * `pre` - The pre-tokens, segmented with non-Burmese coalescing off.
///
/// # Returns
///
/// The words in source order.
pub fn build_words<'s>(
    text: &'s str, base: Pos, pre: &'s [PreLexicalToken<'s>],
) -> Vec<Word<'s>>
{
    let mut grouper = Grouper::builder().text(text).base(base).pre(pre).build();
    (0..pre.len()).for_each(|idx| grouper.feed(idx));
    grouper.flush(pre.len());
    grouper.words
}

/// State of the word builder.
#[derive(Builder)]
struct Grouper<'s>
{
    /// The source text.
    text: &'s str,
    /// Absolute position of `text`.
    base: Pos,
    /// All pre-tokens.
    pre: &'s [PreLexicalToken<'s>],
    /// Words built so far.
    #[builder(default)]
    words: Vec<Word<'s>>,
    /// Start index and kind of the word being built.
    run: Option<(usize, WordKind)>,
    /// Whether the next word is joined to the previous one.
    #[builder(default)]
    joined: bool,
}

impl<'s> Grouper<'s>
{
    /// Take one pre-token.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the pre-token.
    fn feed(&mut self, idx: usize)
    {
        let class = class_of(&self.pre[idx]);
        ret_if!(class == PreClass::Space, self.flush(idx), self.joined = false);
        extract!(PreClass::Word(kind) = class);
        if let WordKind::Delim(_) = kind
        {
            ret_if!(self.ends_reserved(idx), self.flush(idx + 1));
            self.flush(idx);
            self.push(idx, idx + 1, kind);
            return;
        }
        ret_if!(matches!(self.run, Some((_, run_kind)) if run_kind == kind));
        self.flush(idx);
        self.run = Some((idx, kind));
    }

    /// Whether the delimiter `pre[idx]` is the `၏` that ends a reserved word
    /// with the Burmese word being built, as in `ဖြစ်၏`. A `၏` after each
    /// other word is "of", a word of its own: `ကျော်၏`.
    ///
    /// # Arguments
    ///
    /// * `idx` - Index of the delimiter.
    ///
    /// # Returns
    ///
    /// `true` if the word with the `၏` reads as reserved words.
    fn ends_reserved(&self, idx: usize) -> bool
    {
        let Some((start, WordKind::Burmese)) = self.run
        else
        {
            return false;
        };
        ret_false!(self.pre[idx].lexeme != "၏");
        let word = self.word(start, idx + 1, WordKind::Burmese);
        lookup(word.text).is_some() || split_reserved(&word).is_some()
    }

    /// End the word being built, if one is.
    ///
    /// # Arguments
    ///
    /// * `end` - Index one past the last pre-token of the word.
    fn flush(&mut self, end: usize)
    {
        let Some((start, kind)) = self.run.take()
        else
        {
            return;
        };
        self.push(start, end, kind);
    }

    /// Push a word made of `pre[start..end]`.
    ///
    /// # Arguments
    ///
    /// * `start` - Index of the first pre-token.
    /// * `end` - Index one past the last pre-token.
    /// * `kind` - Script class of the word.
    fn push(&mut self, start: usize, end: usize, kind: WordKind)
    {
        let word = self.word(start, end, kind);
        self.words.push(word);
        self.joined = true;
    }

    /// The word made of `pre[start..end]`, joined as the next word is.
    ///
    /// # Arguments
    ///
    /// * `start` - Index of the first pre-token.
    /// * `end` - Index one past the last pre-token.
    /// * `kind` - Script class of the word.
    ///
    /// # Returns
    ///
    /// The word.
    fn word(&self, start: usize, end: usize, kind: WordKind) -> Word<'s>
    {
        let span = Span::new(self.pre[start].span.lo, self.pre[end - 1].span.hi);
        let lo = (span.lo - self.base) as usize;
        let hi = (span.hi - self.base) as usize;
        let pre = &self.pre[start..end];
        let word =
            Word::builder().pre(pre).text(&self.text[lo..hi]).span(span).kind(kind);
        word.joined(self.joined).build()
    }
}

/// Find the class of a pre-token for word building.
///
/// # Arguments
///
/// * `tok` - The pre-token.
///
/// # Returns
///
/// Whitespace, or the kind of word the pre-token belongs to.
fn class_of(tok: &PreLexicalToken<'_>) -> PreClass
{
    let mut chars = tok.lexeme.chars();
    let single = chars.next().filter(|_| chars.next().is_none());
    ret_if!(let Some(ch) = single && is_whitespace(ch), PreClass::Space);
    ret_if!(let Some(ch) = single && is_self_delimiting(ch), PreClass::Word(WordKind::Delim(ch)));
    let foreign = tok.kind == PreLexicalTokenKind::NonBurmeseChar;
    ret_if!(foreign, PreClass::Word(WordKind::Foreign));
    PreClass::Word(WordKind::Burmese)
}
