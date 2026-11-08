use zakac_cli_common::executable_main;
use zakac_lsp::cli::ZakacLsp;

fn main() -> std::process::ExitCode { executable_main::<ZakacLsp>() }
