use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoStaticStr;

/// How a list or struct result reaches the caller. The caller gives each
/// call a slot of its own for the result; the two modes differ in the
/// number of copies, never in what the program prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[derive(Serialize, Deserialize, Display, EnumIter, EnumString, IntoStaticStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ResultMode
{
    /// The callee keeps its result in its own frame, and the return
    /// duplicates it into the caller's slot.
    Duplicate,
    /// The variable the callee gives back, or a literal it returns, is
    /// built in the caller's slot, so the return copies nothing.
    #[default]
    InPlace,
}

/// Where the scratch values of a function live: the lists, structs and
/// slices that a statement builds and uses before it ends, such as a literal
/// that is printed or a slice that is passed. The modes differ in the slots
/// a frame holds, never in what the program prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[derive(Serialize, Deserialize, Display, EnumIter, EnumString, IntoStaticStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ScratchMode
{
    /// Each scratch value has a slot of its own, so each one shows apart in
    /// the RAM view.
    #[default]
    Apart,
    /// Scratch values of one kind share a slot: a value is dead once its
    /// statement ends, so a later statement can take its slot.
    Shared,
}

/// The choices of code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Builder)]
#[derive(Serialize, Deserialize)]
pub struct EmitOptions
{
    /// How a list or struct result reaches the caller.
    #[builder(default)]
    #[serde(default)]
    pub results: ResultMode,
    /// Where the scratch values of a function live.
    #[builder(default)]
    #[serde(default)]
    pub scratch: ScratchMode,
}

impl From<ResultMode> for EmitOptions
{
    /// The choices with this result mode and the default scratch mode.
    ///
    /// # Arguments
    ///
    /// * `results` - The result mode.
    ///
    /// # Returns
    ///
    /// The choices.
    fn from(results: ResultMode) -> Self
    {
        EmitOptions::builder().results(results).build()
    }
}
