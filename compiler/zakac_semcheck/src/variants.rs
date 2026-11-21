//! Enumerations whose variants hold data. Such an enumeration is a struct
//! the compiler makes from its declaration: the first field holds the tag,
//! the position of the variant a value is, and its name lists the variants;
//! each variant that holds data has a field of its own after it, next to
//! the others, as a result keeps its two parts. The fields of the variants
//! a value is not hold zero.
//!
//! The field names start with `%`, which no written name can, so the
//! struct table, the offsets, copies, drops and `==` of structs serve these
//! kinds unchanged.

use zakac_lexer::reserved::AMONG;
use zakac_lexer::reserved::PLURAL;

use crate::enums::stem_of;
use crate::kind::DataKind;
use crate::structs::StructKind;

/// The start of the name of the tag field: `%v`, then `|<variant>` for each
/// variant in order, with `!` before the variant marked as the initial
/// value.
pub const TAG_FIELD: &str = "%v";

/// The mark before the variant that is the initial value.
const INITIAL_MARK: char = '!';

/// The name of the tag field of an enumeration.
///
/// # Arguments
///
/// * `variants` - The variant names in order.
/// * `initial` - The position of the variant marked as the initial value.
///
/// # Returns
///
/// The field name.
pub fn tag_field(variants: &[&str], initial: Option<usize>) -> String
{
    let mark = |at: usize| if initial == Some(at) { "!" } else { "" };
    let parts = variants.iter().enumerate().map(|(at, v)| format!("|{}{v}", mark(at)));
    format!("{TAG_FIELD}{}", parts.collect::<String>())
}

/// The name of the field that holds the data of a variant.
///
/// # Arguments
///
/// * `at` - The position of the variant.
///
/// # Returns
///
/// The field name, `%<position>`.
pub fn data_field(at: usize) -> String { format!("%{at}") }

/// The stem of the name of an enumeration: the name without a closing
/// `များ`; for an instance of a generic one, the stem of the generic with the
/// kinds after it, as `အမှား<isize>` for `အမှားများ<isize>`.
///
/// # Arguments
///
/// * `name` - The name.
///
/// # Returns
///
/// The stem.
pub fn name_stem(name: &str) -> String
{
    match name.split_once('<')
    {
        Some((generic, kinds)) => format!("{}<{kinds}", stem_of(generic)),
        None => stem_of(name).to_string(),
    }
}

/// An enumeration whose variants hold data, seen through its struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Variants(&'static StructKind);

impl Variants
{
    /// The enumeration a struct kind is, when it is one.
    ///
    /// # Arguments
    ///
    /// * `shape` - The struct kind.
    ///
    /// # Returns
    ///
    /// The enumeration, or `None` for another struct.
    pub fn of(shape: &'static StructKind) -> Option<Variants>
    {
        let first = shape.fields().first()?;
        first.0.starts_with(&format!("{TAG_FIELD}|")).then_some(Variants(shape))
    }

    /// The struct the enumeration is.
    ///
    /// # Returns
    ///
    /// The struct kind.
    pub fn shape(self) -> &'static StructKind { self.0 }

    /// The kind of the enumeration.
    ///
    /// # Returns
    ///
    /// The struct kind, as a data kind.
    pub fn kind(self) -> DataKind { DataKind::Struct(self.0) }

    /// The enumeration name.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn name(self) -> &'static str { &self.0.name }

    /// The stem of the name: the name without a closing `များ`; for an
    /// instance of a generic enumeration, the stem of the generic with the
    /// kinds after it, as `အမှား<isize>` for `အမှားများ<isize>`.
    ///
    /// # Returns
    ///
    /// The stem.
    pub fn stem(self) -> String { name_stem(&self.0.name) }

    /// The variant names in order, each with whether it is the initial
    /// value.
    ///
    /// # Returns
    ///
    /// The names.
    fn listed(self) -> impl Iterator<Item = (&'static str, bool)>
    {
        let first = &self.0.fields()[0].0;
        first.split('|').skip(1).map(|v| match v.strip_prefix(INITIAL_MARK)
        {
            Some(name) => (name, true),
            None => (v, false),
        })
    }

    /// The variant names in order.
    ///
    /// # Returns
    ///
    /// The names.
    pub fn names(self) -> Vec<&'static str> { self.listed().map(|(n, _)| n).collect() }

    /// The position of a variant.
    ///
    /// # Arguments
    ///
    /// * `name` - The variant name.
    ///
    /// # Returns
    ///
    /// The position, or `None` for a name the enumeration does not have.
    pub fn position(self, name: &str) -> Option<usize>
    {
        self.listed().position(|(n, _)| n == name)
    }

    /// The position of the variant marked as the initial value.
    ///
    /// # Returns
    ///
    /// The position, or `None` when no variant is marked.
    pub fn initial(self) -> Option<usize> { self.listed().position(|(_, i)| i) }

    /// The kind of the tag.
    ///
    /// # Returns
    ///
    /// The integer kind of the first field.
    pub fn tag_kind(self) -> DataKind { self.0.fields()[0].1 }

    /// The field and the kind of the data of a variant.
    ///
    /// # Arguments
    ///
    /// * `at` - The position of the variant.
    ///
    /// # Returns
    ///
    /// The position of the field and the kind of the data, or `None` for a
    /// variant with no data.
    pub fn data(self, at: usize) -> Option<(usize, DataKind)>
    {
        self.0.field(&data_field(at))
    }

    /// The text of a variant with no data, the form that reads back:
    /// `<stem>များထဲမှ <variant>`.
    ///
    /// # Arguments
    ///
    /// * `at` - The position of the variant.
    ///
    /// # Returns
    ///
    /// The text.
    pub fn text_at(self, at: usize) -> String
    {
        let name = self.names().get(at).copied().unwrap_or_default();
        format!("{}{PLURAL}{AMONG} {name}", self.stem())
    }
}
