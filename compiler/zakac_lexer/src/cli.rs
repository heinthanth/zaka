use std::sync::Arc;

use zakac_cli_common::artifact_output::PhaseArtifactOutput;
use zakac_cli_common::artifact_output::output_format_from_matches;
use zakac_cli_common::artifact_output::resolve_artifact_output;
use zakac_cli_common::clap;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::clap::crate_name;
use zakac_cli_common::clap_utils::CommandInfo;
use zakac_cli_common::clap_utils::attach_common_io_args;
use zakac_cli_common::clap_utils::report_command_error;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_diagnostics::has_errors;
use zakac_i18n::t;
use zakac_reporter::render::write_diagnostics;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::dump_artifact;
use zakac_session::artifact::load_artifact;
use zakac_session::config::Input;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

use crate::artifact::LexArtifact;
use crate::lexer::LexerOptions;
use crate::lexer::lex;

const LEX_RAW_EXTENSION: &str = ".zktok";
const LEX_TEXT_EXTENSION: &str = ".zktokt";

/// The `zakac-lexer` command.
pub struct ZakacLexer
{
    cmd_instance: clap::Command,
}

/// Configure the lexer command.
///
/// # Arguments
///
/// * `cmd` - The command to configure.
///
/// # Returns
///
/// The configured command.
fn configure_lexer_command(cmd: clap::Command) -> clap::Command
{
    attach_common_io_args(cmd).about(t!("lexer.about"))
}

impl Command for ZakacLexer
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
        setup.with(configure_lexer_command)
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
        let (cmd_instance, _) = setup.into_parts();
        Self { cmd_instance }
    }

    /// Get the [`clap::Command`] instance stored in the command.
    ///
    /// # Returns
    ///
    /// The [`clap::Command`] instance stored in the command.
    fn clap_command(&self) -> &clap::Command { &self.cmd_instance }

    /// The main function for the command.
    ///
    /// # Arguments
    ///
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

/// Output capabilities for the lex phase.
fn lex_output_caps() -> PhaseArtifactOutput
{
    PhaseArtifactOutput::builder()
        .formats(&[ArtifactFormat::Text, ArtifactFormat::Raw])
        .default_format(ArtifactFormat::Raw)
        .raw_extension(LEX_RAW_EXTENSION)
        .text_extension(LEX_TEXT_EXTENSION)
        .build()
}

/// The main function for this "zakac_lexer" command.
///
/// The artifact is written even when the lexer reports errors; the
/// diagnostics go to stderr and the exit code is then
/// [`ExitCode::Failure`].
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `io` - The I/O configuration for the command.
///
/// # Returns
///
/// The exit code of the command.
fn command_main(
    _cmd: &ZakacLexer, matches: &ArgMatches, io: &mut CommandIO<'_>,
) -> ExitCode
{
    let cmd_info = CommandInfo::new(ZakacLexer::name(), ZakacLexer::version());

    let input: Input =
        Input::from(matches.get_one::<String>("input").map(|s| s.as_str()));
    let output_arg = matches.get_one::<String>("output").map(|s| s.as_str());
    let caps = lex_output_caps();
    let format_flag = output_format_from_matches(matches, &caps);
    let resolved = resolve_artifact_output(&input, output_arg, format_flag, &caps);
    ret_if!(let Err(error) = resolved, report_command_error(io, &cmd_info, error));
    let resolved = resolved.unwrap();

    let mut source_map = SourceMap::new();
    let source = load_artifact::<Arc<SourceFile>>(
        &input,
        ArtifactFormat::Text,
        io.stdin,
        &mut source_map,
    );
    ret_if!(let Err(error) = source, report_command_error(io, &cmd_info, error));
    let source = source.unwrap();

    let out = lex(&source, &LexerOptions::default());
    let failed = has_errors(&out.diagnostics);
    let _ = write_diagnostics(io.stderr, &source_map, &out.diagnostics);
    let artifact = LexArtifact::from_output(&source, out);

    let mut output = resolved.output;
    let result = dump_artifact(&mut output, io.stdout, &artifact, resolved.format);
    ret_if!(let Err(error) = result, report_command_error(io, &cmd_info, error));
    tern!(failed, ExitCode::Failure, ExitCode::Success)
}
