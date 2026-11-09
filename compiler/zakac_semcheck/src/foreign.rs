//! The check's side of C functions, `"<lib>" မှလာသော`: the name is the C
//! name, and C takes each parameter and gives the result as a whole value.

pub use zakac_ast::stmt::is_lib_path;

use crate::kind::DataKind;

/// Whether a name can be a C name: an ASCII letter or `_`, then ASCII
/// letters, digits and `_`.
///
/// # Arguments
///
/// * `name` - The function name.
///
/// # Returns
///
/// `true` for a C identifier.
pub fn is_c_name(name: &str) -> bool
{
    let mut chars = name.chars();
    let lead = chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
    lead && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Whether a value of the kind can be a field of a struct that crosses to C
/// or a result of a C function, and is a parameter kind: an integer up to 64
/// bits, an `f32` or `f64`, a boolean, a pointer whose outer level borrows or
/// is raw, or a struct whose fields are such kinds, nested structs or counted
/// lists of them. An owner would be freed when the call ends, and C has no
/// kind for text, plain lists or slices.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `platform` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// `true` if it can.
pub fn crosses(kind: DataKind, platform: u16) -> bool
{
    match kind
    {
        DataKind::Int(n) => n.bits(platform) <= 64,
        DataKind::Float(bits) => matches!(bits, 32 | 64),
        DataKind::Bool => true,
        DataKind::Ptr(_) => !kind.is_owner_ptr(),
        DataKind::Struct(s) => s.fields().iter().all(|(_, k)| in_struct(*k, platform)),
        _ => false,
    }
}

/// Whether a value of the kind can be a parameter of a C function: a kind
/// [`crosses`] takes; text, which C takes as the address of a duplicate that
/// ends in a zero octet; or a function value C can call back, see
/// [`is_callback`].
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `platform` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// `true` if it can.
pub fn crosses_param(kind: DataKind, platform: u16) -> bool
{
    kind.is_textual() || crosses(kind, platform) || is_callback(kind, platform)
}

/// Whether a function value of the kind can go to C as a callback, the
/// address of a C function: each parameter and the result are scalars or
/// addresses C passes as C holds them, with no edit mark, and the result is no
/// borrow. A closure cannot go: C keeps no locals alive for it.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `platform` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// `true` if it can.
pub fn is_callback(kind: DataKind, platform: u16) -> bool
{
    let Some(f) = kind.as_fn()
    else
    {
        return false;
    };
    let scalar = |k: DataKind| !k.is_struct() && crosses(k, platform);
    let params = f.params.iter().all(|p| !p.edit && scalar(p.kind));
    params && f.ret.is_none_or(|k| !k.is_borrow() && scalar(k))
}

/// Whether a value of the kind can be the result of a C function: a kind
/// [`crosses`] takes, but no borrow, since C gives no place to lend from;
/// or text or a failure, which C gives as a string or NULL.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `platform` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// `true` if it can.
pub fn crosses_back(kind: DataKind, platform: u16) -> bool
{
    is_c_text_result(kind) || (!kind.is_borrow() && crosses(kind, platform))
}

/// Whether a result kind is text or a failure with no value,
/// `စာသား ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း`: the kind of a C function that gives
/// back a string. NULL is the failure; else Zaka copies the octets into text
/// of its own, and C keeps what it gave.
///
/// # Arguments
///
/// * `kind` - The result kind.
///
/// # Returns
///
/// `true` for that kind.
pub fn is_c_text_result(kind: DataKind) -> bool
{
    matches!(kind.fallible_parts(), Some((Some(ok), None)) if ok.is_text())
}

/// Whether a field of the kind can sit in a struct that crosses to C: a kind
/// [`crosses`] takes, or a counted list of one, which C holds as a run of
/// cells.
///
/// # Arguments
///
/// * `kind` - The kind of the field or cell.
/// * `platform` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// `true` if it can.
fn in_struct(kind: DataKind, platform: u16) -> bool
{
    kind.as_list()
        .map_or_else(|| crosses(kind, platform), |l| in_struct(l.cell, platform))
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn c_names_are_ascii_identifiers()
    {
        assert!(is_c_name("abs"));
        assert!(is_c_name("_exit2"));
        assert!(!is_c_name("2x"));
        assert!(!is_c_name("ရေး"));
        assert!(!is_c_name(""));
    }

    #[test]
    fn owners_and_text_remain_in_zaka()
    {
        assert!(crosses(DataKind::ISIZE, 64));
        assert!(crosses(DataKind::F64, 64));
        assert!(!crosses(DataKind::Float(16), 64));
        assert!(!crosses(DataKind::OPAQUE, 64));
        assert!(crosses(DataKind::OPAQUE.as_borrowed(), 64));
        assert!(!crosses_back(DataKind::OPAQUE.as_borrowed(), 64));
        assert!(!crosses(DataKind::TEXT, 64));
        assert!(!crosses_back(DataKind::TEXT, 64));
        assert!(crosses_back(DataKind::fallible(Some(DataKind::TEXT), None), 64));
        let with_err = DataKind::fallible(Some(DataKind::TEXT), Some(DataKind::ISIZE));
        assert!(!crosses_back(with_err, 64));
    }
}
