use zakac_rust_sugars::do_for;
use zakac_rust_sugars::ret_if;

/// Burmese medial signs in recommended storage order (ယပင့်၊ ရရစ်၊ ဝဆွဲ and ဟထိုး)
const MEDIAL_ORDER: &[(char, u8)] = &[
    ('\u{103b}', 0),
    ('\u{103c}', 1),
    ('\u{103d}', 2),
    ('\u{103e}', 3),
];

/// Burmese dependent vowel / tone signs sorted after medials.
#[rustfmt::skip]
const VOWEL_ORDER: &[(char, u8)] = &[
    ('\u{102b}', 0),  // မောက်ချ
    ('\u{102c}', 1),  // ဝိုက်ချ
    ('\u{102d}', 2),  // လုံးတင်
    ('\u{102e}', 3),  // လုံးကြီးတင်ဆန်ခတ်
    ('\u{102f}', 4),  // တစ်ချောင်းငင်
    ('\u{1030}', 5),  // နှစ်ချောင်းငင်
    ('\u{1031}', 6),  // သဝေထိုး
    ('\u{1032}', 7),  // နောက်ပစ်
    ('\u{1036}', 8),  // သေးသေးတင်
    ('\u{1037}', 9),  // အောက်မြစ်
    ('\u{1038}', 10)]; // ဝစ္စပေါက်

/// Return `true` if `ch` is a Burmese consonant.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` if `ch` is a Burmese consonant, `false` otherwise.
fn is_consonant(ch: char) -> bool { matches!(ch, '\u{1000}'..='\u{102a}') }

/// Return `true` if `ch` is a Burmese virama.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` if `ch` is a Burmese virama, `false` otherwise.
fn is_virama(ch: char) -> bool { ch == '\u{1039}' }

/// Return the rank of `ch` in the Burmese medial order.
///
/// # Arguments
///
/// * `ch` - The character to get the rank of.
///
/// # Returns
///
/// The rank of `ch` in the Burmese medial order.
/// If `ch` is not a Burmese medial sign, `None` is returned.
fn medial_rank(ch: char) -> Option<u8>
{
    MEDIAL_ORDER.iter().find(|(c, _)| *c == ch).map(|(_, r)| *r)
}

/// Return the rank of `ch` in the Burmese vowel order.
///
/// # Arguments
///
/// * `ch` - The character to get the rank of.
///
/// # Returns
///
/// The rank of `ch` in the Burmese vowel order.
/// If `ch` is not a Burmese vowel sign, `None` is returned.
fn vowel_rank(ch: char) -> Option<u8>
{
    VOWEL_ORDER.iter().find(|(c, _)| *c == ch).map(|(_, r)| *r)
}

/// Record one medial or dependent sign from a cluster tail.
///
/// Returns `false` when `ch` ends the medial/vowel/asat run.
fn push_cluster_trailing(
    ch: char, medials: &mut Vec<(u8, char)>, dependents: &mut Vec<char>,
) -> bool
{
    ret_if!(let Some(rank) = medial_rank(ch), medials.push((rank, ch)), true);
    ret_if!(vowel_rank(ch).is_none() && ch != '\u{103a}', false);
    dependents.push(ch);
    true
}

/// Return `true` if `ch` is a Burmese cluster character.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` if `ch` is a Burmese cluster character, `false` otherwise.
fn is_burmese_cluster_char(ch: char) -> bool
{
    is_consonant(ch)
        || is_virama(ch)
        || medial_rank(ch).is_some()
        || vowel_rank(ch).is_some()
        || ch == '\u{103a}' // asat
}

/// Canonicalize one Burmese wanna cluster starting at `chars`.
///
/// # Arguments
///
/// * `chars` - The characters to canonicalize.
///
/// # Returns
///
/// A tuple containing canonical form and the number of characters consumed.
fn canonicalize_wanna(chars: &[char]) -> (String, usize)
{
    ret_if!(chars.is_empty(), (String::new(), 0));

    // Precomposed mapping: convert အက္ခရာ ဥ + လုံးကြီးတင် into ဦ character
    if chars.len() >= 2 && chars[0] == '\u{1025}' && chars[1] == '\u{102e}'
    {
        return ('\u{1026}'.to_string(), 2);
    }

    let mut i = 0usize;
    let mut base = String::new();
    let mut medials: Vec<(u8, char)> = Vec::new();

    // Preserve dependent-sign order (vowels, asat, tone marks): not
    // commutative.
    let mut dependents: Vec<char> = Vec::new();

    if is_consonant(chars[0])
    {
        base.push(chars[0]);
        i = 1;
    }

    // Stacked consonant sequences (consonant virama consonant ...)
    while i + 1 < chars.len() && is_virama(chars[i]) && is_consonant(chars[i + 1])
    {
        base.push(chars[i]);
        base.push(chars[i + 1]);
        i += 2;
    }

    while i < chars.len()
        && push_cluster_trailing(chars[i], &mut medials, &mut dependents)
    {
        i += 1;
    }

    medials.sort_by_key(|(r, _)| *r);

    let mut out = base;
    do_for!((_, ch) in medials, out.push(ch));
    do_for!(ch in dependents, out.push(ch));

    (out, i)
}

/// Append canonicalized Burmese text from `input` starting at `start` into
/// `out`.
///
/// Returns number of **bytes** consumed from `input`.
pub fn push_canonical_burmese(out: &mut String, input: &str, start: usize) -> usize
{
    let slice = &input[start..];
    let chars: Vec<char> = slice.chars().collect();
    if chars.is_empty() || !is_burmese_cluster_char(chars[0])
    {
        return 0;
    }

    let (canonical, consumed) = canonicalize_wanna(&chars);
    let used = if consumed == 0 { 1 } else { consumed };
    let byte_len: usize = chars[..used].iter().map(|c| c.len_utf8()).sum();
    out.push_str(&canonical);
    byte_len
}

/// Return `true` if `ch` begins a canonicalizable Burmese cluster.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` if `ch` begins a canonicalizable Burmese cluster, `false` otherwise.
pub fn starts_burmese_cluster(ch: char) -> bool { is_burmese_cluster_char(ch) }

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn maps_u_plus_i_to_uu()
    {
        let input = "\u{1025}\u{102e}";
        let mut out = String::new();
        let n = push_canonical_burmese(&mut out, input, 0);
        assert_eq!(out, "\u{1026}");
        assert_eq!(n, input.len());
    }

    fn normalize_word(word: &str) -> (String, usize)
    {
        let mut out = String::new();
        let mut pos = 0usize;
        while pos < word.len()
        {
            let n = push_canonical_burmese(&mut out, word, pos);
            pos += n;
        }
        (out, pos)
    }

    #[test]
    fn preserves_compound_vowel_and_asat_order()
    {
        let words = ["လုပ်ဆောင်ချက်", "ပေါင်းလဒ်", "ကိန်းပြည့်", "ဖြင့်"];
        for word in words
        {
            let (normalized, pos) = normalize_word(word);
            assert_eq!(normalized, word, "normalize changed {word}");
            assert_eq!(pos, word.len(), "normalize changed {word} at offset {pos}");
        }
    }
}
