//! The integration tests, built as one test program so the crate links once.

mod callbacks;
mod cli;
mod common;
mod comptime_fns;
mod heap;
mod host;
mod machine;
mod mem;
mod permissions;
mod print;
mod samples;
mod spoken;
mod stepping;
mod text_consts;
mod traps;
mod values;
mod vet;
