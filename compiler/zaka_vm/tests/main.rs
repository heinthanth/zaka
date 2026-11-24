//! The integration tests, built as one test program so the crate links once.

mod callbacks;
mod cli;
mod common;
mod comptime_fns;
mod embedding;
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
mod test_run;
mod text_consts;
mod threads;
mod trait_values;
mod traps;
mod values;
mod vet;
