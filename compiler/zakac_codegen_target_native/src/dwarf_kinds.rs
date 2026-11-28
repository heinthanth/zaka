//! The DWARF of each data kind a debugger shows: a base, a pointer, a fixed
//! list, a structure or an enumeration, named as the messages of the compiler
//! name the kind, so `frame variable` prints each value with its fields and
//! cells. An enumeration of no values is an `isize` in the IR, so it shows as
//! the position of its member.

use std::collections::HashMap;

use cranelift_codegen::gimli;
use cranelift_codegen::gimli::write::AttributeValue;
use cranelift_codegen::gimli::write::Unit;
use cranelift_codegen::gimli::write::UnitEntryId;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::enums::MemberValue;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::value::Value;

/// The names of the three words of text, a plain list and a box, in order: the
/// address of the first cell, the count of cells, and the room of the heap
/// block in octets. The formatters of lldb and gdb read them through these
/// names.
pub const SEQ_PARTS: [&str; 3] = ["cells", "count", "room"];

/// The DIEs of the kinds of one unit, each made once.
pub struct Kinds
{
    /// The DIE of each kind made so far.
    ids: HashMap<DataKind, UnitEntryId>,
    /// Bit count of the platform-sized kinds.
    platform: u16,
}

impl Kinds
{
    /// No kind made so far.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The table.
    pub fn new(platform: u16) -> Self { Self { ids: HashMap::new(), platform } }

    /// The DIE of a kind, made on its first use. A kind enters the table
    /// before its parts, so a struct that points at itself ends.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit.
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The DIE.
    pub fn die_of(&mut self, unit: &mut Unit, kind: DataKind) -> UnitEntryId
    {
        ret_if!(let Some(id) = self.ids.get(&kind), *id);
        ret_if!(let Some(id) = self.plain_enum(unit, kind), id);
        let root = unit.root();
        let id = unit.add(root, tag_of(kind));
        self.ids.insert(kind, id);
        let size = kind.checked_size(self.platform).unwrap_or(0);
        let die = unit.get_mut(id);
        die.set(gimli::DW_AT_name, AttributeValue::String(kind.name().into_bytes()));
        die.set(gimli::DW_AT_byte_size, AttributeValue::Udata(u64::from(size)));
        self.fill(unit, id, kind);
        id
    }

    /// The DIE of an enumeration whose members hold no integer, such as
    /// one of text: the DIE of the kind each member value has.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit.
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The DIE, or `None` for each other kind.
    fn plain_enum(&mut self, unit: &mut Unit, kind: DataKind) -> Option<UnitEntryId>
    {
        let e = kind.as_enum()?;
        ret_if!(int_members(e.members()).is_some(), None);
        let id = self.die_of(unit, e.under());
        self.ids.insert(kind, id);
        Some(id)
    }

    /// Fill in the parts of the DIE of a kind.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit.
    /// * `id` - The DIE.
    /// * `kind` - The kind.
    fn fill(&mut self, unit: &mut Unit, id: UnitEntryId, kind: DataKind)
    {
        let platform = self.platform;
        let words = u64::from(platform / 8);
        match kind
        {
            DataKind::Int(n) => encode(
                unit,
                id,
                tern!(n.signed, gimli::DW_ATE_signed, gimli::DW_ATE_unsigned),
            ),
            DataKind::Float(_) => encode(unit, id, gimli::DW_ATE_float),
            DataKind::Char => encode(unit, id, gimli::DW_ATE_UTF),
            DataKind::Bool => encode(unit, id, gimli::DW_ATE_boolean),
            DataKind::Task(_) => encode(unit, id, gimli::DW_ATE_unsigned),
            DataKind::Ptr(_) => self.pointer(unit, id, kind.pointee(), words),
            DataKind::Null => self.pointer(unit, id, None, words),
            DataKind::Fn(_) => self.fn_pointer(unit, id, words),
            DataKind::List(l) => self.fixed_list(unit, id, l.cell, l.len),
            DataKind::Struct(s) =>
            {
                self.fields(unit, id, s.fields(), |i| s.offset(i, platform))
            }
            DataKind::Seq(s) => self.seq(unit, id, s.cell, words),
            DataKind::Enum(e) => self.members(unit, id, e.under(), e.members()),
        }
    }

    /// A pointer to the DIE of its pointee, or to nothing known (`void *`).
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit.
    /// * `id` - The DIE of the pointer.
    /// * `pointee` - The kind it points at.
    /// * `words` - The size of a word, in octets.
    fn pointer(
        &mut self, unit: &mut Unit, id: UnitEntryId, pointee: Option<DataKind>,
        words: u64,
    )
    {
        unit.get_mut(id).set(gimli::DW_AT_byte_size, AttributeValue::Udata(words));
        let Some(to) = pointee
        else
        {
            return;
        };
        let to = self.die_of(unit, to);
        unit.get_mut(id).set(gimli::DW_AT_type, AttributeValue::UnitRef(to));
    }

    /// A function value: a pointer to a function of no known signature.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit.
    /// * `id` - The DIE of the pointer.
    /// * `words` - The size of a word, in octets.
    fn fn_pointer(&mut self, unit: &mut Unit, id: UnitEntryId, words: u64)
    {
        let root = unit.root();
        let routine = unit.add(root, gimli::DW_TAG_subroutine_type);
        let die = unit.get_mut(id);
        die.set(gimli::DW_AT_byte_size, AttributeValue::Udata(words));
        die.set(gimli::DW_AT_type, AttributeValue::UnitRef(routine));
    }

    /// A fixed list: the DWARF of its cells, with one range of its count.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit.
    /// * `id` - The DIE of the list.
    /// * `cell` - The kind of each cell.
    /// * `len` - The count of cells.
    fn fixed_list(&mut self, unit: &mut Unit, id: UnitEntryId, cell: DataKind, len: u32)
    {
        let cell = self.die_of(unit, cell);
        unit.get_mut(id).set(gimli::DW_AT_type, AttributeValue::UnitRef(cell));
        let range = unit.add(id, gimli::DW_TAG_subrange_type);
        unit.get_mut(range)
            .set(gimli::DW_AT_count, AttributeValue::Udata(u64::from(len)));
    }

    /// A struct: a member for each field, at its offset.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit.
    /// * `id` - The DIE of the struct.
    /// * `fields` - The name and kind of each field.
    /// * `offset` - The offset of the field at an index.
    fn fields(
        &mut self, unit: &mut Unit, id: UnitEntryId, fields: &[(String, DataKind)],
        offset: impl Fn(usize) -> u32,
    )
    {
        do_for!((i, (name, kind)) in fields.iter().enumerate(), {
            self.member(unit, id, name, *kind, u64::from(offset(i)));
        });
    }

    /// Text, a plain list or a box: three words, the address of the first
    /// cell, the count and the room.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit.
    /// * `id` - The DIE of the structure.
    /// * `cell` - The kind of each cell.
    /// * `words` - The size of a word, in octets.
    fn seq(&mut self, unit: &mut Unit, id: UnitEntryId, cell: DataKind, words: u64)
    {
        let kinds = [cell.raw_pointer_to(), DataKind::USIZE, DataKind::USIZE];
        do_for!((i, (name, kind)) in SEQ_PARTS.iter().zip(kinds).enumerate(), {
            self.member(unit, id, name, kind, i as u64 * words);
        });
    }

    /// One member of a structure.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit.
    /// * `id` - The DIE of the structure.
    /// * `name` - The name of the member.
    /// * `kind` - Its kind.
    /// * `offset` - Its offset, in octets.
    fn member(
        &mut self, unit: &mut Unit, id: UnitEntryId, name: &str, kind: DataKind,
        offset: u64,
    )
    {
        let to = self.die_of(unit, kind);
        let member = unit.add(id, gimli::DW_TAG_member);
        let die = unit.get_mut(member);
        die.set(gimli::DW_AT_name, AttributeValue::String(name.as_bytes().to_vec()));
        die.set(gimli::DW_AT_type, AttributeValue::UnitRef(to));
        die.set(gimli::DW_AT_data_member_location, AttributeValue::Udata(offset));
    }

    /// An enumeration of integer members: an enumerator for each, over the
    /// kind of its values.
    ///
    /// # Arguments
    ///
    /// * `unit` - The unit.
    /// * `id` - The DIE of the enumeration.
    /// * `under` - The kind of the member values.
    /// * `members` - The members.
    fn members(
        &mut self, unit: &mut Unit, id: UnitEntryId, under: DataKind,
        members: &[(String, MemberValue)],
    )
    {
        let under_id = self.die_of(unit, under);
        unit.get_mut(id).set(gimli::DW_AT_type, AttributeValue::UnitRef(under_id));
        let values = int_members(members).unwrap_or_default();
        do_for!(((name, _), value) in members.iter().zip(values), {
            let member = unit.add(id, gimli::DW_TAG_enumerator);
            let die = unit.get_mut(member);
            die.set(gimli::DW_AT_name, AttributeValue::String(name.as_bytes().to_vec()));
            die.set(gimli::DW_AT_const_value, AttributeValue::Sdata(value));
        });
    }
}

/// The tag of the DIE of a kind.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The tag.
fn tag_of(kind: DataKind) -> gimli::DwTag
{
    match kind
    {
        DataKind::Ptr(_) | DataKind::Null | DataKind::Fn(_) =>
        {
            gimli::DW_TAG_pointer_type
        }
        DataKind::List(_) => gimli::DW_TAG_array_type,
        DataKind::Struct(_) | DataKind::Seq(_) => gimli::DW_TAG_structure_type,
        DataKind::Enum(_) => gimli::DW_TAG_enumeration_type,
        _ => gimli::DW_TAG_base_type,
    }
}

/// Set the encoding of a base kind.
///
/// # Arguments
///
/// * `unit` - The unit.
/// * `id` - The DIE.
/// * `ate` - The encoding.
fn encode(unit: &mut Unit, id: UnitEntryId, ate: gimli::DwAte)
{
    unit.get_mut(id).set(gimli::DW_AT_encoding, AttributeValue::Encoding(ate));
}

/// The value of each member of an enumeration, when each holds an integer.
///
/// # Arguments
///
/// * `members` - The members.
///
/// # Returns
///
/// The values, in the order of the members, or `None` when a member holds
/// something else.
fn int_members(members: &[(String, MemberValue)]) -> Option<Vec<i64>>
{
    let value = |m: &MemberValue| match m
    {
        MemberValue::Scalar(Value::Int { raw, .. }) => Some(*raw as i64),
        _ => None,
    };
    ret_if!(members.is_empty(), None);
    members.iter().map(|(_, m)| value(m)).collect()
}
