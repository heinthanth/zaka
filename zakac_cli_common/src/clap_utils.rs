use chrono::Datelike;
use chrono::Utc;
use clap::Arg;
use clap::ArgAction;
use clap::ArgMatches;
use termcolor::WriteColor;
use zakac_consts::ZAKA_BUILD_NUMBER;
use zakac_consts::ZAKA_LICENSE;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

use crate::clap_error_handler::handle_args_error;
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
    O: WriteColor,
    E: WriteColor,
{
    let cmd_info = command_info(cmd, &args);
    cmd.clone()
        .try_get_matches_from(args)
        .map_err(|error| handle_args_error(error, &cmd, cmd_info, stdout, stderr))
}

/// The information about the current command or subcommand.
pub struct CommandInfo
{
    /// The command or subcommand name.
    pub name: String,
    /// The command or subcommand version.
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
        Self { name: name.to_string(), version: version.to_string() }
    }
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
    CommandInfo { name: component, version }
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

/// Helper function to create a [`clap::Command`] with shared help/version flags
/// and a localized help template.
///
/// # Arguments
///
/// * `name` - The name used to execute the command.
/// * `version` - Command version.
/// * `canonical_name` - The canonical name of the command used in help,
///   version, and errors.
/// * `aliases` - The aliases of the command.
///
/// # Returns
///
/// A command with shared help/version flags and a localized help template.
pub fn create_clap_command(
    name: &'static str, version: &'static str, canonical_name: Option<&'static str>,
    aliases: Option<&[&'static str]>,
) -> clap::Command
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

    let mut cmd = clap::Command::new(name)
        .version(version)
        .propagate_version(true)
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .arg(g_help_flag)
        .arg(g_version_flag);

    do_if!(let Some(canonical_name) = canonical_name, cmd = cmd.display_name(canonical_name));
    do_if!(let Some(aliases) = aliases, cmd = cmd.aliases(aliases));

    let help = create_help_template(&cmd);
    cmd.help_template(help)
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
fn create_help_template(cmd: &clap::Command) -> String
{
    let name = component_name(cmd);
    let version = cmd.get_version().unwrap_or("0.0.0");

    let header = create_version_string(name, version);

    let subcmd_piece = tern!(
        cmd.get_subcommands().count() > 0,
        format!("\n\n{}\n{{subcommands}}", t!("cli.subcommands_header")),
        "".into()
    );
    let option_piece = tern!(
        cmd.get_opts().count() > 0,
        format!("\n\n{}\n{{options}}", t!("cli.options_header")),
        "".into()
    );
    let positional_piece = tern!(
        cmd.get_positionals().count() > 0,
        format!("\n\n{}\n{{positionals}}", t!("cli.positionals_header")),
        "".into()
    );
    let about_piece = tern!(
        let Some(about) = cmd.get_about(),
        format!("{}\n{{tab}}{about}\n\n", t!("cli.about_header")),
        "".into()
    );

    let usage_header = t!("cli.usage_header");
    #[rustfmt::skip]
    let help_output = format!(
        "{header}\n\n{about_piece}{usage_header}
{{tab}}{{usage}}{subcmd_piece}{option_piece}{positional_piece}");

    help_output
}
