//! The runtime of a native Zaka program, in Rust: each `zk_*` function that
//! generated code calls, under the C name and signature
//! `zaka_target_c_runtime.h` declares.
//!
//! The C target links it as a static lib after the program objects; the
//! Cranelift JIT links it into `zakac` and finds each function through
//! [`exports`]. The runtime reads what it knows of a program (the settings of
//! its build, its rights, its texts and tables) through the table `zk_start`
//! receives, never through names the program defines.

// Each export is a C function: generated code hands it the pointers of the
// values it names, as each C function takes them.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod callbacks;
pub mod ffi;
pub mod heap;
pub mod host;
pub mod native;
pub mod out;
pub mod program;
pub mod rights;
pub mod seqs;
pub mod show;
pub mod start;
pub mod traps;

pub use rights::set_gate;

/// Each function of the runtime that generated code reaches, with its
/// address, for a JIT that resolves names itself.
///
/// # Returns
///
/// The names and addresses.
pub fn exports() -> &'static [(&'static str, *const u8)]
{
    // The table holds addresses alone, which never change.
    struct Table(&'static [(&'static str, *const u8)]);
    // SAFETY: the addresses are of functions, read and never written.
    unsafe impl Sync for Table {}
    static TABLE: Table = Table(include!(concat!(env!("OUT_DIR"), "/exports.rs")));
    TABLE.0
}
