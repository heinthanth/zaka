use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeStruct;
use strum::AsRefStr;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::expr::Ident;

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
    /// A struct, named in [`Kind::name`].
    Named,
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
    /// A raw pointer level: `လိပ်စာအကြမ်း` or `၏ လိပ်စာအကြမ်း`.
    RawPtr,
    /// A list level: `စာရင်း` or `၏ စာရင်း`.
    List(Cells),
    /// A slice level: a count-less `စာရင်း` followed with `တစ်စိတ်တစ်ပိုင်း`.
    Slice,
}

/// A data kind as written in the source, such as
/// `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Builder)]
pub struct Kind
{
    /// The base kind.
    pub base: BaseKind,
    /// The struct name of a [`BaseKind::Named`] base.
    #[serde(default)]
    pub name: Option<Ident>,
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
    /// Whether the kind starts with the `ငှားထားသော` marker.
    #[builder(default)]
    #[serde(default)]
    pub borrowed: bool,
    /// The count of `ငှားထားသော` markers after the first one; a kind
    /// with more than one is a misplaced borrow that a later phase rejects.
    #[builder(default)]
    #[serde(default)]
    pub extra_borrows: u8,
    /// Source region of the whole kind phrase.
    pub span: Span,
}

impl Serialize for Kind
{
    /// Write each field. The text artifacts leave out a name or borrow field
    /// that holds its default, so older goldens are unchanged. The raw
    /// artifacts are not self-describing: all fields are written.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
    {
        let skip = serializer.is_human_readable();
        let skip_name = skip && self.name.is_none();
        let skip_borrowed = skip && !self.borrowed;
        let skip_extra = skip && self.extra_borrows == 0;
        /// The number of fields, the name and borrow fields included.
        const FIELDS: usize = 8;
        let skipped = [skip_name, skip_borrowed, skip_extra].map(usize::from);
        let n = FIELDS - skipped.iter().sum::<usize>();
        let mut state = serializer.serialize_struct("Kind", n)?;
        state.serialize_field("base", &self.base)?;
        tern!(
            skip_name,
            state.skip_field("name")?,
            state.serialize_field("name", &self.name)?
        );
        state.serialize_field("bits", &self.bits)?;
        state.serialize_field("signed", &self.signed)?;
        state.serialize_field("wraps", &self.wraps)?;
        tern!(
            skip_borrowed,
            state.skip_field("borrowed")?,
            state.serialize_field("borrowed", &self.borrowed)?
        );
        tern!(
            skip_extra,
            state.skip_field("extra_borrows")?,
            state.serialize_field("extra_borrows", &self.extra_borrows)?
        );
        state.serialize_field("span", &self.span)?;
        state.end()
    }
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

    /// Whether some wrap of the kind is a raw pointer level.
    ///
    /// # Returns
    ///
    /// `true` when a `လိပ်စာအကြမ်း` level is present.
    pub fn has_raw(&self) -> bool { self.wraps.contains(&Wrap::RawPtr) }

    /// The outermost wrap of the kind when it is a pointer level.
    ///
    /// # Returns
    ///
    /// `Wrap::Ptr` or `Wrap::RawPtr` for the outer level, `None` when the
    /// kind is plain or its outer wrap is a list.
    pub fn outer_ptr_wrap(&self) -> Option<Wrap>
    {
        self.wraps.last().copied().filter(|w| matches!(w, Wrap::Ptr | Wrap::RawPtr))
    }
}
