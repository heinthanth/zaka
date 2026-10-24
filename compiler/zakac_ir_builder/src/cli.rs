use std::sync::Arc;

use zakac_cli_common::artifact_output::PhaseArtifactOutput;
use zakac_cli_common::artifact_output::output_format_from_matches;
use zakac_cli_common::artifact_output::resolve_artifact_output;
use zakac_cli_common::clap;
use zakac_cli_common::clap::Arg;
use zakac_cli_common::clap::ArgAction;
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
use zakac_lexer::lexer::LexerOptions;
use zakac_reporter::render::write_diagnostics;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::dump_artifact;
use zakac_session::artifact::load_artifact;
use zakac_session::config::Input;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

use crate::artifact::IrArtifact;
use crate::artifact::run_phases;
use crate::ir::FoldOptions;

const IR_RAW_EXTENSION: &str = ".zkir";
const IR_TEXT_EXTENSION: &str = ".zkirt";
const PLATFORM_BITS_ARG: &str = "platform_bits";
const STRICT_PARTICLES_ARG: &str = "strict_particles";
const FOLD_ARG: &str = "fold";
/// Words the fold flag takes.
const FOLD_WORDS: &[&str] = &["all", "none", "exprs", "consts", "branches"];
/// Bit counts the platform-sized integer kinds can have.
const PLATFORM_WIDTHS: &[&str] = &["8", "16", "32", "64", "128"];

/// The `zakac_ir_builder` command, `zakac irgen` (or `zakac mkir`) as a
/// subcommand.
pub struct ZakacIrBuilder
{
    cmd_instance: clap::Command,
}

/// Configure the lowering command.
///
/// # Arguments
///
/// * `cmd` - The command to configure.
///
/// # Returns
///
/// The configured command.
fn configure_ir_command(cmd: clap::Command) -> clap::Command
{
    let platform_bits = Arg::new(PLATFORM_BITS_ARG)
        .long("platform-bits")
        .value_name("BITS")
        .help(t!("ir.platform_bits"))
        .value_parser(PLATFORM_WIDTHS.to_vec())
        .default_value("64");
    let strict = Arg::new(STRICT_PARTICLES_ARG)
        .long("strict-particles")
        .help(t!("ir.strict_particles"))
        .action(ArgAction::SetTrue);
    let fold = Arg::new(FOLD_ARG)
        .long("fold")
        .value_name("LIST")
        .help(t!("ir.fold"))
        .value_parser(FOLD_WORDS.to_vec())
        .value_delimiter(',')
        .default_value("all");
    let cmd = attach_common_io_args(cmd).arg(platform_bits).arg(strict).arg(fold);
    cmd.about(t!("ir.about"))
}

/// The check options from the command line.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
///
/// # Returns
///
/// The options.
fn semcheck_options(matches: &ArgMatches) -> SemcheckOptions
{
    let bits = matches.get_one::<String>(PLATFORM_BITS_ARG);
    let bits = bits.and_then(|b| b.parse().ok()).unwrap_or(64);
    let strict = matches.get_flag(STRICT_PARTICLES_ARG);
    SemcheckOptions::builder().platform_bits(bits).strict_particles(strict).build()
}

/// The fold options from the command line: `all`, `none`, or a list of
/// `exprs`, `consts` and `branches`.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
///
/// # Returns
///
/// The options.
fn fold_options(matches: &ArgMatches) -> FoldOptions
{
    let words: Vec<&str> = matches
        .get_many::<String>(FOLD_ARG)
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect();
    ret_if!(words.contains(&"all"), FoldOptions::default());
    let has = |w: &str| words.contains(&w);
    FoldOptions::builder()
        .exprs(has("exprs"))
        .consts(has("consts"))
        .branches(has("branches"))
        .build()
}

impl Command for ZakacIrBuilder
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
    fn aliases() -> &'static [&'static str] { &["mkir"] }

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
        setup.with(configure_ir_command)
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

/// Output capabilities for the lowering phase.
fn ir_output_caps() -> PhaseArtifactOutput
{
    PhaseArtifactOutput::builder()
        .formats(&[ArtifactFormat::Text, ArtifactFormat::Raw])
        .default_format(ArtifactFormat::Raw)
        .raw_extension(IR_RAW_EXTENSION)
        .text_extension(IR_TEXT_EXTENSION)
        .build()
}

/// The main function for this "zakac_ir_builder" command.
///
/// The artifact is written even when a phase reports errors; the
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
    _cmd: &ZakacIrBuilder, matches: &ArgMatches, io: &mut CommandIO<'_>,
) -> ExitCode
{
    let cmd_info = CommandInfo::new(ZakacIrBuilder::name(), ZakacIrBuilder::version());

    let input: Input =
        Input::from(matches.get_one::<String>("input").map(|s| s.as_str()));
    let output_arg = matches.get_one::<String>("output").map(|s| s.as_str());
    let caps = ir_output_caps();
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
    let lowered = run_phases(&source, &LexerOptions::default(), &options, &folding);
    let failed = has_errors(&lowered.diagnostics);
    let _ = write_diagnostics(io.stderr, &source_map, &lowered.diagnostics);
    let artifact = IrArtifact::from_lowered(&source, lowered);

    let mut output = resolved.output;
    let result = dump_artifact(&mut output, io.stdout, &artifact, resolved.format);
    ret_if!(let Err(error) = result, report_command_error(io, &cmd_info, error));
    tern!(failed, ExitCode::Failure, ExitCode::Success)
}
