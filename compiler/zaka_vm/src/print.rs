use zakac_ir_builder::value::Value;
use zakac_ir_builder::value::float_pattern_text;

/// The text a print op writes for `true`.
pub const TRUE_TEXT: &str = "မှန်";
/// The text a print op writes for `false`.
pub const FALSE_TEXT: &str = "မမှန်";

/// The text a print op writes for a value, without the line break.
/// Numbers use Western digits; floats use the shortest text that reads
/// back; a character is written as itself.
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
        Value::Int { kind, raw } if kind.signed => (raw as i128).to_string(),
        Value::Int { raw, .. } => raw.to_string(),
        Value::Float { bits, raw } => float_pattern_text(bits, raw),
        Value::Char(c) => c.to_string(),
        Value::Bool(b) => (if b { TRUE_TEXT } else { FALSE_TEXT }).to_string(),
    }
}
