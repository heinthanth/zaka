//! The native target of the Zaka compiler: the folded IR lowered to
//! Cranelift IR (CLIF), then run at once in the `zakac` process (JIT) or
//! written as an object file and linked with the runtime (AOT).

pub mod aot;
mod blob;
pub mod cli;
mod codeview;
mod codeview_kinds;
pub mod debug;
#[cfg(feature = "disasm")]
pub mod disasm;
mod dwarf_kinds;
pub mod emit;
mod exprs;
mod ffi;
mod func;
#[cfg(feature = "jit")]
pub mod jit;
mod ops;
mod stmts;
mod threads;
mod unwind;
