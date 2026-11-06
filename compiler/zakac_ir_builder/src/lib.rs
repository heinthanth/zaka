pub mod artifact;
pub mod cli;
mod comptime;
mod fault;
pub mod fold;
pub mod ir;
mod lends;
pub mod lower;
pub mod s_expr;
pub mod spans;
pub use zakac_semcheck::value;

zakac_i18n::init!();
