pub use zakac_ir_builder::value::FALSE_TEXT;
pub use zakac_ir_builder::value::NULL_TEXT;
pub use zakac_ir_builder::value::TRUE_TEXT;
use zakac_ir_builder::value::Value;
pub use zakac_ir_builder::value::burmese_digits;
pub use zakac_ir_builder::value::value_text as text;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
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
    ret_if!(let Some(s) = kind.as_struct(), struct_text(mem, addr, s));
    ret_if!(kind.is_textual(), text_at(mem, addr));
    if let Some(slice) = kind.as_slice()
    {
        let (base, count) = slice_parts(mem, addr)?;
        return cells_text(mem, base, slice.cell, count);
    }
    let Some(list) = kind.as_list()
    else
    {
        return mem.load(addr, kind).map(text);
    };
    cells_text(mem, addr, list.cell, u64::from(list.len))
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
    let platform = mem.platform();
    let field = |(at, (name, kind)): (usize, &(String, DataKind))| {
        let offset = u64::from(s.offset(at, platform));
        Ok(format!("{name}: {}", nested_text(mem, addr + offset, *kind)?))
    };
    let fields: Vec<String> =
        s.fields().iter().enumerate().map(field).collect::<Result<_, _>>()?;
    Ok(format!("{}({})", s.name, fields.join("၊ ")))
}
