use std::io::BufRead;

use zakac_cli_common::clap;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::clap_error_handler::unsupported_subcommand_error;
use zakac_cli_common::clap_utils;
use zakac_cli_common::clap_utils::CommandInfo;
use zakac_cli_common::clap_utils::parse_cli_args;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::ExitCode;
use zakac_cli_common::termcolor::WriteColor;
use zakac_rust_sugars::ret_if;

pub(crate) struct Zakac;

impl Command for Zakac
{
    /// Get the canonical name of the command.
    ///
    /// # Returns
    ///
    /// The canonical name of the command.
    fn name(&self) -> &'static str { "zakac" }

    /// Get the name of the standalone command.
    ///
    /// # Returns
    ///
    /// The name of the standalone command.
    fn standalone_name(&self) -> &'static str { "zakac" }

    /// Get the aliases of the command.
    ///
    /// # Returns
    ///
    /// The aliases of the command.
    fn aliases(&self) -> &'static [&'static str] { &[] }

    /// Get the version of the command.
    ///
    /// # Returns
    ///
    /// The version of the command.
    fn version(&self) -> &'static str { clap::crate_version!() }

    /// Create the [`clap::Command`] to be used as a standalone command.
    ///
    /// # Returns
    ///
    /// The [`clap::Command`] for the standalone command.
    fn create_as_standalone(&self) -> clap::Command
    {
        clap_utils::create_clap_command(self.name(), self.version(), None, None)
            .subcommand_required(true)
    }

    /// Create the [`clap::Command`] to be used as a subcommand.
    ///
    /// # Returns
    ///
    /// The [`clap::Command`] for the subcommand.
    fn create_as_subcommand(&self) -> clap::Command { unimplemented!() }

    /// Execute the command with given arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments to pass (without the command name).
    /// * `io` - The I/O configuration for the command.
    ///
    /// # Returns
    ///
    /// The exit code of the command.
    fn execute<I, O, E>(
        &self, args: &[String], io: &mut CommandIO<'_, I, O, E>,
    ) -> ExitCode
    where
        I: BufRead,
        O: WriteColor,
        E: WriteColor,
    {
        let clap_cmd = self.create_as_standalone();
        let matches = parse_cli_args(&clap_cmd, args, &mut io.stdout, &mut io.stderr);
        ret_if!(let Err(code) = matches, code);
        command_main(&matches.unwrap(), io)
    }
}

/// The main function for this "zakac" command.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `io` - The I/O configuration for the command.
///
/// # Returns
///
/// The exit code of the command.
fn command_main<I, O, E>(
    matches: &ArgMatches, io: &mut CommandIO<'_, I, O, E>,
) -> ExitCode
where
    I: BufRead,
    O: WriteColor,
    E: WriteColor,
{
    let cmd_info = CommandInfo::new(Zakac.name(), Zakac.version());
    let subcmd = matches.subcommand_name().unwrap_or_default();
    unsupported_subcommand_error(&cmd_info, subcmd, io.stderr)
}
