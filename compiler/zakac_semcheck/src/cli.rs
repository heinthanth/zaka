use std::sync::Arc;

use strum::VariantNames;
use zakac_ast::target::BuildTarget;
use zakac_ast::target::Target;
use zakac_ast::target::TargetAbi;
use zakac_ast::target::TargetArch;
use zakac_ast::target::TargetOs;
use zakac_ast::target::host_arch;
use zakac_ast::target::host_os;
use zakac_cli_common::artifact_output::PhaseArtifactOutput;
use zakac_cli_common::artifact_output::output_format_from_matches;
use zakac_cli_common::artifact_output::resolve_artifact_output;
use zakac_cli_common::clap;
use zakac_cli_common::clap::Arg;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::clap::crate_name;
use zakac_cli_common::clap::parser::ValueSource;
use zakac_cli_common::clap_utils::CommandInfo;
use zakac_cli_common::clap_utils::attach_common_io_args;
use zakac_cli_common::clap_utils::report_command_error;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_cli_common::grants::Granted;
use zakac_cli_common::grants::attach_grant_args;
use zakac_cli_common::grants::grants_from;
use zakac_cli_common::lints::attach_lint_args;
use zakac_cli_common::lints::lint_levels;
use zakac_diagnostics::has_errors;
use zakac_i18n::t;
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

use crate::artifact::SemcheckArtifact;
use crate::artifact::check_loaded;
use crate::semcheck::SemcheckOptions;

const SEMCHECK_RAW_EXTENSION: &str = ".zksem";
const SEMCHECK_TEXT_EXTENSION: &str = ".zksemt";
const TARGET_BITS_ARG: &str = "target_bits";
/// Bit counts the platform-sized integer kinds can have.
const PLATFORM_WIDTHS: &[&str] = &["8", "16", "32", "64"];
const TARGET_OS_ARG: &str = "target_os";
const TARGET_ARCH_ARG: &str = "target_arch";
const TARGET_ABI_ARG: &str = "target_abi";

/// The `zakac-semcheck` command, `zakac semcheck` as a subcommand.
pub struct ZakacSemcheck
{
    cmd_instance: clap::Command,
}

/// Configure the check command.
///
/// # Arguments
///
/// * `cmd` - The command to configure.
///
/// # Returns
///
/// The configured command.
fn configure_semcheck_command(cmd: clap::Command) -> clap::Command
{
    let target_bits = Arg::new(TARGET_BITS_ARG)
        .long("target-bits")
        .value_name("BITS")
        .help(t!("semcheck.platform_bits"))
        .value_parser(PLATFORM_WIDTHS.to_vec());
    let cmd = attach_lint_args(attach_common_io_args(cmd).arg(target_bits));
    let cmd = attach_grant_args(cmd, Granted::Comptime);
    attach_target_os_args(cmd).about(t!("semcheck.about"))
}

/// Add `--target-os`, `--target-arch` and `--target-abi`, the OS, the CPU and
/// the ABI a program is built for: the OS and the CPU are the host's unless
/// given, and the ABI is the usual one of the two.
///
/// # Arguments
///
/// * `cmd` - The command.
///
/// # Returns
///
/// The command with the flags.
pub fn attach_target_os_args(cmd: clap::Command) -> clap::Command
{
    let os = Arg::new(TARGET_OS_ARG)
        .long("target-os")
        .value_name("OS")
        .help(t!("semcheck.target_os"))
        .value_parser(TargetOs::VARIANTS.to_vec())
        .default_value(<&'static str>::from(host_os()));
    let arch = Arg::new(TARGET_ARCH_ARG)
        .long("target-arch")
        .value_name("ARCH")
        .help(t!("semcheck.target_arch"))
        .value_parser(TargetArch::VARIANTS.to_vec())
        .default_value(<&'static str>::from(host_arch()));
    let abi = Arg::new(TARGET_ABI_ARG)
        .long("target-abi")
        .value_name("ABI")
        .help(t!("semcheck.target_abi"))
        .value_parser(TargetAbi::VARIANTS.to_vec());
    cmd.arg(os).arg(arch).arg(abi)
}

/// The target a command builds for, from `--target-os`, `--target-arch`,
/// `--target-abi` and `--platform-bits`.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `output` - The code the command gives out.
///
/// # Returns
///
/// The target; the host's facts for a flag the command lacks.
pub fn target_from(matches: &ArgMatches, output: BuildTarget) -> Target
{
    let word = |id: &str| matches.try_get_one::<String>(id).ok().flatten();
    let os = word(TARGET_OS_ARG).and_then(|w| w.parse().ok()).unwrap_or_else(host_os);
    let abi = word(TARGET_ABI_ARG).and_then(|w| w.parse().ok());
    let (arch, bits) = (target_arch(matches), platform_bits(matches));
    #[rustfmt::skip]
    return Target::builder().os(os).arch(arch).maybe_abi(abi).bits(bits).output(output)
        .build();
}

/// Whether a flag was given on the command line, not left at its default.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `id` - The flag.
///
/// # Returns
///
/// `true` when the user gave it.
fn given(matches: &ArgMatches, id: &str) -> bool
{
    let source =
        matches.try_contains_id(id).ok().and_then(|_| matches.value_source(id));
    source.is_some_and(|s| s != ValueSource::DefaultValue)
}

/// The CPU a command builds for, from `--target-arch`: WASI runs 32-bit
/// WebAssembly alone, so `--target-os wasi` with no CPU given takes `wasm32`.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
///
/// # Returns
///
/// The CPU; the host's for a command that lacks the flag.
fn target_arch(matches: &ArgMatches) -> TargetArch
{
    let word = |id: &str| matches.try_get_one::<String>(id).ok().flatten();
    let wasi = word(TARGET_OS_ARG).is_some_and(|w| w == "wasi");
    ret_if!(wasi && !given(matches, TARGET_ARCH_ARG), TargetArch::Wasm32);
    word(TARGET_ARCH_ARG).and_then(|w| w.parse().ok()).unwrap_or_else(host_arch)
}

/// The bit count of the platform-sized kinds, from `--target-bits` or a config
/// file: 32 for a WebAssembly CPU and 64 for the others, unless given. The flag
/// has no default of its own, so that its help names no fixed width.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
///
/// # Returns
///
/// The bit count.
pub fn platform_bits(matches: &ArgMatches) -> u16
{
    let word = matches.try_get_one::<String>(TARGET_BITS_ARG).ok().flatten();
    let usual = tern!(target_arch(matches) == TargetArch::Wasm32, 32, 64);
    word.and_then(|b| b.parse().ok()).unwrap_or(usual)
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
    let bits = platform_bits(matches);
    // The comptime runs of the command take the rights its flags grant.
    crate::comptime_calls::set_grants(grants_from(matches, Granted::Comptime));
    #[rustfmt::skip]
    return SemcheckOptions::builder().platform_bits(bits).lint(lint_levels(matches))
        .build();
}

impl Command for ZakacSemcheck
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

    /// Get the section of a config file that holds the defaults of this
    /// command.
    ///
    /// # Returns
    ///
    /// The name of its `zakac` subcommand.
    fn config_section() -> Option<&'static str> { Some("semcheck") }

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
        setup.with(configure_semcheck_command)
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

/// Output capabilities for the check phase.
fn semcheck_output_caps() -> PhaseArtifactOutput
{
    PhaseArtifactOutput::builder()
        .formats(&[ArtifactFormat::Text, ArtifactFormat::Raw])
        .default_format(ArtifactFormat::Raw)
        .raw_extension(SEMCHECK_RAW_EXTENSION)
        .text_extension(SEMCHECK_TEXT_EXTENSION)
        .build()
}

/// The main function for this "zakac_semcheck" command.
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
    _cmd: &ZakacSemcheck, matches: &ArgMatches, io: &mut CommandIO<'_>,
) -> ExitCode
{
    let cmd_info = CommandInfo::new(ZakacSemcheck::name(), ZakacSemcheck::version());

    let input: Input =
        Input::from(matches.get_one::<String>("input").map(|s| s.as_str()));
    let output_arg = matches.get_one::<String>("output").map(|s| s.as_str());
    let caps = semcheck_output_caps();
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
    let loaded = crate::open_chains::load_settled(
        &source,
        &mut source_map,
        &LexerOptions::default(),
        &target_from(matches, BuildTarget::Vm),
        &semcheck_options(matches),
    );
    ret_if!(let Err(error) = loaded, report_command_error(io, &cmd_info, error));
    let loaded = loaded.unwrap();

    let options = semcheck_options(matches);
    let checked = check_loaded(loaded, &options);
    let failed = has_errors(&checked.diagnostics);
    let _ = write_diagnostics(io.stderr, &source_map, &checked.diagnostics);
    let artifact = SemcheckArtifact::from_checked(&source, checked);

    let mut output = resolved.output;
    let result = dump_artifact(&mut output, io.stdout, &artifact, resolved.format);
    ret_if!(let Err(error) = result, report_command_error(io, &cmd_info, error));
    tern!(failed, ExitCode::Failure, ExitCode::Success)
}
