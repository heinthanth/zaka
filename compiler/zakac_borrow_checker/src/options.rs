use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoStaticStr;
use zakac_diagnostics::Code;

/// How the checker reports what it finds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[derive(Serialize, Deserialize, Display, EnumIter, EnumString, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum MemCheck
{
    /// Each finding is an error and stops code generation.
    Error,
    /// Each finding is a warning.
    #[default]
    Warn,
    /// The check reports nothing.
    Off,
}

/// What integer arithmetic does with a result outside its kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[derive(Serialize, Deserialize, Display, EnumIter, EnumString, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum OverflowMode
{
    /// The result wraps around, as two's complement arithmetic does.
    Wrap,
    /// The VM stops the program, and the check warns about an operation
    /// that overflows for each value it can see. A stop with a message
    /// teaches more than a silent wrap, so this is the default.
    #[default]
    Stop,
}

/// The codes whose level `--allow`, `--warn` and `--error` set: the pointer
/// lint, and the warnings of the checks that a proof can leave in doubt.
pub const LINT_CODES: [Code; 13] = [
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
];

/// A named set of lint codes, for `--allow`, `--warn` and `--error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Display, EnumIter, EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum LintGroup
{
    /// The pointer lint of marked code, which `--mem-check` sets.
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
        }
    }
}

/// A lint name that `--allow`, `--warn` or `--error` cannot take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BadLintName
{
    /// The name is no group and no code.
    Unknown,
    /// The name is a code whose level cannot change.
    NotALint,
}

/// The level each lint code is set to through its name or group, over the level
/// `--mem-check` gives the pointer lint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LintLevels([Option<MemCheck>; LINT_CODES.len()]);

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
    pub fn get(&self, code: Code) -> Option<MemCheck>
    {
        LINT_CODES.iter().position(|c| *c == code).and_then(|at| self.0[at])
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
    pub fn with(mut self, name: &str, level: MemCheck) -> Result<Self, BadLintName>
    {
        let codes = lint_codes(name)?;
        for code in codes
        {
            let at = LINT_CODES.iter().position(|c| c == code);
            at.into_iter().for_each(|at| self.0[at] = Some(level));
        }
        Ok(self)
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

/// Options of the checker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Builder)]
pub struct BorrowCheckOptions
{
    /// How the findings are reported.
    #[builder(default)]
    pub check: MemCheck,
    /// Also report the accesses that cannot be proven safe.
    #[builder(default)]
    pub strict: bool,
    /// What integer arithmetic does with a result outside its kind.
    #[builder(default)]
    pub overflow: OverflowMode,
    /// The level of each lint code set through a name, over `check`.
    #[builder(default)]
    pub lint: LintLevels,
}

impl Default for BorrowCheckOptions
{
    /// Errors on, strict off.
    ///
    /// # Returns
    ///
    /// The options.
    fn default() -> Self { Self::builder().build() }
}

#[cfg(test)]
mod tests
{
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn names_set_groups_and_codes()
    {
        let levels = LintLevels::default().with("mem", MemCheck::Error).unwrap();
        assert_eq!(levels.get(Code::BlockLeaked), Some(MemCheck::Error));
        assert_eq!(levels.get(Code::OverflowProven), None);
        let levels = levels.with("zk0708", MemCheck::Off).unwrap();
        assert_eq!(levels.get(Code::BlockLeaked), Some(MemCheck::Off));
        assert_eq!(levels.get(Code::MixedBlocks), Some(MemCheck::Error));
    }

    #[test]
    fn names_outside_the_lint_are_refused()
    {
        let levels = LintLevels::default();
        assert_eq!(levels.with("ZK0701", MemCheck::Off), Err(BadLintName::NotALint));
        assert_eq!(levels.with("leak", MemCheck::Off), Err(BadLintName::Unknown));
        assert!(
            LintGroup::iter().all(|g| g.codes().iter().all(|c| LINT_CODES.contains(c)))
        );
    }
}
