use std::collections::BTreeMap;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;

use zakac_ast::name::Name;
use zakac_rust_sugars::ret_if;

use crate::kind::DataKind;

/// The struct kind of each declared name.
pub type StructTable = BTreeMap<Name, DataKind>;

/// The fields of each struct, as written: each field name with its kind in
/// text form, where a struct base is its bare name, such as `*လူ`.
pub type StructDefs = BTreeMap<String, Vec<(String, String)>>;

/// A struct kind: named fields laid out in C order.
///
/// Two struct kinds are equal when their signatures are: the name, then each
/// field with its kind, a struct inside written out in full except for a
/// struct that encloses it, which is written as its bare name. The signature
/// is also the name of the kind in artifacts, so a kind reads back from its
/// text alone.
pub struct StructKind
{
    /// The struct name.
    pub name: String,
    /// The signature, such as `လူ{အသက်:isize,နောက်:*လူ}`.
    sig: String,
    /// The fields in order, set once each kind the fields name exists.
    fields: OnceLock<Vec<(String, DataKind)>>,
}

impl StructKind
{
    /// The signature, which names the kind in artifacts.
    ///
    /// # Returns
    ///
    /// The signature.
    pub fn sig(&self) -> &str { &self.sig }

    /// The fields in order, each a name and a kind.
    ///
    /// # Returns
    ///
    /// The fields.
    pub fn fields(&self) -> &[(String, DataKind)]
    {
        self.fields.get().map_or(&[], Vec::as_slice)
    }

    /// The field with a name.
    ///
    /// # Arguments
    ///
    /// * `name` - The field name.
    ///
    /// # Returns
    ///
    /// The position of the field and its kind, or `None` for a name the
    /// struct does not have.
    pub fn field(&self, name: &str) -> Option<(usize, DataKind)>
    {
        self.fields()
            .iter()
            .position(|(n, _)| n == name)
            .map(|at| (at, self.fields()[at].1))
    }

    /// The offset of each field: each field starts at the next multiple of
    /// its alignment after the field before it.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The offsets, in octets, and the end of the last field; `None` when a
    /// size passes `u32::MAX`.
    fn arrange(&self, platform: u16) -> Option<(Vec<u32>, u32)>
    {
        let mut offsets = Vec::with_capacity(self.fields().len());
        let mut end = 0u32;
        for (_, kind) in self.fields()
        {
            let at = end.checked_next_multiple_of(kind.align(platform))?;
            offsets.push(at);
            end = at.checked_add(kind.checked_size(platform)?)?;
        }
        Some((offsets, end))
    }

    /// The offset of the field at a position.
    ///
    /// # Arguments
    ///
    /// * `at` - The position of the field.
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The offset in octets; 0 for a position past the last field.
    pub fn offset(&self, at: usize, platform: u16) -> u32
    {
        self.arrange(platform).and_then(|(o, _)| o.get(at).copied()).unwrap_or(0)
    }

    /// The alignment of the struct: the largest alignment of its fields.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The alignment, at least 1.
    pub fn align(&self, platform: u16) -> u32
    {
        self.fields().iter().map(|(_, k)| k.align(platform)).max().unwrap_or(1)
    }

    /// The size of the struct: the end of its last field, rounded up to its
    /// alignment.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The size in octets, or `None` when it passes `u32::MAX`.
    pub fn size(&self, platform: u16) -> Option<u32>
    {
        let (_, end) = self.arrange(platform)?;
        end.checked_next_multiple_of(self.align(platform))
    }
}

impl PartialEq for StructKind
{
    fn eq(&self, other: &Self) -> bool
    {
        std::ptr::eq(self, other) || self.sig == other.sig
    }
}

impl Eq for StructKind {}

impl Hash for StructKind
{
    fn hash<H: Hasher>(&self, state: &mut H) { self.sig.hash(state) }
}

impl std::fmt::Debug for StructKind
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(&self.sig)
    }
}

/// Each finished struct kind made so far, on its signature, so a program
/// compiled again reuses its kinds.
static MADE: OnceLock<Mutex<HashMap<String, &'static StructKind>>> = OnceLock::new();

/// The made kind with a signature.
///
/// # Arguments
///
/// * `sig` - The signature.
///
/// # Returns
///
/// The kind, or `None` when none is made.
fn made(sig: &str) -> Option<&'static StructKind>
{
    let made = MADE.get_or_init(Mutex::default);
    made.lock().unwrap_or_else(PoisonError::into_inner).get(sig).copied()
}

/// Keep finished kinds for reuse. A kind enters once all the kinds it
/// points at have their fields, so no thread reads a kind half made.
///
/// # Arguments
///
/// * `kinds` - The kinds.
fn keep(kinds: impl Iterator<Item = &'static StructKind>)
{
    let made = MADE.get_or_init(Mutex::default);
    let mut made = made.lock().unwrap_or_else(PoisonError::into_inner);
    kinds.for_each(|k| _ = made.entry(k.sig.clone()).or_insert(k));
}

/// Make the struct kinds of a program.
///
/// # Arguments
///
/// * `defs` - The fields of each struct. No struct holds itself, at once or
///   through other structs, other than through an address.
///
/// # Returns
///
/// The kind of each struct whose field kinds all read.
pub fn make_all(defs: &StructDefs) -> StructTable
{
    let mut memo = HashMap::new();
    let table: StructTable = defs
        .keys()
        .filter_map(|name| {
            Some((Name::new(name), DataKind::Struct(resolve(name, defs, &mut memo)?)))
        })
        .collect();
    keep(memo.into_values());
    table
}

/// Read a struct kind from its signature.
///
/// # Arguments
///
/// * `text` - The signature.
///
/// # Returns
///
/// The kind, or `None` for text that is no signature.
pub fn parse_sig(text: &str) -> Option<&'static StructKind>
{
    ret_if!(let Some(found) = made(text), Some(found));
    let mut defs = StructDefs::new();
    let name = collect_defs(text, &mut defs)?;
    let mut memo = HashMap::new();
    let kind = resolve(&name, &defs, &mut memo);
    keep(memo.into_values());
    kind
}

/// The kind of a struct, made the first time it is asked for.
///
/// # Arguments
///
/// * `name` - The struct name.
/// * `defs` - The fields of each struct.
/// * `memo` - The kinds made in this run, so a struct that points at itself
///   finds its own kind.
///
/// # Returns
///
/// The kind, or `None` when a field kind does not read.
fn resolve(
    name: &str, defs: &StructDefs, memo: &mut HashMap<String, &'static StructKind>,
) -> Option<&'static StructKind>
{
    ret_if!(let Some(found) = memo.get(name), Some(*found));
    let sig = sig_of(name, defs, &mut Vec::new());
    ret_if!(let Some(found) = made(&sig), Some(found));
    let fresh = StructKind { name: name.to_string(), sig, fields: OnceLock::new() };
    let kind: &'static StructKind = Box::leak(Box::new(fresh));
    memo.insert(name.to_string(), kind);
    let field = |(f, k): &(String, String)| Some((f.clone(), kind_of(k, defs, memo)?));
    let fields: Option<Vec<_>> = defs.get(name)?.iter().map(field).collect();
    _ = kind.fields.set(fields?);
    Some(kind)
}

/// The signature of a struct.
///
/// # Arguments
///
/// * `name` - The struct name.
/// * `defs` - The fields of each struct.
/// * `around` - The structs that enclose this one, each written as its bare
///   name.
///
/// # Returns
///
/// The signature.
fn sig_of(name: &str, defs: &StructDefs, around: &mut Vec<String>) -> String
{
    around.push(name.to_string());
    let fields = defs.get(name).map_or(&[][..], Vec::as_slice);
    let field = |(f, k): &(String, String)| format!("{f}:{}", expand(k, defs, around));
    let inner_text: Vec<String> = fields.iter().map(field).collect();
    around.pop();
    format!("{name}{{{}}}", inner_text.join(","))
}

/// A field kind with its struct base written out in full, unless the struct
/// encloses it.
///
/// # Arguments
///
/// * `kind` - The field kind in text form.
/// * `defs` - The fields of each struct.
/// * `around` - The structs that enclose the field.
///
/// # Returns
///
/// The text.
fn expand(kind: &str, defs: &StructDefs, around: &mut Vec<String>) -> String
{
    let (prefix, base) = split_base(kind);
    let inner = defs.contains_key(base) && !around.iter().any(|n| n == base);
    ret_if!(!inner, kind.to_string());
    format!("{prefix}{}", sig_of(base, defs, around))
}

/// The kind of a field kind in text form.
///
/// # Arguments
///
/// * `kind` - The text, with each struct base as its bare name.
/// * `defs` - The fields of each struct.
/// * `memo` - The kinds made in this run.
///
/// # Returns
///
/// The kind, or `None` for text that names no kind.
fn kind_of(
    kind: &str, defs: &StructDefs, memo: &mut HashMap<String, &'static StructKind>,
) -> Option<DataKind>
{
    let (prefix, base) = split_base(kind);
    ret_if!(!defs.contains_key(base), DataKind::parse(kind));
    let inner = DataKind::Struct(resolve(base, defs, memo)?);
    wrap_in(inner, prefix)
}

/// Wrap a kind in the levels of a kind prefix, the innermost (rightmost)
/// first.
///
/// # Arguments
///
/// * `inner` - The kind under the prefix.
/// * `prefix` - The marks `&`, `*`, `~` and `[n]`.
///
/// # Returns
///
/// The kind, or `None` for a bad count.
fn wrap_in(inner: DataKind, prefix: &str) -> Option<DataKind>
{
    let mut levels = Vec::new();
    let mut rest = prefix;
    while let Some(ch) = rest.chars().next()
    {
        let len = level_len(rest, ch)?;
        levels.push(&rest[..len]);
        rest = &rest[len..];
    }
    levels.into_iter().rev().try_fold(inner, |kind, level| match level
    {
        "*" => Some(kind.pointer_to()),
        "~" => Some(kind.raw_pointer_to()),
        "&" => Some(kind.as_borrowed()),
        list => list_level(kind, list),
    })
}

/// The length of the prefix level that starts with `ch`.
///
/// # Arguments
///
/// * `rest` - The prefix from the level on.
/// * `ch` - Its first char.
///
/// # Returns
///
/// The length in octets, or `None` for an unclosed `[`.
fn level_len(rest: &str, ch: char) -> Option<usize>
{
    ret_if!(ch != '[', Some(ch.len_utf8()));
    rest.find(']').map(|at| at + 1)
}

/// The list of `[n]` cells of a kind.
///
/// # Arguments
///
/// * `cell` - The cell kind.
/// * `level` - The level text, `[n]` or `[]`.
///
/// # Returns
///
/// The list kind, or `None` for a bad count.
fn list_level(cell: DataKind, level: &str) -> Option<DataKind>
{
    let count = level.strip_prefix('[')?.strip_suffix(']')?;
    let len = if count.is_empty()
    {
        0
    }
    else
    {
        count.parse().ok()?
    };
    Some(DataKind::list_of(cell, len))
}

/// Split a kind in text form into its prefix of levels and its base.
///
/// # Arguments
///
/// * `kind` - The text.
///
/// # Returns
///
/// The prefix (`&`, `*`, `~` and `[n]` marks) and the base after it.
pub fn split_base(kind: &str) -> (&str, &str)
{
    let mut at = 0;
    while let Some(ch) = kind[at..].chars().next()
    {
        let len = match ch
        {
            '&' | '*' | '~' => 1,
            '[' => kind[at..].find(']').map_or(0, |end| end + 1),
            _ => 0,
        };
        ret_if!(len == 0, kind.split_at(at));
        at += len;
    }
    kind.split_at(at)
}

/// Record the fields of a signature and of each struct written out in it,
/// each field kind with its struct base as a bare name.
///
/// # Arguments
///
/// * `text` - The signature: `name{field:kind,...}`.
/// * `defs` - The fields found so far.
///
/// # Returns
///
/// The struct name, or `None` for text that is no signature.
fn collect_defs(text: &str, defs: &mut StructDefs) -> Option<String>
{
    let (name, inner_text) = text.strip_suffix('}')?.split_once('{')?;
    ret_if!(name.is_empty(), None);
    let mut fields = Vec::new();
    for part in top_level_parts(inner_text)
    {
        let (field, kind) = part.split_once(':')?;
        let (prefix, base) = split_base(kind);
        let base = if base.contains('{')
        {
            collect_defs(base, defs)?
        }
        else
        {
            base.to_string()
        };
        fields.push((field.to_string(), format!("{prefix}{base}")));
    }
    defs.insert(name.to_string(), fields);
    Some(name.to_string())
}

/// Split the inner_text of a signature at the commas outside braces.
///
/// # Arguments
///
/// * `inner_text` - The text between the outer braces.
///
/// # Returns
///
/// The parts.
fn top_level_parts(inner_text: &str) -> Vec<&str>
{
    let mut parts = Vec::new();
    let (mut depth, mut start) = (0usize, 0);
    for (at, ch) in inner_text.char_indices()
    {
        match ch
        {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 =>
            {
                parts.push(&inner_text[start..at]);
                start = at + 1;
            }
            _ => (),
        }
    }
    parts.push(&inner_text[start..]);
    parts
}
