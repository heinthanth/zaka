pub mod code;
pub mod diagnostic;
pub mod kinds;
pub mod lexer;
pub mod parser;
pub mod resolve;

pub use code::Code;
pub use diagnostic::Diagnostic;
pub use diagnostic::Level;
pub use diagnostic::has_errors;

zakac_i18n::init!();
