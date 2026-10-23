use zakac_cli_common::executable_main;
use zakac_lexer::cli::ZakacLexer;

fn main() -> std::process::ExitCode { executable_main::<ZakacLexer>() }
