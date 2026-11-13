use std::collections::HashMap;

use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeStruct;
use strum::AsRefStr;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::expr::Expr;
use crate::expr::Ident;
use crate::name::Name;

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
    /// `စာသား`: owned UTF-8 text.
    Text,
    /// A bare `လိပ်စာ`: an address whose target kind is not known.
    #[strum(serialize = "ptr")]
    Opaque,
    /// A struct, named in [`Kind::name`].
    Named,
    /// A result of a function that can fail, with its sides in
    /// [`Kind::parts`]: `<kind> ရလဒ် သို့မဟုတ် <kind> အမှားအယွင်း`.
    Result,
    /// A function value, with its parameters and result in [`Kind::sig`]:
    /// `<params> လက်ခံပြီး <kind> ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား`.
    Fn,
    /// A ဇယား, with its index kind and value kind in [`Kind::table`]:
    /// `<index kind> ဖြင့် ညွှန်းသော <value kind> ဇယား`.
    Table,
}

/// The two kinds of a ဇယား kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct TableParts
{
    /// The kind of the index of each row.
    pub index: Kind,
    /// The kind of the value of each row.
    pub value: Kind,
}

/// One parameter of a function kind: its kind, with the name and the
/// preposition when written. The name documents the kind and takes no part in a
/// match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct SigParam
{
    /// Whether `ဒေတာ ပြောင်းလဲရန်` marks it: what the function leaves in it goes
    /// back to the caller's place.
    #[builder(default)]
    #[serde(default)]
    pub edit: bool,
    /// Whether `အပိုင်အနေဖြင့်` marks it: the function takes over the block the
    /// pointer holds.
    #[builder(default)]
    #[serde(default)]
    pub owned: bool,
    /// The kind.
    pub kind: Kind,
    /// The name, when written.
    #[serde(default)]
    pub name: Option<Ident>,
    /// The preposition after the argument in a preposition call,
    /// `ဝိဘတ် "<preposition>" ကို သုံး၍`.
    #[serde(default)]
    pub preposition: Option<Ident>,
    /// Source region.
    pub span: Span,
}

/// The parameters and the result of a function kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct FnSig
{
    /// The parameters, in order.
    #[builder(default)]
    #[serde(default)]
    pub params: Vec<SigParam>,
    /// The result kind, with its measure word; `None` for a function with no
    /// result.
    #[serde(default)]
    pub ret: Option<Kind>,
}

/// The two sides of a result kind; a side with no value is `None`, as
/// `အောင်မြင်သည့်သတင်း` and `မအောင်မြင်သည့်သတင်း` write it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct ResultParts
{
    /// The kind of the good result.
    pub ok: Option<Kind>,
    /// The kind of the error value.
    pub err: Option<Kind>,
    /// `true` for an optional kind, `T ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`: the good
    /// side is the data, and an optional with no data holds none.
    #[builder(default)]
    #[serde(default)]
    pub optional: bool,
}

impl BaseKind
{
    /// Whether the base is text.
    ///
    /// # Returns
    ///
    /// `true` for `စာသား`.
    pub fn is_text(self) -> bool { self == BaseKind::Text }
}

/// The cell count of a list kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cells
{
    /// `အကွက် <n> ခုရှိသော`: a count written before the kind.
    Fixed(u32),
    /// `အကွက် <expr> ခုရှိသော`: a count that names constants or works one
    /// out, such as `N * ၂`. The check works it out with the folder's math
    /// and writes it back as [`Cells::Fixed`].
    Expr(Box<Expr>),
    /// No count, `စာရင်း`: a plain list, whose count is known at run time.
    Inferred,
}

/// One wrap around the base of a kind, such as the `လိပ်စာ` of a pointer or
/// the `စာရင်း` of a list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Deserialize, Builder)]
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
    /// The sides of a [`BaseKind::Result`] base.
    #[serde(default)]
    pub parts: Option<Box<ResultParts>>,
    /// The parameters and result of a [`BaseKind::Fn`] base.
    #[serde(default)]
    pub sig: Option<Box<FnSig>>,
    /// The index kind and the value kind of a [`BaseKind::Table`] base.
    #[serde(default)]
    pub table: Option<Box<TableParts>>,
    /// The measure word after `တစ်`, written after the outermost kind of a
    /// slot: `ကိန်းပြည့် စာရင်း တစ်ခု`. A kind inside another kind, and the
    /// original kind of an alias, have none.
    #[serde(default)]
    pub classifier: Option<Ident>,
    /// The alias the kind was written as, when the loader put the kind of
    /// the alias in its place and the use adds no levels to it; the measure
    /// word is the alias's.
    #[serde(default)]
    pub alias: Option<Ident>,
    /// The kind arguments of a generic kind, `<name><A၊ B>` or
    /// `A နှင့် B တို့ကို အစားထိုးအသုံးပြုသည့် <name>`; none for each other kind.
    #[builder(default)]
    #[serde(default)]
    pub args: Vec<Kind>,
    /// Source region of the whole kind phrase, the measure word left out.
    pub span: Span,
}

impl Serialize for Kind
{
    /// Write each field. The text artifacts leave out a name, parts or
    /// signature field that holds its default. The raw
    /// artifacts are not self-describing: all fields are written.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
    {
        let skip = serializer.is_human_readable();
        let skip_name = skip && self.name.is_none();
        let skip_parts = skip && self.parts.is_none();
        let skip_sig = skip && self.sig.is_none();
        let skip_table = skip && self.table.is_none();
        let skip_word = skip && self.classifier.is_none();
        let skip_alias = skip && self.alias.is_none();
        let skip_args = skip && self.args.is_empty();
        /// The number of fields, each that can be left out included.
        const FIELDS: usize = 12;
        let skipped = [
            skip_name, skip_parts, skip_sig, skip_table, skip_word, skip_alias,
            skip_args,
        ]
        .map(usize::from);
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
            skip_parts,
            state.skip_field("parts")?,
            state.serialize_field("parts", &self.parts)?
        );
        tern!(
            skip_sig,
            state.skip_field("sig")?,
            state.serialize_field("sig", &self.sig)?
        );
        tern!(
            skip_table,
            state.skip_field("table")?,
            state.serialize_field("table", &self.table)?
        );
        tern!(
            skip_word,
            state.skip_field("classifier")?,
            state.serialize_field("classifier", &self.classifier)?
        );
        tern!(
            skip_alias,
            state.skip_field("alias")?,
            state.serialize_field("alias", &self.alias)?
        );
        tern!(
            skip_args,
            state.skip_field("args")?,
            state.serialize_field("args", &self.args)?
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

    /// The outermost wrap of the kind when it is a pointer level.
    ///
    /// # Returns
    ///
    /// `Wrap::Ptr` for an outer pointer level, `None` when the kind is plain
    /// or its outer wrap is a list.
    pub fn outer_ptr_wrap(&self) -> Option<Wrap>
    {
        self.wraps.last().filter(|w| matches!(w, Wrap::Ptr)).cloned()
    }

    /// Put the kind each kind parameter stands for in place of its name, in
    /// this kind and in each kind inside it. The levels written after the
    /// parameter wrap its kind, and the measure word remains the one written.
    ///
    /// # Arguments
    ///
    /// * `bound` - The kind of each kind parameter.
    pub fn substitute(&mut self, bound: &HashMap<Name, Kind>)
    {
        if let Some(parts) = self.parts.as_deref_mut()
        {
            parts
                .ok
                .iter_mut()
                .chain(parts.err.iter_mut())
                .for_each(|k| k.substitute(bound));
        }
        if let Some(table) = self.table.as_deref_mut()
        {
            table.index.substitute(bound);
            table.value.substitute(bound);
        }
        if let Some(sig) = self.sig.as_deref_mut()
        {
            sig.params.iter_mut().for_each(|p| p.kind.substitute(bound));
            sig.ret.iter_mut().for_each(|k| k.substitute(bound));
        }
        self.args.iter_mut().for_each(|k| k.substitute(bound));
        let named = self.base == BaseKind::Named;
        let found =
            self.name.as_ref().filter(|_| named).and_then(|n| bound.get(&n.name));
        let Some(found) = found
        else
        {
            return;
        };
        let mut phrase = found.clone();
        phrase.wraps.append(&mut self.wraps);
        phrase.classifier = self.classifier.take().or(phrase.classifier);
        phrase.span = self.span;
        *self = phrase;
    }
}
