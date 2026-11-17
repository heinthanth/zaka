//! `zakac run` with the JIT at hand, for the tests of the native target: a
//! program that stops ends this process alone.

use zakac_cli_common::executable_main;

fn main() -> std::process::ExitCode
{
    zaka_vm::comptime::install();
    let _ = zaka_vm::cli::JIT_RUNNER.set(zakac_codegen_target_native::cli::run_jit);
    executable_main::<zaka_vm::cli::ZakaVm>()
}
