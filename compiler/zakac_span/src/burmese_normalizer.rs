use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::do_while;
use zakac_rust_sugars::ret_if;

/// Burmese medial signs in recommended storage order (ယပင့်၊ ရရစ်၊ ဝဆွဲ and ဟထိုး)
#[rustfmt::skip]
const MEDIAL_ORDER: &[(char, u8)] = &[
    ('\u{103b}', 0),
    ('\u{103c}', 1),
    ('\u{103d}', 2),
    ('\u{103e}', 3), ];

/// Canonical rank of `ch` among the Burmese dependent vowel and tone signs.
#[rustfmt::skip]
const VOWEL_RANK: &[(char, u8)] = &[
    ('\u{1031}', 0),  // သဝေထိုး
    ('\u{102b}', 1),  // မောက်ချ
    ('\u{102c}', 2),  // ဝိုက်ချ
    ('\u{102d}', 3),  // လုံးတင်
    ('\u{102e}', 4),  // လုံးကြီးတင်ဆန်ခတ်
    ('\u{102f}', 5),  // တစ်ချောင်းငင်
    ('\u{1030}', 6),  // နှစ်ချောင်းငင်
    ('\u{1032}', 7),  // နောက်ပစ်
    ('\u{1036}', 8),  // သေးသေးတင်
    ('\u{1038}', 9),  // ဝစ္စပေါက်
    ('\u{1037}', 10), ]; // အောက်မြစ်

/// Return `true` if `ch` is a Burmese dependent vowel or tone sign.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` if `ch` is a dependent vowel or tone sign, `false` otherwise.
fn is_vowel_sign(ch: char) -> bool { vowel_rank(ch).is_some() }

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

/// Return the rank of `ch` in the Burmese vowel and tone order.
///
/// # Arguments
///
/// * `ch` - The character to get the rank of.
///
/// # Returns
///
/// The rank of `ch` in the Burmese vowel and tone order.
/// If `ch` is not a Burmese vowel or tone sign, `None` is returned.
fn vowel_rank(ch: char) -> Option<u8>
{
    VOWEL_RANK.iter().find(|(c, _)| *c == ch).map(|(_, r)| *r)
}

/// Record one medial or dependent sign from a cluster tail.
///
/// # Arguments
///
/// * `ch` - The character to push.
/// * `medials` - The medials to push.
/// * `dependents` - The dependents to push.
///
/// # Returns
///
/// `true` if `ch` is a medial or dependent sign, `false` otherwise.
fn push_cluster_trailing(
    ch: char, medials: &mut Vec<(u8, char)>, dependents: &mut Vec<char>,
) -> bool
{
    ret_if!(let Some(rank) = medial_rank(ch), medials.push((rank, ch)), true);
    ret_if!(!is_vowel_sign(ch) && ch != '\u{103a}', false);
    dependents.push(ch);
    true
}

/// Sort each run of vowel signs between asats with [`vowel_rank`], in place.
///
/// # Arguments
///
/// * `signs` - The dependent signs of one cluster.
fn sort_vowel_runs(signs: &mut [char])
{
    do_for!(r in signs.split_mut(|ch| *ch == '\u{103a}'), r.sort_by_key(|ch| vowel_rank(*ch)))
}

/// Map common Burmese clusters to their canonical form.
///
/// # Arguments
///
/// * `chars` - The characters to map.
///
/// # Returns
///
/// A tuple containing the canonical form and the number of characters consumed.
/// If no mapping is found, `None` is returned.
fn map_common_burmese_clusters(chars: &[char]) -> Option<(String, usize)>
{
    // Precomposed mapping: convert အက္ခရာ စျ + ယပင့် into ဈ character.
    if chars.len() >= 2 && chars[0] == '\u{1005}' && chars[1] == '\u{103b}'
    {
        return Some(('\u{1008}'.to_string(), 2));
    }
    None
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
    ret_if!(let Some((canonical, consumed)) = map_common_burmese_clusters(chars), (canonical, consumed));

    let mut i = 0usize;
    let mut base = String::new();
    let mut medials: Vec<(u8, char)> = Vec::new();

    // Collected in input order; canonicalized after the scan.
    let mut dependents: Vec<char> = Vec::new();
    do_if!(is_consonant(chars[0]), base.push(chars[0]), i = 1);

    // Stacked consonant sequences (consonant virama consonant ...)
    while i + 1 < chars.len() && is_virama(chars[i]) && is_consonant(chars[i + 1])
    {
        base.push(chars[i]);
        base.push(chars[i + 1]);
        i += 2;
    }

    let mut push = |ch| push_cluster_trailing(ch, &mut medials, &mut dependents);
    do_while!(i < chars.len() && push(chars[i]), i += 1);

    medials.sort_by_key(|(r, _)| *r);
    sort_vowel_runs(&mut dependents);

    let mut out = base;
    do_for!((_, ch) in medials, out.push(ch));
    do_for!(ch in dependents, out.push(ch));

    (out, i)
}

/// Append the canonical form of the cluster at the start of `chars` to `out`.
///
/// # Arguments
///
/// * `out` - The output string.
/// * `chars` - The chars to canonicalize, at least one.
///
/// # Returns
///
/// The number of chars consumed from `chars`, at least one.
fn push_cluster(out: &mut String, chars: &[char]) -> usize
{
    let (canonical, consumed) = canonicalize_wanna(chars);
    ret_if!(consumed == 0, out.push(chars[0]), 1);
    out.push_str(&canonical);
    consumed
}

/// Canonicalize the Burmese clusters in `text`.
///
/// # Arguments
///
/// * `text` - The text to canonicalize.
///
/// # Returns
///
/// The canonicalized text. Chars outside Burmese clusters are passed through
/// unchanged.
pub fn canonicalize(text: &str) -> String
{
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    do_while!(i < chars.len(), i += push_cluster(&mut out, &chars[i..]));
    out
}

/// Return `true` if `ch` extends the Burmese cluster that `prev` ends.
///
/// # Arguments
///
/// * `prev` - The char before `ch`.
/// * `ch` - The char to check.
///
/// # Returns
///
/// `true` if `ch` is a medial, vowel, asat or virama, or follows a virama.
pub fn continues_cluster(prev: char, ch: char) -> bool
{
    is_virama(prev)
        || is_virama(ch)
        || medial_rank(ch).is_some()
        || is_vowel_sign(ch)
        || ch == '\u{103a}' // asat
}

#[cfg(test)]
pub(crate) mod tests
{
    use super::*;

    /// Canonicalize a word.
    ///
    /// # Arguments
    ///
    /// * `word` - The word to canonicalize.
    ///
    /// # Returns
    ///
    /// The canonical form of `word`.
    fn normalize_word(word: &str) -> String { canonicalize(word) }

    #[test]
    fn maps_zms()
    {
        assert_eq!(normalize_word("\u{1005}\u{103b}"), "\u{1008}");
    }

    #[test]
    fn preserves_compound_vowel_and_asat_order()
    {
        let words = ["လုပ်ဆောင်ချက်", "ပေါင်းလဒ်", "ဖြင့်"];
        do_for!(w in words, assert_eq!(normalize_word(w), w, "normalize changed {w}"));
    }

    /// Guard the asat barrier in [`sort_vowel_runs`].
    #[test]
    fn preserves_asat_split_sign_order()
    {
        let words = [
            "\u{1016}\u{103c}\u{1004}\u{1037}\u{103a}",
            "\u{1000}\u{103b}\u{103d}\u{1014}\u{103a}\u{102f}\u{1015}\u{103a}",
            "\u{1000}\u{102d}\u{102f}\u{1037}",
        ];
        do_for!(w in words, assert_eq!(normalize_word(w), w, "normalize changed {w}"));
    }

    /// Written as escapes so the exact sequence is unambiguous.
    pub(crate) const SAMPLE_WORDS: [&str; 9] = [
        // ကျောင်း
        "\u{1000}\u{103b}\u{1031}\u{102c}\u{1004}\u{103a}\u{1038}",
        // ဆောင်
        "\u{1006}\u{1031}\u{102c}\u{1004}\u{103a}",
        // ကော်
        "\u{1000}\u{1031}\u{102c}\u{103a}",
        // ကို
        "\u{1000}\u{102d}\u{102f}",
        // ဖြင့်
        "\u{1016}\u{103c}\u{1004}\u{1037}\u{103a}",
        // ကျွန်ုပ်
        "\u{1000}\u{103b}\u{103d}\u{1014}\u{103a}\u{102f}\u{1015}\u{103a}",
        // ကို့
        "\u{1000}\u{102d}\u{102f}\u{1037}",
        // ပေါင်းလဒ်
        "\u{1015}\u{1031}\u{102b}\u{1004}\u{103a}\u{1038}\u{101c}\u{1012}\u{103a}",
        // လုပ်ဆောင်ချက်
        concat!(
            "\u{101c}\u{102f}\u{1015}\u{103a}\u{1006}\u{1031}\u{102c}\u{1004}",
            "\u{103a}\u{1001}\u{103b}\u{1000}\u{103a}"
        ),
    ];
    const MEDIALS: [char; 4] = ['\u{103b}', '\u{103c}', '\u{103d}', '\u{103e}'];
    pub(crate) const VOWELS: [char; 11] = [
        '\u{102b}', '\u{102c}', '\u{102d}', '\u{102e}', '\u{102f}', '\u{1030}',
        '\u{1031}', '\u{1032}', '\u{1036}', '\u{1037}', '\u{1038}',
    ];
    const COMBOS: usize = 16 * (VOWELS.len() + 1) * 2;

    /// Build one cluster from a packed combination index.
    ///
    /// # Arguments
    ///
    /// * `base` - The base consonant.
    /// * `combo` - The packed combination index, below [`COMBOS`].
    ///
    /// # Returns
    ///
    /// The generated cluster.
    fn cluster_at(base: char, combo: usize) -> String
    {
        let mask = combo % 16;
        let vowel = (combo / 16) % (VOWELS.len() + 1);
        let asat = combo / 16 / (VOWELS.len() + 1) == 1;

        let mut out = String::new();
        out.push(base);
        let picked = MEDIALS.iter().enumerate().filter(|&(i, _)| mask & (1 << i) != 0);
        do_for!((_, m) in picked, out.push(*m));
        do_if!(vowel < VOWELS.len(), out.push(VOWELS[vowel]));
        do_if!(asat, out.push('\u{103a}'));
        out
    }

    /// All consonants crossed with all medial, vowel and asat combinations.
    ///
    /// # Returns
    ///
    /// One cluster per combination, each set in canonical order.
    pub(crate) fn generate_clusters() -> Vec<String>
    {
        ('\u{1000}'..='\u{102a}')
            .flat_map(|base| (0..COMBOS).map(move |combo| cluster_at(base, combo)))
            .collect()
    }

    /// Whether `word` hits the U+1008 mapping.
    ///
    /// # Arguments
    ///
    /// * `word` - The cluster to check.
    ///
    /// # Returns
    ///
    /// `true` when a mapping applies, `false` otherwise.
    fn maps_to_precomposed(word: &str) -> bool { word.starts_with("\u{1005}\u{103b}") }

    #[test]
    fn generated_clusters_round_trip()
    {
        let words: Vec<String> = generate_clusters()
            .into_iter()
            .filter(|w| !maps_to_precomposed(w))
            .collect();
        assert!(words.len() > 10_000, "generator made too few cases: {}", words.len());
        do_for!(w in &words, assert_eq!(&normalize_word(w), w, "normalize changed {w:?}"));
    }

    #[test]
    fn real_words_round_trip()
    {
        do_for!(w in SAMPLE_WORDS, assert_eq!(normalize_word(w), w, "normalize changed {w}"));
    }

    #[test]
    fn normalize_is_idempotent()
    {
        let mut words = generate_clusters();
        do_for!(w in SAMPLE_WORDS, words.push(w.to_string()));

        for word in &words
        {
            let once = normalize_word(word);
            let twice = normalize_word(&once);
            assert_eq!(once, twice, "not idempotent for {word:?}");
        }
    }

    /// Text outside the Burmese cluster set has to pass through untouched.
    #[test]
    fn non_burmese_passes_through()
    {
        let samples = [
            "a",
            "Z",
            "0",
            " ",
            "\n",
            "!",
            "\u{1f600}",
            "\u{00e9}",
            "\u{65e5}",
            "\u{104a}", // Burmese section mark
            "\u{1040}", // Burmese digit zero
        ];
        do_for!(s in samples, assert_eq!(normalize_word(s), s, "changed {s:?}"));
    }

    /// A virama outside a stacked pair passes through unchanged.
    #[test]
    fn lone_virama_is_preserved()
    {
        let words = ["\u{1039}", "\u{1000}\u{1039}", "\u{1000}\u{1039}\u{102c}"];
        do_for!(w in words, assert_eq!(normalize_word(w), w, "normalize dropped signs from {w:?}"));
    }

    /// Scrambled-but-equal sign orders collapse to the canonical form.
    #[test]
    fn repairs_scrambled_sign_order()
    {
        let cases = [
            // ကုိ -> ကို
            ("\u{1000}\u{102f}\u{102d}", "\u{1000}\u{102d}\u{102f}"),
            // ဆာေ -> ဆော
            ("\u{1006}\u{102c}\u{1031}", "\u{1006}\u{1031}\u{102c}"),
            // ကးာ -> ကား
            ("\u{1000}\u{1038}\u{102c}", "\u{1000}\u{102c}\u{1038}"),
        ];
        for (scrambled, canonical) in cases
        {
            assert_eq!(
                normalize_word(scrambled),
                canonical,
                "wrong repair for {scrambled:?}"
            );
            let again = normalize_word(canonical);
            assert_eq!(again, canonical, "canonical form of {scrambled:?} not stable");
        }
    }

    fn normalize_two_clusters(first: char, second: char)
    {
        let word = format!("\u{1000}{first}{second}");
        let mut signs = [first, second];
        signs.sort_by_key(|ch| vowel_rank(*ch));
        let expected: String = std::iter::once('\u{1000}').chain(signs).collect();

        let normalized = normalize_word(&word);
        assert_eq!(normalized, expected, "wrong order for {word:?}");
        let again = normalize_word(&normalized);
        assert_eq!(again, normalized, "not idempotent for {word:?}");
    }

    /// Both orders of each vowel-sign pair land in rank order, and stick.
    #[test]
    fn two_vowel_clusters_canonicalize()
    {
        do_for!(f in VOWELS, do_for!(s in VOWELS, normalize_two_clusters(f, s)));
    }
}
