use lsp_server::Connection;
use zakac_cli_common::clap;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::clap::crate_name;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_i18n::t;

use crate::server::serve;

/// The `lsp` command: a language server for editors, over standard input
/// and output.
pub struct ZakacLsp
{
    /// The clap command.
    cmd_instance: clap::Command,
}

impl Command for ZakacLsp
{
    fn name() -> &'static str { crate_name!() }

    fn standalone_name() -> &'static str { Self::name() }

    fn aliases() -> &'static [&'static str] { &[] }

    fn version() -> &'static str { clap::crate_version!() }

    fn configure(setup: CommandSetup, _mode: CommandMode) -> CommandSetup
    {
        setup.with(|cmd| cmd.about(t!("lsp.about")))
    }

    fn from_setup(setup: CommandSetup) -> Self
    {
        let (cmd_instance, _) = setup.into_parts();
        Self { cmd_instance }
    }

    fn clap_command(&self) -> &clap::Command { &self.cmd_instance }

    fn main(&self, _matches: &ArgMatches, io: &mut CommandIO<'_>) -> ExitCode
    {
        let (connection, threads) = Connection::stdio();
        let served = serve(&connection);
        // Dropping the connection closes its channels, so the I/O threads end.
        drop(connection);
        let result = served.and(threads.join().map_err(Into::into));
        let Err(error) = result
        else
        {
            return ExitCode::Success;
        };
        let _ = writeln!(io.stderr, "{}", t!("lsp.failed", reason = error.to_string()));
        ExitCode::Failure
    }
}
