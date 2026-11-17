//! Rules for the particle warnings (ZK0501 to ZK0508): each finds, in the
//! source text, the word the warning is about and writes the idiomatic form.

use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Fix;
use zakac_i18n::t;
use zakac_span::source_file::SourceFile;

use crate::check::offsets_in;
use crate::check::rule_fix;

/// The statement particle.
const THI: &str = "သည်";
/// The polite particle.
const PA: &str = "ပါ";
/// The two particles together.
const PA_THI: &str = "ပါသည်";
/// The particle that marks a thing.
const KO: &str = "ကို";
/// The particle that quotes.
const HU: &str = "ဟု";
/// The verb "to be", which takes the attached word when the particle remains.
const BE_WORD: &str = "ဖြစ်";
/// The postfix negation.
const NOT: &str = "မဟုတ်";

/// The rule fixes of a particle warning.
///
/// # Arguments
///
/// * `root` - The file the warning points into.
/// * `d` - The warning.
///
/// # Returns
///
/// The fixes; none for another code or a span outside the file.
pub(crate) fn fixes(root: &SourceFile, d: &Diagnostic) -> Vec<Fix>
{
    let Some((lo, hi)) = offsets_in(root, &d.span)
    else
    {
        return Vec::new();
    };
    let at = Site { root, lo, hi, d };
    match d.code
    {
        Code::NegativeBeforeOr => at.swap_tail(THI, PA),
        Code::PaBeforeAnd | Code::PaInValue => at.swap_tail(PA, THI),
        Code::ParticleBeforeAttached => at.attached(),
        Code::NotWithThi => at.not_forms(),
        Code::PrintSentenceWithKo => at.swap_next(KO, HU),
        Code::PrintValueWithHu => at.swap_next(HU, KO),
        Code::RedundantGroup => at.unwrap_group(),
        _ => Vec::new(),
    }
}

/// The span of a warning in its file.
struct Site<'a>
{
    /// The file.
    root: &'a SourceFile,
    /// The start offset of the span.
    lo: usize,
    /// Its end offset.
    hi: usize,
    /// The warning.
    d: &'a Diagnostic,
}

impl Site<'_>
{
    /// The text of the file.
    fn src(&self) -> &str { &self.root.src }

    /// The start of the last word of the span: the clause-final word with its
    /// particle.
    fn word_start(&self) -> usize
    {
        let text = &self.src()[self.lo..self.hi];
        text.rfind([' ', '(']).map_or(self.lo, |i| self.lo + i + 1)
    }

    /// The help text of the warning, or "write ..." when it has none.
    fn message(&self, new: &str) -> String
    {
        let write = || t!("fixes.write", w = new).to_string();
        self.d.help.clone().unwrap_or_else(write)
    }

    /// Change the particle at the end of the last word.
    fn swap_tail(&self, from: &str, to: &str) -> Vec<Fix>
    {
        let start = self.word_start();
        let Some(verb) = self.src()[start..self.hi].strip_suffix(from)
        else
        {
            return Vec::new();
        };
        let new = format!("{verb}{to}");
        vec![rule_fix(self.root, self.message(&new), start, self.hi, new)]
    }

    /// A word with a particle before an attached `လျှင်` or `နေသရွေ့`: drop
    /// the particle, or keep it and put `ဖြစ်` before the attached word.
    fn attached(&self) -> Vec<Fix>
    {
        let start = self.word_start();
        let word = &self.src()[start..self.hi];
        let Some(verb) = bare(word)
        else
        {
            return Vec::new();
        };
        let rest = &self.src()[self.hi..];
        let next = rest.split(char::is_whitespace).next().unwrap_or_default();
        let bare = t!("fixes.write", w = format!("{verb}{next}"));
        let keep = t!("fixes.keep_particle", w = format!("{word} {BE_WORD}{next}"));
        vec![
            rule_fix(self.root, bare, start, self.hi, verb),
            rule_fix(self.root, keep, self.hi, self.hi, format!(" {BE_WORD}")),
        ]
    }

    /// `မဟုတ်` with a particle: `မဟုတ်ပါ`, or `မဟုတ်` alone.
    fn not_forms(&self) -> Vec<Fix>
    {
        let start = self.word_start();
        let word = &self.src()[start..self.hi];
        let Some(NOT) = bare(word)
        else
        {
            return Vec::new();
        };
        let with_pa = format!("{NOT}{PA}");
        let write = |w: &str| t!("fixes.write", w = w);
        vec![
            rule_fix(self.root, write(&with_pa), start, self.hi, with_pa.clone()),
            rule_fix(self.root, write(NOT), start, self.hi, NOT),
        ]
    }

    /// Change the particle written after the span.
    fn swap_next(&self, from: &str, to: &str) -> Vec<Fix>
    {
        let rest = &self.src()[self.hi..];
        let at = self.hi + rest.len() - rest.trim_start_matches(' ').len();
        let after = &self.src()[at..];
        let Some(tail) = after.strip_prefix(from)
        else
        {
            return Vec::new();
        };
        // `ကိုယ်` starts with `ကို`; the particle ends where its word ends.
        let ends = tail.chars().next().is_none_or(|c| c.is_whitespace() || c == '။');
        let fix = || rule_fix(self.root, self.message(to), at, at + from.len(), to);
        ends.then(fix).into_iter().collect()
    }

    /// Remove the parentheses around the span.
    fn unwrap_group(&self) -> Vec<Fix>
    {
        let text = &self.src()[self.lo..self.hi];
        let inner = text.strip_prefix('(').and_then(|t| t.strip_suffix(')'));
        let fix = |inner: &str| {
            rule_fix(self.root, self.message(inner), self.lo, self.hi, inner)
        };
        inner.map(fix).into_iter().collect()
    }
}

/// A word without its sentence-final particle.
///
/// # Arguments
///
/// * `word` - The word.
///
/// # Returns
///
/// The word before `ပါသည်`, `သည်` or `ပါ`, or `None` when it ends in none.
fn bare(word: &str) -> Option<&str>
{
    let strip = |p: &&str| word.strip_suffix(*p);
    [PA_THI, THI, PA].iter().find_map(strip).filter(|v| !v.is_empty())
}
