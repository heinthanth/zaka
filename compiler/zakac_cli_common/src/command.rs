use std::collections::HashMap;
use std::io::BufRead;
use std::process::ExitCode as PExitCode;

use bon::Builder;
use clap::ArgMatches;
use num_enum::FromPrimitive;
use num_enum::IntoPrimitive;
use termcolor::WriteColor;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::clap;
use crate::clap::Arg;
use crate::clap::ArgAction;
use crate::clap_utils::create_help_template;
use crate::clap_utils::parse_cli_args;

/// The exit code of the command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ExitCode
{
    /// Nothing went wrong during command execution.
    #[default]
    Success = 0,
    /// Generic failure.
    Failure,
    /// Invalid CLI usage.
    InvalidUsage,
    /// IO error.
    IoError,
}

impl From<ExitCode> for PExitCode
{
    /// Convert the ExitCode to a [`std::process::ExitCode`].
    ///
    /// # Arguments
    ///
    /// * `self` - The ExitCode to convert.
    ///
    /// # Returns
    ///
    /// A [`std::process::ExitCode`].
    fn from(code: ExitCode) -> Self { PExitCode::from(u8::from(code)) }
}

/// The I/O configuration for the command.
#[derive(Builder)]
pub struct CommandIO<'a>
{
    /// The standard input.
    pub stdin: &'a mut dyn BufRead,
    /// The standard output.
    pub stdout: &'a mut dyn WriteColor,
    /// The standard error.
    pub stderr: &'a mut dyn WriteColor,
}

/// Whether a command is built as a standalone executable or as a subcommand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandMode
{
    /// Built as the root command of a standalone executable.
    Standalone,
    /// Built as a subcommand of another command.
    Subcommand,
}

/// Builds a [`clap::Command`] with shared help/version flags.
///
/// The localized help template is applied later using
/// [`CommandSetup::attach_help_template`], after subcommands are registered.
pub struct CommandBuilder
{
    cmd: clap::Command,
}

impl CommandBuilder
{
    /// Create a new command with shared global help and version flags.
    ///
    /// # Arguments
    ///
    /// * `name` - The name used to execute the command.
    /// * `version` - Command version.
    pub fn new(name: &'static str, version: &'static str) -> Self
    {
        let g_help_flag = Arg::new("help")
            .help(t!("cli.help"))
            .short('h')
            .long("help")
            .action(ArgAction::Help)
            .required(false)
            .global(true);

        let g_version_flag = Arg::new("version")
            .help(t!("cli.version"))
            .short('v')
            .long("version")
            .action(ArgAction::Version)
            .required(false)
            .global(true);

        let cmd = clap::Command::new(name)
            .version(version)
            .propagate_version(true)
            .disable_help_flag(true)
            .disable_help_subcommand(true)
            .disable_version_flag(true)
            .arg(g_help_flag)
            .arg(g_version_flag);

        Self { cmd }
    }

    /// Set the canonical name used in help, version, and errors.
    pub fn canonical_name(self, name: &'static str) -> Self
    {
        Self { cmd: self.cmd.display_name(name) }
    }

    /// Set command aliases.
    pub fn aliases(self, aliases: &'static [&'static str]) -> Self
    {
        Self { cmd: self.cmd.aliases(aliases) }
    }

    /// Take the [`clap::Command`] before the help template is applied.
    pub fn command(self) -> clap::Command { self.cmd }
}

/// A [`clap::Command`] being configured, together with the subcommand handlers
/// registered through [`CommandSetup::add_subcommand`].
#[derive(Builder)]
#[builder(start_fn(vis = ""), builder_type(vis = ""))]
pub struct CommandSetup
{
    cmd: clap::Command,
    #[builder(default)]
    subcommands: HashMap<String, Box<dyn Command>>,
}

impl CommandSetup
{
    /// Wrap a [`clap::Command`] that does not have a help template.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The command to configure.
    pub fn new(cmd: clap::Command) -> Self { Self::builder().cmd(cmd).build() }

    /// Configure the internal [`clap::Command`].
    ///
    /// # Arguments
    ///
    /// * `f` - The function to configure the command.
    ///
    /// # Returns
    ///
    /// The setup with the command configured.
    pub fn with(self, f: impl FnOnce(clap::Command) -> clap::Command) -> Self
    {
        Self::builder().cmd(f(self.cmd)).subcommands(self.subcommands).build()
    }

    /// Register `C` as a subcommand.
    ///
    /// Stores the handler for later dispatch and attaches its
    /// [`clap::Command`] to this command.
    ///
    /// # Arguments
    ///
    /// * `name` - Name override. [`None`] keeps [`Command::name`].
    ///
    /// # Returns
    ///
    /// The setup with the subcommand registered.
    pub fn add_subcommand<C: Command>(mut self, name: Option<&'static str>) -> Self
    {
        let child = C::create_as_subcommand(name);
        let clap_child = child.clap_command().clone();
        let registered_name = clap_child.get_name().to_string();
        self.cmd = self.cmd.subcommand(clap_child);
        self.subcommands.insert(registered_name, child);
        self
    }

    /// Attach the localized help template.
    /// Call this after subcommands are registered so the template can include
    /// them.
    ///
    /// # Returns
    ///
    /// The setup with the help template attached.
    pub(crate) fn attach_help_template(mut self) -> Self
    {
        let help = create_help_template(&self.cmd);
        self.cmd = self.cmd.help_template(help);
        self
    }

    /// Split the setup into the finished [`clap::Command`] and its handlers.
    ///
    /// # Returns
    ///
    /// The finished [`clap::Command`] and its handlers.
    pub fn into_parts(self) -> (clap::Command, HashMap<String, Box<dyn Command>>)
    {
        (self.cmd, self.subcommands)
    }
}

/// An interface that all Zaka commands must implement.
pub trait Command: 'static
{
    /// Get the canonical name of the command.
    ///
    /// # Returns
    ///
    /// The canonical name of the command.
    fn name() -> &'static str
    where
        Self: Sized;

    /// Get the name of the standalone command.
    ///
    /// # Returns
    ///
    /// The name of the standalone command.
    fn standalone_name() -> &'static str
    where
        Self: Sized;

    /// Get the aliases of the command.
    ///
    /// # Returns
    ///
    /// The aliases of the command.
    fn aliases() -> &'static [&'static str]
    where
        Self: Sized;

    /// Get the version of the command.
    ///
    /// # Returns
    ///
    /// The version of the command.
    fn version() -> &'static str
    where
        Self: Sized;

    /// Customize the [`clap::Command`] for standalone or subcommand use.
    ///
    /// # Arguments
    ///
    /// * `setup` - The command after shared flags are applied.
    /// * `mode` - Whether this command is standalone or a subcommand.
    ///
    /// # Returns
    ///
    /// The configured command (before the help template is applied).
    fn configure(setup: CommandSetup, mode: CommandMode) -> CommandSetup
    where
        Self: Sized;

    /// Construct this command wrapper from a finished [`CommandSetup`].
    fn from_setup(setup: CommandSetup) -> Self
    where
        Self: Sized;

    /// Create the [`clap::Command`] to be used as a standalone command.
    ///
    /// # Returns
    ///
    /// The command instance.
    fn create_as_standalone() -> Box<dyn Command>
    where
        Self: Sized,
    {
        let mut builder = CommandBuilder::new(Self::standalone_name(), Self::version());
        let aliases = Self::aliases();
        do_if!(!aliases.is_empty(), builder = builder.aliases(aliases));

        let setup = Self::configure(
            CommandSetup::new(builder.command()),
            CommandMode::Standalone,
        );
        Box::new(Self::from_setup(setup.attach_help_template()))
    }

    /// Create the [`clap::Command`] to be used as a subcommand.
    ///
    /// # Arguments
    ///
    /// * `name_override` - The name of the subcommand to override the default
    ///   name.
    ///
    /// # Returns
    ///
    /// The command instance.
    fn create_as_subcommand(name_override: Option<&'static str>) -> Box<dyn Command>
    where
        Self: Sized,
    {
        let name = name_override.unwrap_or(Self::name());
        let mut builder = CommandBuilder::new(name, Self::version());
        let aliases = Self::aliases();
        do_if!(!aliases.is_empty(), builder = builder.aliases(aliases));

        let setup = Self::configure(
            CommandSetup::new(builder.command()),
            CommandMode::Subcommand,
        );
        Box::new(Self::from_setup(setup.attach_help_template()))
    }

    /// Get the [`Command`] instance of the registered subcommand.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the subcommand.
    ///
    /// # Returns
    ///
    /// The [`Command`] instance of the registered subcommand.
    fn get_subcommand(&self, _name: &str) -> Option<&dyn Command>
    where
        Self: Sized,
    {
        None
    }

    /// Get the [`clap::Command`] instance stored in the command.
    ///
    /// # Returns
    ///
    /// The [`clap::Command`] instance stored in the command.
    fn clap_command(&self) -> &clap::Command;

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
    fn execute(&self, args: &[String], io: &mut CommandIO<'_>) -> ExitCode
    {
        let matches = parse_cli_args(&self.clap_command(), args, io.stdout, io.stderr);
        ret_if!(let Err(code) = matches, code);
        self.main(&matches.unwrap(), io)
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
    fn main(&self, matches: &ArgMatches, io: &mut CommandIO<'_>) -> ExitCode;
}
