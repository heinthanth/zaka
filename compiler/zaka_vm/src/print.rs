use zakac_ir_builder::value::Value;
use zakac_ir_builder::value::float_pattern_text;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::structs::StructKind;

use crate::mem::Mem;
use crate::trap::TrapKind;

/// The text a print op writes for `true`.
pub const TRUE_TEXT: &str = "မှန်";
/// The text a print op writes for `false`.
pub const FALSE_TEXT: &str = "မမှန်";
/// The text a print op writes for the null address.
pub const NULL_TEXT: &str = "ဗလာ";

/// The text a print op writes for a value, without the line break.
/// Numbers use Burmese digits; floats use the shortest text that reads
/// back; a character is written as itself; an address is written as `0x`
/// and lowercase hex digits, the null address as `ဗလာ`.
///
/// # Arguments
///
/// * `v` - The value.
///
/// # Returns
///
/// The text.
pub fn text(v: Value) -> String
{
    match v
    {
        Value::Int { kind, raw } if kind.signed =>
        {
            burmese_digits(&(raw as i128).to_string())
        }
        Value::Int { raw, .. } => burmese_digits(&raw.to_string()),
        Value::Float { bits, raw } => burmese_digits(&float_pattern_text(bits, raw)),
        Value::Char(c) => c.to_string(),
        Value::Bool(b) => (if b { TRUE_TEXT } else { FALSE_TEXT }).to_string(),
        Value::Ptr(0) => NULL_TEXT.to_string(),
        Value::Ptr(a) => format!("{a:#x}"),
    }
}

/// Swap each Western digit for the Burmese digit of the same value; signs,
/// points, exponent marks and words such as `inf` stand as written.
///
/// # Arguments
///
/// * `num` - The number text in Western digits.
///
/// # Returns
///
/// The text in Burmese digits.
pub fn burmese_digits(num: &str) -> String
{
    num.chars()
        .map(|c| {
            c.to_digit(10).and_then(|d| char::from_u32('၀' as u32 + d)).unwrap_or(c)
        })
        .collect()
}

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
    let one = |i: u64| list_text(mem, addr + i * stride, cell);
    let cells: Vec<String> = (0..count).map(one).collect::<Result<_, _>>()?;
    Ok(format!("[{}]", cells.join("၊ ")))
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
        Ok(format!("{name}: {}", list_text(mem, addr + offset, *kind)?))
    };
    let fields: Vec<String> =
        s.fields().iter().enumerate().map(field).collect::<Result<_, _>>()?;
    Ok(format!("{}({})", s.name, fields.join("၊ ")))
}
