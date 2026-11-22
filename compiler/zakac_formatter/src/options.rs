//! The settings of the formatter. Each one is a flag of `zakac fmt` and a key
//! of the `[fmt]` section of `zakac.toml`.

use std::str::FromStr;

use bon::Builder;
use strum::EnumString;
use strum::IntoStaticStr;
use strum::VariantNames;

/// The settings of the formatter.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct FmtOptions
{
    /// Spaces for each level of block depth; `0` keeps each line flush left.
    #[builder(default)]
    pub indent: usize,
    /// The indent of the later lines of one statement.
    #[builder(default)]
    pub continuation_indent: ContinuationIndent,
    /// The longest run of blank lines that is kept.
    #[builder(default = 1)]
    pub blank_lines_max: usize,
    /// A line break after `။` when two statements share a line.
    #[builder(default = true)]
    pub one_statement_per_line: bool,
    /// Write the Burmese marks in their canonical order; off, each word keeps
    /// the order of the marks as written.
    #[builder(default = true)]
    pub mark_order: bool,
    /// The form each reserved word is written in.
    #[builder(default)]
    pub speech_mode: SpeechMode,
    /// The script of the digits of number literals.
    #[builder(default)]
    pub digits: Digits,
    /// The order of the imports at the head of a file.
    #[builder(default)]
    pub import_order: ImportOrder,
}

impl Default for FmtOptions
{
    fn default() -> Self { Self::builder().build() }
}

/// The indent of the later lines of one statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContinuationIndent
{
    /// Each line keeps the spaces the author put in front of it.
    #[default]
    Preserve,
    /// This count of spaces past the depth of the statement.
    Spaces(usize),
}

impl FromStr for ContinuationIndent
{
    type Err = std::num::ParseIntError;

    /// Read `preserve` or a count of spaces.
    ///
    /// # Arguments
    ///
    /// * `text` - The setting as written.
    ///
    /// # Returns
    ///
    /// The setting, or the error of a count that is not a number.
    fn from_str(text: &str) -> Result<Self, Self::Err>
    {
        if text == "preserve"
        {
            return Ok(Self::Preserve);
        }
        text.parse().map(Self::Spaces)
    }
}

/// The form each reserved word is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumString, IntoStaticStr)]
#[derive(VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum SpeechMode
{
    /// The written words: `ထားမည်`, `တွင်`.
    #[default]
    Written,
    /// The spoken spellings: `ထားမယ်`, `မှာ`.
    Spoken,
}

/// The script of the digits of number literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumString, IntoStaticStr)]
#[derive(VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum Digits
{
    /// Each literal keeps its digits.
    #[default]
    Preserve,
    /// Burmese digits: `၁၀`.
    Burmese,
    /// Latin digits: `10`.
    Latin,
}

/// The order of the imports at the head of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumString, IntoStaticStr)]
#[derive(VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum ImportOrder
{
    /// The imports keep their order.
    #[default]
    Preserve,
    /// Each run of imports is sorted: the stdlib first, then the others, each
    /// group in the order of its path.
    Sorted,
}
