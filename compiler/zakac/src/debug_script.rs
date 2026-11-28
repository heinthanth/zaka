//! The `zakac debug-script` command: print the formatters that show Zaka text
//! as a string and a plain list as its cells in lldb or gdb, for a native
//! build with debug info.

use zakac_cli_common::clap;
use zakac_cli_common::clap::Arg;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_codegen_target_native::debug::GDB_SCRIPT;
use zakac_codegen_target_native::debug::LLDB_SCRIPT;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

/// The argument that names the debugger.
const DEBUGGER_ARG: &str = "debugger";

/// The `zakac debug-script` command.
pub(crate) struct ZakacDebugScript
{
    /// The clap command.
    cmd_instance: clap::Command,
}

/// Print the script of the debugger the command names.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `io` - The standard streams.
///
/// # Returns
///
/// The exit code.
fn command_main(matches: &ArgMatches, io: &mut CommandIO<'_>) -> ExitCode
{
    let gdb = matches.get_one::<String>(DEBUGGER_ARG).is_some_and(|d| d == "gdb");
    let _ = io.stdout.write_all(tern!(gdb, GDB_SCRIPT, LLDB_SCRIPT).as_bytes());
    ExitCode::Success
}

impl Command for ZakacDebugScript
{
    /// Get the canonical name of the command.
    ///
    /// # Returns
    ///
    /// The canonical name of the command.
    fn name() -> &'static str { "debug-script" }

    /// Get the name of the command when run on its own.
    ///
    /// # Returns
    ///
    /// The name.
    fn standalone_name() -> &'static str { Self::name() }

    /// Get the aliases of the command.
    ///
    /// # Returns
    ///
    /// The aliases.
    fn aliases() -> &'static [&'static str] { &[] }

    /// Get the version of the command.
    ///
    /// # Returns
    ///
    /// The version.
    fn version() -> &'static str { clap::crate_version!() }

    /// Configure the command.
    ///
    /// # Arguments
    ///
    /// * `setup` - The setup to configure.
    /// * `_mode` - The mode the command runs in.
    ///
    /// # Returns
    ///
    /// The configured setup.
    fn configure(setup: CommandSetup, _mode: CommandMode) -> CommandSetup
    {
        let debugger = Arg::new(DEBUGGER_ARG)
            .value_name("DEBUGGER")
            .value_parser(["lldb", "gdb"])
            .required(true)
            .help(t!("debug_script.debugger"));
        setup.with(|cmd| cmd.about(t!("debug_script.about")).arg(debugger))
    }

    /// Build the command from its setup.
    ///
    /// # Arguments
    ///
    /// * `setup` - The setup.
    ///
    /// # Returns
    ///
    /// The command.
    fn from_setup(setup: CommandSetup) -> Self
    {
        let (cmd_instance, _) = setup.into_parts();
        Self { cmd_instance }
    }

    /// Get the clap command.
    ///
    /// # Returns
    ///
    /// The clap command.
    fn clap_command(&self) -> &clap::Command { &self.cmd_instance }

    /// Run the command.
    ///
    /// # Arguments
    ///
    /// * `matches` - The matches of the CLI arguments.
    /// * `io` - The standard streams.
    ///
    /// # Returns
    ///
    /// The exit code.
    fn main(&self, matches: &ArgMatches, io: &mut CommandIO<'_>) -> ExitCode
    {
        command_main(matches, io)
    }
}
