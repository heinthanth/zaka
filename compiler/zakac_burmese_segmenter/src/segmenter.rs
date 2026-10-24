use bon::Builder;
use const_for::const_for;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoStaticStr;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;
use zakac_span::pos::Pos;
use zakac_span::span::Span;

/// Kind of pre-lexical token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PreLexicalTokenKind
{
    /// ဝဏ္ဏ in Burmese
    BurmeseVanna,
    /// Burmese digit
    BurmeseDigit,
    /// Other non-Burmese characters
    NonBurmeseChar,
}

/// A pre-lexical token is a unit of input for a lexer.
/// Lexer lexes one or more of them into lexical tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct PreLexicalToken<'s>
{
    /// Kind of the token.
    pub kind: PreLexicalTokenKind,
    /// Lexeme of the token.
    pub lexeme: &'s str,
    /// Source span of the token.
    pub span: Span,
}

/// Options for the segment function.
#[derive(Builder)]
pub struct SegmenterOptions
{
    /// Maximum look-ahead limit.
    #[builder(default = 16)]
    pub max_lookahead_limit: usize,
    /// Whether to coalesce non-Burmese characters into a single token.
    #[builder(default)]
    pub coalesce_non_burmese_chars: bool,
}

impl Default for SegmenterOptions
{
    fn default() -> Self { Self::builder().build() }
}

/// Segment the input text into pre-lexical tokens.
///
/// # Arguments
///
/// * `text`: The input text to segment.
/// * `options`: The options for the segmenter.
///
/// # Returns
///
/// The sequence of pre-lexical tokens.
pub fn segment<'s>(
    text: &'s str, options: &SegmenterOptions,
) -> Vec<PreLexicalToken<'s>>
{
    segment_at(text, Pos::ZERO, options)
}

/// Segment the input text into pre-lexical tokens at absolute u8 positions.
///
/// # Arguments
///
/// * `text`: The input text to segment.
/// * `base_pos`: Absolute start position of `text` in the source map.
/// * `options`: The options for the segmenter.
///
/// # Returns
///
/// The sequence of pre-lexical tokens.
pub fn segment_at<'s>(
    text: &'s str, base_pos: Pos, options: &SegmenterOptions,
) -> Vec<PreLexicalToken<'s>>
{
    let mut chars = text.char_indices();
    let Some((_, first_ch)) = chars.next()
    else
    {
        return Vec::new();
    };

    let mut prev = bur_class(first_ch as u32);
    let mut start = 0;
    let mut s_class = prev;

    // Educated guess:
    // One vanna is often 1–3 chars, so fewer tokens than chars.
    // ASCII characters with no coalesce option will have 1 token per character.
    // So, half the length of the text plus 1 is a good middle ground.
    let mut tokens = Vec::with_capacity(text.len() / 2 + 1);

    // One-character lookahead buffer: `(offset, code point, class)`.
    let mut p = decode_next(&mut chars);

    while let Some((off, _, curr)) = p
    {
        p = decode_next(&mut chars);
        #[rustfmt::skip]
        handle_break(&mut tokens, text, base_pos, &mut start, &mut s_class, prev, off, curr, p, options);
        prev = curr;
    }
    emit(&mut tokens, text, base_pos, start, s_class, text.len());
    tokens
}

/// First code point of the coburon Burmese character set.
const BUR_FIRST: u32 = 0x1000;
/// Number of code points in `U+1000..=U+104F`.
const BUR_COUNT: usize = (0x104F - 0x1000 + 1) as usize;
/// Dot below (့), the one sign allowed between a killed consonant and its asat.
const DOT_BELOW: u32 = 0x1037;
/// Compile-time class table for `U+1000..=U+104F`.
const BUR_CLASS: [u8; BUR_COUNT] = build_bur_class_table();

/// Build the class table for `U+1000..=U+104F` at compile time.
const fn build_bur_class_table() -> [u8; BUR_COUNT]
{
    let mut table = [0u8; BUR_COUNT];
    const_for!(i in 0..BUR_COUNT => { table[i] = bur_flags(BUR_FIRST + i as u32); });
    table
}

const F_FOREIGN: u8 = 1 << 0;
const F_DIGIT: u8 = 1 << 1;
const F_VIRAMA: u8 = 1 << 2;
const F_ASAT: u8 = 1 << 3;
const F_MODIFIER: u8 = 1 << 4;
const F_START_VANNA: u8 = 1 << 5;
const F_STANDALONE: u8 = 1 << 6;
const F_U: u8 = 1 << 7;
const VISARGA: u32 = 0x1038;

/// Returns `flag` if `cond` is `true`, otherwise `0`.
///
/// # Arguments
///
/// * `cond`: The condition to check.
/// * `flag`: The flag to return if `cond` is `true`.
///
/// # Returns
///
/// The flag if `cond` is `true`, otherwise `0`.
const fn bit(cond: bool, flag: u8) -> u8 { if cond { flag } else { 0 } }

/// Convert a code point to a class.
///
/// # Arguments
///
/// * `cp`: The code point to convert.
///
/// # Returns
///
/// A bitmask of flags indicating the class of the code point.
const fn bur_flags(cp: u32) -> u8
{
    bit(is_foreign_char(cp), F_FOREIGN)
        | bit(is_burmese_digit(cp), F_DIGIT)
        | bit(is_virama(cp), F_VIRAMA)
        | bit(is_asat(cp), F_ASAT)
        | bit(is_burmese_modifier(cp), F_MODIFIER)
        | bit(is_vanna_starter(cp), F_START_VANNA)
        | bit(can_standalone(cp), F_STANDALONE)
        | bit(cp == 0x1026, F_U)
}

/// Look up the class of `cp`.
///
/// # Arguments
///
/// * `cp`: The code point to look up.
///
/// # Returns
///
/// The class of the code point.
#[inline]
fn bur_class(cp: u32) -> u8
{
    let idx = cp.wrapping_sub(BUR_FIRST);
    tern!(idx < BUR_COUNT as u32, BUR_CLASS[idx as usize], F_FOREIGN)
}

/// Check if a character is a Burmese digit.
///
/// # Arguments
///
/// * `cp`: The code point of the character to check.
///
/// # Returns
///
/// `true` if the character is a Burmese digit, `false` otherwise.
#[inline]
const fn is_burmese_digit(cp: u32) -> bool { cp >= 0x1040 && cp <= 0x1049 }

/// Check if a character is a Virama (္).
///
/// # Arguments
///
/// * `cp`: The code point of the character to check.
///
/// # Returns
///
/// `true` if the character is a Virama, `false` otherwise.
#[inline]
const fn is_virama(cp: u32) -> bool { cp == 0x1039 }

/// Check if a character is a Asat (်).
///
/// # Arguments
///
/// * `cp`: The code point of the character to check.
///
/// # Returns
///
/// `true` if the character is a Asat, `false` otherwise.
#[inline]
const fn is_asat(cp: u32) -> bool { cp == 0x103A }

/// Check if a character is a foreign character (i.e. outside of coburon Burmese
/// character set).
///
/// # Arguments
///
/// * `cp`: The code point of the character to check.
///
/// # Returns
///
/// `true` if the character is a foreign character, `false` otherwise.
#[inline]
const fn is_foreign_char(cp: u32) -> bool
{
    // characters outside of Burmese character set
    cp < 0x1000 || cp > 0x104F ||
    // Shan A (ဢ) and Mon E (ဨ)
    cp == 0x1022 || cp == 0x1028 ||
    // Mon vowel signs (◌ဳ, ◌ဴ, ◌ဵ)
    cp == 0x1033 || cp == 0x1034 || cp == 0x1035
}

/// Check if a character is a Burmese independent vowel, medials or tone mark.
///
/// # Arguments
///
/// * `cp`: The code point of the character to check.
///
/// # Returns
///
/// `true` if the character is a Burmese independent vowel, medials or tone
/// mark, `false` otherwise.
#[inline]
const fn is_burmese_modifier(cp: u32) -> bool
{
    // vowel signs excluding virama and asat
    (cp >= 0x102B && cp <= 0x1038) ||
    // medial diacritics
    (cp >= 0x103B && cp <= 0x103E)
}

/// Check if a character can start a vanna without looking at the next
/// character. ဦ is handled via [`F_U`].
///
/// # Arguments
///
/// * `cp`: The code point of the character to check.
///
/// # Returns
///
/// `true` if the character can start a vanna, `false` otherwise.
#[inline]
const fn is_vanna_starter(cp: u32) -> bool
{
    // Burmese consonants
    (cp >= 0x1000 && cp <= 0x1021) ||
    // Burmese independent vowels (ဣ, ဥ)
    cp == 0x1023 || cp == 0x1025 ||
    // Burmese Aforementioned characters - can accept -င်း vowel
    cp == 0x104E
}

/// Check if a character can stand alone.
///
/// # Arguments
///
/// * `cp`: The code point of the character to check.
///
/// # Returns
///
/// `true` if the character can stand alone, `false` otherwise.
#[inline]
const fn can_standalone(cp: u32) -> bool
{
    // Burmese independent vowels (ဤ, ဧ)
    cp == 0x1024 || cp == 0x1027 ||
    // Burmese independent vowels (ဩ, ဪ)
    cp == 0x1029 || cp == 0x102A ||
    // Burmese punctuation (၊, ။)
    cp == 0x104A || cp == 0x104B ||
    // Burmese alphabet (၌, ၍)
    cp == 0x104C || cp == 0x104D ||
    // Burmese independent vowel (၏)
    cp == 0x104F
}

/// Check if an Asat or Virama follows the base (within the modifier span). If
/// so, this base is a killed / stacked consonant and must NOT be split off from
/// the preceding vanna.
///
/// # Arguments
///
/// * `rest`: The remainder of the input after the base being checked.
/// * `first`: The class of the first character of `rest`
/// * `max_lookahead`: The maximum look-ahead limit.
///
/// # Returns
///
/// `true` if the base has a structural tail, `false` otherwise.
#[inline]
fn has_structural_tail(rest: &str, first: u8, max_lookahead: usize) -> bool
{
    // lookahead limit is 0, no structural tail
    ret_false!(max_lookahead == 0);

    // Not a vowel/medial: either virama/asat (do not break) or a new base
    // (no structural tail). No UTF-8 walk.
    ret_if!(first & F_MODIFIER == 0, first & (F_VIRAMA | F_ASAT) != 0);

    // lookahead limit is 1, no structural tail
    ret_false!(max_lookahead == 1);

    // A killed consonant can hold the dot below (့) before its asat, but no
    // other sign. Vowel signs and medials mean the base starts a new vanna,
    // as in `ကျော်`, where the asat belongs to the vowel.
    rest.chars()
        .take(max_lookahead)
        .find(|&ch| ch as u32 != DOT_BELOW)
        .is_some_and(|ch| bur_class(ch as u32) & (F_VIRAMA | F_ASAT) != 0)
}

/// Get the kind of a token from the class of its first character.
///
/// # Arguments
///
/// * `class`: The class of the first character of the token.
///
/// # Returns
///
/// The kind of the token.
#[inline]
fn kind_from_class(class: u8) -> PreLexicalTokenKind
{
    if class & F_FOREIGN != 0
    {
        PreLexicalTokenKind::NonBurmeseChar
    }
    else if class & F_DIGIT != 0
    {
        PreLexicalTokenKind::BurmeseDigit
    }
    else
    {
        PreLexicalTokenKind::BurmeseVanna
    }
}

/// Decide whether a new token should start at the current character.
///
/// # Arguments
///
/// * `prev`: The class of the previous character.
/// * `curr`: The class of the current character.
/// * `next_cp`: The code point of the next character, or `0` if none.
/// * `next`: The class of the next character (`F_FOREIGN` if none).
/// * `rest`: The remainder of the input after the current character.
/// * `options`: The options for the segmenter.
///
/// # Returns
///
/// `true` if a token should be started at the current character.
#[inline]
fn should_break(
    prev: u8, curr: u8, next_cp: u32, next: u8, rest: &str, options: &SegmenterOptions,
) -> bool
{
    let prev_non_bur = prev & F_FOREIGN != 0;
    let curr_non_bur = curr & F_FOREIGN != 0;

    // Test the [`SegmenterOptions::coalesce_non_burmese_chars`] option first so
    // mixed-script, digit, punctuation never reach the structural-tail scan.
    // And isolate non-Burmese characters
    if options.coalesce_non_burmese_chars
    {
        ret_true!(curr_non_bur != prev_non_bur);
    }
    else if curr_non_bur || prev_non_bur
    {
        return true;
    }

    // isolate Burmese digits and standalone characters
    ret_true!(curr & (F_STANDALONE | F_DIGIT) != 0);

    // Break before a new vanna base, unless it is stacked after virama or
    // carries asat/virama of its own. `F_U` and `VISARGA` is a special case
    // where visarga (း) can follow the independent vowel (ဦ)
    let starts = curr & F_START_VANNA != 0 || (curr & F_U != 0 && next_cp == VISARGA);
    starts
        && prev & F_VIRAMA == 0
        && !has_structural_tail(rest, next, options.max_lookahead_limit)
}

/// Emit a token for `text[start_off..off]` if we should break at `off`.
/// This logic is moved out of the [`segment`] function to avoid over nesting.
///
/// On a break, `curr` becomes the start class of the new token so [`emit`]
/// does not have to re-decode the lexeme.
///
/// # Arguments
///
/// * `tokens`: The vector of tokens to push the new token to.
/// * `text`: The input text.
/// * `start_off`: The start offset of the new token.
/// * `start_class`: The start class of the new token.
/// * `prev`: The previous character.
/// * `off`: The offset of the character to break at.
/// * `curr`: The current character.
/// * `pending`: The pending character.
/// * `options`: The options for the segmenter.
#[inline]
fn handle_break<'s>(
    tokens: &mut Vec<PreLexicalToken<'s>>, text: &'s str, base_pos: Pos,
    start_off: &mut usize, start_class: &mut u8, prev: u8, off: usize, curr: u8,
    pending: Option<(usize, u32, u8)>, options: &SegmenterOptions,
)
{
    let (next_cp, next, rest) = lookahead(text, pending);
    ret_if!(!should_break(prev, curr, next_cp, next, rest, options));
    emit(tokens, text, base_pos, *start_off, *start_class, off);
    *start_off = off;
    *start_class = curr; // first character of the token that now begins
}

/// Get the code point and class of a character at a given offset.
///
/// # Arguments
///
/// * `off`: The u8 offset of the character.
/// * `ch`: The character to decode.
///
/// # Returns
///
/// The offset, code point, and class of the character.
#[inline]
fn decode_at(off: usize, ch: char) -> (usize, u32, u8)
{
    let cp = ch as u32;
    (off, cp, bur_class(cp))
}

/// Get the code point and class of the next character from an iterator.
///
/// # Arguments
///
/// * `chars`: The iterator to get the next character from.
///
/// # Returns
///
/// The offset, code point, and class of the next character.
#[inline]
fn decode_next(
    chars: &mut impl Iterator<Item = (usize, char)>,
) -> Option<(usize, u32, u8)>
{
    chars.next().map(|(off, ch)| decode_at(off, ch))
}

/// Look ahead one character from the pending slot.
///
/// # Arguments
///
/// * `text`: The input text.
/// * `pending`: The pending character.
///
/// # Returns
///
/// The code point, class, and remainder of the text.
#[inline]
fn lookahead(text: &str, pending: Option<(usize, u32, u8)>) -> (u32, u8, &str)
{
    tern!(let Some((off, cp, class)) = pending, (cp, class, &text[off..]), (0, F_FOREIGN, ""))
}

/// Emit a token for `text[start_off..end_off]` and
/// push it to the vector of tokens.
///
/// # Arguments
///
/// * `tokens`: The vector of tokens to push the new token to.
/// * `text`: The input text.
/// * `start_off`: The start offset of the new token.
/// * `start_class`: The start class of the new token.
/// * `end_off`: The end offset of the new token.
#[inline]
fn emit<'s>(
    tokens: &mut Vec<PreLexicalToken<'s>>, text: &'s str, base_pos: Pos,
    start_off: usize, start_class: u8, end_off: usize,
)
{
    let kind = kind_from_class(start_class);
    let lo = base_pos + start_off;
    let hi = base_pos + end_off;
    let lexeme = &text[start_off..end_off];
    let token =
        PreLexicalToken::builder().kind(kind).lexeme(lexeme).span(Span::new(lo, hi));
    tokens.push(token.build());
}
