//! The functions the host serves to a Zaka program, in three parts:
//!
//! - The signature: a program declares each one as an external function
//!   (`ပြင်ပမှလာသော`), in the stdlib; [`HOST_FNS`] holds the name and the kinds
//!   it must have, so the check can match a declaration to a row.
//! - The values: the VM turns each argument into a [`HostValue`] from the kind
//!   of its parameter, and the result back from the kind of the result, so a
//!   new function over kinds the VM knows needs no VM code.
//! - The effects: each row runs plain Rust against [`Effects`], which reaches a
//!   [`World`] (the terminal, or storage for the browser and the tests) and the
//!   output of the VM.
//!
//! To add a host function: a row in [`HOST_FNS`] with the function that runs
//! it, a [`World`] method when it needs a new effect (written for both
//! worlds), and its declaration in the stdlib.

mod kinds;
mod table;
mod value;
mod world;

pub use kinds::*;
pub use table::HOST_FNS;
pub use table::HostDef;
pub use table::HostFn;
pub use table::listing;
pub use value::Args;
pub use value::HostValue;
pub use world::Effects;
pub use world::MemWorld;
pub use world::TermWorld;
pub use world::World;
