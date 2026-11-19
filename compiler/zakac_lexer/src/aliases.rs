//! The aliases of the parts of reserved words: parts that mean the same, so a
//! reserved word written with one part reads as the word written with another.
//!
//! [`ALIASES`] is the table to edit. Each row is a group of parts:
//!
//! - `written(&[a, b, c])`: each part is plain Zaka. A reserved word that holds
//!   one of them reads the same with each other one: with `written(&["၍",
//!   "ပြီး", "ကာ"])`, `ဖြစ်ပြီး` and `ဖြစ်ကာ` read as `ဖြစ်၍`.
//! - `spoken(&[written, spoken])`: the first part is written Zaka and the
//!   second its spoken form, which the lint reports (ZK0510) unless a program
//!   allows it. A word takes each of its spoken parts at once: `ထားမည်` is
//!   `ထားမယ်`.
//!
//! A row can tell where it holds:
//!
//! - `.at_end()`: the part ends the word, as the ending of a verb does.
//! - `.within(&[words])`: the row holds in these reserved words alone.
//! - `.except(&[words])`: the row holds in each reserved word but these.
//!
//! The words of `within` and `except` are spelled as [`RESERVED`] spells them.
//! A part matches whole vannas, so the `မှ` of `မှတ်` is no part `မှ`; a spoken
//! part is one vanna.
//!
//! The tests write each spelling the table gives to
//! `tests/expected/aliases.tsv` and `tests/expected/spoken.tsv` for review, and
//! fail when a spelling reads as two reserved words.

use std::collections::HashSet;
use std::sync::LazyLock;

use zakac_burmese_segmenter::segmenter::SegmenterOptions;
use zakac_burmese_segmenter::segmenter::segment;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;

use crate::reserved::RESERVED;
use crate::token::Kw;

#[rustfmt::skip]
pub static ALIASES: &[Alias] = &[
    // ---- written: each spelling is plain Zaka ----

    // "and then": the ending of a verb that the sentence goes on from
    written(&["၍", "ပြီး", "ကာ"]).at_end().except(&["အကယ်၍", "ပြီးဆုံးပါပြီး"]),
    // "that": the ending of a verb before its noun
    written(&["သော", "သည့်"]).at_end().within(&["သုံးသော", "လက်ခံသော", "ပြန်ပေးသော"]),
    // the closer of a block
    written(&["ပြီးဆုံးပါပြီး", "ပြီးဆုံးပါပြီ", "ပြီးပါပြီး", "ပြီးပါပြီ"]),
    // "storage"
    written(&["မင်မိုရီ", "မှတ်ဉာဏ်"]),
    // the classifier of a count of items
    written(&["ခု", "လုံး"]).within(&["ခုစာ", "ခုမြောက်"]),

    // ---- spoken: the lint reports each (ZK0510) ----

    // In `မည်သည်ကိုမှ` ("nothing") and `အမည်ဖြင့်` ("under the name") the parts
    // are no particles, so these words keep their written form.
    spoken(&["မည်", "မယ်"]).except(&["မည်သည်ကိုမှ", "အမည်ဖြင့်"]),
    spoken(&["မည့်", "မယ့်"]),
    spoken(&["သည်", "တယ်"]).except(&["မည်သည်ကိုမှ"]),
    spoken(&["သည့်", "တဲ့"]),
    spoken(&["သော", "တဲ့"]),
    spoken(&["တွင်", "မှာ"]),
    spoken(&["လျှင်", "ရင်"]),
    spoken(&["ဟု", "လို့"]),
    spoken(&["မှ", "ကနေ"]).except(&["မည်သည်ကိုမှ"]),
    spoken(&["နှင့်", "နဲ့"]),
    spoken(&["ဤ", "ဒီ"]),
];

/// Whether the parts of a row are plain Zaka or spoken forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register
{
    /// Plain Zaka.
    Written,
    /// A spoken form, which the lint reports.
    Spoken,
}

/// A row of [`ALIASES`]: a group of parts that mean the same.
#[derive(Debug, Clone, Copy)]
pub struct Alias
{
    /// The parts; for a spoken row, the written part, then its spoken form.
    pub parts: &'static [&'static str],
    /// Whether the parts are plain Zaka or spoken forms.
    pub register: Register,
    /// Whether a part matches at the end of the word alone.
    pub at_end: bool,
    /// The reserved words the row holds in; each word when it lists none.
    pub within: &'static [&'static str],
    /// The reserved words the row does not hold in.
    pub except: &'static [&'static str],
}

/// A row of written parts.
///
/// # Arguments
///
/// * `parts` - The parts.
///
/// # Returns
///
/// The row, which holds at each place in each reserved word.
pub const fn written(parts: &'static [&'static str]) -> Alias
{
    Alias {
        parts,
        register: Register::Written,
        at_end: false,
        within: &[],
        except: &[],
    }
}

/// A row of a written part and its spoken form.
///
/// # Arguments
///
/// * `parts` - The written part, then the spoken one.
///
/// # Returns
///
/// The row, which holds at each place in each reserved word.
pub const fn spoken(parts: &'static [&'static str]) -> Alias
{
    Alias { register: Register::Spoken, ..written(parts) }
}

impl Alias
{
    /// The row, with its parts at the end of a word alone.
    ///
    /// # Returns
    ///
    /// The row.
    pub const fn at_end(self) -> Alias { Alias { at_end: true, ..self } }

    /// The row, holding in some reserved words alone.
    ///
    /// # Arguments
    ///
    /// * `words` - The words, as [`RESERVED`] spells them.
    ///
    /// # Returns
    ///
    /// The row.
    pub const fn within(self, words: &'static [&'static str]) -> Alias
    {
        Alias { within: words, ..self }
    }

    /// The row, holding in each reserved word but some.
    ///
    /// # Arguments
    ///
    /// * `words` - The words, as [`RESERVED`] spells them.
    ///
    /// # Returns
    ///
    /// The row.
    pub const fn except(self, words: &'static [&'static str]) -> Alias
    {
        Alias { except: words, ..self }
    }

    /// Whether the row holds in a reserved word.
    ///
    /// # Arguments
    ///
    /// * `word` - The word, as [`RESERVED`] spells it.
    ///
    /// # Returns
    ///
    /// `true` if it does.
    fn holds_in(&self, word: &str) -> bool
    {
        let listed = self.within.is_empty() || self.within.contains(&word);
        listed && !self.except.contains(&word)
    }
}

/// A written spelling of a reserved word: the spelling [`RESERVED`] gives, or
/// one that the written rows of [`ALIASES`] make from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written
{
    /// The spelling.
    pub text: String,
    /// The reserved word.
    pub kw: Kw,
    /// The spelling [`RESERVED`] gives the word.
    pub first: &'static str,
}

/// Each written spelling: each word of [`RESERVED`], in its order, then the
/// spellings the written rows make from it.
pub static WRITTEN: LazyLock<Vec<Written>> = LazyLock::new(|| {
    let mut seen = HashSet::new();
    let each = RESERVED.iter().flat_map(|(word, kw)| {
        let spellings = written_spellings(word);
        spellings.into_iter().map(|text| Written { text, kw: *kw, first: word })
    });
    each.filter(|w| seen.insert((w.text.clone(), w.kw))).collect()
});

/// The places in a text where a vanna ends, and its start.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The offsets in u8s, from 0 to the length.
fn vanna_ends(text: &str) -> Vec<usize>
{
    let vannas = segment(text, &SegmenterOptions::default());
    let lens = vannas.iter().map(|v| v.lexeme.len());
    std::iter::once(0)
        .chain(lens.scan(0, |at, len| {
            *at += len;
            Some(*at)
        }))
        .collect()
}

/// Each place a part stands in a text, on vanna ends.
///
/// # Arguments
///
/// * `text` - The text.
/// * `part` - The part.
/// * `at_end` - Whether the part must end the text.
///
/// # Returns
///
/// The start of each place, in u8s.
fn places(text: &str, part: &str, at_end: bool) -> Vec<usize>
{
    let ends = vanna_ends(text);
    let fits = |start: &usize| {
        let stop = start + part.len();
        let tail = !at_end || stop == text.len();
        text[*start..].starts_with(part) && ends.contains(&stop) && tail
    };
    ends.iter().copied().filter(fits).collect()
}

/// The text with the part at a place written as another part.
///
/// # Arguments
///
/// * `text` - The text.
/// * `start` - The place, in u8s.
/// * `from` - The part there.
/// * `to` - The part to write.
///
/// # Returns
///
/// The new text.
fn swapped(text: &str, start: usize, from: &str, to: &str) -> String
{
    format!("{}{to}{}", &text[..start], &text[start + from.len()..])
}

/// Each text one written row makes from a text, with one part written as
/// another.
///
/// # Arguments
///
/// * `row` - The row.
/// * `text` - The text.
///
/// # Returns
///
/// The texts.
fn written_swaps(row: &Alias, text: &str) -> Vec<String>
{
    let mut out = Vec::new();
    for from in row.parts
    {
        let starts = places(text, from, row.at_end);
        let others = row.parts.iter().filter(|to| *to != from);
        let each =
            others.flat_map(|to| starts.iter().map(|s| swapped(text, *s, from, to)));
        out.extend(each);
    }
    out
}

/// Each written spelling of a reserved word: the word itself first, then each
/// spelling that one or more written parts swapped make.
///
/// # Arguments
///
/// * `word` - The word, as [`RESERVED`] spells it.
///
/// # Returns
///
/// The spellings, with no repeat.
pub fn written_spellings(word: &str) -> Vec<String>
{
    let rows: Vec<&Alias> = ALIASES
        .iter()
        .filter(|r| r.register == Register::Written && r.holds_in(word))
        .collect();
    let mut all = vec![word.to_string()];
    let mut next = 0;
    while next < all.len()
    {
        let text = all[next].clone();
        next += 1;
        let made: Vec<String> =
            rows.iter().flat_map(|r| written_swaps(r, &text)).collect();
        do_for!(t in made, do_if!(!all.contains(&t), all.push(t)));
    }
    all
}

/// The spoken spelling of a written spelling: each spoken row that holds in
/// the reserved word writes its spoken part in place of its written one, at
/// each place at once.
///
/// # Arguments
///
/// * `text` - The written spelling.
/// * `word` - The reserved word, as [`RESERVED`] spells it.
///
/// # Returns
///
/// The spoken spelling, or `None` when no spoken row holds in it.
pub fn spoken_spelling(text: &str, word: &str) -> Option<String>
{
    let ends = vanna_ends(text);
    let rows: Vec<&Alias> = ALIASES
        .iter()
        .filter(|r| r.register == Register::Spoken && r.holds_in(word))
        .collect();
    let vannas = ends.windows(2).map(|w| &text[w[0]..w[1]]);
    let swap = |v: &str| rows.iter().find(|r| r.parts[0] == v).map(|r| r.parts[1]);
    let spoken: String = vannas.clone().map(|v| swap(v).unwrap_or(v)).collect();
    let changed = vannas.into_iter().any(|v| swap(v).is_some());
    changed.then_some(spoken)
}
