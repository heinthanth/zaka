use zakac_rust_sugars::ret_if;

use crate::burmese_normalizer::push_canonical_burmese;
use crate::burmese_normalizer::starts_burmese_cluster;

/// A position in normalized source where some characters were removed.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NormalizedPos
{
    /// Offset in the normalized text.
    pub pos: u32,
    /// Cumulative u8 removed from the original up to this point.
    pub diff: u32,
}

impl NormalizedPos
{
    /// Create a new normalized position.
    pub fn new(pos: u32, diff: u32) -> Self { NormalizedPos { pos, diff } }
}

/// Result of normalizing source text.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NormalizedSource
{
    /// Normalized source text.
    pub src: String,
    /// Length of the original (pre-normalize) text.
    pub unnormalized_source_len: u32,
    /// Positions where normalization removed characters.
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
fn normalize_carriage_returns(
    rest: &str, out: &mut String, cumulative_diff: &mut u32,
    normalized_pos: &mut Vec<NormalizedPos>,
) -> usize
{
    out.push('\n');
    ret_if!(rest.len() <= 1 || !rest[1..].starts_with('\n'), 1);

    *cumulative_diff += 1;
    normalized_pos.push(NormalizedPos::new(out.len() as u32, *cumulative_diff));
    return 2;
}

/// Normalize a Burmese cluster.
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
/// If no characters were consumed, `None` is returned.
fn normalize_burmese_cluster(
    rest: &str, out: &mut String, cumulative_diff: &mut u32,
    normalized_pos: &mut Vec<NormalizedPos>,
) -> Option<usize>
{
    let consumed = push_canonical_burmese(out, rest, 0);
    ret_if!(consumed == 0, None);
    record_diff(out.len(), consumed, cumulative_diff, normalized_pos);
    Some(consumed)
}

/// Record the difference between the original and normalized text.
///
/// # Arguments
///
/// * `out_len` - The length of the output string.
/// * `orig_consumed` - The number of characters consumed from the original
///   text.
/// * `cumulative_diff` - The cumulative difference.
/// * `normalized_pos` - The normalized positions.
fn record_diff(
    out_len: usize, orig_consumed: usize, cumulative_diff: &mut u32,
    normalized_pos: &mut Vec<NormalizedPos>,
)
{
    ret_if!(orig_consumed <= out_len);
    *cumulative_diff += (orig_consumed - out_len) as u32;
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
    let mut diff_acc = 0u32;

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

/// Normalize one scalar or cluster at the start of `rest`.
///
/// Returns the number of bytes consumed from `rest`.
fn normalize_next(
    rest: &str, out: &mut String, acc_diff: &mut u32, npos: &mut Vec<NormalizedPos>,
) -> usize
{
    let ch = rest.chars().next().unwrap();
    ret_if!(ch == '\r', normalize_carriage_returns(rest, out, acc_diff, npos));

    #[rustfmt::skip]
    ret_if!(starts_burmese_cluster(ch) && let Some(consumed)
        = normalize_burmese_cluster(rest, out, acc_diff, npos), consumed);

    out.push(ch);
    ch.len_utf8()
}
