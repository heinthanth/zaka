//! The CodeView of each data kind, for `.debug$T`: a primitive index for a
//! number, a character, a boolean or an address, and a record for a pointer,
//! a fixed list, a struct, text, a plain list or an enumeration. A struct
//! comes first as a forward record under its unique name, so a struct that
//! points at itself ends; text is `text`, and a plain list `list<cell>`, the
//! names the natvis of the build matches.

use std::collections::HashMap;

use ms_codeview::types::Leaf;
use ms_codeview::types::TypeIndex;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::enums::MemberValue;
use zakac_semcheck::kind::DataKind;

use crate::dwarf_kinds::SEQ_PARTS;
use crate::dwarf_kinds::int_members;

/// The first index of a record; the indexes below name primitives.
const FIRST_INDEX: u32 = 0x1000;

/// The attributes of a pointer: 64-bit (`CV_PTR_64`), plain, eight octets.
const POINTER_64: u32 = 0x0C | (8 << 13);

/// The properties of a struct that names itself with a unique name, and of
/// one that is a forward record alone.
const UNIQUE: u16 = 0x0200;
const FORWARD: u16 = 0x0080;

/// The access of a member or an enumerator: public.
const PUBLIC: u16 = 3;

/// The records of the kinds of one object, each made once.
pub struct Kinds
{
    /// The records so far, after the signature.
    pub out: Vec<u8>,
    /// The index of each kind made so far.
    ids: HashMap<DataKind, u32>,
    /// The index of the next record.
    next: u32,
    /// Bit count of the platform-sized kinds.
    platform: u16,
}

impl Kinds
{
    /// No record so far.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The table.
    pub fn new(platform: u16) -> Self
    {
        let out = 4u32.to_le_bytes().to_vec();
        Self { out, ids: HashMap::new(), next: FIRST_INDEX, platform }
    }

    /// The index of a kind, its records made on its first use.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The index.
    pub fn index(&mut self, kind: DataKind) -> u32
    {
        ret_if!(let Some(p) = primitive(kind, self.platform), p.0);
        ret_if!(let Some(id) = self.ids.get(&kind), *id);
        let id = match kind
        {
            DataKind::Ptr(_) => self.pointer(kind),
            DataKind::List(l) => self.fixed_list(l.cell, kind),
            DataKind::Enum(e) => self.enumeration(kind, e.under(), e.members()),
            _ => self.structure(kind),
        };
        self.ids.insert(kind, id);
        id
    }

    /// A pointer to the record of its pointee.
    ///
    /// # Arguments
    ///
    /// * `kind` - The pointer kind.
    ///
    /// # Returns
    ///
    /// The index.
    fn pointer(&mut self, kind: DataKind) -> u32
    {
        let to = kind.pointee().map_or(TypeIndex::T_VOID.0, |p| self.index(p));
        let mut content = to.to_le_bytes().to_vec();
        content.extend_from_slice(&POINTER_64.to_le_bytes());
        self.record(Leaf::LF_POINTER, &content)
    }

    /// A fixed list: the record of its cells, with an unsigned word as its
    /// index.
    ///
    /// # Arguments
    ///
    /// * `cell` - The kind of each cell.
    /// * `kind` - The list kind, whose size gives the count of cells.
    ///
    /// # Returns
    ///
    /// The index.
    fn fixed_list(&mut self, cell: DataKind, kind: DataKind) -> u32
    {
        let cell = self.index(cell);
        let mut content = cell.to_le_bytes().to_vec();
        content.extend_from_slice(&TypeIndex::T_UINT8.0.to_le_bytes());
        number(&mut content, u64::from(kind.size(self.platform)));
        content.push(0);
        self.record(Leaf::LF_ARRAY, &content)
    }

    /// A struct, text, a plain list or a box: a forward record, the members,
    /// then the full record. The kind takes the index of the forward record
    /// while its members are made, so a member that points at it ends.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The index of the full record.
    fn structure(&mut self, kind: DataKind) -> u32
    {
        let name = cv_name(kind);
        let unique = kind.sig_name();
        let forward = self.record(
            Leaf::LF_STRUCTURE,
            &struct_content(&Head::forward(&name, &unique)),
        );
        self.ids.insert(kind, forward);
        let members = self.members(kind);
        let count = members.len() as u16;
        let mut list = Vec::new();
        do_for!((name, index, offset) in &members, member(&mut list, name, *index, *offset));
        let list = self.record(Leaf::LF_FIELDLIST, &list);
        let size = kind.size(self.platform);
        let head =
            Head { count, props: UNIQUE, list, size, name: &name, unique: &unique };
        self.record(Leaf::LF_STRUCTURE, &struct_content(&head))
    }

    /// The members of a struct, text, a plain list or a box: the name, the
    /// index and the offset of each.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The members.
    fn members(&mut self, kind: DataKind) -> Vec<(String, u32, u32)>
    {
        let platform = self.platform;
        // Text too: `as_seq` leaves it out.
        if let DataKind::Seq(seq) = kind
        {
            let kinds = [seq.cell.raw_pointer_to(), DataKind::USIZE, DataKind::USIZE];
            let parts = SEQ_PARTS.iter().zip(kinds).enumerate();
            let words = u32::from(platform / 8);
            return parts
                .map(|(i, (name, k))| {
                    (name.to_string(), self.index(k), i as u32 * words)
                })
                .collect();
        }
        let Some(s) = kind.as_struct()
        else
        {
            return Vec::new();
        };
        let fields = s.fields().iter().enumerate();
        fields
            .map(|(i, (name, k))| (name.clone(), self.index(*k), s.offset(i, platform)))
            .collect()
    }

    /// An enumeration of integer members: its enumerators, then the record
    /// over the primitive of its values; another enumeration is the kind of
    /// its values.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `under` - The kind of the member values.
    /// * `members` - The members.
    ///
    /// # Returns
    ///
    /// The index.
    fn enumeration(
        &mut self, kind: DataKind, under: DataKind, members: &[(String, MemberValue)],
    ) -> u32
    {
        let Some(values) = int_members(members)
        else
        {
            return self.index(under);
        };
        let mut list = Vec::new();
        do_for!(((name, _), value) in members.iter().zip(values), enumerator(&mut list, name, value));
        let list = self.record(Leaf::LF_FIELDLIST, &list);
        let mut content = (members.len() as u16).to_le_bytes().to_vec();
        content.extend_from_slice(&0u16.to_le_bytes());
        content.extend_from_slice(&self.index(under).to_le_bytes());
        content.extend_from_slice(&list.to_le_bytes());
        content.extend_from_slice(kind.name().as_bytes());
        content.push(0);
        self.record(Leaf::LF_ENUM, &content)
    }

    /// Append one record, padded to four octets, and give it the next index.
    ///
    /// # Arguments
    ///
    /// * `leaf` - The kind of the record.
    /// * `content` - Its content after the kind.
    ///
    /// # Returns
    ///
    /// Its index.
    fn record(&mut self, leaf: Leaf, content: &[u8]) -> u32
    {
        // The length and the kind take four octets, so the content pads to four
        // and the next record starts aligned.
        let padded = content.len().next_multiple_of(4);
        self.out.extend_from_slice(&((padded + 2) as u16).to_le_bytes());
        self.out.extend_from_slice(&leaf.0.to_le_bytes());
        self.out.extend_from_slice(content);
        pad(&mut self.out, padded - content.len());
        let id = self.next;
        self.next += 1;
        id
    }
}

/// The head of a struct record.
struct Head<'n>
{
    /// The count of members.
    count: u16,
    /// The properties.
    props: u16,
    /// The index of the members.
    list: u32,
    /// The size, in octets.
    size: u32,
    /// The name a debugger shows.
    name: &'n str,
    /// The unique name, which matches a forward record to the full one.
    unique: &'n str,
}

impl<'n> Head<'n>
{
    /// The head of a forward record.
    ///
    /// # Arguments
    ///
    /// * `name` - The name a debugger shows.
    /// * `unique` - The unique name.
    ///
    /// # Returns
    ///
    /// The head.
    fn forward(name: &'n str, unique: &'n str) -> Self
    {
        Self { count: 0, props: UNIQUE | FORWARD, list: 0, size: 0, name, unique }
    }
}

/// The content of an `LF_STRUCTURE`.
///
/// # Arguments
///
/// * `head` - Its head.
///
/// # Returns
///
/// The content.
fn struct_content(head: &Head<'_>) -> Vec<u8>
{
    let mut content = head.count.to_le_bytes().to_vec();
    content.extend_from_slice(&head.props.to_le_bytes());
    content.extend_from_slice(&head.list.to_le_bytes());
    // No base list and no table of virtual functions.
    content.extend_from_slice(&[0; 8]);
    number(&mut content, u64::from(head.size));
    do_for!(text in [head.name, head.unique], {
        content.extend_from_slice(text.as_bytes());
        content.push(0);
    });
    content
}

/// Append one `LF_MEMBER` to a list of members, padded to four octets.
///
/// # Arguments
///
/// * `list` - The list so far.
/// * `name` - The name of the member.
/// * `index` - Its kind.
/// * `offset` - Its offset.
fn member(list: &mut Vec<u8>, name: &str, index: u32, offset: u32)
{
    list.extend_from_slice(&Leaf::LF_MEMBER.0.to_le_bytes());
    list.extend_from_slice(&PUBLIC.to_le_bytes());
    list.extend_from_slice(&index.to_le_bytes());
    number(list, u64::from(offset));
    list.extend_from_slice(name.as_bytes());
    list.push(0);
    pad(list, list.len().next_multiple_of(4) - list.len());
}

/// Append one `LF_ENUMERATE` to a list of enumerators, padded to four octets.
///
/// # Arguments
///
/// * `list` - The list so far.
/// * `name` - The name of the member.
/// * `value` - Its value.
fn enumerator(list: &mut Vec<u8>, name: &str, value: i64)
{
    list.extend_from_slice(&Leaf::LF_ENUMERATE.0.to_le_bytes());
    list.extend_from_slice(&PUBLIC.to_le_bytes());
    signed(list, value);
    list.extend_from_slice(name.as_bytes());
    list.push(0);
    pad(list, list.len().next_multiple_of(4) - list.len());
}

/// Append the octets that pad a record: `LF_PAD` marks that count down.
///
/// # Arguments
///
/// * `out` - The record so far.
/// * `count` - The count of octets, at most three.
fn pad(out: &mut Vec<u8>, count: usize)
{
    do_for!(n in (1..=count).rev(), out.push(0xF0 + n as u8));
}

/// Append an unsigned number as CodeView writes one: two octets below
/// `0x8000`, else a leaf that gives its width.
///
/// # Arguments
///
/// * `out` - The record so far.
/// * `value` - The number.
fn number(out: &mut Vec<u8>, value: u64)
{
    if value < 0x8000
    {
        out.extend_from_slice(&(value as u16).to_le_bytes());
        return;
    }
    if let Ok(word) = u32::try_from(value)
    {
        out.extend_from_slice(&Leaf::LF_ULONG.0.to_le_bytes());
        out.extend_from_slice(&word.to_le_bytes());
        return;
    }
    out.extend_from_slice(&Leaf::LF_UQUADWORD.0.to_le_bytes());
    out.extend_from_slice(&value.to_le_bytes());
}

/// Append a signed number as CodeView writes one.
///
/// # Arguments
///
/// * `out` - The record so far.
/// * `value` - The number.
fn signed(out: &mut Vec<u8>, value: i64)
{
    ret_if!(value >= 0, number(out, value as u64));
    out.extend_from_slice(&Leaf::LF_QUADWORD.0.to_le_bytes());
    out.extend_from_slice(&value.to_le_bytes());
}

/// The name a debugger shows for a struct, text, a plain list or a box: the
/// name of the kind, but `list<cell>` for a plain list, since natvis matches
/// a name with `*` inside angle brackets alone.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The name.
fn cv_name(kind: DataKind) -> String
{
    let plain = kind.as_seq().filter(|s| !s.text && !s.boxed);
    plain.map_or_else(|| kind.name(), |s| format!("list<{}>", s.cell.name()))
}

/// The primitive index of a number, a character, a boolean or an address.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The index, or `None` for a kind that needs a record.
pub fn primitive(kind: DataKind, platform: u16) -> Option<TypeIndex>
{
    let index = match kind
    {
        DataKind::Int(n) => int_index(n.signed, n.bits(platform)),
        DataKind::Float(32) => TypeIndex::T_REAL32,
        DataKind::Float(_) => TypeIndex::T_REAL64,
        DataKind::Char => TypeIndex::T_CHAR32,
        DataKind::Bool => TypeIndex::T_BOOL8,
        DataKind::Task(_) => TypeIndex::T_UINT8,
        DataKind::Null | DataKind::Fn(_) => TypeIndex::T_64PVOID,
        DataKind::Ptr(_) if kind.pointee().is_none() => TypeIndex::T_64PVOID,
        _ => return None,
    };
    Some(index)
}

/// The CodeView index of an integer kind.
///
/// # Arguments
///
/// * `signed` - Whether it is signed.
/// * `bits` - Its width.
///
/// # Returns
///
/// The index.
fn int_index(signed: bool, bits: u16) -> TypeIndex
{
    match (signed, bits)
    {
        (true, 8) => TypeIndex::T_INT1,
        (false, 8) => TypeIndex::T_UINT1,
        (true, 16) => TypeIndex::T_INT2,
        (false, 16) => TypeIndex::T_UINT2,
        (true, 32) => TypeIndex::T_INT4,
        (false, 32) => TypeIndex::T_UINT4,
        (true, _) => TypeIndex::T_INT8,
        (false, _) => TypeIndex::T_UINT8,
    }
}
