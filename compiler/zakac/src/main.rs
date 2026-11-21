mod build;
mod cli;
mod repl;

use zakac_cli_common::executable_main;

zakac_i18n::init!();

fn main() -> std::process::ExitCode
{
    // The language server checks on each edit: its comptime runs take fewer
    // ops and never write or call C.
    let editor = std::env::args().nth(1).is_some_and(|a| a == "lsp");
    let steps = zakac_rust_sugars::tern!(
        editor,
        zaka_vm::comptime::EDITOR_STEPS,
        zaka_vm::comptime::MAX_STEPS
    );
    zaka_vm::comptime::install_with(steps, editor);
    let _ = zaka_vm::cli::JIT_RUNNER.set(zakac_codegen_target_native::cli::run_jit);
    executable_main::<cli::Zakac>()
}
