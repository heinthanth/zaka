use strum::Display;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoEnumIterator;
use strum::IntoStaticStr;
use zakac_borrow_checker::cli::attach_borrow_check_args;
use zakac_cli_common::clap;
use zakac_cli_common::clap::Arg;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::clap::builder::PossibleValuesParser;
use zakac_cli_common::clap::parser::ValueSource;
use zakac_cli_common::clap_utils::CommandInfo;
use zakac_cli_common::clap_utils::report_command_error;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_codegen_target_c::cli as target_c;
use zakac_codegen_target_vm::cli as target_vm;
use zakac_host_errors::Error;
use zakac_i18n::t;
use zakac_ir_builder::cli::attach_compile_args;
use zakac_rust_sugars::ret_if;

/// The flag that picks the target.
const TARGET_ARG: &str = "target";
/// The flag that picks what a build writes; each target has its own words.
const EMIT_ARG: &str = "emit";

/// What a build makes a program for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[derive(Display, EnumIter, EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
enum Target
{
    /// Ops for the Zaka VM.
    #[default]
    Vm,
    /// A native executable, through C.
    C,
}

/// The `zakac build` command: it checks the program, then hands it to the
/// target `--target` names.
pub(crate) struct ZakacBuild
{
    /// The clap command.
    cmd_instance: clap::Command,
}

/// Configure the build command: the flags each target shares, then the
/// flags of each target.
///
/// # Arguments
///
/// * `cmd` - The command to configure.
///
/// # Returns
///
/// The configured command.
fn configure_build(cmd: clap::Command) -> clap::Command
{
    let input = Arg::new("input").value_name("INPUT").help(t!("build.input"));
    let output = Arg::new("output")
        .short('o')
        .long("output")
        .value_name("FILE")
        .help(t!("build.output"));
    let targets: Vec<&'static str> = Target::iter().map(Into::into).collect();
    let target = Arg::new(TARGET_ARG)
        .long("target")
        .value_name("TARGET")
        .help(t!("build.target"))
        .value_parser(PossibleValuesParser::new(targets))
        .default_value(<&'static str>::from(Target::default()));
    let mut emits = target_vm::emit_words();
    emits.extend(target_c::emit_words());
    let emit = Arg::new(EMIT_ARG)
        .long("emit")
        .value_name("WHAT")
        .help(t!("build.emit"))
        .value_parser(PossibleValuesParser::new(emits));
    let cmd = cmd.arg(input).arg(output).arg(target).arg(emit);
    let cmd = attach_borrow_check_args(attach_compile_args(cmd));
    let cmd = target_vm::attach_codegen_args(cmd);
    target_c::attach_target_args(cmd).about(t!("build.about"))
}

/// The first flag given on the command line that does not go with a target:
/// a flag of the other target, or an `--emit` word of the other target.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `target` - The target.
///
/// # Returns
///
/// The flag as the user wrote it, or `None`.
fn misplaced_flag(matches: &ArgMatches, target: Target) -> Option<String>
{
    let (others, own_emits) = match target
    {
        Target::Vm => (target_c::TARGET_ARGS, target_vm::emit_words()),
        Target::C => (target_vm::TARGET_ARGS, target_c::emit_words()),
    };
    let given = |id: &&&str| matches.value_source(id) == Some(ValueSource::CommandLine);
    let flag =
        others.iter().find(given).map(|id| format!("--{}", id.replace('_', "-")));
    let emit = matches.get_one::<String>(EMIT_ARG);
    let emit = emit.filter(|w| !own_emits.contains(&w.as_str()));
    flag.or_else(|| emit.map(|w| format!("--emit {w}")))
}

/// Build a program: refuse a flag of the other target, then run the build
/// of the target.
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
    let info = CommandInfo::new(ZakacBuild::name(), ZakacBuild::version());
    let word = matches.get_one::<String>(TARGET_ARG);
    let target: Target = word.and_then(|w| w.parse().ok()).unwrap_or_default();
    let misplaced = misplaced_flag(matches, target);
    let error = |flag| Error::FlagNotForTarget { flag, target: target.to_string() };
    ret_if!(let Some(flag) = misplaced, report_command_error(io, &info, error(flag)));
    match target
    {
        Target::Vm => target_vm::command_main(matches, io),
        Target::C => target_c::command_main(matches, io),
    }
}

impl Command for ZakacBuild
{
    /// Get the canonical name of the command.
    ///
    /// # Returns
    ///
    /// The canonical name of the command.
    fn name() -> &'static str { "build" }

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
        setup.with(configure_build)
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
