pub mod callbacks;
pub mod cli;
pub mod descriptors;
pub mod driver;
pub mod emit;
pub mod exports;
mod exprs;
pub mod ffi;
pub mod jobs;
pub mod kinds;
pub mod lib_names;
pub mod options;
mod stmts;
pub mod toolchain;
pub mod wasi;

zakac_i18n::init!();

/// The prefix of each name the runtime this compiler carries exports, in
/// place of `zk_`: the version of the runtime and a hash of its sources.
pub const RUNTIME_PREFIX: &str = env!("ZK_NAME_PREFIX");

/// The name a function or a static of the runtime links under.
///
/// # Arguments
///
/// * `plain` - The name as C writes it, such as `zk_alloc`.
///
/// # Returns
///
/// The linked name.
pub fn linked_name(plain: &str) -> String
{
    let prefix = crate::driver::runtime_prefix();
    format!("{prefix}{}", plain.strip_prefix("zk_").unwrap_or(plain))
}
