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
mod rings;
mod samples;
mod select;
mod spoken;
mod stepping;
mod text_consts;
mod threads;
mod traps;
mod values;
mod vet;
