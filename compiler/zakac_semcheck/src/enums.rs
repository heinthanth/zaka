use std::collections::HashMap;
use std::fmt::Write;
use std::hash::Hash;
use std::hash::Hasher;
use std::str::FromStr;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;

use zakac_lexer::reserved::AMONG;
use zakac_lexer::reserved::PLURAL;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::kind::DataKind;
use crate::value::Value;

/// The constant value of an enumeration member.
#[derive(Debug, Clone, PartialEq)]
pub enum MemberValue
{
    /// A number, a character, a true/false value or an address.
    Scalar(Value),
    /// Text.
    Text(String),
    /// The cells of a list or the fields of a struct, in order.
    Items(Vec<MemberValue>),
}

impl MemberValue
{
    /// The value as a scalar.
    ///
    /// # Returns
    ///
    /// The value, or `None` for text, a list or a struct.
    pub fn scalar(&self) -> Option<Value>
    {
        let MemberValue::Scalar(v) = self
        else
        {
            return None;
        };
        Some(*v)
    }

    /// Write the value in the form a signature holds: `v<hex>.` for a scalar
    /// (the hex of its artifact text), `t<hex>.` for text and `[...]` for
    /// items. The form holds none of the signs a signature is cut at.
    ///
    /// # Arguments
    ///
    /// * `out` - Where the form goes.
    fn encode(&self, out: &mut String)
    {
        match self
        {
            MemberValue::Scalar(v) => write_hex(out, 'v', &String::from(*v)),
            MemberValue::Text(t) => write_hex(out, 't', t),
            MemberValue::Items(items) =>
            {
                out.push('[');
                items.iter().for_each(|i| i.encode(out));
                out.push(']');
            }
        }
    }

    /// Read a value from the form [`MemberValue::encode`] writes.
    ///
    /// # Arguments
    ///
    /// * `text` - The form; the read value is taken off its front.
    ///
    /// # Returns
    ///
    /// The value, or `None` for a form that does not read.
    fn decode(text: &mut &str) -> Option<MemberValue>
    {
        let mark = text.chars().next()?;
        *text = &text[1..];
        if mark == '['
        {
            let mut items = Vec::new();
            while !text.starts_with(']')
            {
                items.push(MemberValue::decode(text)?);
            }
            *text = &text[1..];
            return Some(MemberValue::Items(items));
        }
        let (hex, rest) = text.split_once('.')?;
        *text = rest;
        let plain = read_hex(hex)?;
        match mark
        {
            'v' => Value::from_str(&plain).ok().map(MemberValue::Scalar),
            't' => Some(MemberValue::Text(plain)),
            _ => None,
        }
    }
}

/// Write a mark, the hex of a text and a closing `.`.
///
/// # Arguments
///
/// * `out` - Where it goes.
/// * `mark` - The mark.
/// * `text` - The text.
fn write_hex(out: &mut String, mark: char, text: &str)
{
    out.push(mark);
    text.as_bytes().iter().for_each(|b| _ = write!(out, "{b:02x}"));
    out.push('.');
}

/// Read text from its hex.
///
/// # Arguments
///
/// * `hex` - Two hex digits for each u8.
///
/// # Returns
///
/// The text, or `None` for hex that is no UTF-8.
fn read_hex(hex: &str) -> Option<String>
{
    let pair = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
    let raw: Option<Vec<u8>> = (0..hex.len()).step_by(2).map(pair).collect();
    String::from_utf8(raw?).ok()
}

/// An enumeration kind: named members, each a constant of one value
/// kind, which a value of the enumeration holds in storage.
///
/// Two enumeration kinds are equal when their signatures are: the name, then
/// each member with its value, then the value kind, such as
/// `အရောင်<နီ=v...|ဝါ=v...;isize>`; `!` stands in front of the member marked
/// `ကနဦးတန်ဖိုး`. The signature is also the name of the kind in artifacts, so
/// a kind reads back from its text alone.
pub struct EnumKind
{
    /// The enumeration name.
    pub name: String,
    /// The signature.
    sig: String,
    /// The kind each member value has.
    under: DataKind,
    /// The members in order, each a name and its value.
    members: Vec<(String, MemberValue)>,
    /// The position of the member marked `ကနဦးတန်ဖိုး`, the initial value of
    /// the enumeration; `None` when no member is marked.
    initial: Option<usize>,
}

impl EnumKind
{
    /// The signature, which names the kind in artifacts.
    ///
    /// # Returns
    ///
    /// The signature.
    pub fn sig(&self) -> &str { &self.sig }

    /// The kind each member value has, which a value of the enumeration holds
    /// in storage.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn under(&self) -> DataKind { self.under }

    /// The members in order, each a name and its value.
    ///
    /// # Returns
    ///
    /// The members.
    pub fn members(&self) -> &[(String, MemberValue)] { &self.members }

    /// The position of a member.
    ///
    /// # Arguments
    ///
    /// * `name` - The member name.
    ///
    /// # Returns
    ///
    /// The position, or `None` for a name the enumeration does not have.
    pub fn position(&self, name: &str) -> Option<usize>
    {
        self.members.iter().position(|(n, _)| n == name)
    }

    /// The value of a member.
    ///
    /// # Arguments
    ///
    /// * `name` - The member name.
    ///
    /// # Returns
    ///
    /// The value, or `None` for a name the enumeration does not have.
    pub fn member(&self, name: &str) -> Option<&MemberValue>
    {
        self.position(name).map(|at| &self.members[at].1)
    }

    /// The value of the first member, which a value of the enumeration starts
    /// at.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn first(&self) -> &MemberValue { &self.members[0].1 }

    /// The position of the member marked `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်`.
    ///
    /// # Returns
    ///
    /// The position, or `None` when the enumeration has no initial value.
    pub fn initial(&self) -> Option<usize> { self.initial }

    /// The stem of the name: the name without a closing `များ`, the form the
    /// default arm of a switch writes.
    ///
    /// # Returns
    ///
    /// The stem.
    pub fn stem(&self) -> &str { stem_of(&self.name) }

    /// The text a member prints as: `<stem>များထဲမှ <member>`, the form that
    /// reads back.
    ///
    /// # Arguments
    ///
    /// * `at` - The position of the member.
    ///
    /// # Returns
    ///
    /// The text.
    pub fn text_at(&self, at: usize) -> String
    {
        format!("{}{PLURAL}{AMONG} {}", self.stem(), self.members[at].0)
    }

    /// The position of the first member whose value passes a test.
    ///
    /// # Arguments
    ///
    /// * `holds` - The test.
    ///
    /// # Returns
    ///
    /// The position, or `None` when no member passes.
    pub fn find(&self, holds: impl Fn(&MemberValue) -> bool) -> Option<usize>
    {
        self.members.iter().position(|(_, v)| holds(v))
    }
}

impl PartialEq for EnumKind
{
    fn eq(&self, other: &Self) -> bool
    {
        std::ptr::eq(self, other) || self.sig == other.sig
    }
}

impl Eq for EnumKind {}

impl Hash for EnumKind
{
    fn hash<H: Hasher>(&self, state: &mut H) { self.sig.hash(state) }
}

impl std::fmt::Debug for EnumKind
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(&self.sig)
    }
}

/// The stem of an enumeration name: the name without a closing `များ`.
///
/// # Arguments
///
/// * `name` - The name.
///
/// # Returns
///
/// The stem; the name itself when it does not end in `များ`.
pub fn stem_of(name: &str) -> &str { name.strip_suffix(PLURAL).unwrap_or(name) }

/// Each enumeration kind made so far, on its signature, so a program compiled
/// again reuses its kinds.
static MADE: OnceLock<Mutex<HashMap<String, &'static EnumKind>>> = OnceLock::new();

/// The enumeration kind of a name, its value kind and its members, made
/// once and kept for the rest of the run.
///
/// # Arguments
///
/// * `name` - The enumeration name.
/// * `under` - The kind of each member value.
/// * `members` - The members in order, each a name and its value; at least one.
///
/// # Returns
///
/// The kind.
pub fn make(
    name: &str, under: DataKind, members: Vec<(String, MemberValue)>,
    initial: Option<usize>,
) -> &'static EnumKind
{
    let mut parts = Vec::with_capacity(members.len());
    for (at, (n, v)) in members.iter().enumerate()
    {
        let mark = tern!(initial == Some(at), "!", "");
        let mut part = format!("{mark}{n}=");
        v.encode(&mut part);
        parts.push(part);
    }
    let sig = format!("{name}<{};{}>", parts.join("|"), under.sig_name());
    let made = MADE.get_or_init(Mutex::default);
    let mut made = made.lock().unwrap_or_else(PoisonError::into_inner);
    let fresh = || -> &'static EnumKind {
        #[rustfmt::skip]
        let kind = EnumKind { name: name.to_string(), sig: sig.clone(), under, members, initial };
        Box::leak(Box::new(kind))
    };
    made.entry(sig.clone()).or_insert_with(fresh)
}

/// Read an enumeration kind from its signature.
///
/// # Arguments
///
/// * `text` - The signature: `name<member=value|...;kind>`.
///
/// # Returns
///
/// The kind, or `None` for text that is no signature.
pub fn parse_sig(text: &str) -> Option<&'static EnumKind>
{
    let (name, inner) = text.strip_suffix('>')?.split_once('<')?;
    let (listed, under) = inner.split_once(';')?;
    ret_if!(name.is_empty() || listed.is_empty(), None);
    let under = DataKind::parse(under)?;
    let parts: Vec<&str> = listed.split('|').collect();
    let initial = parts.iter().position(|p| p.starts_with('!'));
    let member = |part: &&str| {
        let (n, mut v) = part.trim_start_matches('!').split_once('=')?;
        let value = MemberValue::decode(&mut v)?;
        v.is_empty().then(|| (n.to_string(), value))
    };
    let members: Option<Vec<_>> = parts.iter().map(member).collect();
    Some(make(name, under, members?, initial))
}
