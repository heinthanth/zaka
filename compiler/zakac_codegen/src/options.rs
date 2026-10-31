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
