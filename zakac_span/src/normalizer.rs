use std::iter::once;

use unicode_normalization::IsNormalized;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::canonical_combining_class;
use unicode_normalization::is_nfc_quick;
use zakac_rust_sugars::ret_if;

use crate::burmese_normalizer;

/// A position in normalized source where normalization changed the text length.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NormalizedPos
{
    /// Offset in the normalized text.
    pub pos: u32,
    /// Cumulative u8 count the original text has over the normalized text up
    /// to this point. Negative where NFC grew the text.
    pub diff: i32,
}

impl NormalizedPos
{
    /// Create a new normalized position.
    pub fn new(pos: u32, diff: i32) -> Self { NormalizedPos { pos, diff } }
}

/// Result of normalizing source text.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NormalizedSource
{
    /// Normalized source text.
    pub src: String,
    /// Length of the original (pre-normalize) text.
    pub unnormalized_source_len: u32,
    /// Positions where normalization changed the text length.
    pub normalized_pos: Vec<NormalizedPos>,
}

impl From<&str> for NormalizedSource
{
    /// Convert a string to a normalized source.
    ///
    /// # Arguments
    ///
    /// * `original` - The original source text.
    ///
    /// # Returns
    ///
    /// The normalized source text.
    fn from(original: &str) -> Self { normalize_source(original) }
}

impl From<String> for NormalizedSource
{
    /// Convert a string to a normalized source.
    ///
    /// # Arguments
    ///
    /// * `original` - The original source text.
    ///
    /// # Returns
    ///
    /// The normalized source text.
    fn from(original: String) -> Self { normalize_source(&original) }
}

/// Normalize carriage returns.
/// convert carriage returns to line feeds.
///
/// # Arguments
///
/// * `rest` - The rest of the string to normalize.
/// * `out` - The output string.
/// * `cumulative_diff` - The cumulative difference.
/// * `normalized_pos` - The normalized positions.
///
/// # Returns
///
/// The number of characters to skip.
fn normalize_cr(
    rest: &str, out: &mut String, cumulative_diff: &mut i32,
    normalized_pos: &mut Vec<NormalizedPos>,
) -> usize
{
    out.push('\n');
    ret_if!(rest.len() <= 1 || !rest[1..].starts_with('\n'), 1);

    *cumulative_diff += 1;
    normalized_pos.push(NormalizedPos::new(out.len() as u32, *cumulative_diff));
    return 2;
}

/// Return `true` if no char before `ch` can combine or reorder with it under
/// NFC.
///
/// # Arguments
///
/// * `ch` - The char to check.
///
/// # Returns
///
/// `true` if an NFC segment starts at `ch`, `false` otherwise.
fn starts_nfc_segment(ch: char) -> bool
{
    ch.is_ascii()
        || (canonical_combining_class(ch) == 0
            && is_nfc_quick(once(ch)) == IsNormalized::Yes)
}

/// Return `true` if a new chunk starts at `ch`.
///
/// # Arguments
///
/// * `prev` - The char before `ch`.
/// * `ch` - The char to check.
///
/// # Returns
///
/// `true` if `ch` starts a new chunk, `false` if it extends the current one.
fn starts_chunk(prev: char, ch: char) -> bool
{
    ch == '\r'
        || (starts_nfc_segment(ch) && !burmese_normalizer::continues_cluster(prev, ch))
}

/// Return the length of the chunk at the start of `rest`.
///
/// A chunk holds whole NFC segments and whole Burmese clusters, so it
/// normalizes on its own, without context from the text around it.
///
/// # Arguments
///
/// * `rest` - The rest of the string to normalize, at least one char.
///
/// # Returns
///
/// The chunk length in u8s.
fn chunk_len(rest: &str) -> usize
{
    rest.chars()
        .zip(rest.char_indices().skip(1))
        .find(|(prev, (_, ch))| starts_chunk(*prev, *ch))
        .map_or(rest.len(), |(_, (idx, _))| idx)
}

/// Append the normalized form of one chunk to `out`.
///
/// NFC first folds canonical equivalents together, the Burmese pass then
/// sorts medials and vowels, and a second NFC composes what the sort brought
/// together.
///
/// # Arguments
///
/// * `out` - The output string.
/// * `chunk` - The chunk to normalize.
///
/// # Returns
///
/// The normalized chunk.
fn push_normalized_chunk(out: &mut String, chunk: &str)
{
    ret_if!(chunk.is_ascii(), out.push_str(chunk));
    let composed: String = chunk.nfc().collect();
    out.extend(burmese_normalizer::canonicalize(&composed).nfc());
}

/// Record how much a chunk changed the text length.
///
/// # Arguments
///
/// * `added` - u8s this chunk pushed into the normalized output.
/// * `orig_consumed` - u8s this chunk consumed from the original text.
/// * `out_len` - Total length of the normalized output so far.
/// * `cumulative_diff` - The cumulative difference.
/// * `normalized_pos` - The normalized positions.
fn record_diff(
    added: usize, orig_consumed: usize, out_len: usize, cumulative_diff: &mut i32,
    normalized_pos: &mut Vec<NormalizedPos>,
)
{
    ret_if!(orig_consumed == added);
    *cumulative_diff += orig_consumed as i32 - added as i32;
    normalized_pos.push(NormalizedPos::new(out_len as u32, *cumulative_diff));
}

/// Normalize source text.
///
/// # Arguments
///
/// * `original` - The original source text.
///
/// # Returns
///
/// The normalized source text.
fn normalize_source(original: &str) -> NormalizedSource
{
    let unnormalized_source_len = original.len() as u32;
    let mut npos = Vec::new();
    let mut diff_acc = 0i32;

    let mut src = original;
    if src.starts_with('\u{FEFF}')
    {
        diff_acc = 3;
        npos.push(NormalizedPos { pos: 0, diff: diff_acc });
        src = &src[3..];
    }

    let mut out = String::with_capacity(src.len());
    let mut idx = 0usize;

    while idx < src.len()
    {
        idx += normalize_next(&src[idx..], &mut out, &mut diff_acc, &mut npos);
    }
    NormalizedSource { src: out, unnormalized_source_len, normalized_pos: npos }
}

/// Normalize one line break or chunk at the start of `rest`.
///
/// # Arguments
///
/// * `rest` - The rest of the string to normalize.
/// * `out` - The output string.
/// * `acc_diff` - The cumulative difference.
/// * `npos` - The normalized positions.
///
/// # Returns
///
/// Returns the number of u8s consumed from `rest`.
fn normalize_next(
    rest: &str, out: &mut String, acc_diff: &mut i32, npos: &mut Vec<NormalizedPos>,
) -> usize
{
    ret_if!(rest.starts_with('\r'), normalize_cr(rest, out, acc_diff, npos));

    let chunk = &rest[..chunk_len(rest)];
    let out_len_before = out.len();
    push_normalized_chunk(out, chunk);
    record_diff(out.len() - out_len_before, chunk.len(), out.len(), acc_diff, npos);
    chunk.len()
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A shrinking cluster leaves a mark wherever it sits in the input.
    ///
    /// Each case maps ဥ + လုံးကြီးတင် to ဦ, dropping 3 u8s. [`record_diff`]
    /// must push one [`NormalizedPos`] per case — the mark sits at the end
    /// of the shrunk cluster in normalized text, with the cumulative loss.
    #[test]
    fn records_diff_for_later_clusters()
    {
        // (input, expected pos of the mark, expected cumulative diff)
        let cases = [
            ("\u{1025}\u{102e}", 3u32, 3i32),
            ("\u{1000}\u{1025}\u{102e}", 6, 3),
            ("\u{1000}\u{1000}\u{1000}\u{1025}\u{102e}", 12, 3),
        ];
        for (input, pos, diff) in cases
        {
            let normalized = normalize_source(input);
            let lost = input.len() - normalized.src.len();
            assert_eq!(lost, 3, "expected to drop 3 u8s from {input:?}");
            assert_eq!(
                normalized.normalized_pos,
                vec![NormalizedPos::new(pos, diff)],
                "wrong diff entries for {input:?}",
            );
        }
    }

    /// End-to-end position mapping through shrinks and a CRLF.
    ///
    /// Input: `a` + CRLF + ဦ + ဦ + `b`. The normalized text is
    /// `a` + LF + ဦ + ဦ + `b`, 3 u8s lost per cluster plus 1 for the CR.
    /// [`SourceFile::original_relative_pos`] has to return the exact original
    /// offset for each normalized position, in particular past the second
    /// shrink.
    #[test]
    fn maps_positions_through_shrinks()
    {
        use crate::filename::FileName;
        use crate::hash::SourceFileHashAlgorithm;
        use crate::pos::Pos;
        use crate::pos::RelativePos;
        use crate::source_file::SourceFile;

        let original = "a\r\n\u{1025}\u{102e}\u{1025}\u{102e}b";
        let file = SourceFile::new(
            FileName::Custom("test".into()),
            original.to_string(),
            Pos::ZERO,
            SourceFileHashAlgorithm::Sha256,
        );
        assert_eq!(file.src, "a\n\u{1026}\u{1026}b");

        // (normalized offset, original offset): 'a' 0->0, LF 1->1 (the CR,
        // where the break starts), first ဦ 2->3, second ဦ 5->9, 'b' 8->15,
        // end 9->16.
        let cases = [(0u32, 0u32), (1, 1), (2, 3), (5, 9), (8, 15), (9, 16)];
        for (normalized, original_pos) in cases
        {
            let mapped = file.original_relative_pos(&RelativePos::new(normalized));
            assert_eq!(
                mapped.as_u32(),
                original_pos,
                "wrong original offset for normalized {normalized}",
            );
        }
    }

    /// Run [`normalize_source`] and keep just the text.
    ///
    /// # Arguments
    ///
    /// * `input` - The text to normalize.
    ///
    /// # Returns
    ///
    /// The normalized text.
    fn normalize(input: &str) -> String { normalize_source(input).src }

    /// Canonical equivalents collapse to one NFC form, in and out of Burmese.
    #[test]
    fn applies_nfc()
    {
        let a = concat!(
            "\u{1000}\u{102d}\u{1014}\u{103a}\u{1038}\u{1015}\u{103c}",
            "\u{100a}\u{103a}\u{1037}"
        );
        let b = concat!(
            "\u{1000}\u{102d}\u{1014}\u{103a}\u{1038}\u{1015}\u{103c}",
            "\u{100a}\u{1037}\u{103a}"
        );
        let cases = [
            // ဥ + လုံးကြီးတင် -> ဦ
            ("\u{1025}\u{102e}", "\u{1026}"),
            // င ် ့ -> င ့ ် (ccc 7 sorts before ccc 9)
            ("\u{1004}\u{103a}\u{1037}", "\u{1004}\u{1037}\u{103a}"),
            // ကိန်းပြည့် as written (asat then dot below) -> NFC order
            (a, b),
            // The vowel sort brings U+102E next to ဥ; the second NFC composes.
            ("\u{1025}\u{102f}\u{102e}", "\u{1026}\u{102f}"),
            // Latin e + combining acute -> é
            ("e\u{0301}", "\u{00e9}"),
            // Combining marks out of ccc order: dot below (220) before acute (230)
            ("e\u{0301}\u{0323}", "\u{1eb9}\u{0301}"),
        ];
        for (input, expected) in cases
        {
            assert_eq!(normalize(input), expected, "wrong NFC for {input:?}");
            assert_eq!(normalize(expected), expected, "{expected:?} not stable");
        }
    }

    /// Normalized output is NFC, and a second pass leaves it alone.
    ///
    /// Covers generated clusters, real words, and each base + four-sign
    /// tail drawn from the vowels and asat, which mixes the Burmese sort with
    /// the NFC pair order in odd orders.
    #[test]
    fn output_is_stable_nfc()
    {
        use crate::burmese_normalizer::tests::SAMPLE_WORDS;
        use crate::burmese_normalizer::tests::VOWELS;
        use crate::burmese_normalizer::tests::generate_clusters;

        let signs: Vec<char> = VOWELS.into_iter().chain(once('\u{103a}')).collect();
        let signs = signs.as_slice();
        let n = signs.len();
        let tails =
            (0..n.pow(4)).map(|k| (0..4u32).map(move |d| signs[k / n.pow(d) % n]));
        let mut words = generate_clusters();
        words.extend(SAMPLE_WORDS.map(String::from));
        for base in ['\u{1000}', '\u{1025}']
        {
            words
                .extend(tails.clone().map(|t| once(base).chain(t).collect::<String>()));
        }

        for word in &words
        {
            let first = normalize(word);
            assert!(unicode_normalization::is_nfc(&first), "not NFC for {word:?}");
            assert_eq!(normalize(&first), first, "not idempotent for {word:?}");
        }
    }

    /// A Latin composition shrinks the text and leaves a mark.
    #[test]
    fn records_diff_for_latin_composition()
    {
        let normalized = normalize_source("e\u{0301}x");
        assert_eq!(normalized.src, "\u{00e9}x");
        assert_eq!(normalized.normalized_pos, vec![NormalizedPos::new(2, 1)]);
    }

    /// NFC can grow text, and the mark then carries a negative diff.
    ///
    /// U+0958 is a composition exclusion: NFC decomposes it to U+0915 U+093C,
    /// 3 u8s more. Positions past it still map back to the right original
    /// offset.
    #[test]
    fn maps_positions_through_growth()
    {
        use crate::filename::FileName;
        use crate::hash::SourceFileHashAlgorithm;
        use crate::pos::Pos;
        use crate::pos::RelativePos;
        use crate::source_file::SourceFile;

        let original = "a\u{0958}b";
        let normalized = normalize_source(original);
        assert_eq!(normalized.src, "a\u{0915}\u{093c}b");
        assert_eq!(normalized.normalized_pos, vec![NormalizedPos::new(7, -3)]);

        let file = SourceFile::new(
            FileName::Custom("test".into()),
            original.to_string(),
            Pos::ZERO,
            SourceFileHashAlgorithm::Sha256,
        );
        // (normalized offset, original offset): 'a' 0->0, the cluster 1->1,
        // 'b' 7->4, end 8->5.
        let cases = [(0u32, 0u32), (1, 1), (7, 4), (8, 5)];
        for (normalized, original_pos) in cases
        {
            let mapped = file.original_relative_pos(&RelativePos::new(normalized));
            assert_eq!(
                mapped.as_u32(),
                original_pos,
                "wrong original offset for normalized {normalized}",
            );
        }
    }
}
