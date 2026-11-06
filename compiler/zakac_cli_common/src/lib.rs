pub mod artifact_output;
pub mod clap_error_handler;
pub mod clap_utils;
pub mod command;

pub use clap;
pub use termcolor;
use termcolor::ColorChoice;
use termcolor::StandardStream;

use crate::command::Command;
use crate::command::CommandIO;

zakac_i18n::init!();

/// Shared main function to use in executables.
///
/// # Template Parameters
///
/// * `C` - The command trait compatible item (i.e. a command implementation).
///
/// # Returns
///
/// The exit code of the command.
pub fn executable_main<C: Command>() -> std::process::ExitCode
{
    let command = C::create_as_standalone();
    // Stdin is locked for each read alone, so a program that reads its input
    // while it runs (the VM's terminal world) can take the lock too.
    let mut sin = std::io::BufReader::new(std::io::stdin());
    let mut sout = StandardStream::stdout(ColorChoice::Auto);
    let mut serr = StandardStream::stderr(ColorChoice::Auto);
    let mut io = CommandIO { stdin: &mut sin, stdout: &mut sout, stderr: &mut serr };
    command.execute(&std::env::args().collect::<Vec<String>>(), &mut io).into()
}
