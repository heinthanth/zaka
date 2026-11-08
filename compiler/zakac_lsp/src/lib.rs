pub mod check;
#[cfg(feature = "server")]
pub mod cli;
pub mod complete;
pub mod hover;
pub mod lines;
pub mod names;
pub mod outline;
#[cfg(feature = "server")]
pub mod server;
pub mod tokens;

zakac_i18n::init!();
