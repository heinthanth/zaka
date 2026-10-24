use bon::Builder;
use chrono::Datelike;
use chrono::Utc;
use clap::Arg;
use clap::ArgMatches;
use termcolor::WriteColor;
use zakac_consts::ZAKA_BUILD_NUMBER;
use zakac_consts::ZAKA_LICENSE;
use zakac_host_errors::Error;
use zakac_i18n::t;
use zakac_reporter::writer::write_line;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

use crate::clap_error_handler::handle_args_error;
use crate::command::CommandIO;
use crate::command::ExitCode;

/// Parse the command line arguments.
///
/// # Arguments
///
/// * `cmd` - The command to parse arguments against.
/// * `args` - The command line arguments to parse.
/// * `stdout` - The writer to use for version output.
/// * `stderr` - The writer to use for error output.
///
/// # Returns
///
/// The parsed argument matches, or an exit code if parsing fails.
pub fn parse_cli_args<O, E>(
    cmd: &clap::Command, args: &[String], stdout: &mut O, stderr: &mut E,
) -> Result<ArgMatches, ExitCode>
where
    O: WriteColor + ?Sized,
    E: WriteColor + ?Sized,
{
    let cmd_info = command_info(cmd, &args);
    cmd.clone()
        .try_get_matches_from(args)
        .map_err(|error| handle_args_error(error, &cmd, cmd_info, stdout, stderr))
}

/// The information about the current command or subcommand.
#[derive(Builder)]
pub struct CommandInfo
{
    /// The command or subcommand name.
    #[builder(into)]
    pub name: String,
    /// The command or subcommand version.
    #[builder(into)]
    pub version: String,
}

impl CommandInfo
{
    /// Create a new CommandInfo.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the command or subcommand.
    /// * `version` - The version of the command or subcommand.
    ///
    /// # Returns
    ///
    /// A new CommandInfo.
    pub fn new(name: &'static str, version: &'static str) -> Self
    {
        Self::builder().name(name).version(version).build()
    }
}

/// Format a command error using the shared CLI error template.
///
/// # Arguments
///
/// * `cmd_info` - The command or subcommand information.
/// * `msg` - The error message content.
///
/// # Returns
///
/// A localized `name: msg` string.
pub fn format_command_error(cmd_info: &CommandInfo, msg: impl AsRef<str>) -> String
{
    t!("cli_errors.template", name = cmd_info.name, msg = msg.as_ref()).to_string()
}

/// Write a localized operational error for a command to stderr.
///
/// # Arguments
///
/// * `io` - Command I/O streams.
/// * `cmd_info` - Command name and version for the error prefix.
/// * `error` - Operational error to report.
///
/// # Returns
///
/// [`ExitCode::IoError`].
pub fn report_command_error(
    io: &mut CommandIO<'_>, cmd_info: &CommandInfo, error: Error,
) -> ExitCode
{
    let msg = format_command_error(cmd_info, error.to_string());
    let _ = write_line(io.stderr, &msg);
    ExitCode::IoError
}

/// Get the component name and version from a command.
///
/// # Arguments
///
/// * `cmd` - The command to get the command or subcommand information from.
/// * `args` - The arguments to get the command or subcommand information from.
///
/// # Returns
///
/// The command or subcommand information.
pub(crate) fn command_info(cmd: &clap::Command, args: &[String]) -> CommandInfo
{
    let active = active_command(cmd, args);
    let component = component_name(active).to_string();
    let version = active.get_version().unwrap_or("0.0.0").to_string();
    CommandInfo::builder().name(component).version(version).build()
}

/// Resolve the deepest subcommand referenced before a terminator
/// flag.
///
/// # Arguments
///
/// * `cmd` - The command to get the active command from.
/// * `args` - The arguments to get the active command from.
///
/// # Returns
///
/// The active command.
fn active_command<'a>(mut cmd: &'a clap::Command, args: &[String])
-> &'a clap::Command
{
    let terminator_flags = ["-h", "--help", "-V", "--version", "--"];

    for arg in args.iter().skip(1)
    {
        do_if!(terminator_flags.contains(&arg.as_str()), break);
        tern!(let Some(sub) = cmd.find_subcommand(arg), cmd = sub, break);
    }
    cmd
}

/// Create a version string for a command.
///
/// # Arguments
///
/// * `component` - The component name.
/// * `version` - The component version.
///
/// # Returns
///
/// A localized version string.
pub(crate) fn create_version_string(component: &str, version: &str) -> String
{
    #[rustfmt::skip]
    let version_string = t!("cli.version_header",
        start = "2023", current = Utc::now().year(),
        component = component, version = version,
        license = ZAKA_LICENSE, build = ZAKA_BUILD_NUMBER).to_string();
    version_string
}

/// Attach the common input and output arguments to a command.
///
/// # Arguments
///
/// * `cmd` - The command to attach the input and output arguments to.
///
/// # Returns
///
/// The command with the input and output arguments attached.
pub fn attach_common_io_args(cmd: clap::Command) -> clap::Command
{
    let input = Arg::new("input").value_name("INPUT").help(t!("cli.input"));
    let output = Arg::new("output")
        .short('o')
        .long("output")
        .value_name("FILE")
        .help(t!("cli.output"));
    let output_format = Arg::new("output_format")
        .long("output-format")
        .value_name("FORMAT")
        .help(t!("cli.output_format"))
        .value_parser(["text", "raw"]);

    cmd.arg(input).arg(output).arg(output_format)
}

/// Get the component name from a command.
///
/// # Arguments
///
/// * `cmd` - The command to get the component name from.
///
/// # Returns
///
/// The component name.
pub(crate) fn component_name(cmd: &clap::Command) -> &str
{
    cmd.get_display_name().unwrap_or_else(|| cmd.get_name())
}

/// Create a help template for a command.
///
/// # Arguments
///
/// * `cmd` - The command to create a help template for.
///
/// # Returns
///
/// A help template for a command.
pub(crate) fn create_help_template(cmd: &clap::Command) -> String
{
    let name = component_name(cmd);
    let version = cmd.get_version().unwrap_or("0.0.0");

    let header = create_version_string(name, version);

    let has_subcmds = cmd.get_subcommands().count() > 0;
    let has_options = cmd.get_opts().count() > 0;
    let has_positionals = cmd.get_positionals().count() > 0;
    let subcmd_header = t!("cli.subcommands_header");
    let option_header = t!("cli.options_header");
    let positional_header = t!("cli.positionals_header");
    let subcmd_piece = section(has_subcmds, &subcmd_header, "subcommands");
    let option_piece = section(has_options, &option_header, "options");
    let positional_piece = section(has_positionals, &positional_header, "positionals");
    let about = |a| format!("{}\n{{tab}}{a}\n\n", t!("cli.about_header"));
    let about_piece = cmd.get_about().map(about).unwrap_or_default();

    let usage_header = t!("cli.usage_header");
    #[rustfmt::skip]
    let help_output = format!(
        "{header}\n\n{about_piece}{usage_header}
{{tab}}{{usage}}{subcmd_piece}{option_piece}{positional_piece}");

    help_output
}

/// One section of a help template.
///
/// # Arguments
///
/// * `shown` - Whether the command has entries for the section.
/// * `header` - The section header.
/// * `slot` - The template placeholder that lists the entries.
///
/// # Returns
///
/// The header and placeholder, or a blank string when `shown` is `false`.
fn section(shown: bool, header: &str, slot: &str) -> String
{
    tern!(shown, format!("\n\n{header}\n{{{slot}}}"), String::new())
}
