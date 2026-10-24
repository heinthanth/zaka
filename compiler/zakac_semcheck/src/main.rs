use zakac_cli_common::executable_main;
use zakac_semcheck::cli::ZakacSemcheck;

fn main() -> std::process::ExitCode { executable_main::<ZakacSemcheck>() }
