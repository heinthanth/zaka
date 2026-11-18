//! The integration tests, built as one test program so the crate links once.

mod common;
mod debug_info;
#[cfg(feature = "disasm")]
mod disasm;
mod ffi_lib;
mod forms;
mod permissions;
mod samples;
mod traps;
mod unwind;
