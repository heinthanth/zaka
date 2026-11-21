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

/// The name an export of the runtime links under: the prefix of this build of
/// the runtime, then the name without its `zk_`. C and generated code write
/// the plain name, which the header of the runtime maps to this one.
macro_rules! stamped {
    ($name:literal) => {
        concat!(env!("ZK_NAME_PREFIX"), $name)
    };
}

// WASI has no libffi: a program for it calls no C function and hands C no
// closure.
#[cfg(not(target_os = "wasi"))]
pub mod callbacks;
pub mod ctx;
pub mod exchange;
#[cfg(not(target_os = "wasi"))]
pub mod ffi;
#[cfg(zk_tasks)]
mod fiber;
pub mod heap;
pub mod host;
pub mod libs;
mod local;
pub mod native;
pub mod out;
pub mod program;
pub mod rights;
pub mod seqs;
pub mod show;
pub mod start;
mod tasks;
pub mod threads;
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

#[cfg(feature = "guard")]
unsafe extern "C" {
    /// The guard of a thread, in C: runs the step, and gives 1 once a trap
    /// jumped back to it.
    #[link_name = stamped!("thread_guard")]
    fn thread_guard(step: threads::ZkStep, data: *mut std::ffi::c_void) -> i32;
    /// The jump back to the guard of the thread of this OS thread.
    #[link_name = stamped!("thread_leave")]
    fn thread_leave(trap: i32) -> !;
}

/// The guard of a thread and its jump, under the plain names the code of a
/// thread calls, for a JIT: a build of a program compiles its own.
///
/// # Returns
///
/// The names and addresses.
#[cfg(feature = "guard")]
pub fn thread_glue() -> [(&'static str, *const u8); 2]
{
    [
        ("zk_thread_guard", thread_guard as *const u8),
        ("zk_thread_leave", thread_leave as *const u8),
    ]
}
