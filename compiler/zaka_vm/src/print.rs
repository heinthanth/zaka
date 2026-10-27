use zakac_ir_builder::value::Value;
use zakac_ir_builder::value::float_pattern_text;

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
