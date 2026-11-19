use std::collections::HashMap;
use std::fmt::Write;

use zakac_semcheck::enums::MemberValue;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::FALLIBLE;
use zakac_semcheck::value::value_text;

use crate::emit::c_string;

/// The word of a good result, as the VM prints it.
pub const GOOD_TEXT: &str = "ရလဒ်ကောင်း";
/// The word of an error value, as the VM prints it.
pub const FAULT_TEXT: &str = "အမှားအယွင်း";
/// The word of a good result with no value.
pub const GOOD_BARE: &str = "အောင်မြင်သည့်သတင်း";
/// The word of a failure with no value.
pub const FAULT_BARE: &str = "မအောင်မြင်သည့်သတင်း";
/// The word of a print of an optional that holds data, before the data.
pub const DATA_TEXT: &str = "ဒေတာ";
/// The print of an optional that holds no data.
pub const NO_DATA_TEXT: &str = "အလွတ်";
/// The text before the name of a function value.
pub const FN_LEAD: &str = "";
/// The text after the name of a function value.
pub const FN_TAIL: &str = " ဟူသော လုပ်ဆောင်ချက်";

/// One kind descriptor as the runtime reads it, `struct zk_kind`: each
/// target writes it in its own form (C text, or data of an object).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindRow
{
    /// The shape, a member of `enum zk_tag`.
    pub tag: i32,
    /// The size in octets.
    pub size: u32,
    /// The number of the descriptor of the cell, or of the value kind of an
    /// enumeration.
    pub cell: Option<usize>,
    /// The cell count, the field count or the member count.
    pub len: usize,
    /// Each field: its name, its offset, and the number of its descriptor.
    pub fields: Vec<(String, u32, usize)>,
    /// Each member: its value as text, and the text a print writes.
    pub members: Vec<(String, String)>,
    /// The name of a struct.
    pub name: Option<String>,
    /// Whether a value holds text, a plain list or a closure on the heap.
    pub holds_text: bool,
    /// Whether a value holds a borrow.
    pub holds_borrow: bool,
    /// Whether the kind is an owner pointer, or holds one.
    pub owns: bool,
    /// Whether a plain list is a box.
    pub boxed: bool,
}

/// The kind descriptors a program needs: one `zk_kind` in `zaka_kinds.c` for
/// each data kind the runtime walks, named `zk_k<n>`.
#[derive(Default)]
pub struct Descriptors
{
    /// The number of each kind with a descriptor.
    index: HashMap<DataKind, usize>,
    /// The kinds, in the order of their numbers.
    kinds: Vec<DataKind>,
}

impl Descriptors
{
    /// The C text of the address of the descriptor of a kind, made on first
    /// use with the descriptors of the kinds inside it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The C text, such as `&zk_k3`.
    pub fn of(&mut self, kind: DataKind) -> String
    {
        format!("&zk_k{}", self.number(kind))
    }

    /// The number of the descriptor of a kind, made on first use with the
    /// descriptors of the kinds inside it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The number.
    pub fn index_of(&mut self, kind: DataKind) -> usize { self.number(kind) }

    /// The count of descriptors so far.
    ///
    /// # Returns
    ///
    /// The count.
    pub fn count(&self) -> usize { self.kinds.len() }

    /// Each descriptor, in the order of its number.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The rows.
    pub fn rows(&self, platform: u16) -> Vec<KindRow>
    {
        self.kinds.iter().map(|k| self.row(*k, platform)).collect()
    }

    /// The row of one descriptor.
    ///
    /// # Arguments
    ///
    /// * `kind` - Its kind.
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The row.
    fn row(&self, kind: DataKind, platform: u16) -> KindRow
    {
        let at = |k: DataKind| self.index[&k];
        let mut row = KindRow {
            tag: tag(kind),
            size: kind.size(platform),
            cell: None,
            len: 0,
            fields: Vec::new(),
            members: Vec::new(),
            name: None,
            holds_text: kind.holds_text(),
            holds_borrow: kind.holds_borrow(),
            owns: kind.holds_owner_ptr(),
            boxed: kind.is_box(),
        };
        if let Some(s) = kind.as_struct()
        {
            let each = s.fields().iter().enumerate();
            row.fields = each
                .map(|(i, (f, k))| (f.clone(), s.offset(i, platform), at(*k)))
                .collect();
            (row.len, row.name) = (s.fields().len(), Some(s.name.clone()));
        }
        if let Some(e) = kind.as_enum()
        {
            let each = e.members().iter().enumerate();
            row.members = each
                .map(|(i, (_, v))| (member_text(e.under(), v, false), e.text_at(i)))
                .collect();
            (row.cell, row.len) = (Some(at(e.under())), e.members().len());
        }
        let inner = kind.as_list().map(|l| (l.cell, l.len as usize));
        let inner = inner.or_else(|| kind.cell_kind().map(|c| (c, 0)));
        if let Some((c, count)) = inner.filter(|_| kind.as_enum().is_none())
        {
            (row.cell, row.len) = (Some(at(c)), count);
        }
        row
    }

    /// The number of the descriptor of a kind, made on first use.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The number.
    fn number(&mut self, kind: DataKind) -> usize
    {
        if let Some(n) = self.index.get(&kind)
        {
            return *n;
        }
        let n = self.kinds.len();
        self.index.insert(kind, n);
        self.kinds.push(kind);
        inner_kinds(kind).into_iter().for_each(|k| _ = self.number(k));
        n
    }

    /// The declaration of each descriptor, for the shared header.
    ///
    /// # Returns
    ///
    /// The C text.
    pub fn declarations(&self) -> String
    {
        let decl = |n: usize| format!("extern const zk_kind zk_k{n};\n");
        (0..self.kinds.len()).map(decl).collect()
    }

    /// The definition of each descriptor, for `zaka_kinds.c`.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The C text.
    pub fn definitions(&self, platform: u16) -> String
    {
        let mut out = String::new();
        for (n, kind) in self.kinds.iter().enumerate()
        {
            out.push_str(&self.definition(n, *kind, platform));
        }
        out
    }

    /// The definition of one descriptor, with the table of its fields or
    /// members before it.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of the descriptor.
    /// * `kind` - Its kind.
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The C text.
    fn definition(&self, n: usize, kind: DataKind, platform: u16) -> String
    {
        let r = self.row(kind, platform);
        let at = |i: Option<usize>| {
            i.map_or_else(|| "NULL".to_string(), |i| format!("&zk_k{i}"))
        };
        let mut out = String::new();
        let mut fields = "NULL".to_string();
        if !r.fields.is_empty() || kind.as_struct().is_some()
        {
            let _ = writeln!(out, "static const zk_field zk_k{n}_f[] = {{");
            for (name, offset, k) in &r.fields
            {
                let _ = writeln!(
                    out,
                    "    {{.name = {}, .offset = {offset}, .kind = &zk_k{k}}},",
                    c_string(name)
                );
            }
            out.push_str("};\n");
            fields = format!("zk_k{n}_f");
        }
        let mut members = "NULL".to_string();
        if kind.as_enum().is_some()
        {
            let _ = writeln!(out, "static const zk_member zk_k{n}_m[] = {{");
            for (value, shown) in &r.members
            {
                let _ = writeln!(
                    out,
                    "    {{.value = {}, .shown = {}}},",
                    c_string(value),
                    c_string(shown)
                );
            }
            out.push_str("};\n");
            members = format!("zk_k{n}_m");
        }
        let name = r.name.as_deref().map_or_else(|| "NULL".to_string(), c_string);
        let (tag, size, cell, len) = (TAGS[r.tag as usize], r.size, at(r.cell), r.len);
        let flags = [r.holds_text, r.holds_borrow, r.owns, r.boxed].map(u8::from);
        let [text, borrow, owns, boxed] = flags;
        let _ = writeln!(
            out,
            "const zk_kind zk_k{n} = {{.tag = {tag}, .size = {size}, .cell = {cell}, \
             .len = {len}, .fields = {fields}, .members = {members}, .name = {name}, \
             .holds_text = {text}, .holds_borrow = {borrow}, .owns = {owns}, .boxed = \
             {boxed}}};"
        );
        out
    }
}

/// The kinds inside a kind that need descriptors of their own: the cell,
/// the fields, the value kind of an enumeration.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The kinds.
fn inner_kinds(kind: DataKind) -> Vec<DataKind>
{
    let mut out = Vec::new();
    out.extend(
        kind.as_struct().into_iter().flat_map(|s| s.fields().iter().map(|(_, k)| *k)),
    );
    out.extend(kind.as_enum().map(|e| e.under()));
    out.extend(kind.as_enum().is_none().then(|| kind.cell_kind()).flatten());
    out
}

/// The C names of the members of `enum zk_tag`, in their order.
pub const TAGS: [&str; 13] = [
    "ZK_TAG_SINT",
    "ZK_TAG_UINT",
    "ZK_TAG_FLOAT",
    "ZK_TAG_CHAR",
    "ZK_TAG_BOOL",
    "ZK_TAG_PTR",
    "ZK_TAG_FN",
    "ZK_TAG_LIST",
    "ZK_TAG_STRUCT",
    "ZK_TAG_FALLIBLE",
    "ZK_TAG_SEQ",
    "ZK_TAG_TEXT",
    "ZK_TAG_ENUM",
];

/// The tag of a kind in the runtime: the number of its member of
/// `enum zk_tag`.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The number.
fn tag(kind: DataKind) -> i32
{
    match kind
    {
        DataKind::Int(k) if k.signed => 0,
        // A job or a channel is the unsigned number that names it.
        DataKind::Int(_) | DataKind::Task(_) => 1,
        DataKind::Float(_) => 2,
        DataKind::Char => 3,
        DataKind::Bool => 4,
        DataKind::Ptr(_) | DataKind::Null => 5,
        DataKind::Fn(_) => 6,
        DataKind::List(_) => 7,
        DataKind::Struct(s) if s.name == FALLIBLE => 9,
        DataKind::Struct(_) => 8,
        DataKind::Seq(s) if s.text => 11,
        DataKind::Seq(_) => 10,
        DataKind::Enum(_) => 12,
    }
}

/// The text of a member value as a print of its value kind writes it, which
/// the runtime holds up to the print of the value it reads.
///
/// # Arguments
///
/// * `kind` - The value kind.
/// * `value` - The member value.
/// * `nested` - Whether it stands inside a list or a struct, where text is
///   quoted.
///
/// # Returns
///
/// The text.
fn member_text(kind: DataKind, value: &MemberValue, nested: bool) -> String
{
    match value
    {
        MemberValue::Scalar(v) => value_text(*v),
        MemberValue::Text(t) if nested => quoted(t),
        MemberValue::Text(t) => t.clone(),
        MemberValue::Items(items) => items_text(kind.repr(), items),
    }
}

/// The text of the cells or fields of a member value.
///
/// # Arguments
///
/// * `kind` - The list, plain list or struct kind.
/// * `items` - The cells or fields.
///
/// # Returns
///
/// The text.
fn items_text(kind: DataKind, items: &[MemberValue]) -> String
{
    let sep = "\u{104A} ";
    if let Some(s) = kind.as_struct()
    {
        let field = |((name, k), v): (&(String, DataKind), &MemberValue)| {
            format!("{name}: {}", member_text(*k, v, true))
        };
        let fields: Vec<String> = s.fields().iter().zip(items).map(field).collect();
        return format!("{}({})", s.name, fields.join(sep));
    }
    let cell = kind.cell_kind().unwrap_or(kind);
    let cells: Vec<String> = items.iter().map(|v| member_text(cell, v, true)).collect();
    format!("[{}]", cells.join(sep))
}

/// Text in quotes, as a print writes text inside a list or a struct: a
/// backslash and a quote escaped, each brace doubled.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The quoted text.
fn quoted(text: &str) -> String
{
    let mut out = String::from("\"");
    for c in text.chars()
    {
        match c
        {
            '\\' | '"' => out.extend(['\\', c]),
            '{' | '}' => out.extend([c, c]),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}
