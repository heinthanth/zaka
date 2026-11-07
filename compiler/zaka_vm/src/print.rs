pub use zakac_ir_builder::value::FALSE_TEXT;
pub use zakac_ir_builder::value::NULL_TEXT;
pub use zakac_ir_builder::value::TRUE_TEXT;
use zakac_ir_builder::value::Value;
pub use zakac_ir_builder::value::burmese_digits;
pub use zakac_ir_builder::value::value_text as text;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::enums::EnumKind;
use zakac_semcheck::enums::MemberValue;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::FALLIBLE;
use zakac_semcheck::structs::StructKind;

use crate::mem::Mem;
use crate::trap::TrapKind;

/// The text a print op writes for a list or a struct, without the line
/// break: the cells of a list between `[` and `]`, the fields of a struct as
/// `name: value` between `(` and `)` after the struct name, each joined with
/// `၊ `; a list or struct inside is written the same.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the value.
/// * `addr` - The address of the value.
/// * `kind` - The list or struct kind; another kind gives the text of one
///   value.
///
/// # Returns
///
/// The text, or the trap a read of a cell or field hits.
pub fn list_text(mem: &Mem, addr: u64, kind: DataKind) -> Result<String, TrapKind>
{
    ret_if!(let Some(e) = kind.as_enum(), member_text(mem, addr, e));
    ret_if!(kind.is_fn(), mem.load(addr, kind).map(|v| fn_value_text(mem, v)));
    ret_if!(let Some(s) = kind.as_struct(), struct_text(mem, addr, s));
    ret_if!(kind.is_textual(), text_at(mem, addr));
    if let Some(cell) =
        kind.as_slice().map(|s| s.cell).or_else(|| kind.as_seq().map(|s| s.cell))
    {
        let (base, count) = slice_parts(mem, addr)?;
        return cells_text(mem, base, cell, count);
    }
    let Some(list) = kind.as_list()
    else
    {
        return mem.load(addr, kind).map(text);
    };
    cells_text(mem, addr, list.cell, u64::from(list.len))
}

/// The text a print writes for a value: a member of an enumeration as
/// `<stem>များထဲမှ <member>` and a function value as
/// `လုပ်ဆောင်ချက် <name> ၏ လိပ်စာ`, the forms that read back; a list, a struct or
/// text as [`list_text`] writes it; each other value as [`text`] writes it.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the value.
/// * `v` - The value, or the address of one that storage holds as a list, a
///   struct or text.
/// * `kind` - Its kind.
///
/// # Returns
///
/// The text, or the trap a read hits.
pub fn shown_text(mem: &Mem, v: Value, kind: DataKind) -> Result<String, TrapKind>
{
    let stored = kind.repr();
    if stored.is_aggregate()
    {
        let reason = "a printed list, struct or text is no address".to_string();
        let addr = v.as_ptr().ok_or(TrapKind::Malformed { reason })?;
        return list_text(mem, addr, kind);
    }
    ret_if!(kind.is_fn(), Ok(fn_value_text(mem, v)));
    let Some(e) = kind.as_enum()
    else
    {
        return Ok(text(v));
    };
    let at = e.find(|m| m.scalar() == Some(v));
    Ok(at.map_or_else(|| text(v), |at| e.text_at(at)))
}

/// The text of a function value: the function it holds, in the form that
/// reads back, or `ဗလာ`.
///
/// # Arguments
///
/// * `mem` - The RAM, which knows the function names.
/// * `v` - The value.
///
/// # Returns
///
/// The text.
fn fn_value_text(mem: &Mem, v: Value) -> String
{
    v.as_ptr().map_or_else(|| text(v), |word| mem.fn_text(word))
}

/// The text of a member of an enumeration held at an address: its name, or
/// the value as its value kind prints when no member has it.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the value.
/// * `addr` - The address of the value.
/// * `e` - The enumeration.
///
/// # Returns
///
/// The text, or the trap a read hits.
fn member_text(mem: &Mem, addr: u64, e: &EnumKind) -> Result<String, TrapKind>
{
    for (at, (_, v)) in e.members().iter().enumerate()
    {
        ret_if!(holds_value(mem, addr, e.under(), v)?, Ok(e.text_at(at)));
    }
    list_text(mem, addr, e.under())
}

/// Whether the value at an address is a member value.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the value.
/// * `addr` - The address of the value.
/// * `kind` - The value kind.
/// * `v` - The member value.
///
/// # Returns
///
/// `true` if it is, or the trap a read hits.
fn holds_value(
    mem: &Mem, addr: u64, kind: DataKind, v: &MemberValue,
) -> Result<bool, TrapKind>
{
    let kind = kind.repr();
    match v
    {
        MemberValue::Scalar(x) => Ok(mem.load(addr, kind)? == *x),
        MemberValue::Text(t) => Ok(text_at(mem, addr)? == *t),
        MemberValue::Items(items) => items_held(mem, addr, kind, items),
    }
}

/// Whether the cells or fields at an address are the items of a member
/// value.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the value.
/// * `addr` - The address of the list or the struct.
/// * `kind` - Its kind.
/// * `items` - The cells or fields of the member value.
///
/// # Returns
///
/// `true` when so, or the trap a read hits.
fn items_held(
    mem: &Mem, addr: u64, kind: DataKind, items: &[MemberValue],
) -> Result<bool, TrapKind>
{
    let platform = mem.platform();
    if let Some(s) = kind.as_struct()
    {
        for (at, (v, (_, k))) in items.iter().zip(s.fields()).enumerate()
        {
            let field = addr + u64::from(s.offset(at, platform));
            ret_if!(!holds_value(mem, field, *k, v)?, Ok(false));
        }
        return Ok(true);
    }
    let Some(cell) = kind.cell_kind()
    else
    {
        return Ok(false);
    };
    let (base, count) =
        tern!(kind.is_seq(), slice_parts(mem, addr)?, (addr, items.len() as u64));
    ret_if!(count != items.len() as u64, Ok(false));
    let stride = u64::from(cell.size(platform));
    for (i, v) in items.iter().enumerate()
    {
        ret_if!(!holds_value(mem, base + i as u64 * stride, cell, v)?, Ok(false));
    }
    Ok(true)
}

/// The text of a run of cells, between `[` and `]` and joined with `၊ `.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the cells.
/// * `addr` - The address of the first cell.
/// * `cell` - The kind of each cell.
/// * `count` - The number of cells.
///
/// # Returns
///
/// The text, or the trap a read of a cell hits.
fn cells_text(
    mem: &Mem, addr: u64, cell: DataKind, count: u64,
) -> Result<String, TrapKind>
{
    let stride = u64::from(cell.size(mem.platform()));
    let one = |i: u64| nested_text(mem, addr + i * stride, cell);
    let cells: Vec<String> = (0..count).map(one).collect::<Result<_, _>>()?;
    Ok(format!("[{}]", cells.join("၊ ")))
}

/// The text of text, owned or a view: its u8s read as UTF-8, a broken
/// sequence as `U+FFFD`.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the text.
/// * `addr` - The address of the text; it starts with the two words of a slice.
///
/// # Returns
///
/// The text, or the trap a read hits.
pub fn text_at(mem: &Mem, addr: u64) -> Result<String, TrapKind>
{
    let (base, count) = slice_parts(mem, addr)?;
    Ok(String::from_utf8_lossy(&mem.octets(base, count)?).into_owned())
}

/// The text of a cell or a field: as [`list_text`] writes it, with text in
/// quotes, so the whole reads back as notation.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the value.
/// * `addr` - The address of the value.
/// * `kind` - Its kind.
///
/// # Returns
///
/// The text, or the trap a read hits.
fn nested_text(mem: &Mem, addr: u64, kind: DataKind) -> Result<String, TrapKind>
{
    let text = list_text(mem, addr, kind)?;
    Ok(tern!(kind.is_textual(), quoted(&text), text))
}

/// Text in double quotes as a literal writes it: a backslash, a quote and
/// each brace escaped.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The literal.
fn quoted(text: &str) -> String
{
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
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

/// The two words of a slice: the address of its first cell and its count.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the slice.
/// * `addr` - The address of the slice.
///
/// # Returns
///
/// The address and the count, or the trap a read hits.
pub fn slice_parts(mem: &Mem, addr: u64) -> Result<(u64, u64), TrapKind>
{
    let word = u64::from(mem.platform() / 8);
    let base = mem.load(addr, DataKind::OPAQUE)?.as_ptr().unwrap_or(0);
    let count = match mem.load(addr + word, DataKind::USIZE)?
    {
        Value::Int { raw, .. } => u64::try_from(raw).unwrap_or(u64::MAX),
        _ => 0,
    };
    Ok((base, count))
}

/// The text of a result of a function that can fail: `ရလဒ်ကောင်း(<value>)`
/// for a good result and `အမှားအယွင်း(<value>)` for an error value; a side
/// with no value writes `အောင်မြင်သည့်သတင်း` or `မအောင်မြင်သည့်သတင်း`.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the result.
/// * `addr` - The address of the result.
/// * `s` - The struct kind of the result.
///
/// # Returns
///
/// The text, or the trap a read of a part hits.
fn fallible_text(mem: &Mem, addr: u64, s: &StructKind) -> Result<String, TrapKind>
{
    let platform = mem.platform();
    let part = |name: &str| {
        s.field(name).map(|(at, kind)| (addr + u64::from(s.offset(at, platform)), kind))
    };
    let (tag_at, tag_kind) =
        part("tag").ok_or(TrapKind::BadValue { addr, kind: s.name.clone() })?;
    // Tag 0, the blank of a result, is a failure.
    let failed = mem.load(tag_at, tag_kind)? == Value::default_of(tag_kind);
    let (word, shown) =
        tern!(failed, ("အမှားအယွင်း", part("err")), ("ရလဒ်ကောင်း", part("ok")));
    let bare = tern!(failed, "မအောင်မြင်သည့်သတင်း", "အောင်မြင်သည့်သတင်း");
    let Some((at, kind)) = shown
    else
    {
        return Ok(bare.to_string());
    };
    Ok(format!("{word}({})", nested_text(mem, at, kind)?))
}

/// The text of a struct: `<struct>(name: value၊ ...)`.
///
/// # Arguments
///
/// * `mem` - The RAM that holds the struct.
/// * `addr` - The address of the struct.
/// * `s` - The struct kind.
///
/// # Returns
///
/// The text, or the trap a read of a field hits.
fn struct_text(mem: &Mem, addr: u64, s: &StructKind) -> Result<String, TrapKind>
{
    ret_if!(s.name == FALLIBLE, fallible_text(mem, addr, s));
    let platform = mem.platform();
    let field = |(at, (name, kind)): (usize, &(String, DataKind))| {
        let offset = u64::from(s.offset(at, platform));
        Ok(format!("{name}: {}", nested_text(mem, addr + offset, *kind)?))
    };
    let fields: Vec<String> =
        s.fields().iter().enumerate().map(field).collect::<Result<_, _>>()?;
    Ok(format!("{}({})", s.name, fields.join("၊ ")))
}
