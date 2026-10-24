use zakac_cli_common::executable_main;
use zakac_ir_builder::cli::ZakacIrBuilder;

fn main() -> std::process::ExitCode { executable_main::<ZakacIrBuilder>() }
