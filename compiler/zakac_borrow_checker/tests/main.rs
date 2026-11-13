//! The integration tests, built as one test program so the crate links once.

mod artifact;
mod bounds;
mod calls;
mod cli;
mod closures;
mod common;
mod escape;
mod free_rules;
mod last_use;
mod leaks;
mod lists;
mod loans;
mod modes;
mod moves;
mod nullness;
mod oracle;
mod pipeline;
mod samples;
mod still_works;
mod unset;
mod vm_inputs;
