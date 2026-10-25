use std::sync::Arc;

use zakac_cli_common::clap;
use zakac_cli_common::clap::Arg;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::clap::crate_name;
use zakac_cli_common::clap::value_parser;
use zakac_cli_common::clap_utils::CommandInfo;
use zakac_cli_common::clap_utils::report_command_error;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_codegen::artifact::CodegenArtifact;
use zakac_codegen::artifact::TEXT_HEADER;
use zakac_codegen::artifact::decode_artifact;
use zakac_codegen::artifact::run_phases;
use zakac_codegen::bc::Module;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::has_errors;
use zakac_host_errors::Error;
use zakac_i18n::t;
use zakac_ir_builder::cli::attach_compile_args;
use zakac_ir_builder::cli::fold_options;
use zakac_ir_builder::cli::semcheck_options;
use zakac_lexer::lexer::LexerOptions;
use zakac_reporter::render::write_diagnostics;
use zakac_rust_sugars::ret_if;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::RAW_ARTIFACT_MAGIC;
use zakac_session::artifact::load_artifact;
use zakac_session::config::Input;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

use crate::machine::RunOptions;
use crate::machine::run_with;

/// The flag for the call depth limit.
const MAX_DEPTH_ARG: &str = "max_depth";
/// The flag for the op budget.
const MAX_STEPS_ARG: &str = "max_steps";

/// The `zaka_vm` command, `zakac run` as a subcommand.
pub struct ZakaVm
{
    /// The clap command.
    cmd_instance: clap::Command,
}

/// Configure the run command.
///
/// # Arguments
///
/// * `cmd` - The command to configure.
///
/// # Returns
///
/// The configured command.
fn configure_vm_command(cmd: clap::Command) -> clap::Command
{
    let input = Arg::new("input").value_name("INPUT").help(t!("vm.about"));
    let depth = Arg::new(MAX_DEPTH_ARG)
        .long("max-depth")
        .value_name("CALLS")
        .help(t!("vm.max_depth"))
        .value_parser(value_parser!(u32).range(1..))
        .default_value("4096");
    let steps = Arg::new(MAX_STEPS_ARG)
        .long("max-steps")
        .value_name("OPS")
        .help(t!("vm.max_steps"))
        .value_parser(value_parser!(u64).range(1..));
    attach_compile_args(cmd.arg(input)).arg(depth).arg(steps).about(t!("vm.about"))
}

impl Command for ZakaVm
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
        setup.with(configure_vm_command)
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

/// The ops to run and the diagnostics that came with them.
struct Loaded
{
    /// The ops, or `None` when a phase reported an error.
    module: Option<Module>,
    /// The diagnostics, with absolute spans.
    diagnostics: Vec<Diagnostic>,
}

/// Load the program, run it and write its output to stdout. A source file
/// is compiled first; a codegen artifact, raw or text, runs as it is.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `io` - The standard streams.
///
/// # Returns
///
/// The exit code: [`ExitCode::Failure`] for compile errors or a trap.
fn command_main(matches: &ArgMatches, io: &mut CommandIO<'_>) -> ExitCode
{
    let cmd_info = CommandInfo::new(ZakaVm::name(), ZakaVm::version());
    let input = Input::from(matches.get_one::<String>("input").map(|s| s.as_str()));
    let mut source_map = SourceMap::new();
    let loaded = load(matches, &input, io, &mut source_map);
    ret_if!(let Err(error) = loaded, report_command_error(io, &cmd_info, error));
    let Loaded { module, diagnostics } = loaded.unwrap();
    let _ = write_diagnostics(io.stderr, &source_map, &diagnostics);
    let module = module.filter(|_| !has_errors(&diagnostics));
    let Some(module) = module
    else
    {
        return ExitCode::Failure;
    };

    let options = run_options(matches);
    let result = run_with(&module, &mut *io.stdout, &options);
    let Err(trap) = result
    else
    {
        return ExitCode::Success;
    };
    let _ = report_command_error(
        io,
        &cmd_info,
        Error::RuntimeTrap { msg: trap.to_string() },
    );
    ExitCode::Failure
}

/// The run limits from the command line.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
///
/// # Returns
///
/// The options.
fn run_options(matches: &ArgMatches) -> RunOptions
{
    let depth = matches.get_one::<u32>(MAX_DEPTH_ARG).copied().unwrap_or(4096);
    let steps = matches.get_one::<u64>(MAX_STEPS_ARG).copied();
    RunOptions::builder().max_depth(depth).maybe_max_steps(steps).build()
}

/// Read the input and turn it into ops: decode a codegen artifact, or
/// compile source text.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `input` - The input.
/// * `io` - The standard streams.
/// * `source_map` - The source map.
///
/// # Returns
///
/// The ops and diagnostics, or a host error.
fn load(
    matches: &ArgMatches, input: &Input, io: &mut CommandIO<'_>,
    source_map: &mut SourceMap,
) -> zakac_host_errors::Result<Loaded>
{
    let (_, content) = input.read(io.stdin)?;
    let raw = content.starts_with(RAW_ARTIFACT_MAGIC);
    ret_if!(raw, decoded(&content, input, ArtifactFormat::Raw, source_map));
    let text = content.starts_with(TEXT_HEADER.as_bytes());
    ret_if!(text, decoded(&content, input, ArtifactFormat::Text, source_map));
    let again = match input
    {
        Input::Stdin => Input::Data { name: "<stdin>".to_string(), content },
        other => other.clone(),
    };
    let source = load_artifact::<Arc<SourceFile>>(
        &again,
        ArtifactFormat::Text,
        io.stdin,
        source_map,
    )?;
    Ok(compiled(matches, &source))
}

/// Decode a codegen artifact and register its source for diagnostics.
///
/// # Arguments
///
/// * `content` - The encoded artifact.
/// * `input` - The input, for error paths.
/// * `format` - The format.
/// * `source_map` - The source map.
///
/// # Returns
///
/// The ops and diagnostics, or a decode error.
fn decoded(
    content: &[u8], input: &Input, format: ArtifactFormat, source_map: &mut SourceMap,
) -> zakac_host_errors::Result<Loaded>
{
    let artifact: CodegenArtifact =
        decode_artifact(content, &input.error_path(), format)?;
    let file = artifact.register_source(source_map);
    let diagnostics = artifact.absolute_diagnostics(&file);
    Ok(Loaded { module: artifact.module, diagnostics })
}

/// Compile source text to ops.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `source` - The source file.
///
/// # Returns
///
/// The ops and diagnostics.
fn compiled(matches: &ArgMatches, source: &SourceFile) -> Loaded
{
    let options = semcheck_options(matches);
    let folding = fold_options(matches);
    let emitted = run_phases(source, &LexerOptions::default(), &options, &folding);
    let diagnostics = emitted.diagnostics().to_vec();
    Loaded { module: emitted.module, diagnostics }
}
