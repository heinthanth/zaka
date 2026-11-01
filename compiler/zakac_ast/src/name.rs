use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

/// Each name made so far, kept for the rest of the run, so equal text gives
/// one address and a name stays `Copy`.
static NAMES: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();

/// A name from the source, such as a variable, a field or a function: text
/// kept once for the whole run. Two names are equal when both share an
/// address, as equal text does; a name hashes and orders as its
/// text, so a map from names can be read with a `&str`. Artifacts write
/// a name as its text, and reading one back keeps it again.
#[derive(Clone, Copy, Eq)]
pub struct Name(&'static str);

impl Name
{
    /// The name with this text, kept on first use.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn new(text: &str) -> Name
    {
        let names = NAMES.get_or_init(Mutex::default);
        let mut names = names.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(found) = names.get(text)
        {
            return Name(found);
        }
        let kept: &'static str = Box::leak(text.to_owned().into_boxed_str());
        names.insert(kept);
        Name(kept)
    }

    /// The text of the name.
    ///
    /// # Returns
    ///
    /// The text, kept for the whole run.
    pub fn as_str(self) -> &'static str { self.0 }
}

impl PartialEq for Name
{
    fn eq(&self, other: &Name) -> bool { std::ptr::eq(self.0, other.0) }
}

impl PartialEq<str> for Name
{
    fn eq(&self, other: &str) -> bool { self.0 == other }
}

impl PartialEq<&str> for Name
{
    fn eq(&self, other: &&str) -> bool { self.0 == *other }
}

impl Hash for Name
{
    fn hash<H: Hasher>(&self, state: &mut H) { self.0.hash(state) }
}

impl PartialOrd for Name
{
    fn partial_cmp(&self, other: &Name) -> Option<std::cmp::Ordering>
    {
        Some(self.cmp(other))
    }
}

impl Ord for Name
{
    fn cmp(&self, other: &Name) -> std::cmp::Ordering { self.0.cmp(other.0) }
}

impl Deref for Name
{
    type Target = str;

    fn deref(&self) -> &str { self.0 }
}

impl Borrow<str> for Name
{
    fn borrow(&self) -> &str { self.0 }
}

impl AsRef<str> for Name
{
    fn as_ref(&self) -> &str { self.0 }
}

impl fmt::Display for Name
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.0) }
}

impl fmt::Debug for Name
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        fmt::Debug::fmt(self.0, f)
    }
}

impl From<&str> for Name
{
    fn from(text: &str) -> Name { Name::new(text) }
}

impl From<&String> for Name
{
    fn from(text: &String) -> Name { Name::new(text) }
}

impl From<String> for Name
{
    fn from(text: String) -> Name { Name::new(&text) }
}

impl From<Name> for String
{
    fn from(name: Name) -> String { name.0.to_owned() }
}

impl Serialize for Name
{
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error>
    {
        s.serialize_str(self.0)
    }
}

impl<'de> Deserialize<'de> for Name
{
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Name, D::Error>
    {
        String::deserialize(d).map(Name::from)
    }
}

#[cfg(test)]
mod tests
{
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn equal_text_gives_one_name_that_reads_as_text()
    {
        let a = Name::new("အသက်");
        let b = Name::from(String::from("အသက်"));
        assert_eq!(a, b);
        assert!(std::ptr::eq(a.as_str(), b.as_str()));
        assert_ne!(a, Name::new("အမည်"));
        assert_eq!(a, "အသက်");
        let map = HashMap::from([(a, 1)]);
        assert_eq!(map.get("အသက်"), Some(&1));
        assert_eq!(format!("{a}"), "အသက်");
    }
}
