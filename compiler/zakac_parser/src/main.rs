use zakac_cli_common::executable_main;
use zakac_parser::cli::ZakacParser;

fn main() -> std::process::ExitCode { executable_main::<ZakacParser>() }
