//! The text a print writes for a scalar: numbers in Burmese digits, floats as
//! the shortest text that reads back, and the words for truth values and the
//! null address. The VM, the check and the native runtime print through these,
//! so their output cannot drift.

use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

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

/// Read text as a 64-bit float: an optional sign, digits `0`-`9` or `၀`-`၉`, an
/// optional point with digits, and an optional exponent, `e` or `E` with an
/// optional sign and digits. White space and words such as `inf` are no
/// number.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The float, or the member of [`crate::NUMBER_ERROR`] that gives the reason
/// the text is no float: blank text, a character no float holds, or a float too
/// large for 64 bits.
pub fn parse_float(text: &str) -> Result<f64, u32>
{
    ret_if!(text.is_empty(), Err(crate::NUMBER_BLANK));
    let western: String = text.chars().map(western_digit).collect();
    let allowed =
        |c: char| c.is_ascii_digit() || matches!(c, '+' | '-' | '.' | 'e' | 'E');
    ret_if!(!western.chars().all(allowed), Err(crate::NUMBER_CHAR));
    let value: f64 = western.parse().map_err(|_| crate::NUMBER_CHAR)?;
    tern!(value.is_finite(), Ok(value), Err(crate::NUMBER_RANGE))
}

/// The Western digit of a Burmese digit; another character stands as it is.
///
/// # Arguments
///
/// * `c` - The character.
///
/// # Returns
///
/// The character, with `၀`-`၉` read as `0`-`9`.
fn western_digit(c: char) -> char
{
    let burmese = ('၀'..='၉').contains(&c);
    let shift = |c: char| char::from_u32(c as u32 - '၀' as u32 + '0' as u32);
    tern!(burmese, shift(c).unwrap_or(c), c)
}
