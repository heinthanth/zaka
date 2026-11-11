//! The check's side of C functions, `"<lib>" မှလာသော`: the name is the C
//! name, and C takes each parameter and gives the result as a whole value.

pub use zakac_ast::stmt::is_lib_path;
use zakac_rust_sugars::ret_if;

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
/// bits, an `f32` or `f64`, a boolean, a pointer, or a struct whose fields
/// are such kinds, nested structs or counted lists of them. C has no kind for
/// text or plain lists.
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
        DataKind::Ptr(_) => true,
        // A result and an optional are cells of Zaka, which C has no form of.
        DataKind::Struct(_) if kind.fallible_parts().is_some() => false,
        DataKind::Struct(s) => s.fields().iter().all(|(_, k)| in_struct(*k, platform)),
        _ => false,
    }
}

/// Whether a value of the kind can be a parameter of a C function: a kind
/// [`crosses`] takes; text, which C takes as the address of a duplicate that
/// ends in a zero octet; a list of numbers, which C takes as the address of
/// its first cell; or a function value C can call back, see
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
    let numbers = number_cells(kind, platform).is_some();
    let handle = kind.as_external().is_some();
    kind.is_text()
        || numbers
        || handle
        || crosses(kind, platform)
        || is_callback(kind, platform)
}

/// The cell kind of a list of numbers that C takes as the address of its
/// first cell: a plain or counted list whose cells are integers up to 64
/// bits, `f32` or `f64`.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `platform` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// The cell kind, or `None` for another kind.
pub fn number_cells(kind: DataKind, platform: u16) -> Option<DataKind>
{
    let seq = kind.as_seq().filter(|s| !s.boxed).map(|s| s.cell);
    let cell = seq.or_else(|| kind.as_list().map(|l| l.cell))?;
    let number = matches!(cell, DataKind::Int(_) | DataKind::Float(_));
    (number && crosses(cell, platform)).then_some(cell)
}

/// Whether C can write a parameter of the kind back, for `ဒေတာ ပြောင်းလဲရန်`: a
/// list of numbers, which C takes as the address of its first cell, or a struct
/// that crosses, which C takes as its address (`T *`).
///
/// # Arguments
///
/// * `kind` - The parameter kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// `true` if it can.
pub fn edit_crosses(kind: DataKind, platform: u16) -> bool
{
    number_cells(kind, platform).is_some()
        || (kind.is_struct() && crosses(kind, platform))
}

/// Whether a function value of the kind can go to C as a callback, the
/// address C calls: each parameter is one [`callback_param`] takes, with no
/// `ဒေတာ ပြောင်းလဲရန်`, and the result is a number, a boolean or an address, no
/// borrow.
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
    let params = f.params.iter().all(|p| !p.edit && callback_param(p.kind, platform));
    params && f.ret.is_none_or(|k| !k.is_borrow() && scalar(k))
}

/// Whether a parameter of a function value can take what C passes when it
/// calls the value back: a number, a boolean or an address as it is; a
/// plain struct as a whole; a `char *` as text, or as an optional of text,
/// where NULL holds no data; and the address of a struct C keeps as a
/// handle, or as an optional of one.
///
/// # Arguments
///
/// * `kind` - The parameter kind.
/// * `platform` - Bit count of the platform-sized integer kinds.
///
/// # Returns
///
/// `true` if it can.
pub fn callback_param(kind: DataKind, platform: u16) -> bool
{
    let pointer = kind.is_text() || kind.as_external().is_some();
    let optional = is_c_text_result(kind) || is_c_handle_result(kind);
    pointer || optional || (!kind.is_borrow() && crosses(kind, platform))
}

/// Whether a value of the kind can be the result of a C function: a kind
/// [`crosses`] takes, but no borrow, since C gives no place to lend from;
/// a handle; or an optional of text or of a handle, which C gives as an
/// address or NULL.
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
    let handle = kind.as_external().is_some() || is_c_handle_result(kind);
    is_c_text_result(kind) || handle || (!kind.is_borrow() && crosses(kind, platform))
}

/// Whether a kind is an optional of a handle, `<kind> ရှိကောင်းရှိနိုင်သော
/// ဒေတာဆဲလ်` for a kind from outside: C gives an address, and NULL is the
/// optional with no data.
///
/// # Arguments
///
/// * `kind` - The result kind.
///
/// # Returns
///
/// `true` for that kind.
pub fn is_c_handle_result(kind: DataKind) -> bool
{
    kind.optional_data().is_some_and(|k| k.as_external().is_some())
}

/// Whether a kind is the old form of a C result that can be NULL, a result
/// of text or of a handle that fails with no value: ZK0465 names the
/// optional in its place.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` for that kind.
pub fn is_failing_pointer(kind: DataKind) -> bool
{
    let parts = kind.fallible_parts().filter(|_| !kind.is_optional());
    let pointer = |k: DataKind| k.is_text() || k.as_external().is_some();
    matches!(parts, Some((Some(ok), None)) if pointer(ok))
}

/// Whether a kind is an optional of text, `စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`:
/// the kind of a string C gives. NULL is the optional with no data; else Zaka
/// copies the octets into text of its own, and C keeps what it gave.
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
    kind.optional_data().is_some_and(DataKind::is_text)
}

/// Whether a value of the kind passes a pointer to or from C: a pointer, or
/// a struct with one in a field or a cell of a counted list.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` if it holds one.
pub fn holds_ptr(kind: DataKind) -> bool
{
    ret_if!(let Some(l) = kind.as_list(), holds_ptr(l.cell));
    ret_if!(let Some(s) = kind.as_struct(), s.fields().iter().any(|(_, k)| holds_ptr(*k)));
    kind.is_ptr()
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
    fn pointers_cross_and_text_remains_in_zaka()
    {
        assert!(crosses(DataKind::ISIZE, 64));
        assert!(crosses(DataKind::F64, 64));
        assert!(!crosses(DataKind::Float(16), 64));
        assert!(crosses(DataKind::OPAQUE, 64));
        assert!(crosses_back(DataKind::OPAQUE, 64));
        assert!(!crosses(DataKind::TEXT, 64));
        assert!(!crosses_back(DataKind::TEXT, 64));
        assert!(crosses_back(DataKind::optional(DataKind::TEXT), 64));
        // NULL from C is the optional, not a failure.
        assert!(!crosses_back(DataKind::fallible(Some(DataKind::TEXT), None), 64));
        let with_err = DataKind::fallible(Some(DataKind::TEXT), Some(DataKind::ISIZE));
        assert!(!crosses_back(with_err, 64));
    }
}
