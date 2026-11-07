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
/// The text before the name of a function value.
pub const FN_LEAD: &str = "လုပ်ဆောင်ချက် ";
/// The text after the name of a function value.
pub const FN_TAIL: &str = " ၏ လိပ်စာ";

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
        let at = |k: DataKind| format!("&zk_k{}", self.index[&k]);
        let mut out = String::new();
        let (mut cell, mut len, mut fields, mut members, mut name) = (
            "NULL".to_string(),
            0,
            "NULL".to_string(),
            "NULL".to_string(),
            "NULL".to_string(),
        );
        if let Some(s) = kind.as_struct()
        {
            let _ = writeln!(out, "static const zk_field zk_k{n}_f[] = {{");
            for (i, (field, k)) in s.fields().iter().enumerate()
            {
                let offset = s.offset(i, platform);
                let _ =
                    writeln!(out, "    {{{}, {offset}, {}}},", c_string(field), at(*k));
            }
            out.push_str("};\n");
            (len, fields, name) =
                (s.fields().len(), format!("zk_k{n}_f"), c_string(&s.name));
        }
        if let Some(e) = kind.as_enum()
        {
            let _ = writeln!(out, "static const zk_member zk_k{n}_m[] = {{");
            for (i, (_, value)) in e.members().iter().enumerate()
            {
                let text = member_text(e.under(), value, false);
                let _ = writeln!(
                    out,
                    "    {{{}, {}}},",
                    c_string(&text),
                    c_string(&e.text_at(i))
                );
            }
            out.push_str("};\n");
            (cell, len, members) =
                (at(e.under()), e.members().len(), format!("zk_k{n}_m"));
        }
        let inner = kind.as_list().map(|l| (l.cell, l.len as usize));
        let inner = inner.or_else(|| kind.cell_kind().map(|c| (c, 0)));
        if let Some((c, count)) = inner.filter(|_| kind.as_enum().is_none())
        {
            (cell, len) = (at(c), count);
        }
        let tag = tag(kind);
        let size = kind.size(platform);
        let text = u8::from(kind.holds_text());
        let _ = writeln!(
            out,
            "const zk_kind zk_k{n} = {{{tag}, {size}, {cell}, {len}, {fields}, \
             {members}, {name}, {text}}};"
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

/// The tag of a kind in the runtime, a member of `enum zk_tag`.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// The C name of the tag.
fn tag(kind: DataKind) -> &'static str
{
    match kind
    {
        DataKind::Int(k) if k.signed => "ZK_K_SINT",
        DataKind::Int(_) => "ZK_K_UINT",
        DataKind::Float(_) => "ZK_K_FLOAT",
        DataKind::Char => "ZK_K_CHAR",
        DataKind::Bool => "ZK_K_BOOL",
        DataKind::Ptr(_) | DataKind::Null => "ZK_K_PTR",
        DataKind::Fn(_) => "ZK_K_FN",
        DataKind::List(_) => "ZK_K_LIST",
        DataKind::Struct(s) if s.name == FALLIBLE => "ZK_K_FALLIBLE",
        DataKind::Struct(_) => "ZK_K_STRUCT",
        DataKind::Slice(s) if s.text => "ZK_K_VIEW",
        DataKind::Slice(_) => "ZK_K_SLICE",
        DataKind::Seq(s) if s.text => "ZK_K_TEXT",
        DataKind::Seq(_) => "ZK_K_SEQ",
        DataKind::Enum(_) => "ZK_K_ENUM",
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
