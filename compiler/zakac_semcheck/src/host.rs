//! The check's side of the functions the host serves: an external function
//! (`ပြင်ပမှလာသော`) must match a row of the table in `zaka_host`, with the same
//! name, parameter kinds and result kind.

pub use zaka_host::HostFn;
pub use zaka_host::IO_ERROR;
pub use zaka_host::listing;

use crate::kind::DataKind;

/// The host function an external declaration names.
///
/// # Arguments
///
/// * `name` - The declared name, or the inner name an overload takes after the
///   check, `<name>#<n>`.
/// * `params` - The declared parameter kinds.
/// * `ret` - The declared result kind, which must be one that can fail.
///
/// # Returns
///
/// The host function, or `None` when no row matches.
pub fn find(name: &str, params: &[DataKind], ret: Option<DataKind>) -> Option<HostFn>
{
    let name = name.split_once('#').map_or(name, |(written, _)| written);
    let (ok, err) = ret.and_then(DataKind::fallible_parts)?;
    let params: Vec<String> = params.iter().map(|k| k.name()).collect();
    let (ok, err) = (ok.map(DataKind::name), err.map(DataKind::name));
    HostFn::find(name, &params, ok.as_deref(), err.as_deref())
}
