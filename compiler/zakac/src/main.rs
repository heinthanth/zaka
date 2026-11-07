mod build;
mod cli;

use zakac_cli_common::executable_main;

zakac_i18n::init!();

fn main() -> std::process::ExitCode { executable_main::<cli::Zakac>() }
