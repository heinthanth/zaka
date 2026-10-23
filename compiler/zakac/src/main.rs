mod cli;

use zakac_cli_common::executable_main;

fn main() -> std::process::ExitCode { executable_main::<cli::Zakac>() }
