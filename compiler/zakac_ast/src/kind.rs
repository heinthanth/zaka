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
    /// `ကိန်းပြည့်`
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

/// A data kind as written in the source, such as
/// `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Kind
{
    /// The base kind.
    pub base: BaseKind,
    /// The bit count, when one is written.
    pub bits: Option<u16>,
    /// `Some(true)` for `လက္ခဏာရှိ`, `Some(false)` for `လက္ခဏာမဲ့`, `None`
    /// when no sign word is written.
    pub signed: Option<bool>,
    /// The number of pointer levels; zero for a plain value. After a base
    /// word, a bare `လိပ်စာ` and each `၏ လိပ်စာ` add one. The opaque kind's
    /// own `လိပ်စာ` counts too, so an [`BaseKind::Opaque`] kind has at least
    /// one.
    #[builder(default)]
    pub depth: u8,
    /// Source region of the whole kind phrase.
    pub span: Span,
}
