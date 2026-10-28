use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use strum::AsRefStr;
use zakac_span::span::Span;

/// The base of a kind, without its size or sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "lowercase")]
pub enum BaseKind
{
    /// `ကိန်းပြည့်`
    Int,
    /// `ဒဿမကိန်း`
    Float,
    /// `စာလုံး`
    Char,
    /// `မှန်မှားတန်ဖိုး` or `ဘူလီယန်`
    Bool,
    /// A bare `လိပ်စာ`: an address whose target kind is not known.
    #[strum(serialize = "ptr")]
    Opaque,
}

/// The cell count of a list kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cells
{
    /// `အကွက် <n> ခုရှိသော`: a count written before the kind.
    Fixed(u32),
    /// No count: the initializer gives it.
    Inferred,
}

/// One wrap around the base of a kind, such as the `လိပ်စာ` of a pointer or
/// the `စာရင်း` of a list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Wrap
{
    /// A pointer level: a bare `လိပ်စာ` or `၏ လိပ်စာ`.
    Ptr,
    /// A list level: `စာရင်း` or `၏ စာရင်း`.
    List(Cells),
}

/// A data kind as written in the source, such as
/// `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Kind
{
    /// The base kind.
    pub base: BaseKind,
    /// The bit count, when one is written.
    pub bits: Option<u16>,
    /// `Some(true)` for `လက္ခဏာရှိ`, `Some(false)` for `လက္ခဏာမဲ့`, `None`
    /// when no sign word is written.
    pub signed: Option<bool>,
    /// The wraps around the base, innermost first; none for a plain value.
    /// The opaque kind's own `လိပ်စာ` counts as a pointer wrap, so an
    /// [`BaseKind::Opaque`] kind starts with one.
    #[builder(default)]
    #[serde(default)]
    pub wraps: Vec<Wrap>,
    /// Source region of the whole kind phrase.
    pub span: Span,
}

impl Kind
{
    /// Whether the kind is a plain value: no pointer and no list wraps.
    ///
    /// # Returns
    ///
    /// `true` for a plain value.
    pub fn is_plain(&self) -> bool { self.wraps.is_empty() }

    /// Whether a list wrap of the kind leaves its count to the initializer.
    ///
    /// # Returns
    ///
    /// `true` when some `စာရင်း` has no count.
    pub fn has_inferred_cells(&self) -> bool
    {
        self.wraps.contains(&Wrap::List(Cells::Inferred))
    }
}
