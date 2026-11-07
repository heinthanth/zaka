use zakac_cli_common::executable_main;
use zakac_codegen_target_c::cli::ZakacTargetC;

fn main() -> std::process::ExitCode { executable_main::<ZakacTargetC>() }
