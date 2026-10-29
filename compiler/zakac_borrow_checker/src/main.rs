use zakac_borrow_checker::cli::ZakacBorrowChecker;
use zakac_cli_common::executable_main;

fn main() -> std::process::ExitCode { executable_main::<ZakacBorrowChecker>() }
