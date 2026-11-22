//! The lint: the codes whose level a reader sets through a name with
//! `--lint-allow`, `--lint-warn` and `--lint-error`, alone or as a group, and
//! the levels set so far.

use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoStaticStr;

use crate::Code;
use crate::Diagnostic;
use crate::Level;

/// The level a lint code is reported at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[derive(Serialize, Deserialize, Display, EnumIter, EnumString, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum LintLevel
{
    /// Each finding is an error and stops code generation.
    Error,
    /// Each finding is a warning.
    #[default]
    Warn,
    /// The check reports nothing.
    Off,
}

/// The codes whose level `--lint-allow`, `--lint-warn` and `--lint-error` set:
/// the pointer lint, the warnings of the checks that a proof can leave in
/// doubt, the naturalness rules and the duplicates in trait values.
pub const LINT_CODES: [Code; 26] = [
    Code::BadFree,
    Code::BorrowStored,
    Code::InvalidatedLoan,
    Code::LocalBorrowReturned,
    Code::AliasedArgument,
    Code::BlockLeaked,
    Code::NullDeref,
    Code::OffsetPastEnd,
    Code::NullDerefPossible,
    Code::MixedBlocks,
    Code::OverflowProven,
    Code::BlitRangesOverlap,
    Code::ReadUnsetDoubt,
    Code::NegativeBeforeOr,
    Code::ParticleBeforeAttached,
    Code::NotWithThi,
    Code::PaBeforeAnd,
    Code::PrintSentenceWithKo,
    Code::PrintValueWithHu,
    Code::RedundantGroup,
    Code::PaInValue,
    Code::ClassifierMismatch,
    Code::SpokenForm,
    Code::MixedForms,
    Code::VerbBeforeNot,
    Code::TraitValueDuplicates,
];

/// The spoken spellings. Their mix with the written ones (ZK0511) is no
/// part of it, so `--lint-allow spoken` leaves that warning on.
const SPOKEN_CODES: &[Code] = &[Code::SpokenForm];

/// The naturalness rules, the `ZK05` codes.
const NATURALNESS_CODES: &[Code] = &[
    Code::NegativeBeforeOr,
    Code::ParticleBeforeAttached,
    Code::NotWithThi,
    Code::PaBeforeAnd,
    Code::PrintSentenceWithKo,
    Code::PrintValueWithHu,
    Code::RedundantGroup,
    Code::PaInValue,
    Code::ClassifierMismatch,
    Code::SpokenForm,
    Code::MixedForms,
    Code::VerbBeforeNot,
];

/// A named set of lint codes, for `--lint-allow`, `--lint-warn` and
/// `--lint-error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Display, EnumIter, EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum LintGroup
{
    /// The pointer lint of unsafe code, which `--mem-check` sets.
    Mem,
    /// A block that is never given back.
    Leaks,
    /// A free through the wrong pointer, or of a block another argument
    /// reaches.
    Frees,
    /// A borrow that is stored, outlived or returned.
    Borrows,
    /// A read through an address that is or can be null.
    Nulls,
    /// Pointer arithmetic and block copies.
    Pointers,
    /// The findings the checker cannot prove: shown under `--mem-strict`.
    Doubts,
    /// An integer operation that overflows for each value it can see.
    Overflow,
    /// The naturalness rules: particles, classifiers, parentheses and the
    /// spoken spellings.
    Naturalness,
    /// The spoken spellings: `--lint-allow spoken` lets a program use them.
    Spoken,
    /// A plain value that goes into a trait value as a duplicate, where a
    /// function of the trait changes the duplicate alone.
    Duplicates,
}

impl LintGroup
{
    /// The codes of the group.
    ///
    /// # Returns
    ///
    /// The codes.
    pub fn codes(self) -> &'static [Code]
    {
        match self
        {
            LintGroup::Mem => &[
                Code::BadFree,
                Code::BorrowStored,
                Code::InvalidatedLoan,
                Code::LocalBorrowReturned,
                Code::AliasedArgument,
                Code::BlockLeaked,
                Code::NullDeref,
                Code::OffsetPastEnd,
                Code::NullDerefPossible,
                Code::MixedBlocks,
                Code::BlitRangesOverlap,
            ],
            LintGroup::Leaks => &[Code::BlockLeaked],
            LintGroup::Frees => &[Code::BadFree, Code::AliasedArgument],
            LintGroup::Borrows =>
            {
                &[Code::BorrowStored, Code::InvalidatedLoan, Code::LocalBorrowReturned]
            }
            LintGroup::Nulls => &[Code::NullDeref, Code::NullDerefPossible],
            LintGroup::Pointers =>
            {
                &[Code::OffsetPastEnd, Code::MixedBlocks, Code::BlitRangesOverlap]
            }
            LintGroup::Doubts => &[Code::NullDerefPossible, Code::ReadUnsetDoubt],
            LintGroup::Overflow => &[Code::OverflowProven],
            LintGroup::Naturalness => NATURALNESS_CODES,
            LintGroup::Spoken => SPOKEN_CODES,
            LintGroup::Duplicates => &[Code::TraitValueDuplicates],
        }
    }
}

/// A lint name that `--lint-allow`, `--lint-warn` or `--lint-error` cannot
/// take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BadLintName
{
    /// The name is no group and no code.
    Unknown,
    /// The name is a code whose level cannot change.
    NotALint,
}

/// The level each lint code is set to through its name or group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct LintLevels([Option<LintLevel>; LINT_CODES.len()]);

impl LintLevels
{
    /// The level a code is set to through its name or group.
    ///
    /// # Arguments
    ///
    /// * `code` - The code.
    ///
    /// # Returns
    ///
    /// The level, or `None` when no name sets it.
    pub fn get(&self, code: Code) -> Option<LintLevel>
    {
        LINT_CODES.iter().position(|c| *c == code).and_then(|at| self.0[at])
    }

    /// The levels with one lint code set to a level, or back to the level of
    /// its own with `None`; a code outside the lint changes nothing.
    ///
    /// # Arguments
    ///
    /// * `code` - The code.
    /// * `level` - The level, or `None`.
    ///
    /// # Returns
    ///
    /// The levels.
    pub fn set(mut self, code: Code, level: Option<LintLevel>) -> Self
    {
        let at = LINT_CODES.iter().position(|c| *c == code);
        at.into_iter().for_each(|at| self.0[at] = level);
        self
    }

    /// The levels with one group or code set.
    ///
    /// # Arguments
    ///
    /// * `name` - A group, or a code such as `ZK0708`.
    /// * `level` - The level.
    ///
    /// # Returns
    ///
    /// The levels, or the reason the name cannot be set.
    pub fn with(mut self, name: &str, level: LintLevel) -> Result<Self, BadLintName>
    {
        let codes = lint_codes(name)?;
        for code in codes
        {
            let at = LINT_CODES.iter().position(|c| c == code);
            at.into_iter().for_each(|at| self.0[at] = Some(level));
        }
        Ok(self)
    }

    /// The levels with each of `codes` that no name sets put at a level.
    ///
    /// # Arguments
    ///
    /// * `codes` - The codes, such as the codes of a group.
    /// * `level` - The level.
    ///
    /// # Returns
    ///
    /// The levels.
    pub fn with_default(mut self, codes: &[Code], level: LintLevel) -> Self
    {
        for code in codes
        {
            let at = LINT_CODES.iter().position(|c| c == code);
            at.into_iter().for_each(|at| _ = self.0[at].get_or_insert(level));
        }
        self
    }

    /// Diagnostics at the levels their names set: a code set to
    /// [`LintLevel::Off`] is left out, one set to `Warn` or `Error` takes
    /// that level, and a code no name sets keeps its own.
    ///
    /// # Arguments
    ///
    /// * `diags` - The diagnostics.
    ///
    /// # Returns
    ///
    /// The diagnostics that remain, at their levels.
    pub fn relevel(&self, diags: Vec<Diagnostic>) -> Vec<Diagnostic>
    {
        let level = |d: Diagnostic| match self.get(d.code)
        {
            Some(LintLevel::Off) => None,
            Some(LintLevel::Warn) => Some(d.with_level(Level::Warning)),
            Some(LintLevel::Error) => Some(d.with_level(Level::Error)),
            None => Some(d),
        };
        diags.into_iter().filter_map(level).collect()
    }
}

/// The codes a lint name names.
///
/// # Arguments
///
/// * `name` - A group, or a code such as `ZK0708`.
///
/// # Returns
///
/// The codes, or the reason the name cannot be set.
pub fn lint_codes(name: &str) -> Result<&'static [Code], BadLintName>
{
    if let Ok(group) = name.parse::<LintGroup>()
    {
        return Ok(group.codes());
    }
    let found = Code::ALL.iter().position(|c| c.id().eq_ignore_ascii_case(name));
    let at = found.ok_or(BadLintName::Unknown)?;
    let code = &Code::ALL[at..=at];
    LINT_CODES.contains(&code[0]).then_some(code).ok_or(BadLintName::NotALint)
}

#[cfg(test)]
mod tests
{
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn names_set_groups_and_codes()
    {
        let levels = LintLevels::default().with("mem", LintLevel::Error).unwrap();
        assert_eq!(levels.get(Code::BlockLeaked), Some(LintLevel::Error));
        assert_eq!(levels.get(Code::OverflowProven), None);
        let levels = levels.with("zk0708", LintLevel::Off).unwrap();
        assert_eq!(levels.get(Code::BlockLeaked), Some(LintLevel::Off));
        assert_eq!(levels.get(Code::MixedBlocks), Some(LintLevel::Error));
    }

    #[test]
    fn names_outside_the_lint_are_refused()
    {
        let levels = LintLevels::default();
        assert_eq!(levels.with("ZK0701", LintLevel::Off), Err(BadLintName::NotALint));
        assert_eq!(levels.with("leak", LintLevel::Off), Err(BadLintName::Unknown));
        assert!(
            LintGroup::iter().all(|g| g.codes().iter().all(|c| LINT_CODES.contains(c)))
        );
    }

    #[test]
    fn a_group_default_leaves_a_named_code_as_it_is()
    {
        let levels = LintLevels::default().with("ZK0502", LintLevel::Off).unwrap();
        let levels =
            levels.with_default(LintGroup::Naturalness.codes(), LintLevel::Error);
        assert_eq!(levels.get(Code::ParticleBeforeAttached), Some(LintLevel::Off));
        assert_eq!(levels.get(Code::NegativeBeforeOr), Some(LintLevel::Error));
    }
}
