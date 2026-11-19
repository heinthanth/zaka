//! The aliases of the parts of reserved words: parts that mean the same, so a
//! reserved word written with one part reads as the word written with another.
//!
//! [`ALIASES`] is the table to edit. Each row is one meaning:
//!
//! - `base(part)`: the part, as plain Zaka writes it.
//! - `.written(&[forms])`: the other written forms of the part, each plain
//!   Zaka. A reserved word that holds the part or one of these reads the same
//!   with each other one: with `base("၍").written(&["ပြီး", "ကာ"])`, `ဖြစ်ပြီး`
//!   and `ဖြစ်ကာ` read as `ဖြစ်၍`.
//! - `.spoken(&[forms])`: the spoken forms, each of which stands for the part
//!   and each written form; the lint reports them (ZK0510) unless a program
//!   allows them. A word takes each of its spoken parts at once: `ထားမည်` is
//!   `ထားမယ်`, and both `ဟူသော` and `ဟူသည့်` are `ဟူတဲ့`.
//!
//! A row can tell where it holds:
//!
//! - `.at_end()`: a part counts at the end of a word alone, as the ending of a
//!   verb does.
//! - `.swaps_within(&[words])`: the written forms swap in these reserved words
//!   alone; the spoken forms still hold in each word.
//! - `.except(&[words])`: neither the written forms nor the spoken forms hold
//!   in these words, where the part is no particle.
//!
//! The words of `swaps_within` and `except` are spelled as [`RESERVED`] spells
//! them. A part matches whole vannas, so the `မှ` of `မှတ်` is no part `မှ`.
//! Where parts of several rows start at one place, the longest part wins, and
//! each row of that part gives its spoken forms: `base("ရှိစေရန်")` speaks for
//! `ရှိစေရန်` where `base("ရန်")` would speak for its `ရန်`.
//! - `.alone()`: the part is a word of its own, a mark such as `၏`, and no part
//!   of a reserved word; the lexer reads its spoken forms as the mark.
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
use zakac_rust_sugars::tern;

use crate::reserved::RESERVED;
use crate::token::Kw;

#[rustfmt::skip]
pub static ALIASES: &[Alias] = &[
    // "and then": the ending of a verb that the sentence goes on from
    base("၍").written(&["ပြီး", "ကာ"]).at_end().except(&["အကယ်၍", "ပြီးဆုံးပါပြီး"]),
    // "that": the ending of a verb before its noun
    base("သော").written(&["သည့်"]).spoken(&["တဲ့"])
        .swaps_within(&["သုံးသော", "လက်ခံသော", "ပြန်ပေးသော"]),
    // "at", "in"; `၌` is the literary form
    base("တွင်").written(&["၌"]).spoken(&["မှာ"]),
    // "will": the end of a statement; in `မည်သည်ကိုမှ` ("nothing") and
    // `အမည်ဖြင့်` ("under the name") it is no particle
    base("မည်").spoken(&["မယ်"]).except(&["မည်သည်ကိုမှ", "အမည်ဖြင့်"]),
    // "that will", before a name
    base("မည့်").spoken(&["မယ့်"]),
    // the end of a sentence, and the particle on a verb
    base("သည်").spoken(&["တယ်"]).except(&["မည်သည်ကိုမှ"]),
    // the literary end of a sentence, `ဖြစ်၏`; `၏` alone is "of", so the
    // word `သည်` keeps one spelling
    base("သည်").written(&["၏"]).at_end().except(&["သည်", "မည်သည်ကိုမှ"]),
    // "if"
    base("လျှင်").spoken(&["ရင်"]),
    // the mark of a quote
    base("ဟု").spoken(&["လို့"]),
    // "from"
    base("မှ").spoken(&["ကနေ"]).except(&["မည်သည်ကိုမှ"]),
    // "to"
    base("သို့").spoken(&["ကို"]).except(&["သို့မဟုတ်", "သို့မဟုတ်ပါက", "ထိုသို့မဟုတ်ဘဲ"]),
    // "of"
    base("၏").spoken(&["ရဲ့"]).alone(),
    // "and", "with"
    base("နှင့်").spoken(&["နဲ့"]),
    // "this"
    base("ဤ").spoken(&["ဒီ"]),
    // "for", "so as to"
    base("ရန်").spoken(&["ဖို့"]),
    // "so as to have", in the head of a trait
    base("ရှိစေရန်").spoken(&["ရှိစေဖို့", "ရှိစေအောင်"]),
    // "note that"
    base("ရန်").written(&["ရမည်"]).swaps_within(&["သတိပြုရန်မှာ"]),
    // "although"
    base("သော်လည်း").spoken(&["ပေမယ့်"]),
    // the closer of a block
    base("ပြီးဆုံးပါပြီး").written(&["ပြီးဆုံးပါပြီ", "ပြီးပါပြီး", "ပြီးပါပြီ"]),
    // the opener of an else-if
    base("ထိုသို့မဟုတ်ဘဲ").spoken(&["ဒါမှမဟုတ်ဘဲ"]),
    // the opener of an else; a bare `မဟုတ်လျှင်` is "if not", so it is none
    base("သို့မဟုတ်ပါက").spoken(&["ဒါမှမဟုတ်လျှင်", "ဒါမှမဟုတ်ရင်"]),
    // "or"
    base("သို့မဟုတ်").spoken(&["ဒါမှမဟုတ်"]),
    // "storage"
    base("မင်မိုရီ").written(&["မှတ်ဉာဏ်"]),
];

/// A row of [`ALIASES`]: one part and its other forms.
#[derive(Debug, Clone, Copy)]
pub struct Alias
{
    /// The part, as plain Zaka writes it.
    pub base: &'static str,
    /// The other written forms of the part.
    pub written: &'static [&'static str],
    /// The spoken forms of the part; none when speech has no other form.
    pub spoken: &'static [&'static str],
    /// Whether a part counts at the end of a word alone.
    pub at_end: bool,
    /// The reserved words the written forms swap in; each word when it lists
    /// none.
    pub swaps_within: &'static [&'static str],
    /// The reserved words the row does not hold in.
    pub except: &'static [&'static str],
    /// Whether the part is a word of its own, a mark, and no part of a
    /// reserved word.
    pub alone: bool,
}

/// A row of one part, with no other forms so far.
///
/// # Arguments
///
/// * `part` - The part, as plain Zaka writes it.
///
/// # Returns
///
/// The row, which holds at each place in each reserved word.
pub const fn base(part: &'static str) -> Alias
{
    Alias {
        base: part,
        written: &[],
        spoken: &[],
        at_end: false,
        swaps_within: &[],
        except: &[],
        alone: false,
    }
}

impl Alias
{
    /// The row, with the other written forms of its part.
    ///
    /// # Arguments
    ///
    /// * `forms` - The forms, each plain Zaka.
    ///
    /// # Returns
    ///
    /// The row.
    pub const fn written(self, forms: &'static [&'static str]) -> Alias
    {
        Alias { written: forms, ..self }
    }

    /// The part and each written form of it.
    ///
    /// # Returns
    ///
    /// The forms, the part first.
    pub fn forms(&self) -> impl Iterator<Item = &'static str>
    {
        std::iter::once(self.base).chain(self.written.iter().copied())
    }

    /// The row, with the spoken forms of its part.
    ///
    /// # Arguments
    ///
    /// * `forms` - The spoken forms, each of which stands for the part and each
    ///   written form.
    ///
    /// # Returns
    ///
    /// The row.
    pub const fn spoken(self, forms: &'static [&'static str]) -> Alias
    {
        Alias { spoken: forms, ..self }
    }

    /// The row, with its forms at the end of a word alone.
    ///
    /// # Returns
    ///
    /// The row.
    pub const fn at_end(self) -> Alias { Alias { at_end: true, ..self } }

    /// The row, with its written forms swapping in some reserved words alone.
    ///
    /// # Arguments
    ///
    /// * `words` - The words, as [`RESERVED`] spells them.
    ///
    /// # Returns
    ///
    /// The row.
    pub const fn swaps_within(self, words: &'static [&'static str]) -> Alias
    {
        Alias { swaps_within: words, ..self }
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

    /// Whether the written forms of the row swap in a reserved word.
    ///
    /// # Arguments
    ///
    /// * `word` - The word, as [`RESERVED`] spells it.
    ///
    /// # Returns
    ///
    /// `true` if they do.
    fn swaps_in(&self, word: &str) -> bool
    {
        let listed = self.swaps_within.is_empty() || self.swaps_within.contains(&word);
        !self.alone
            && !self.written.is_empty()
            && listed
            && !self.except.contains(&word)
    }

    /// Whether the spoken forms of the row hold in a reserved word.
    ///
    /// # Arguments
    ///
    /// * `word` - The word, as [`RESERVED`] spells it.
    ///
    /// # Returns
    ///
    /// `true` if they do.
    fn speaks_in(&self, word: &str) -> bool
    {
        !self.alone && !self.spoken.is_empty() && !self.except.contains(&word)
    }

    /// The row, with its part a word of its own: a mark, and no part of a
    /// reserved word.
    ///
    /// # Returns
    ///
    /// The row.
    pub const fn alone(self) -> Alias { Alias { alone: true, ..self } }
}

/// The mark a spoken word stands for, by the rows of [`ALIASES`] marked
/// `.alone()`: `၏` for `ရဲ့`.
///
/// # Arguments
///
/// * `spoken` - The word.
///
/// # Returns
///
/// The mark, or `None` for a word that is no spoken form of a mark.
pub fn mark_of(spoken: &str) -> Option<&'static str>
{
    let row = ALIASES.iter().find(|r| r.alone && r.spoken.contains(&spoken))?;
    Some(row.base)
}

/// Each mark with its spoken forms, by the rows of [`ALIASES`] marked
/// `.alone()`.
///
/// # Returns
///
/// Each mark and spoken form.
pub fn marks() -> impl Iterator<Item = (&'static str, &'static str)>
{
    let alone = ALIASES.iter().filter(|r| r.alone);
    alone.flat_map(|r| r.spoken.iter().map(|s| (r.base, *s)))
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
    for from in row.forms()
    {
        let starts = places(text, from, row.at_end);
        let others = row.forms().filter(|to| *to != from);
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
    let rows: Vec<&Alias> = ALIASES.iter().filter(|r| r.swaps_in(word)).collect();
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

/// The spoken spellings of a written spelling: each spoken row that holds in
/// the reserved word writes a spoken form in place of its part, at each place
/// at once, one spelling for each choice of the forms. Where parts start at
/// one place, the longest wins, and each row of it gives its forms.
///
/// # Arguments
///
/// * `text` - The written spelling.
/// * `word` - The reserved word, as [`RESERVED`] spells it.
///
/// # Returns
///
/// The spoken spellings; none when no spoken row holds in it.
pub fn spoken_spellings(text: &str, word: &str) -> Vec<String>
{
    let ends = vanna_ends(text);
    let rows: Vec<&Alias> = ALIASES.iter().filter(|r| r.speaks_in(word)).collect();
    let mut all = vec![String::new()];
    let mut spoke = false;
    let mut at = 0;
    while at + 1 < ends.len()
    {
        let (stop, forms) = spoken_at(text, &ends, ends[at], &rows);
        spoke |= !forms.is_empty();
        let choices = tern!(forms.is_empty(), vec![&text[ends[at]..stop]], forms);
        let grown =
            all.iter().flat_map(|a| choices.iter().map(move |c| format!("{a}{c}")));
        all = grown.collect();
        at = ends.iter().position(|e| *e == stop).unwrap_or(ends.len());
    }
    tern!(spoke, all, Vec::new())
}

/// The spoken forms of the longest part of a spoken row that starts at a
/// place, on vanna ends.
///
/// # Arguments
///
/// * `text` - The written spelling.
/// * `ends` - Its vanna ends.
/// * `start` - The place, in u8s.
/// * `rows` - The spoken rows that hold in the word.
///
/// # Returns
///
/// The end of the part and the forms of each row of it; the end of the vanna
/// and no forms when no part starts there.
fn spoken_at(
    text: &str, ends: &[usize], start: usize, rows: &[&Alias],
) -> (usize, Vec<&'static str>)
{
    let fits = |r: &Alias, f: &str| {
        let stop = start + f.len();
        let tail = !r.at_end || stop == text.len();
        text[start..].starts_with(f) && ends.contains(&stop) && tail
    };
    let parts = rows.iter().flat_map(|r| {
        r.forms().filter(|f| fits(r, f)).map(move |f| (start + f.len(), *r))
    });
    let parts: Vec<(usize, &Alias)> = parts.collect();
    let next = ends.iter().copied().find(|e| *e > start).unwrap_or(text.len());
    let Some(stop) = parts.iter().map(|(stop, _)| *stop).max()
    else
    {
        return (next, Vec::new());
    };
    let mut forms: Vec<&'static str> = Vec::new();
    let longest = parts.iter().filter(|(s, _)| *s == stop).flat_map(|(_, r)| r.spoken);
    do_for!(f in longest, do_if!(!forms.contains(f), forms.push(f)));
    (stop, forms)
}
