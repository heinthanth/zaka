use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoStaticStr;
pub use zakac_diagnostics::lint::BadLintName;
pub use zakac_diagnostics::lint::LINT_CODES;
pub use zakac_diagnostics::lint::LintGroup;
/// How the checker reports what it finds: the level of the pointer lint.
pub use zakac_diagnostics::lint::LintLevel as MemCheck;
pub use zakac_diagnostics::lint::LintLevels;
pub use zakac_diagnostics::lint::lint_codes;

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
    /// A run of a REPL session, whose module leaves out the statements of
    /// earlier chunks: no last use turns into a move, since a later chunk can
    /// read the variable again, and each check of the run remains, since the
    /// proofs miss the sets of earlier chunks.
    #[builder(default)]
    pub repl: bool,
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
