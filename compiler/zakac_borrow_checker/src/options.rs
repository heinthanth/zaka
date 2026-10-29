use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoStaticStr;

/// How the ownership check reports what it finds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[derive(Serialize, Deserialize, Display, EnumIter, EnumString, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum MemCheck
{
    /// Each finding is an error and stops code generation.
    #[default]
    Error,
    /// Each finding is a warning.
    Warn,
    /// The check reports nothing.
    Off,
}

/// Options of the ownership check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Builder)]
pub struct BorrowCheckOptions
{
    /// How the findings are reported.
    #[builder(default)]
    pub check: MemCheck,
    /// Also report the accesses that cannot be proven safe.
    #[builder(default)]
    pub strict: bool,
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
