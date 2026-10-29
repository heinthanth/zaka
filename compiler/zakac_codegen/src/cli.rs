use std::path::PathBuf;
use std::sync::Arc;

use zakac_borrow_checker::cli::attach_borrow_check_args;
use zakac_borrow_checker::cli::borrow_check_options;
use zakac_cli_common::artifact_output::PhaseArtifactOutput;
use zakac_cli_common::artifact_output::ResolvedArtifactOutput;
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
use zakac_session::config::Output;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

use crate::artifact::CodegenArtifact;
use crate::artifact::Emitted;
use crate::artifact::run_phases;
use crate::listing::listing;

/// Extension of a raw artifact.
pub const CODEGEN_RAW_EXTENSION: &str = ".zkbc";
/// Extension of a text artifact.
pub const CODEGEN_TEXT_EXTENSION: &str = ".zkbct";
/// The flag that writes the listing.
const ASM_ARG: &str = "asm";

/// The `zakac_codegen` command, `zakac codegen` (or `zakac mkbc`) as a
/// subcommand.
pub struct ZakacCodegen
{
    /// The clap command.
    cmd_instance: clap::Command,
}

/// Configure the code generation command.
///
/// # Arguments
///
/// * `cmd` - The command to configure.
///
/// # Returns
///
/// The configured command.
fn configure_codegen_command(cmd: clap::Command) -> clap::Command
{
    let asm = Arg::new(ASM_ARG)
        .long("asm")
        .help(t!("codegen.asm"))
        .action(ArgAction::SetTrue);
    let cmd = attach_borrow_check_args(attach_compile_args(attach_common_io_args(cmd)));
    cmd.arg(asm).about(t!("codegen.about"))
}

impl Command for ZakacCodegen
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
    fn aliases() -> &'static [&'static str] { &["mkbc"] }

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
        setup.with(configure_codegen_command)
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

/// Output capabilities for the code generation phase.
///
/// # Returns
///
/// The capabilities.
pub fn codegen_output_caps() -> PhaseArtifactOutput
{
    PhaseArtifactOutput::builder()
        .formats(&[ArtifactFormat::Text, ArtifactFormat::Raw])
        .default_format(ArtifactFormat::Raw)
        .raw_extension(CODEGEN_RAW_EXTENSION)
        .text_extension(CODEGEN_TEXT_EXTENSION)
        .build()
}

/// Generate ops from a source file and write the artifact, or the listing
/// with `--asm`. The artifact is written even when a phase reports errors;
/// the diagnostics go to stderr and the exit code is then
/// [`ExitCode::Failure`].
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
    let cmd_info = CommandInfo::new(ZakacCodegen::name(), ZakacCodegen::version());

    let input: Input =
        Input::from(matches.get_one::<String>("input").map(|s| s.as_str()));
    let output_arg = matches.get_one::<String>("output").map(|s| s.as_str());
    let caps = codegen_output_caps();
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
    let emitted = run_phases(&source, &lexer, &options, &folding, &mem);
    let failed = has_errors(emitted.diagnostics());
    let _ = write_diagnostics(io.stderr, &source_map, emitted.diagnostics());

    let asm = matches.get_flag(ASM_ARG);
    let result = tern!(
        asm,
        write_listing(io, output_arg, &emitted),
        write_artifact(io, resolved, &source, emitted)
    );
    ret_if!(let Err(error) = result, report_command_error(io, &cmd_info, error));
    tern!(failed, ExitCode::Failure, ExitCode::Success)
}

/// Write the listing of the generated ops.
///
/// # Arguments
///
/// * `io` - The standard streams.
/// * `output_arg` - The output path, or `None` for stdout.
/// * `emitted` - The output of the phases.
///
/// # Returns
///
/// The result of the write.
fn write_listing(
    io: &mut CommandIO<'_>, output_arg: Option<&str>, emitted: &Emitted,
) -> zakac_host_errors::Result<()>
{
    let text = emitted.module.as_ref().map(listing).unwrap_or_default();
    let path = output_arg.map(PathBuf::from);
    let mut output = path.map_or(Output::Stdout, Output::File);
    output.write_text(io.stdout, &text)
}

/// Write the artifact.
///
/// # Arguments
///
/// * `io` - The standard streams.
/// * `resolved` - Where and how to write it.
/// * `source` - The source file.
/// * `emitted` - The output of the phases.
///
/// # Returns
///
/// The result of the write.
fn write_artifact(
    io: &mut CommandIO<'_>, resolved: ResolvedArtifactOutput, source: &SourceFile,
    emitted: Emitted,
) -> zakac_host_errors::Result<()>
{
    let artifact = CodegenArtifact::from_emitted(source, emitted);
    let mut output = resolved.output;
    dump_artifact(&mut output, io.stdout, &artifact, resolved.format)
}
