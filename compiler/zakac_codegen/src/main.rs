use zakac_cli_common::executable_main;
use zakac_codegen::cli::ZakacCodegen;

fn main() -> std::process::ExitCode { executable_main::<ZakacCodegen>() }
