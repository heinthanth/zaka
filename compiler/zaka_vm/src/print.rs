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
/// `name: value` between `{` and `}`, each joined with `၊ `; a list or struct
/// inside is written the same.
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
    let Some(list) = kind.as_list()
    else
    {
        return mem.load(addr, kind).map(text);
    };
    let stride = u64::from(list.cell.size(mem.platform()));
    let cell = |i: u64| list_text(mem, addr + i * stride, list.cell);
    let cells: Vec<String> =
        (0..u64::from(list.len)).map(cell).collect::<Result<_, _>>()?;
    Ok(format!("[{}]", cells.join("၊ ")))
}

/// The text of a struct: `{name: value၊ ...}`.
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
    Ok(format!("{{{}}}", fields.join("၊ ")))
}
