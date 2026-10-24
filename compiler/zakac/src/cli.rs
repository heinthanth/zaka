use std::collections::HashMap;

use bon::Builder;
use zakac_burmese_segmenter::cli::ZakacBurmeseSegmenter;
use zakac_cli_common::clap;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::clap::crate_name;
use zakac_cli_common::clap_error_handler::unsupported_subcommand_error;
use zakac_cli_common::clap_utils::CommandInfo;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_ir_builder::cli::ZakacIrBuilder;
use zakac_lexer::cli::ZakacLexer;
use zakac_parser::cli::ZakacParser;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::cli::ZakacSemcheck;

/// The `zakac` command.
#[derive(Builder)]
pub(crate) struct Zakac
{
    /// private [`clap::Command`] instance.
    cmd_instance: clap::Command,
    /// map of subcommand names to their [`Command`] instances.
    subcommands: HashMap<String, Box<dyn Command>>,
}

impl Command for Zakac
{
    /// Get the canonical name of the command.
    ///
    /// # Returns
    ///
    /// The canonical name of the command.
    fn name() -> &'static str { crate_name!() }

    /// Get the name of the standalone command.
    ///
    /// # Returns
    ///
    /// The name of the standalone command.
    fn standalone_name() -> &'static str { Self::name() }

    /// Get the aliases of the command.
    ///
    /// # Returns
    ///
    /// The aliases of the command.
    fn aliases() -> &'static [&'static str] { &[] }

    /// Get the version of the command.
    ///
    /// # Returns
    ///
    /// The version of the command.
    fn version() -> &'static str { clap::crate_version!() }

    /// Customize the [`clap::Command`] for standalone or subcommand use.
    ///
    /// # Arguments
    ///
    /// * `setup` - The command to configure.
    /// * `mode` - Whether this command is standalone or a subcommand.
    ///
    /// # Returns
    ///
    /// The configured command.
    fn configure(setup: CommandSetup, _mode: CommandMode) -> CommandSetup
    {
        setup
            .with(|cmd| cmd.subcommand_required(true))
            .add_subcommand::<ZakacBurmeseSegmenter>(Some("segment"))
            .add_subcommand::<ZakacLexer>(Some("lex"))
            .add_subcommand::<ZakacParser>(Some("parse"))
            .add_subcommand::<ZakacSemcheck>(Some("semcheck"))
            .add_subcommand::<ZakacIrBuilder>(Some("irgen"))
    }

    /// Construct this command wrapper from a finished [`CommandSetup`].
    ///
    /// # Arguments
    ///
    /// * `setup` - The finished [`CommandSetup`].
    ///
    /// # Returns
    ///
    /// The constructed command wrapper.
    fn from_setup(setup: CommandSetup) -> Self
    {
        let (cmd_instance, subcommands) = setup.into_parts();
        Self::builder().cmd_instance(cmd_instance).subcommands(subcommands).build()
    }

    /// Get the [`clap::Command`] instance stored in the command.
    ///
    /// # Returns
    ///
    /// The [`clap::Command`] instance stored in the command.
    fn clap_command(&self) -> &clap::Command { &self.cmd_instance }

    /// Get the [`Command`] instance of the registered subcommand.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the subcommand.
    ///
    /// # Returns
    ///
    /// The [`Command`] instance of the registered subcommand.
    fn get_subcommand(&self, name: &str) -> Option<&dyn Command>
    {
        self.subcommands.get(name).map(|cmd| cmd.as_ref())
    }

    /// The main function for the command.
    ///
    /// # Arguments
    ///
    /// * `self` - The command instance.
    /// * `matches` - The matches of the CLI arguments.
    /// * `io` - The I/O configuration for the command.
    ///
    /// # Returns
    ///
    /// The exit code of the command.
    fn main(&self, matches: &ArgMatches, io: &mut CommandIO<'_>) -> ExitCode
    {
        command_main(self, matches, io)
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
fn command_main(cmd: &Zakac, matches: &ArgMatches, io: &mut CommandIO<'_>) -> ExitCode
{
    let found = matches
        .subcommand()
        .and_then(|(name, sub)| cmd.get_subcommand(name).map(|subcmd| (subcmd, sub)));
    ret_if!(let Some((subcmd, sub_matches)) = found, subcmd.main(sub_matches, io));

    let cmd_info = CommandInfo::new(Zakac::name(), Zakac::version());
    let subcmd = matches.subcommand_name().unwrap_or_default();
    unsupported_subcommand_error(&cmd_info, subcmd, io.stderr)
}
