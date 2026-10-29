use std::sync::Arc;

use strum::IntoEnumIterator;
use zakac_cli_common::artifact_output::PhaseArtifactOutput;
use zakac_cli_common::artifact_output::output_format_from_matches;
use zakac_cli_common::artifact_output::resolve_artifact_output;
use zakac_cli_common::clap;
use zakac_cli_common::clap::Arg;
use zakac_cli_common::clap::ArgAction;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::clap::builder::PossibleValuesParser;
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
use zakac_ir_builder::cli::attach_compile_args;
use zakac_ir_builder::cli::fold_options;
use zakac_ir_builder::cli::semcheck_options;
use zakac_lexer::lexer::LexerOptions;
use zakac_reporter::render::write_diagnostics;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::dump_artifact;
use zakac_session::artifact::load_artifact;
use zakac_session::config::Input;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

use crate::artifact::BorrowCheckArtifact;
use crate::artifact::run_phases;
use crate::options::BorrowCheckOptions;
use crate::options::MemCheck;

/// Extension of a raw artifact.
pub const BORROW_CHECK_RAW_EXTENSION: &str = ".zkbck";
/// Extension of a text artifact.
pub const BORROW_CHECK_TEXT_EXTENSION: &str = ".zkbckt";
const MEM_CHECK_ARG: &str = "mem_check";
const MEM_STRICT_ARG: &str = "mem_strict";

/// The `zakac_borrow_checker` command, `zakac borrow-check` (or `zakac mkbck`)
/// as a subcommand.
pub struct ZakacBorrowChecker
{
    /// The clap command.
    cmd_instance: clap::Command,
}

/// Configure the ownership check command.
///
/// # Arguments
///
/// * `cmd` - The command to configure.
///
/// # Returns
///
/// The configured command.
fn configure_borrow_check_command(cmd: clap::Command) -> clap::Command
{
    let cmd = attach_compile_args(attach_common_io_args(cmd));
    attach_borrow_check_args(cmd).about(t!("borrow_check.about"))
}

/// Add the flags that steer the ownership check: how it reports and how
/// strict it is. Later phases share them.
///
/// # Arguments
///
/// * `cmd` - The command.
///
/// # Returns
///
/// The command with the flags.
pub fn attach_borrow_check_args(cmd: clap::Command) -> clap::Command
{
    let words: Vec<&'static str> = MemCheck::iter().map(Into::into).collect();
    let check = Arg::new(MEM_CHECK_ARG)
        .long("mem-check")
        .value_name("MODE")
        .help(t!("borrow_check.mem_check"))
        .value_parser(PossibleValuesParser::new(words))
        .default_value(<&'static str>::from(MemCheck::default()));
    let strict = Arg::new(MEM_STRICT_ARG)
        .long("mem-strict")
        .help(t!("borrow_check.mem_strict"))
        .action(ArgAction::SetTrue);
    cmd.arg(check).arg(strict)
}

/// The ownership check options from the command line.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
///
/// # Returns
///
/// The options.
pub fn borrow_check_options(matches: &ArgMatches) -> BorrowCheckOptions
{
    let word = matches.get_one::<String>(MEM_CHECK_ARG);
    let check = word.and_then(|w| w.parse().ok()).unwrap_or_default();
    let strict = matches.get_flag(MEM_STRICT_ARG);
    BorrowCheckOptions::builder().check(check).strict(strict).build()
}

impl Command for ZakacBorrowChecker
{
    /// Get the canonical name of the command.
    ///
    /// # Returns
    ///
    /// The canonical name of the command.
    fn name() -> &'static str { crate_name!() }

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
    fn aliases() -> &'static [&'static str] { &["mkbck"] }

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
        setup.with(configure_borrow_check_command)
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

/// Output capabilities for the ownership check phase.
///
/// # Returns
///
/// The capabilities.
fn borrow_check_output_caps() -> PhaseArtifactOutput
{
    PhaseArtifactOutput::builder()
        .formats(&[ArtifactFormat::Text, ArtifactFormat::Raw])
        .default_format(ArtifactFormat::Raw)
        .raw_extension(BORROW_CHECK_RAW_EXTENSION)
        .text_extension(BORROW_CHECK_TEXT_EXTENSION)
        .build()
}

/// Check a source file and write the artifact. The artifact is written even
/// when a phase reports errors; the diagnostics go to stderr and the exit
/// code is then [`ExitCode::Failure`].
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
    let cmd_info =
        CommandInfo::new(ZakacBorrowChecker::name(), ZakacBorrowChecker::version());

    let input: Input =
        Input::from(matches.get_one::<String>("input").map(|s| s.as_str()));
    let output_arg = matches.get_one::<String>("output").map(|s| s.as_str());
    let caps = borrow_check_output_caps();
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

    let options = semcheck_options(matches);
    let folding = fold_options(matches);
    let mem = borrow_check_options(matches);
    let lexer = LexerOptions::default();
    let output = run_phases(&source, &lexer, &options, &folding, &mem);
    let failed = has_errors(&output.diagnostics);
    let _ = write_diagnostics(io.stderr, &source_map, &output.diagnostics);
    let artifact = BorrowCheckArtifact::from_borrowchecked(&source, output);

    let mut output = resolved.output;
    let result = dump_artifact(&mut output, io.stdout, &artifact, resolved.format);
    ret_if!(let Err(error) = result, report_command_error(io, &cmd_info, error));
    tern!(failed, ExitCode::Failure, ExitCode::Success)
}
