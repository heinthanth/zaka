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

/// Whether a value of the kind can be a parameter of a C function: an
/// integer up to 64 bits, an `f32` or `f64`, a boolean, a pointer whose outer
/// level borrows or is raw, or a struct whose fields are such kinds, nested
/// structs or counted lists of them. An owner would be freed when the call
/// ends, and C has no kind for text, plain lists, slices or function values.
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
/// [`crosses`] takes, or text, which C takes as the address of a copy that
/// ends in a zero octet.
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
    kind.is_textual() || crosses(kind, platform)
}

/// Whether a value of the kind can be one of the other values of a C
/// function, which C reads through its `...`: a parameter kind but a
/// struct.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `platform` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// `true` if it can.
pub fn crosses_rest(kind: DataKind, platform: u16) -> bool
{
    !kind.is_struct() && crosses_param(kind, platform)
}

/// Whether a value of the kind can be the result of a C function: a kind
/// [`crosses`] takes, but no borrow, since C gives no place to lend from.
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
    !kind.is_borrow() && crosses(kind, platform)
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
    }
}
