//! The rule for an undeclared name (ZK0301): the words of the file closest to
//! the one written, with look-alike letters counted as the same letter.

use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Fix;
use zakac_i18n::t;
use zakac_rust_sugars::ret_if;
use zakac_span::source_file::SourceFile;

use crate::check::offsets_in;
use crate::check::rule_fix;

/// The most names offered for one undeclared name.
const MAX_NAMES: usize = 3;

/// Letters that look the same: the digit zero and `ဝ`, and the two shapes of
/// `ဥ`. Each pair counts as one letter.
const LOOK_ALIKE: &[(char, char)] =
    &[('\u{1040}', '\u{101D}'), ('\u{1009}', '\u{1025}')];

/// Characters that end a word besides whitespace.
const ENDS: &[char] = &['။', '၊', '(', ')', '"', ','];

/// The fixes of an undeclared name: the closest words of the file, closest
/// first.
///
/// shortcut: the words of the root file stand in for the names in scope;
/// take the names of the checker, imports included, when a file holds few of
/// the names it uses.
///
/// # Arguments
///
/// * `root` - The file.
/// * `d` - The diagnostic.
///
/// # Returns
///
/// The fixes; none for another code.
pub(crate) fn fixes(root: &SourceFile, d: &Diagnostic) -> Vec<Fix>
{
    ret_if!(d.code != Code::UndeclaredName, Vec::new());
    let Some((lo, hi)) = offsets_in(root, &d.span)
    else
    {
        return Vec::new();
    };
    let written = &root.src[lo..hi];
    let limit = (written.chars().count() / 3).max(1);
    let words = root.src.split(|c: char| c.is_whitespace() || ENDS.contains(&c));
    let others = words.filter(|w| !w.is_empty() && *w != written);
    let mut near: Vec<(usize, &str)> = others
        .map(|w| (distance(written, w), w))
        .filter(|(n, _)| *n <= limit)
        .collect();
    near.sort_unstable();
    near.dedup();
    let fix = |(_, w): (usize, &str)| {
        let message = t!("fixes.near_name", w = w);
        rule_fix(root, message, lo, hi, w)
    };
    near.into_iter().take(MAX_NAMES).map(fix).collect()
}

/// The edit distance of two words, in characters, look-alikes counted as one
/// letter.
///
/// # Arguments
///
/// * `a` - One word.
/// * `b` - The other.
///
/// # Returns
///
/// The distance.
fn distance(a: &str, b: &str) -> usize { strsim::levenshtein(&plain(a), &plain(b)) }

/// A word with each look-alike letter written as its pair.
///
/// # Arguments
///
/// * `word` - The word.
///
/// # Returns
///
/// The word.
fn plain(word: &str) -> String
{
    let one =
        |c: char| LOOK_ALIKE.iter().find(|(from, _)| *from == c).map_or(c, |p| p.1);
    word.chars().map(one).collect()
}
