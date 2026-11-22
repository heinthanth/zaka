use zakac_cli_common::executable_main;
use zakac_formatter::cli::ZakacFormatter;

fn main() -> std::process::ExitCode { executable_main::<ZakacFormatter>() }
