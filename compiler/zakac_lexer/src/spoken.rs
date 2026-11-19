//! The spoken spellings of the reserved words: `ထားမယ်` for `ထားမည်`,
//! `ထဲမှာ` for `ထဲတွင်`. Each lexes as its written word; the lint decides
//! whether a program can use them (ZK0510).
//!
//! The spellings come from the spoken rows of
//! [`ALIASES`](crate::aliases::ALIASES), applied to each written spelling of
//! [`WRITTEN`].

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::LazyLock;

use crate::aliases::WRITTEN;
use crate::aliases::Written;
use crate::aliases::spoken_spelling;
use crate::token::Kw;

/// The spoken form of `၏`, which is a mark and no reserved word.
pub const SPOKEN_EI: &str = "ရဲ့";

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

/// Each spoken spelling, in the order of [`WRITTEN`]. Two written words in
/// `သော` and `သည့်` give one spoken spelling in `တဲ့`: it reads as the word in
/// `သော` and also stands for the other.
pub static SPOKEN: LazyLock<Vec<Spoken>> = LazyLock::new(|| {
    // Two written spellings of one word, such as `သုံးသော` and `သုံးသည့်`, give
    // one spoken spelling; the first keeps it.
    let mut seen = HashSet::new();
    let made = |w: &'static Written| {
        let spoken = spoken_spelling(&w.text, w.first)?;
        Some(Spoken { spoken, kw: w.kw, written: w.text.as_str(), also: None })
    };
    let all: Vec<Spoken> = WRITTEN
        .iter()
        .filter_map(made)
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

/// The written spelling of a word: the first spelling of the reserved word
/// a spoken or a second written spelling stands for, `၏` for `ရဲ့`, and each
/// other word as it is.
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
    let written = spoken(text).map_or(text, |s| s.written);
    let first = FIRST.get(written).copied();
    first.or(ei).unwrap_or(written)
}

/// The first spelling of each written spelling of a reserved word.
static FIRST: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| WRITTEN.iter().map(|w| (w.text.as_str(), w.first)).collect());
