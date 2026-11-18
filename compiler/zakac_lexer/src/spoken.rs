//! The spoken spellings of the reserved words: `ထားမယ်` for `ထားမည်`,
//! `ထဲမှာ` for `ထဲတွင်`. Each lexes as its written word; the lint decides
//! whether a program can use them (ZK0510).
//!
//! The spellings come from [`RESERVED`]: each spoken part takes the place of
//! its written part, a whole vanna at a time, so the `မှ` of `မှတ်` remains.

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::LazyLock;

use zakac_burmese_segmenter::segmenter::SegmenterOptions;
use zakac_burmese_segmenter::segmenter::segment;

use crate::reserved::RESERVED;
use crate::token::Kw;

/// Each written vanna with its spoken form.
#[rustfmt::skip]
pub static SPOKEN_PARTS: &[(&str, &str)] = &[
    ("မည်", "မယ်"),
    ("မည့်", "မယ့်"),
    ("သည်", "တယ်"),
    ("သည့်", "တဲ့"),
    ("သော", "တဲ့"),
    ("တွင်", "မှာ"),
    ("လျှင်", "ရင်"),
    ("ဟု", "လို့"),
    ("မှ", "ကနေ"),
    ("နှင့်", "နဲ့"),
    ("ဤ", "ဒီ"),
];

/// The spoken form of `၏`, which is a mark and no reserved word.
pub const SPOKEN_EI: &str = "ရဲ့";

/// Reserved words in which a written part is no particle, so each has no
/// spoken spelling: the `မည်` and `သည်` of `မည်သည်ကိုမှ` ask "which", and the
/// `မည်` of `အမည်ဖြင့်` is the end of `အမည်`, "name".
const NO_SPOKEN: &[&str] = &["မည်သည်ကိုမှ", "အမည်ဖြင့်"];

/// One spoken spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spoken
{
    /// The spoken spelling.
    pub spoken: String,
    /// The reserved word it reads as.
    pub kw: Kw,
    /// The written spelling it stands for.
    pub written: &'static str,
    /// The other reserved word it can stand for, with its written spelling:
    /// `ရှိတဲ့` is `ရှိသော` and also `ရှိသည့်`, which the parser
    /// reads where its grammar asks for that word.
    pub also: Option<(Kw, &'static str)>,
}

/// Each spoken spelling, in the order of [`RESERVED`]. Two written words in
/// `သော` and `သည့်` give one spoken spelling in `တဲ့`: it reads as the word in
/// `သော` and also stands for the other.
pub static SPOKEN: LazyLock<Vec<Spoken>> = LazyLock::new(|| {
    // Two written spellings of one word, such as `သုံးသော` and `သုံးသည့်`, give
    // one spoken spelling; the first in the table keeps it.
    let mut seen = HashSet::new();
    let all: Vec<Spoken> = RESERVED
        .iter()
        .filter_map(|(w, kw)| spoken_of(w, *kw))
        .filter(|s| seen.insert((s.spoken.clone(), s.kw)))
        .collect();
    let rival = |s: &Spoken| all.iter().find(|o| o.spoken == s.spoken && o.kw != s.kw);
    let keep = |s: &Spoken| {
        let Some(other) = rival(s)
        else
        {
            return Some(s.clone());
        };
        let also = Some((other.kw, other.written));
        s.written.ends_with("သော").then(|| Spoken { also, ..s.clone() })
    };
    all.iter().filter_map(keep).collect()
});

/// [`SPOKEN`] as a map from the spoken spelling.
static BY_SPELLING: LazyLock<HashMap<&'static str, &'static Spoken>> =
    LazyLock::new(|| SPOKEN.iter().map(|s| (s.spoken.as_str(), s)).collect());

/// [`SPOKEN`] as a map from the written spelling, with `၏`.
static BY_WRITTEN: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        let also = |s: &'static Spoken| s.also.map(|(_, w)| (w, s.spoken.as_str()));
        let words = SPOKEN.iter().map(|s| (s.written, s.spoken.as_str()));
        let words = words.chain(SPOKEN.iter().filter_map(also));
        words.chain([("၏", SPOKEN_EI)]).collect()
    });

/// The spoken spelling of a written word.
///
/// # Arguments
///
/// * `written` - The written word.
/// * `kw` - The reserved word it is.
///
/// # Returns
///
/// The spelling, or `None` when the word has no spoken part or is one of
/// [`NO_SPOKEN`].
fn spoken_of(written: &'static str, kw: Kw) -> Option<Spoken>
{
    (!NO_SPOKEN.contains(&written)).then_some(())?;
    let vannas = segment(written, &SegmenterOptions::default());
    let part = |s: &str| SPOKEN_PARTS.iter().find(|(w, _)| *w == s).map(|(_, sp)| *sp);
    let swapped = vannas.iter().any(|t| part(t.lexeme).is_some());
    swapped.then_some(())?;
    let spoken = vannas.iter().map(|t| part(t.lexeme).unwrap_or(t.lexeme)).collect();
    Some(Spoken { spoken, kw, written, also: None })
}

/// The spoken spelling a word is.
///
/// # Arguments
///
/// * `text` - The word, in normalized spelling.
///
/// # Returns
///
/// The spelling, or `None` for a word that is no spoken spelling.
pub fn spoken(text: &str) -> Option<&'static Spoken> { BY_SPELLING.get(text).copied() }

/// The length in u8s of the longest spoken spelling.
///
/// # Returns
///
/// The length.
pub fn longest() -> usize { SPOKEN.iter().map(|s| s.spoken.len()).max().unwrap_or(0) }

/// The spoken spelling of a written word, `ရဲ့` for `၏`.
///
/// # Arguments
///
/// * `text` - The written word, in normalized spelling.
///
/// # Returns
///
/// The spoken spelling, or `None` for a word that has none.
pub fn spoken_twin(text: &str) -> Option<&'static str> { BY_WRITTEN.get(text).copied() }

/// The second spellings of reserved words, each with the spelling a
/// preposition or a phrase is written in: `မှတ်ဉာဏ်` reads as `မင်မိုရီ`.
const SYNONYMS: &[(&str, &str)] = &[("မှတ်ဉာဏ်", "မင်မိုရီ")];

/// The written spelling of a word: the written word a spoken spelling stands
/// for, `၏` for `ရဲ့`, the first spelling of a reserved word with two, and
/// each other word as it is.
///
/// # Arguments
///
/// * `text` - The word, in normalized spelling.
///
/// # Returns
///
/// The written spelling.
pub fn written_form(text: &str) -> &str
{
    let ei = (text == SPOKEN_EI).then_some("၏");
    let twin =
        SYNONYMS.iter().find(|(second, _)| *second == text).map(|(_, first)| *first);
    spoken(text).map(|s| s.written).or(ei).or(twin).unwrap_or(text)
}
