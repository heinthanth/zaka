use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use rustc_apfloat::Float;
use rustc_apfloat::FloatConvert;
use rustc_apfloat::Round;
use rustc_apfloat::ieee::Double;
use rustc_apfloat::ieee::Half;
use rustc_apfloat::ieee::Quad;
use rustc_apfloat::ieee::Single;
use serde::Deserialize;
use serde::Serialize;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::IntKind;

use crate::ir::CmpOp;
use crate::ir::InfixOp;
use crate::ir::LogicOp;
use crate::ir::PrefixOp;

/// Rounding of all float operations: to nearest, ties to even.
const NEAREST: Round = Round::NearestTiesToEven;

/// Run `$code` with `$f` naming the float format of `$bits` bits.
macro_rules! per_width {
    ($bits:expr, $f:ident => $code:expr) => {
        match $bits
        {
            16 =>
            {
                type $f = Half;
                $code
            }
            32 =>
            {
                type $f = Single;
                $code
            }
            64 =>
            {
                type $f = Double;
                $code
            }
            _ =>
            {
                type $f = Quad;
                $code
            }
        }
    };
}

/// A constant. Artifacts store it as text with its kind, such as `-5:i8`,
/// `1.5:f32`, `'က':char`, `true:bool` or `0x1000:ptr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum Value
{
    /// An integer. `raw` holds the value's two's-complement bits: sign
    /// extended to 128 bits for a signed kind, zero extended for an
    /// unsigned one. It lies in the range of the kind.
    Int
    {
        /// The kind.
        kind: IntKind,
        /// The bits.
        raw: u128,
    },
    /// An IEEE 754 float of `bits` bits, stored as its bit pattern.
    Float
    {
        /// The bit count of the format.
        bits: u16,
        /// The bit pattern.
        raw: u128,
    },
    /// A character.
    Char(char),
    /// A boolean.
    Bool(bool),
    /// An address. Its kind is `ptr`; the checker alone knows what it
    /// points at.
    Ptr(u64),
}

/// The reason raw octets do not read back as a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawError
{
    /// The octet count differs from the size of the kind.
    WrongLen,
    /// The octets hold no value of the kind: a boolean other than 0 or 1,
    /// or a character that is no Unicode scalar value.
    BadValue,
}

/// The reason an operation on constants has no value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalError
{
    /// The result is outside the range of its kind.
    Overflow,
    /// An integer division or remainder with zero as divisor.
    ZeroDivisor,
    /// A shift distance not below the bit width.
    ShiftTooFar
    {
        /// The distance.
        distance: u128,
        /// The bit width of the shifted value.
        bits: u16,
    },
    /// The operands do not suit the operator; the checker rules this out.
    Mismatch,
}

impl Value
{
    /// The null address.
    pub const NULL: Value = Value::Ptr(0);

    /// Make an integer from a bit pattern, cut to the width of its kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    /// * `pattern` - The bits; those above the width are dropped.
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn int(kind: IntKind, pattern: u128, platform: u16) -> Value
    {
        Value::Int { kind, raw: wrap(kind, platform, pattern) }
    }

    /// Read a decimal float literal at the exact width of its kind.
    ///
    /// # Arguments
    ///
    /// * `text` - Decimal digits with an optional fraction and exponent, `inf`,
    ///   `-inf` or `NaN`.
    /// * `bits` - The bit count of the format.
    ///
    /// # Returns
    ///
    /// The value rounded to nearest, or `None` for text that is no number.
    pub fn float(text: &str, bits: u16) -> Option<Value>
    {
        let raw = per_width!(bits, F => parse_float::<F>(text)?.to_bits());
        Some(Value::Float { bits, raw })
    }

    /// The default value of a kind: zero, `'\0'`, false or the null address.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The value.
    pub fn default_of(kind: DataKind) -> Value
    {
        match kind
        {
            DataKind::Int(kind) => Value::Int { kind, raw: 0 },
            DataKind::Float(bits) => Value::Float { bits, raw: 0 },
            DataKind::Char => Value::Char('\0'),
            DataKind::Bool => Value::Bool(false),
            // A list, a struct or a slice is no single value; its slot is
            // cleared to zero octets instead, the default of each cell or
            // field, and a slice of no cells.
            DataKind::Ptr(_)
            | DataKind::Null
            | DataKind::List(_)
            | DataKind::Struct(_)
            | DataKind::Slice(_) => Value::NULL,
        }
    }

    /// The data kind.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn kind(self) -> DataKind
    {
        match self
        {
            Value::Int { kind, .. } => DataKind::Int(kind),
            Value::Float { bits, .. } => DataKind::Float(bits),
            Value::Char(_) => DataKind::Char,
            Value::Bool(_) => DataKind::Bool,
            Value::Ptr(_) => DataKind::OPAQUE,
        }
    }

    /// The boolean, for a boolean value.
    ///
    /// # Returns
    ///
    /// The boolean, or `None`.
    pub fn as_bool(self) -> Option<bool>
    {
        tern!(let Value::Bool(b) = self, Some(b), None)
    }

    /// The address, for an address value.
    ///
    /// # Returns
    ///
    /// The address, or `None`.
    pub fn as_ptr(self) -> Option<u64>
    {
        tern!(let Value::Ptr(a) = self, Some(a), None)
    }

    /// The bit pattern of the value.
    ///
    /// # Returns
    ///
    /// The bits: an integer's two's complement, a float's pattern, a
    /// character's scalar value, 0 or 1 for a boolean, the address for a
    /// pointer.
    fn pattern(self) -> u128
    {
        match self
        {
            Value::Int { raw, .. } | Value::Float { raw, .. } => raw,
            Value::Char(c) => u128::from(c),
            Value::Bool(b) => u128::from(b),
            Value::Ptr(a) => u128::from(a),
        }
    }
}

/// Cut a bit pattern to the width of an integer kind and extend it again.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `platform` - Bit count of the platform-sized kinds.
/// * `pattern` - The bits.
///
/// # Returns
///
/// The bits, sign extended for a signed kind.
fn wrap(kind: IntKind, platform: u16, pattern: u128) -> u128
{
    let bits = kind.bits(platform);
    ret_if!(bits >= 128, pattern);
    let mask = (1u128 << bits) - 1;
    let low = pattern & mask;
    let negative = kind.signed && (low >> (bits - 1)) & 1 == 1;
    tern!(negative, low | !mask, low)
}

/// Whether a signed value fits `bits` bits.
///
/// # Arguments
///
/// * `v` - The value.
/// * `bits` - The bit count.
///
/// # Returns
///
/// `true` if it fits.
fn signed_fits(v: i128, bits: u16) -> bool
{
    ret_if!(bits >= 128, true);
    let half = 1i128 << (bits - 1);
    -half <= v && v < half
}

/// Whether an unsigned value fits `bits` bits.
///
/// # Arguments
///
/// * `v` - The value.
/// * `bits` - The bit count.
///
/// # Returns
///
/// `true` if it fits.
fn unsigned_fits(v: u128, bits: u16) -> bool { bits >= 128 || v >> bits == 0 }

/// The smallest value of a signed width.
///
/// # Arguments
///
/// * `bits` - The bit count.
///
/// # Returns
///
/// The value.
fn signed_min(bits: u16) -> i128
{
    tern!(bits >= 128, i128::MIN, -(1i128 << (bits - 1)))
}

/// Work out a prefix operator to a constant.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `v` - The operand.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The value, or the reason there is none.
pub fn prefix(op: PrefixOp, v: Value, platform: u16) -> Result<Value, EvalError>
{
    match (op, v)
    {
        (PrefixOp::Neg, Value::Int { kind, raw }) => int_neg(kind, raw, platform),
        (PrefixOp::Neg, Value::Float { bits, raw }) =>
        {
            let raw = per_width!(bits, F => (-F::from_bits(raw)).to_bits());
            Ok(Value::Float { bits, raw })
        }
        (PrefixOp::BitNot, Value::Int { kind, raw }) =>
        {
            Ok(Value::int(kind, !raw, platform))
        }
        (PrefixOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
        _ => Err(EvalError::Mismatch),
    }
}

/// Negate an integer.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `raw` - The bits.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The value, or an overflow.
fn int_neg(kind: IntKind, raw: u128, platform: u16) -> Result<Value, EvalError>
{
    ret_if!(
        !kind.signed,
        tern!(raw == 0, Ok(Value::Int { kind, raw }), Err(EvalError::Overflow))
    );
    let v = (raw as i128).checked_neg().ok_or(EvalError::Overflow)?;
    ret_if!(!signed_fits(v, kind.bits(platform)), Err(EvalError::Overflow));
    Ok(Value::Int { kind, raw: v as u128 })
}

/// Work out an arithmetic, bitwise or shift operator to constants.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `l` - The left operand.
/// * `r` - The right operand.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The value, or the reason there is none.
pub fn infix(op: InfixOp, l: Value, r: Value, platform: u16)
-> Result<Value, EvalError>
{
    let shift = matches!(op, InfixOp::Shl | InfixOp::Shr);
    ret_if!(shift, shift_int(op, l, r, platform));
    match (l, r)
    {
        (Value::Int { kind, raw: a }, Value::Int { kind: k, raw: b }) if kind == k =>
        {
            let raw = tern!(
                kind.signed,
                signed_op(op, a as i128, b as i128, kind.bits(platform))?,
                unsigned_op(op, a, b, kind.bits(platform))?
            );
            Ok(Value::Int { kind, raw })
        }
        (Value::Float { bits, raw: a }, Value::Float { bits: b_bits, raw: b })
            if bits == b_bits =>
        {
            let raw = per_width!(bits, F => float_op::<F>(op, a, b)?);
            Ok(Value::Float { bits, raw })
        }
        _ => Err(EvalError::Mismatch),
    }
}

/// Work out a prefix operator at run time: integer negation wraps to the
/// width of the kind instead of reporting an overflow.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `v` - The operand.
/// * `platform` - The bit count of the platform-sized kinds.
///
/// # Returns
///
/// The value, or the reason there is none.
pub fn prefix_wrapping(
    op: PrefixOp, v: Value, platform: u16,
) -> Result<Value, EvalError>
{
    match (op, v)
    {
        (PrefixOp::Neg, Value::Int { kind, raw }) =>
        {
            Ok(Value::int(kind, raw.wrapping_neg(), platform))
        }
        _ => prefix(op, v, platform),
    }
}

/// Work out an arithmetic, bitwise or shift operator at run time. Integer
/// results wrap to the width of the kind; a zero divisor and a shift
/// distance not below the width are still errors.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `l` - The left operand.
/// * `r` - The right operand.
/// * `platform` - The bit count of the platform-sized kinds.
///
/// # Returns
///
/// The value, or the reason there is none.
pub fn infix_wrapping(
    op: InfixOp, l: Value, r: Value, platform: u16,
) -> Result<Value, EvalError>
{
    let shift = matches!(op, InfixOp::Shl | InfixOp::Shr);
    ret_if!(shift, shift_int(op, l, r, platform));
    let (Value::Int { kind, raw: a }, Value::Int { kind: k, raw: b }) = (l, r)
    else
    {
        return infix(op, l, r, platform);
    };
    ret_if!(kind != k, Err(EvalError::Mismatch));
    let divides = matches!(op, InfixOp::Div | InfixOp::Rem);
    ret_if!(divides && b == 0, Err(EvalError::ZeroDivisor));
    let raw = tern!(
        kind.signed,
        signed_wrapping(op, a as i128, b as i128) as u128,
        unsigned_wrapping(op, a, b)
    );
    Ok(Value::int(kind, raw, platform))
}

/// Work out a wrapping arithmetic or bitwise operator to 128-bit signed
/// integers. The divisor is not zero.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `a` - The left operand.
/// * `b` - The right operand.
///
/// # Returns
///
/// The result, to be cut to the width of the kind.
fn signed_wrapping(op: InfixOp, a: i128, b: i128) -> i128
{
    match op
    {
        InfixOp::Add => a.wrapping_add(b),
        InfixOp::Sub => a.wrapping_sub(b),
        InfixOp::Mul => a.wrapping_mul(b),
        InfixOp::Div => a.wrapping_div(b),
        InfixOp::Rem => a.wrapping_rem(b),
        InfixOp::BitAnd => a & b,
        InfixOp::BitOr => a | b,
        InfixOp::BitXor => a ^ b,
        InfixOp::Shl | InfixOp::Shr => a,
    }
}

/// Work out a wrapping arithmetic or bitwise operator to 128-bit unsigned
/// integers. The divisor is not zero.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `a` - The left operand.
/// * `b` - The right operand.
///
/// # Returns
///
/// The result, to be cut to the width of the kind.
fn unsigned_wrapping(op: InfixOp, a: u128, b: u128) -> u128
{
    match op
    {
        InfixOp::Add => a.wrapping_add(b),
        InfixOp::Sub => a.wrapping_sub(b),
        InfixOp::Mul => a.wrapping_mul(b),
        InfixOp::Div => a / b,
        InfixOp::Rem => a % b,
        InfixOp::BitAnd => a & b,
        InfixOp::BitOr => a | b,
        InfixOp::BitXor => a ^ b,
        InfixOp::Shl | InfixOp::Shr => a,
    }
}

/// Work out an arithmetic or bitwise operator to signed integers.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `a` - The left value.
/// * `b` - The right value.
/// * `bits` - The bit width.
///
/// # Returns
///
/// The bits of the result, or the reason there is none.
fn signed_op(op: InfixOp, a: i128, b: i128, bits: u16) -> Result<u128, EvalError>
{
    let divides = matches!(op, InfixOp::Div | InfixOp::Rem);
    ret_if!(divides && b == 0, Err(EvalError::ZeroDivisor));
    ret_if!(divides && b == -1 && a == signed_min(bits), Err(EvalError::Overflow));
    let v = match op
    {
        InfixOp::Add => a.checked_add(b),
        InfixOp::Sub => a.checked_sub(b),
        InfixOp::Mul => a.checked_mul(b),
        InfixOp::Div => a.checked_div(b),
        InfixOp::Rem => a.checked_rem(b),
        InfixOp::BitAnd => Some(a & b),
        InfixOp::BitOr => Some(a | b),
        InfixOp::BitXor => Some(a ^ b),
        InfixOp::Shl | InfixOp::Shr => None,
    };
    let v = v.filter(|v| signed_fits(*v, bits)).ok_or(EvalError::Overflow)?;
    Ok(v as u128)
}

/// Work out an arithmetic or bitwise operator to unsigned integers.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `a` - The left value.
/// * `b` - The right value.
/// * `bits` - The bit width.
///
/// # Returns
///
/// The bits of the result, or the reason there is none.
fn unsigned_op(op: InfixOp, a: u128, b: u128, bits: u16) -> Result<u128, EvalError>
{
    let divides = matches!(op, InfixOp::Div | InfixOp::Rem);
    ret_if!(divides && b == 0, Err(EvalError::ZeroDivisor));
    let v = match op
    {
        InfixOp::Add => a.checked_add(b),
        InfixOp::Sub => a.checked_sub(b),
        InfixOp::Mul => a.checked_mul(b),
        InfixOp::Div => a.checked_div(b),
        InfixOp::Rem => a.checked_rem(b),
        InfixOp::BitAnd => Some(a & b),
        InfixOp::BitOr => Some(a | b),
        InfixOp::BitXor => Some(a ^ b),
        InfixOp::Shl | InfixOp::Shr => None,
    };
    v.filter(|v| unsigned_fits(*v, bits)).ok_or(EvalError::Overflow)
}

/// Shift an integer. Bits shifted out of the width are dropped.
///
/// # Arguments
///
/// * `op` - `Shl` or `Shr`.
/// * `l` - The shifted value.
/// * `r` - The distance, an unsigned integer.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The value, or the reason there is none.
fn shift_int(op: InfixOp, l: Value, r: Value, platform: u16)
-> Result<Value, EvalError>
{
    let (Value::Int { kind, raw }, Value::Int { raw: distance, .. }) = (l, r)
    else
    {
        return Err(EvalError::Mismatch);
    };
    let bits = kind.bits(platform);
    ret_if!(
        distance >= u128::from(bits),
        Err(EvalError::ShiftTooFar { distance, bits })
    );
    let d = distance as u32;
    let right = tern!(kind.signed, ((raw as i128) >> d) as u128, raw >> d);
    let moved = tern!(op == InfixOp::Shl, raw << d, right);
    Ok(Value::int(kind, moved, platform))
}

/// Work out an arithmetic operator to floats of format `F`.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `a` - The left bit pattern.
/// * `b` - The right bit pattern.
///
/// # Returns
///
/// The bit pattern of the result, rounded to nearest.
fn float_op<F: Float>(op: InfixOp, a: u128, b: u128) -> Result<u128, EvalError>
{
    let (a, b) = (F::from_bits(a), F::from_bits(b));
    let v = match op
    {
        InfixOp::Add => a.add_r(b, NEAREST).value,
        InfixOp::Sub => a.sub_r(b, NEAREST).value,
        InfixOp::Mul => a.mul_r(b, NEAREST).value,
        InfixOp::Div => a.div_r(b, NEAREST).value,
        InfixOp::Rem => a.c_fmod(b).value,
        _ => return Err(EvalError::Mismatch),
    };
    Ok(v.to_bits())
}

/// Compare two constants of one kind.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `l` - The left operand.
/// * `r` - The right operand.
///
/// # Returns
///
/// The result; a NaN operand is unordered, so `!=` alone holds. Addresses
/// compare as unsigned numbers.
pub fn compare(op: CmpOp, l: Value, r: Value) -> Result<bool, EvalError>
{
    ret_if!(l.kind() != r.kind(), Err(EvalError::Mismatch));
    let order = match (l, r)
    {
        (Value::Int { kind, raw: a }, Value::Int { raw: b, .. }) =>
        {
            Some(tern!(kind.signed, (a as i128).cmp(&(b as i128)), a.cmp(&b)))
        }
        (Value::Float { bits, raw: a }, Value::Float { raw: b, .. }) =>
        {
            per_width!(bits, F => F::from_bits(a).partial_cmp(&F::from_bits(b)))
        }
        (Value::Char(a), Value::Char(b)) => Some(a.cmp(&b)),
        (Value::Bool(a), Value::Bool(b)) => Some(a.cmp(&b)),
        (Value::Ptr(a), Value::Ptr(b)) => Some(a.cmp(&b)),
        _ => None,
    };
    Ok(holds(op, order))
}

/// Whether a comparison holds for an ordering.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `order` - The ordering, `None` when unordered.
///
/// # Returns
///
/// The result.
fn holds(op: CmpOp, order: Option<Ordering>) -> bool
{
    let Some(order) = order
    else
    {
        return op == CmpOp::Ne;
    };
    match op
    {
        CmpOp::Eq => order.is_eq(),
        CmpOp::Ne => order.is_ne(),
        CmpOp::Lt => order.is_lt(),
        CmpOp::Gt => order.is_gt(),
        CmpOp::Le => order.is_le(),
        CmpOp::Ge => order.is_ge(),
    }
}

/// Work out a logical operator to two boolean constants.
///
/// # Arguments
///
/// * `op` - The operator.
/// * `l` - The left operand.
/// * `r` - The right operand.
///
/// # Returns
///
/// The result.
pub fn logic(op: LogicOp, l: Value, r: Value) -> Result<bool, EvalError>
{
    let (Some(a), Some(b)) = (l.as_bool(), r.as_bool())
    else
    {
        return Err(EvalError::Mismatch);
    };
    Ok(tern!(op == LogicOp::And, a && b, a || b))
}

/// Convert a constant to another kind. Integers wrap to the target width;
/// floats round to nearest; a float loses its fraction to become an
/// integer, saturating at the range ends, with NaN as zero; an integer that
/// is no Unicode scalar value becomes U+FFFD. An address becomes an integer
/// wrapped to the target width; an integer becomes the address of its low
/// `platform` bits; an address keeps its value under each pointer kind.
///
/// # Arguments
///
/// * `v` - The value.
/// * `to` - The target kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The converted value.
pub fn convert(v: Value, to: DataKind, platform: u16) -> Result<Value, EvalError>
{
    ret_if!(v.kind() == to, Ok(v));
    match (v, to)
    {
        (Value::Int { raw, .. }, DataKind::Int(k)) => Ok(Value::int(k, raw, platform)),
        (Value::Int { kind, raw }, DataKind::Float(bits)) =>
        {
            let raw = per_width!(bits, F => int_to_float::<F>(kind.signed, raw));
            Ok(Value::Float { bits, raw })
        }
        (Value::Float { bits, raw }, DataKind::Float(to_bits)) =>
        {
            let raw = per_width!(bits, F => widen_or_narrow::<F>(to_bits, raw));
            Ok(Value::Float { bits: to_bits, raw })
        }
        (Value::Float { bits, raw }, DataKind::Int(k)) =>
        {
            let width = usize::from(k.bits(platform));
            let raw = per_width!(bits, F => float_to_int::<F>(raw, k.signed, width));
            Ok(Value::Int { kind: k, raw })
        }
        (Value::Char(c), DataKind::Int(k)) =>
        {
            Ok(Value::int(k, u128::from(c), platform))
        }
        (Value::Int { kind, raw }, DataKind::Char) =>
        {
            Ok(Value::Char(int_to_char(kind, raw)))
        }
        (Value::Bool(b), DataKind::Int(k)) =>
        {
            Ok(Value::Int { kind: k, raw: u128::from(b) })
        }
        (Value::Ptr(a), DataKind::Int(k)) => Ok(Value::int(k, u128::from(a), platform)),
        (Value::Int { raw, .. }, DataKind::Ptr(_) | DataKind::Null) =>
        {
            Ok(Value::Ptr(address(raw, platform)))
        }
        (Value::Ptr(_), DataKind::Ptr(_) | DataKind::Null) => Ok(v),
        _ => Err(EvalError::Mismatch),
    }
}

/// The address an integer bit pattern names: its low `platform` bits.
///
/// # Arguments
///
/// * `pattern` - The two's-complement bits.
/// * `platform` - Bit count of an address.
///
/// # Returns
///
/// The address.
fn address(pattern: u128, platform: u16) -> u64
{
    let low = pattern as u64;
    ret_if!(platform >= 64, low);
    low & ((1u64 << platform) - 1)
}

/// Encode a value as little-endian octets.
///
/// # Arguments
///
/// * `v` - The value.
/// * `platform` - Bit count of the platform-sized kinds and addresses.
///
/// # Returns
///
/// The octets, one per 8 bits of the value's kind: a character as its
/// `u32` scalar value, a boolean as one octet 0 or 1, an address as
/// `platform / 8` octets.
pub fn to_raw(v: Value, platform: u16) -> Vec<u8>
{
    let len = v.kind().size(platform) as usize;
    let all = v.pattern().to_le_bytes();
    all.get(..len).unwrap_or(&all).to_vec()
}

/// Decode little-endian octets as a value of a kind.
///
/// # Arguments
///
/// * `kind` - The kind; `null` reads like a pointer.
/// * `raw` - The octets, one per 8 bits of the kind.
/// * `platform` - Bit count of the platform-sized kinds and addresses.
///
/// # Returns
///
/// The value, an integer sign or zero extended per its kind, or the reason
/// the octets hold none.
pub fn from_raw(kind: DataKind, raw: &[u8], platform: u16) -> Result<Value, RawError>
{
    let len = kind.size(platform) as usize;
    ret_if!(raw.len() != len || len > 16, Err(RawError::WrongLen));
    let mut wide = [0u8; 16];
    wide[..len].copy_from_slice(raw);
    let pattern = u128::from_le_bytes(wide);
    match kind
    {
        DataKind::Int(k) => Ok(Value::int(k, pattern, platform)),
        DataKind::Float(bits) => Ok(Value::Float { bits, raw: pattern }),
        DataKind::Char => u32::try_from(pattern)
            .ok()
            .and_then(char::from_u32)
            .map(Value::Char)
            .ok_or(RawError::BadValue),
        DataKind::Bool =>
        {
            tern!(pattern <= 1, Ok(Value::Bool(pattern == 1)), Err(RawError::BadValue))
        }
        DataKind::Ptr(_) | DataKind::Null => Ok(Value::Ptr(pattern as u64)),
        DataKind::List(_) | DataKind::Struct(_) | DataKind::Slice(_) =>
        {
            Err(RawError::WrongLen)
        }
    }
}

/// Convert an integer to a float of format `F`.
///
/// # Arguments
///
/// * `signed` - Whether the bits are signed.
/// * `raw` - The bits.
///
/// # Returns
///
/// The bit pattern, rounded to nearest.
fn int_to_float<F: Float>(signed: bool, raw: u128) -> u128
{
    let v = tern!(
        signed,
        F::from_i128_r(raw as i128, NEAREST).value,
        F::from_u128_r(raw, NEAREST).value
    );
    v.to_bits()
}

/// Convert a float of format `F` to the format of `bits` bits.
///
/// # Arguments
///
/// * `bits` - The bit count of the target format.
/// * `raw` - The bit pattern.
///
/// # Returns
///
/// The bit pattern, rounded to nearest.
fn widen_or_narrow<F>(bits: u16, raw: u128) -> u128
where
    F: FloatConvert<Half> + FloatConvert<Single> + FloatConvert<Double>,
    F: FloatConvert<Quad>,
{
    per_width!(bits, G => float_to_float::<F, G>(raw))
}

/// Convert a float of format `F` to format `G`.
///
/// # Arguments
///
/// * `raw` - The bit pattern.
///
/// # Returns
///
/// The bit pattern, rounded to nearest.
fn float_to_float<F: FloatConvert<G>, G: Float>(raw: u128) -> u128
{
    let mut loses = false;
    let v: G = F::from_bits(raw).convert_r(NEAREST, &mut loses).value;
    v.to_bits()
}

/// Convert a float of format `F` to an integer: the fraction is dropped,
/// values past the range saturate, NaN gives zero.
///
/// # Arguments
///
/// * `raw` - The bit pattern.
/// * `signed` - Whether the target is signed.
/// * `width` - The bit width of the target.
///
/// # Returns
///
/// The bits of the integer.
fn float_to_int<F: Float>(raw: u128, signed: bool, width: usize) -> u128
{
    let v = F::from_bits(raw);
    ret_if!(v.is_nan(), 0);
    tern!(signed, v.to_i128(width).value as u128, v.to_u128(width).value)
}

/// The character with an integer's value as its scalar value.
///
/// # Arguments
///
/// * `kind` - The integer kind.
/// * `raw` - The bits.
///
/// # Returns
///
/// The character, or U+FFFD when the value is no scalar value.
fn int_to_char(kind: IntKind, raw: u128) -> char
{
    let negative = kind.signed && (raw as i128) < 0;
    let scalar = u32::try_from(raw).ok().filter(|_| !negative);
    scalar.and_then(char::from_u32).unwrap_or(char::REPLACEMENT_CHARACTER)
}

/// Read float text in format `F`.
///
/// # Arguments
///
/// * `text` - Decimal text, `inf`, `-inf` or `NaN`.
///
/// # Returns
///
/// The value, or `None` for text that is no number.
fn parse_float<F: Float>(text: &str) -> Option<F>
{
    match text
    {
        "inf" => Some(F::INFINITY),
        "-inf" => Some(-F::INFINITY),
        "NaN" => Some(F::NAN),
        _ => F::from_str_r(text, NEAREST).ok().map(|v| v.value),
    }
}

/// Write a float in format `F` as text that reads back to the same value,
/// as Rust writes `f64`: `3.0`, `0.1`, `1e300`.
///
/// # Arguments
///
/// * `v` - The value.
///
/// # Returns
///
/// The text, `inf`, `-inf` or `NaN`.
fn float_text<F: Float>(v: F) -> String
{
    ret_if!(v.is_nan(), "NaN".to_string());
    ret_if!(v.is_infinite(), tern!(v.is_negative(), "-inf", "inf").to_string());
    let text = v.to_string().replace("E+", "e").replace('E', "e").replace(".0e", "e");
    let whole = text.chars().all(|c| c.is_ascii_digit() || c == '-');
    tern!(whole, format!("{text}.0"), text)
}

/// Write a float pattern as text. The 32- and 64-bit formats use the
/// shortest text that reads back; the others use enough digits to do so.
///
/// # Arguments
///
/// * `bits` - The bit count of the format.
/// * `raw` - The bit pattern.
///
/// # Returns
///
/// The text.
pub fn float_pattern_text(bits: u16, raw: u128) -> String
{
    match bits
    {
        32 => format!("{:?}", f32::from_bits(raw as u32)),
        64 => format!("{:?}", f64::from_bits(raw as u64)),
        _ => per_width!(bits, F => float_text(F::from_bits(raw))),
    }
}

impl fmt::Display for Value
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let kind = self.kind().name();
        match *self
        {
            Value::Int { kind: k, raw } if k.signed =>
            {
                write!(f, "{}:{kind}", raw as i128)
            }
            Value::Int { raw, .. } => write!(f, "{raw}:{kind}"),
            Value::Float { bits, raw } =>
            {
                write!(f, "{}:{kind}", float_pattern_text(bits, raw))
            }
            Value::Char(c) => write!(f, "{c:?}:{kind}"),
            Value::Bool(b) => write!(f, "{b}:{kind}"),
            Value::Ptr(a) => write!(f, "{a:#x}:{kind}"),
        }
    }
}

impl FromStr for Value
{
    type Err = String;

    /// Read the text form, such as `-5:i8` or `0x10:*i32`.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    ///
    /// # Returns
    ///
    /// The value, or an error for text that is no value.
    fn from_str(text: &str) -> Result<Self, Self::Err>
    {
        let bad = || format!("unknown constant {text}");
        let (head, kind) = text.rsplit_once(':').ok_or_else(bad)?;
        let kind = DataKind::parse(kind).ok_or_else(bad)?;
        let value = match kind
        {
            DataKind::Int(k) if k.signed => head
                .parse::<i128>()
                .ok()
                .map(|v| Value::Int { kind: k, raw: v as u128 }),
            DataKind::Int(k) =>
            {
                head.parse::<u128>().ok().map(|raw| Value::Int { kind: k, raw })
            }
            DataKind::Float(bits) => Value::float(head, bits),
            DataKind::Char => read_char(head).map(Value::Char),
            DataKind::Bool => head.parse::<bool>().ok().map(Value::Bool),
            DataKind::Ptr(_) | DataKind::Null => read_address(head).map(Value::Ptr),
            DataKind::List(_) | DataKind::Struct(_) | DataKind::Slice(_) => None,
        };
        value.ok_or_else(bad)
    }
}

/// Read a character written as Rust writes a debug `char`, quotes
/// included.
///
/// # Arguments
///
/// * `text` - The text, such as `'က'`, `'\n'` or `'\u{301}'`.
///
/// # Returns
///
/// The character, or `None`.
fn read_char(text: &str) -> Option<char>
{
    let inner = text.strip_prefix('\'')?.strip_suffix('\'')?;
    let mut chars = inner.chars();
    let first = chars.next()?;
    ret_if!(first != '\\', tern!(chars.next().is_none(), Some(first), None));
    let rest: String = chars.collect();
    match rest.as_str()
    {
        "n" => Some('\n'),
        "r" => Some('\r'),
        "t" => Some('\t'),
        "0" => Some('\0'),
        "\\" => Some('\\'),
        "'" => Some('\''),
        "\"" => Some('"'),
        _ => read_unicode_escape(&rest),
    }
}

/// Read an address written as `0x` and hex digits.
///
/// # Arguments
///
/// * `text` - The text, such as `0x1000`.
///
/// # Returns
///
/// The address, or `None`.
fn read_address(text: &str) -> Option<u64>
{
    let hex = text.strip_prefix("0x")?;
    ret_if!(!hex.chars().all(|c| c.is_ascii_hexdigit()), None);
    u64::from_str_radix(hex, 16).ok()
}

/// Read the `u{HHHH}` part of a `\u{HHHH}` escape.
///
/// # Arguments
///
/// * `text` - The text after the backslash.
///
/// # Returns
///
/// The character, or `None`.
fn read_unicode_escape(text: &str) -> Option<char>
{
    let hex = text.strip_prefix("u{")?.strip_suffix('}')?;
    u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
}

impl From<Value> for String
{
    fn from(v: Value) -> String { v.to_string() }
}

impl TryFrom<String> for Value
{
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> { text.parse() }
}
