//! The formatter of Zaka source: `zakac fmt` and `zakafmt`.
//!
//! The tokens and comments of a file give the text, which the formatter
//! prints again word for word; the parsed blocks give the depth of each line.
//! The formatter changes the space between words alone, never splits or joins
//! a word, and checks that its output lexes to the same words.

pub mod check;
pub mod cli;
pub mod format;
pub mod options;
pub mod rewrite;

pub use format::FmtError;
pub use format::format;
pub use format::format_text;
pub use options::FmtOptions;

zakac_i18n::init!();
