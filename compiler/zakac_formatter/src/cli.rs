//! The `zakac fmt` command and the `zakafmt` executable.

use std::path::Path;
use std::path::PathBuf;

use strum::VariantNames;
use zakac_cli_common::clap;
use zakac_cli_common::clap::Arg;
use zakac_cli_common::clap::ArgAction;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::clap::builder::PossibleValuesParser;
use zakac_cli_common::clap::crate_name;
use zakac_cli_common::clap_utils::CommandInfo;
use zakac_cli_common::clap_utils::format_command_error;
use zakac_cli_common::clap_utils::report_command_error;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_cli_common::config::nearest_local;
use zakac_cli_common::config::parse_text;
use zakac_cli_common::config::set_defaults;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_i18n::t;
use zakac_reporter::render::write_diagnostics;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

use crate::format::FmtError;
use crate::format::format;
use crate::options::ContinuationIndent;
use crate::options::Digits;
use crate::options::FmtOptions;
use crate::options::ImportOrder;
use crate::options::SpeechMode;

const PATHS_ARG: &str = "paths";
const CHECK_ARG: &str = "check";
const INDENT_ARG: &str = "indent";
const CONTINUATION_ARG: &str = "continuation-indent";
const BLANK_ARG: &str = "blank-lines-max";
const ONE_STATEMENT_ARG: &str = "one-statement-per-line";
const MARK_ORDER_ARG: &str = "mark-order";
const SPEECH_ARG: &str = "speech-mode";
const DIGITS_ARG: &str = "digits";
const IMPORT_ARG: &str = "import-order";

/// The `zakac fmt` command.
pub struct ZakacFormatter
{
    cmd_instance: clap::Command,
}

/// A flag that takes a value, with its default.
///
/// # Arguments
///
/// * `name` - The long name, which is also the key of the config file.
/// * `value_name` - The name of the value in the help.
/// * `default` - The default value.
/// * `help` - The help text.
///
/// # Returns
///
/// The flag.
fn setting(
    name: &'static str, value_name: &'static str, default: &'static str, help: String,
) -> Arg
{
    Arg::new(name).long(name).value_name(value_name).default_value(default).help(help)
}

/// Configure the formatter command.
///
/// # Arguments
///
/// * `cmd` - The command to configure.
///
/// # Returns
///
/// The configured command.
fn configure_fmt_command(cmd: clap::Command) -> clap::Command
{
    let paths = Arg::new(PATHS_ARG)
        .value_name("PATH")
        .num_args(0..)
        .help(t!("formatter.paths"));
    let check = Arg::new(CHECK_ARG)
        .long(CHECK_ARG)
        .action(ArgAction::SetTrue)
        .help(t!("formatter.check"));
    let continuation =
        |s: &str| s.parse::<ContinuationIndent>().map_err(|e| e.to_string());
    let settings = [
        setting(INDENT_ARG, "SPACES", "0", t!("formatter.indent").into())
            .value_parser(clap::value_parser!(usize)),
        setting(
            CONTINUATION_ARG,
            "SPACES",
            "preserve",
            t!("formatter.continuation_indent").into(),
        )
        .value_parser(continuation),
        setting(BLANK_ARG, "LINES", "1", t!("formatter.blank_lines_max").into())
            .value_parser(clap::value_parser!(usize)),
        setting(
            ONE_STATEMENT_ARG,
            "BOOL",
            "true",
            t!("formatter.one_statement_per_line").into(),
        )
        .value_parser(clap::value_parser!(bool)),
        setting(MARK_ORDER_ARG, "BOOL", "true", t!("formatter.mark_order").into())
            .value_parser(clap::value_parser!(bool)),
        setting(SPEECH_ARG, "MODE", "written", t!("formatter.speech_mode").into())
            .value_parser(PossibleValuesParser::new(SpeechMode::VARIANTS)),
        setting(DIGITS_ARG, "SCRIPT", "preserve", t!("formatter.digits").into())
            .value_parser(PossibleValuesParser::new(Digits::VARIANTS)),
        setting(IMPORT_ARG, "ORDER", "preserve", t!("formatter.import_order").into())
            .value_parser(PossibleValuesParser::new(ImportOrder::VARIANTS)),
    ];
    cmd.about(t!("formatter.about")).arg(paths).arg(check).args(settings)
}

/// The settings the flags and the config files give.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
///
/// # Returns
///
/// The settings.
pub fn options_from(matches: &ArgMatches) -> FmtOptions
{
    let count = |id: &str| matches.get_one::<usize>(id).copied().unwrap_or_default();
    let flag = |id: &str| matches.get_one::<bool>(id).copied().unwrap_or(true);
    let word = |id: &str| matches.get_one::<String>(id).map_or("", String::as_str);
    let continuation = matches.get_one::<ContinuationIndent>(CONTINUATION_ARG).copied();
    FmtOptions::builder()
        .indent(count(INDENT_ARG))
        .continuation_indent(continuation.unwrap_or_default())
        .blank_lines_max(count(BLANK_ARG))
        .one_statement_per_line(flag(ONE_STATEMENT_ARG))
        .mark_order(flag(MARK_ORDER_ARG))
        .speech_mode(word(SPEECH_ARG).parse().unwrap_or_default())
        .digits(word(DIGITS_ARG).parse().unwrap_or_default())
        .import_order(word(IMPORT_ARG).parse().unwrap_or_default())
        .build()
}

impl Command for ZakacFormatter
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
    fn standalone_name() -> &'static str { "zakafmt" }

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
    /// command when it runs as an executable of its own.
    ///
    /// # Returns
    ///
    /// The section.
    fn config_section() -> Option<&'static str> { Some("fmt") }

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
        setup.with(configure_fmt_command)
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
        command_main(matches, io)
    }
}

/// What one run formats with.
struct Run<'a, 'io>
{
    io: &'a mut CommandIO<'io>,
    options: FmtOptions,
    /// Write nothing; list the files that would change.
    check: bool,
    info: CommandInfo,
}

/// The main function of the formatter: format each file the paths name, or
/// standard input when no path is given.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `io` - The I/O configuration for the command.
///
/// # Returns
///
/// The exit code: a failure when a file could not be formatted, or, with
/// `--check`, when a file would change.
fn command_main(matches: &ArgMatches, io: &mut CommandIO<'_>) -> ExitCode
{
    let paths: Vec<&String> =
        matches.get_many::<String>(PATHS_ARG).into_iter().flatten().collect();
    let info = CommandInfo::new(ZakacFormatter::name(), ZakacFormatter::version());
    let check = matches.get_flag(CHECK_ARG);
    let mut run = Run { io, options: options_from(matches), check, info };
    let piped = paths.is_empty() || paths.iter().all(|p| p.as_str() == "-");
    ret_if!(piped, run.format_stdin());
    let mut code = ExitCode::Success;
    for path in paths.iter().filter(|p| p.as_str() != "-")
    {
        let found = zaka_files(Path::new(path));
        let codes = found.iter().map(|f| run.format_path(f)).collect::<Vec<_>>();
        let failed = codes.iter().find(|c| **c != ExitCode::Success);
        failed.into_iter().for_each(|c| code = *c);
    }
    code
}

impl Run<'_, '_>
{
    /// Format standard input to standard output.
    ///
    /// # Returns
    ///
    /// The exit code.
    fn format_stdin(&mut self) -> ExitCode
    {
        let mut text = String::new();
        let read = self.io.stdin.read_to_string(&mut text);
        let unread = |e| {
            Error::from_io(
                e,
                IOErrorContext::new(PathBuf::from("-"), IOErrorAction::Read),
            )
        };
        ret_if!(let Err(e) = read, report_command_error(self.io, &self.info, unread(e)));
        let mut map = SourceMap::new();
        let out = format(&mut map, FileName::Stdin, &text, &self.options);
        let out = match out
        {
            Ok(out) => out,
            Err(e) => return self.report(&map, "-", e),
        };
        ret_if!(self.check, tern!(out == text, ExitCode::Success, ExitCode::Failure));
        let _ = self.io.stdout.write_all(out.as_bytes());
        ExitCode::Success
    }

    /// Format a file in place.
    ///
    /// # Arguments
    ///
    /// * `path` - The file.
    ///
    /// # Returns
    ///
    /// The exit code.
    fn format_path(&mut self, path: &Path) -> ExitCode
    {
        let action = |a| IOErrorContext::new(path.to_path_buf(), a);
        let text = std::fs::read_to_string(path);
        let text = match text
        {
            Ok(text) => text,
            Err(e) => return self.fail(Error::from_io(e, action(IOErrorAction::Read))),
        };
        let mut map = SourceMap::new();
        let out =
            format(&mut map, FileName::real(path.to_path_buf()), &text, &self.options);
        let shown = shown(path);
        let out = match out
        {
            Ok(out) => out,
            Err(e) => return self.report(&map, &shown, e),
        };
        ret_if!(out == text, ExitCode::Success);
        if self.check
        {
            let _ = writeln!(self.io.stdout, "{shown}");
            return ExitCode::Failure;
        }
        let written = std::fs::write(path, out);
        written.map_or_else(
            |e| self.fail(Error::from_io(e, action(IOErrorAction::Write))),
            |_| ExitCode::Success,
        )
    }

    /// Report why a file was left as it is.
    ///
    /// # Arguments
    ///
    /// * `map` - The source map the file was read into.
    /// * `shown` - The name of the file as messages show it.
    /// * `error` - The reason.
    ///
    /// # Returns
    ///
    /// The exit code.
    fn report(&mut self, map: &SourceMap, shown: &str, error: FmtError) -> ExitCode
    {
        match error
        {
            FmtError::Unparsed(diags) =>
            {
                let _ = write_diagnostics(self.io.stderr, map, &diags);
                ExitCode::Failure
            }
            FmtError::Changed =>
            {
                let msg = format_command_error(
                    &self.info,
                    t!("formatter.changed", file = shown),
                );
                let _ = writeln!(self.io.stderr, "{msg}");
                ExitCode::Failure
            }
            FmtError::Unread(e) => self.fail(e),
        }
    }

    /// Report an error of the disk.
    ///
    /// # Arguments
    ///
    /// * `error` - The error.
    ///
    /// # Returns
    ///
    /// The exit code.
    fn fail(&mut self, error: Error) -> ExitCode
    {
        report_command_error(self.io, &self.info, error)
    }
}

/// A path as messages show it, with `/` between its parts on each host.
///
/// # Arguments
///
/// * `path` - The path.
///
/// # Returns
///
/// The text.
fn shown(path: &Path) -> String
{
    path.display().to_string().replace(std::path::MAIN_SEPARATOR, "/")
}

/// The files a path names: the path itself when it is a file, or each `.zaka`
/// file of a folder and its subfolders, sorted.
///
/// # Arguments
///
/// * `path` - The path.
///
/// # Returns
///
/// The files.
pub fn zaka_files(path: &Path) -> Vec<PathBuf>
{
    ret_if!(!path.is_dir(), vec![path.to_path_buf()]);
    let entries =
        std::fs::read_dir(path).into_iter().flatten().flatten().map(|e| e.path());
    let is_zaka =
        |p: &PathBuf| p.is_dir() || p.extension().is_some_and(|e| e == "zaka");
    let mut found: Vec<PathBuf> =
        entries.filter(is_zaka).flat_map(|p| zaka_files(&p)).collect();
    found.sort();
    found
}

/// The settings of the `[fmt]` section of the `zakac.toml` nearest a folder,
/// as the language server reads them: the defaults for each setting the file
/// leaves out, or for a file that cannot be read.
///
/// # Arguments
///
/// * `dir` - The folder of the file being formatted.
///
/// # Returns
///
/// The settings.
pub fn project_options(dir: &Path) -> FmtOptions
{
    let table = nearest_local(dir)
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| parse_text(&text, "").ok());
    let cmd = configure_fmt_command(clap::Command::new("fmt"));
    let cmd = table.map_or(cmd.clone(), |t| set_defaults(cmd, &t, Some("fmt")));
    let matches = cmd.try_get_matches_from(["fmt"]);
    matches.map(|m| options_from(&m)).unwrap_or_default()
}
