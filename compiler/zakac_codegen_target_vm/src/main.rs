use zakac_cli_common::executable_main;
use zakac_codegen_target_vm::cli::ZakacTargetVm;

fn main() -> std::process::ExitCode { executable_main::<ZakacTargetVm>() }
