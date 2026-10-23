pub mod errors;

zakac_i18n::init!();

/// Re-exported [`errors::Error`].
pub use errors::Error;

/// A result used in Zakac operations.
pub type Result<T> = std::result::Result<T, errors::Error>;
