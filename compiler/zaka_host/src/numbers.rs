//! The text a print writes for a scalar: numbers in Burmese digits, floats as
//! the shortest text that reads back, and the words for truth values and the
//! null address. The VM, the check and the native runtime print through these,
//! so their output cannot drift.

/// The text a print op writes for `true`.
pub const TRUE_TEXT: &str = "မှန်";
/// The text a print op writes for `false`.
pub const FALSE_TEXT: &str = "မမှန်";
/// The text a print op writes for the null address: the phrase the program
/// writes for it, so the output reads as the source does.
pub const NULL_TEXT: &str = "မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ";

/// Write a float pattern as text: the shortest text that reads back, as
/// Rust writes it: `3.0`, `0.1`, `1e300`, `inf`, `NaN`.
///
/// # Arguments
///
/// * `bits` - The bit count of the format: 32, else 64.
/// * `raw` - The bit pattern.
///
/// # Returns
///
/// The text.
pub fn float_pattern_text(bits: u16, raw: u64) -> String
{
    match bits
    {
        32 => format!("{:?}", f32::from_bits(raw as u32)),
        _ => format!("{:?}", f64::from_bits(raw)),
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

/// The text a print writes for an integer, in Burmese digits.
///
/// # Arguments
///
/// * `raw` - The value, sign-extended to 128 bits when `signed`.
/// * `signed` - Whether the kind is signed.
///
/// # Returns
///
/// The text.
pub fn int_text(raw: u128, signed: bool) -> String
{
    let western = match signed
    {
        true => (raw as i128).to_string(),
        false => raw.to_string(),
    };
    burmese_digits(&western)
}

/// The text a print writes for a float, in Burmese digits.
///
/// # Arguments
///
/// * `bits` - The bit count of the format: 32, else 64.
/// * `raw` - The bit pattern.
///
/// # Returns
///
/// The text.
pub fn float_text(bits: u16, raw: u64) -> String
{
    burmese_digits(&float_pattern_text(bits, raw))
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn numbers_print_in_burmese_digits()
    {
        assert_eq!(int_text((-12i128) as u128, true), "-၁၂");
        assert_eq!(int_text(u64::MAX as u128, false), "၁၈၄၄၆၇၄၄၀၇၃၇၀၉၅၅၁၆၁၅");
        assert_eq!(float_text(64, 0.1f64.to_bits()), "၀.၁");
        assert_eq!(float_text(32, u64::from(1.5f32.to_bits())), "၁.၅");
        assert_eq!(float_text(64, f64::INFINITY.to_bits()), "inf");
    }
}
