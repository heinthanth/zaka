pub mod artifact_output;
pub mod clap_error_handler;
pub mod clap_utils;
pub mod command;
pub mod config;

pub use clap;
pub use termcolor;
use termcolor::ColorChoice;
use termcolor::StandardStream;

use crate::clap_utils::CommandInfo;
use crate::clap_utils::parse_cli_args;
use crate::clap_utils::report_command_error;
use crate::command::Command;
use crate::command::CommandIO;

zakac_i18n::init!();

/// The stack of the thread a command runs on. The phases walk the program
/// tree by recursion, so a program nested deep needs more than the main
/// thread gets.
const COMMAND_STACK: usize = 256 << 20;

/// Shared main function to use in executables: the command runs on a
/// thread with a large stack, or on this thread when none can start.
///
/// # Template Parameters
///
/// * `C` - The command trait compatible item (i.e. a command implementation).
///
/// # Returns
///
/// The exit code of the command.
pub fn executable_main<C: Command + 'static>() -> std::process::ExitCode
{
    let spawned = std::thread::Builder::new().stack_size(COMMAND_STACK).spawn(run::<C>);
    let Ok(handle) = spawned
    else
    {
        return run::<C>();
    };
    handle.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}

/// Run the command on the standard streams.
///
/// # Template Parameters
///
/// * `C` - The command trait compatible item (i.e. a command implementation).
///
/// # Returns
///
/// The exit code of the command.
fn run<C: Command>() -> std::process::ExitCode
{
    let command = C::create_as_standalone();
    // Stdin is locked for each read alone, so a program that reads its input
    // while it runs (the VM's terminal world) can take the lock too.
    let mut sin = std::io::BufReader::new(std::io::stdin());
    let mut sout = StandardStream::stdout(ColorChoice::Auto);
    let mut serr = StandardStream::stderr(ColorChoice::Auto);
    let mut io = CommandIO { stdin: &mut sin, stdout: &mut sout, stderr: &mut serr };
    let args: Vec<String> = std::env::args().collect();
    // The config files are read here, in an executable alone, so a test that
    // runs a command in its own process never sees the config of the one
    // who runs it.
    let cmd =
        match config::configure(command.clap_command(), C::config_section(), &args)
        {
            Ok(cmd) => cmd,
            Err(error) =>
            {
                let info = CommandInfo::new(C::standalone_name(), C::version());
                return report_command_error(&mut io, &info, error).into();
            }
        };
    let matches = parse_cli_args(&cmd, &args, io.stdout, io.stderr);
    matches.map_or_else(|code| code, |m| command.main(&m, &mut io)).into()
}
