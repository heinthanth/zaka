use std::fmt;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use zakac_i18n::t;

use crate::diagnostic::Level;

/// Define [`Code`] from one row per code, so the enum, the id table and the
/// level table cannot drift apart.
#[rustfmt::skip]
macro_rules! define_codes {
($($(#[$meta:meta])* $name:ident = ($id:literal, $level:expr)),+ $(,)?) => {
/// A stable diagnostic code, printed as `error[ZK0101]` and stored in
/// artifacts as its id.
///
/// Ids are `ZK` plus four digits; the first two give the phase:
/// `01` lexer, `02` parser, `03` resolve, `04` kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Code { $($(#[$meta])* $name,)+ }

impl Code
{
    /// All codes, in id order.
    pub const ALL: &'static [Code] = &[$(Code::$name),+];

    /// The stable id, for example `ZK0101`.
    ///
    /// # Returns
    ///
    /// The id.
    pub const fn id(self) -> &'static str
    {
        match self { $(Code::$name => $id,)+ }
    }

    /// The level this code is reported at.
    ///
    /// # Returns
    ///
    /// The level.
    pub const fn level(self) -> Level
    {
        match self { $(Code::$name => $level,)+ }
    }
}
};}

define_codes! {
// lexer: ZK01xx
/// A word holds a character that cannot start or continue a token.
InvalidCharacter = ("ZK0101", Level::Error),
/// A word starts with a digit but is not a number.
InvalidNumber = ("ZK0102", Level::Error),
/// A number mixes Burmese and Western digits.
MixedDigitScripts = ("ZK0103", Level::Error),
/// A string literal reaches the end of the line.
UnterminatedString = ("ZK0104", Level::Error),
/// A character literal reaches the end of the line.
UnterminatedChar = ("ZK0105", Level::Error),
/// An escape sequence is not known.
UnknownEscape = ("ZK0106", Level::Error),
/// A character literal does not hold one character.
BadCharLength = ("ZK0107", Level::Error),
/// A raw identifier reaches the end of the line.
UnterminatedRawIdentifier = ("ZK0108", Level::Error),
/// A raw identifier has nothing between its backticks.
BlankRawIdentifier = ("ZK0109", Level::Error) }

impl Code
{
    /// Find a code from its id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id, for example `ZK0101`.
    ///
    /// # Returns
    ///
    /// The code, or `None` for an unknown id.
    pub fn parse(id: &str) -> Option<Code>
    {
        Self::ALL.iter().copied().find(|c| c.id() == id)
    }

    /// Short localized title from the catalogue.
    ///
    /// # Returns
    ///
    /// The title.
    pub fn title(self) -> String { self.catalogue_text("title") }

    /// Long localized explanation from the catalogue.
    ///
    /// # Returns
    ///
    /// The explanation.
    pub fn explain(self) -> String { self.catalogue_text("explain") }

    /// Look up a catalogue text for this code.
    ///
    /// # Arguments
    ///
    /// * `field` - The text name: `title` or `explain`.
    ///
    /// # Returns
    ///
    /// The localized text.
    fn catalogue_text(self, field: &str) -> String
    {
        let name = format!("codes.{}.{field}", self.id());
        t!(name).to_string()
    }
}

impl fmt::Display for Code
{
    /// Write the id.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The result of the write.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.id()) }
}

impl Serialize for Code
{
    /// Serialize as the id string.
    ///
    /// # Arguments
    ///
    /// * `serializer` - The serializer.
    ///
    /// # Returns
    ///
    /// The result of the serialization.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
    {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for Code
{
    /// Deserialize from the id string.
    ///
    /// # Arguments
    ///
    /// * `deserializer` - The deserializer.
    ///
    /// # Returns
    ///
    /// The code, or an error for an unknown id.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>
    {
        let id = String::deserialize(deserializer)?;
        Code::parse(&id).ok_or_else(|| {
            serde::de::Error::custom(format!("unknown diagnostic code {id}"))
        })
    }
}
