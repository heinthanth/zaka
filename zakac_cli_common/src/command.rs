use std::io::BufRead;
use std::process::ExitCode as PExitCode;

use bon::Builder;
use num_enum::FromPrimitive;
use num_enum::IntoPrimitive;
use termcolor::WriteColor;

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
#[derive(Debug, Builder)]
pub struct CommandIO<'a, I, O, E>
{
    /// The standard input.
    pub stdin: &'a mut I,
    /// The standard output.
    pub stdout: &'a mut O,
    /// The standard error.
    pub stderr: &'a mut E,
}

/// An interface that all Zaka commands must implement.
pub trait Command
{
    /// Get the canonical name of the command.
    ///
    /// # Returns
    ///
    /// The canonical name of the command.
    fn name(&self) -> &'static str;

    /// Get the name of the standalone command.
    ///
    /// # Returns
    ///
    /// The name of the standalone command.
    fn standalone_name(&self) -> &'static str;

    /// Get the aliases of the command.
    ///
    /// # Returns
    ///
    /// The aliases of the command.
    fn aliases(&self) -> &'static [&'static str];

    /// Get the version of the command.
    ///
    /// # Returns
    ///
    /// The version of the command.
    fn version(&self) -> &'static str;

    /// Create the [`clap::Command`] to be used as a standalone command.
    ///
    /// # Returns
    ///
    /// The [`clap::Command`] for the standalone command.
    fn create_as_standalone(&self) -> clap::Command;

    /// Create the [`clap::Command`] to be used as a subcommand.
    ///
    /// # Returns
    ///
    /// The [`clap::Command`] for the subcommand.
    fn create_as_subcommand(&self) -> clap::Command;

    /// Execute the command with given arguments.
    ///
    /// # Arguments
    ///
    /// * `io` - The I/O configuration for the command.
    /// * `args` - The arguments to pass (without the command name).
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
        E: WriteColor;
}
