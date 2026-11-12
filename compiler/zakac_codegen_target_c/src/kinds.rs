use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::IntKind;
use zakac_semcheck::value::Value;

/// The C kind of a scalar data kind, or `None` for a kind the C target
/// cannot hold at present.
///
/// # Arguments
///
/// * `kind` - The data kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The C kind name.
pub fn c_kind(kind: DataKind, platform: u16) -> Option<String>
{
    match kind.repr()
    {
        DataKind::Int(k) => Some(int_name(k, platform)),
        DataKind::Float(16) => Some("zk_half".into()),
        DataKind::Float(32) => Some("float".into()),
        DataKind::Float(64) => Some("double".into()),
        DataKind::Float(128) => Some("zk_quad".into()),
        DataKind::Char => Some("uint32_t".into()),
        DataKind::Bool => Some("uint8_t".into()),
        DataKind::Ptr(_) | DataKind::Null => Some("unsigned char *".into()),
        DataKind::Fn(_) => Some("zk_fn".into()),
        _ => None,
    }
}

/// Whether a kind is held as a block of octets: a list, a struct, a slice,
/// a plain list or text. Its C value is the address of its first octet.
///
/// # Arguments
///
/// * `kind` - The data kind.
///
/// # Returns
///
/// `true` for a block.
pub fn is_blob(kind: DataKind) -> bool { kind.repr().is_aggregate() }

/// The name of a scalar kind in the names of `zk_ld_<N>` and `zk_st_<N>`,
/// the load and the store through an address.
///
/// # Arguments
///
/// * `kind` - The scalar kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The name, or `None` for a block or a kind the C target cannot hold.
pub fn mem_name(kind: DataKind, platform: u16) -> Option<String>
{
    match kind.repr()
    {
        k @ DataKind::Int(_) => ops_name(k, platform),
        DataKind::Float(bits) => Some(format!("f{bits}")),
        DataKind::Char => Some("u32".into()),
        DataKind::Bool => Some("u8".into()),
        DataKind::Ptr(_) | DataKind::Null => Some("p".into()),
        DataKind::Fn(_) => Some("fn".into()),
        _ => None,
    }
}

/// The C kind of an integer kind.
///
/// # Arguments
///
/// * `k` - The integer kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The C kind name, such as `int32_t` or `zk_u128`.
fn int_name(k: IntKind, platform: u16) -> String
{
    let bits = k.bits(platform);
    match (k.signed, bits)
    {
        (true, 128) => "zk_i128".into(),
        (false, 128) => "zk_u128".into(),
        (true, n) => format!("int{n}_t"),
        (false, n) => format!("uint{n}_t"),
    }
}

/// The name of an integer kind in the names of the runtime functions of its
/// operations, such as `i32` in `zk_add_i32`.
///
/// # Arguments
///
/// * `kind` - The integer kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The name, or `None` for a kind that is no integer.
pub fn ops_name(kind: DataKind, platform: u16) -> Option<String>
{
    let k = kind.repr().int_kind()?;
    Some(format!("{}{}", if k.signed { 'i' } else { 'u' }, k.bits(platform)))
}

/// The unsigned C kind of the same width as an integer kind.
///
/// # Arguments
///
/// * `kind` - The integer kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The C kind name, or `None` for a kind that is no integer.
pub fn same_unsigned(kind: DataKind, platform: u16) -> Option<String>
{
    let k = kind.repr().int_kind()?;
    Some(int_name(IntKind::new(false, k.width), platform))
}

/// A bit pattern as C text of a kind: the pattern is read as an unsigned
/// number, then converted, so the sign bit comes out as written.
///
/// # Arguments
///
/// * `t` - The C kind name.
/// * `raw` - The bits.
///
/// # Returns
///
/// The C text.
fn pattern(t: &str, raw: u128) -> String
{
    let (hi, lo) = ((raw >> 64) as u64, raw as u64);
    match hi
    {
        0 => format!("(({t})UINT64_C({lo:#x}))"),
        _ => format!("(({t})(((zk_u128)UINT64_C({hi:#x}) << 64) | UINT64_C({lo:#x})))"),
    }
}

/// A constant as C text.
///
/// # Arguments
///
/// * `v` - The constant.
/// * `kind` - The kind of the expression that holds it.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The C text, or `None` for a constant the C target cannot write at
/// present.
pub fn constant(v: Value, kind: DataKind, platform: u16) -> Option<String>
{
    let t = c_kind(kind, platform)?;
    let small = |n: i128| (-0x7fff_ffff..=0x7fff_ffff).contains(&n);
    match v
    {
        Value::Int { kind: k, raw } if k.signed && small(raw as i128) =>
        {
            Some(format!("(({t}){})", raw as i128))
        }
        Value::Int { raw, .. } if raw <= 0x7fff_ffff => Some(format!("(({t}){raw})")),
        Value::Int { kind: k, raw } =>
        {
            let bits = k.bits(platform);
            let mask = u128::MAX >> (128 - bits);
            Some(pattern(&t, raw & mask))
        }
        Value::Float { bits: 32, raw } =>
        {
            Some(float_text(f64::from(f32::from_bits(raw as u32)), "f", &t))
        }
        Value::Float { bits: 64, raw } =>
        {
            Some(float_text(f64::from_bits(raw as u64), "", &t))
        }
        // C11 has no literal of these kinds, so the value comes from its bits.
        Value::Float { bits: 16, raw } =>
        {
            Some(format!("zk_f16_of(UINT16_C({raw:#x}))"))
        }
        Value::Float { bits: 128, raw } =>
        {
            let (hi, lo) = ((raw >> 64) as u64, raw as u64);
            Some(format!("zk_f128_of(UINT64_C({hi:#x}), UINT64_C({lo:#x}))"))
        }
        Value::Char(c) => Some(format!("(({t}){:#x})", c as u32)),
        Value::Bool(b) => Some(format!("(({t}){})", u8::from(b))),
        Value::Ptr(0) => Some(format!("(({t})0)")),
        // A function value folds to the index of its function plus one, as
        // the VM holds it; C holds the address of its record.
        Value::Ptr(a) if kind.is_fn() => Some(format!("((zk_fn)&zk_r{})", a - 1)),
        Value::Ptr(a) => Some(format!("(({t})(uintptr_t){a:#x})")),
        _ => None,
    }
}

/// A float as C text that reads back to the same value.
///
/// # Arguments
///
/// * `v` - The value, widened to `f64` when it is an `f32`.
/// * `suffix` - `f` for an `f32` literal.
/// * `t` - The C kind name.
///
/// # Returns
///
/// The C text.
fn float_text(v: f64, suffix: &str, t: &str) -> String
{
    match v
    {
        _ if v.is_nan() => format!("(({t})NAN)"),
        f64::INFINITY => format!("(({t})INFINITY)"),
        f64::NEG_INFINITY => format!("(({t})-INFINITY)"),
        _ if suffix.is_empty() => format!("(({t}){v:?})"),
        _ => format!("(({t}){:?}{suffix})", v as f32),
    }
}
