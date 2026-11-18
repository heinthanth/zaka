use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use strum::IntoEnumIterator;
use zakac_ast::target::BuildTarget;
use zakac_ast::target::TargetArch;
use zakac_ast::target::TargetOs;
use zakac_borrow_checker::cli::attach_borrow_check_args;
use zakac_borrow_checker::cli::borrow_check_options;
use zakac_borrow_checker::cli::checked_fixes;
use zakac_cli_common::clap;
use zakac_cli_common::clap::Arg;
use zakac_cli_common::clap::ArgAction;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::clap::builder::PossibleValuesParser;
use zakac_cli_common::clap::crate_name;
use zakac_cli_common::clap::parser::ValueSource;
use zakac_cli_common::clap_utils::CommandInfo;
use zakac_cli_common::clap_utils::format_command_error;
use zakac_cli_common::clap_utils::report_command_error;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_cli_common::grants::Granted;
use zakac_cli_common::grants::attach_grant_args;
use zakac_cli_common::grants::grants_from;
use zakac_diagnostics::has_errors;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_i18n::t;
use zakac_ir_builder::cli::attach_compile_args;
use zakac_ir_builder::cli::fold_options;
use zakac_ir_builder::cli::semcheck_options;
use zakac_lexer::lexer::LexerOptions;
use zakac_reporter::render::write_diagnostics;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::cli::target_from;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::load_artifact;
use zakac_session::config::Input;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

use crate::driver::default_cache;
use crate::driver::translate;
use crate::driver::write_cache_for;
use crate::emit::CFile;
use crate::exports;
use crate::exports::exported;
use crate::ffi::link_libs_for;
use crate::options::Emit;
use crate::options::RunCheck;
use crate::options::RunChecks;
use crate::options::TargetCOptions;
use crate::toolchain;
use crate::toolchain::BuildPlan;
use crate::toolchain::CFLAGS;
use crate::toolchain::LDFLAGS;
use crate::wasi;

/// The flag that names the cache folder.
const CACHE_ARG: &str = "cache";
/// The flag that picks what a build makes.
const EMIT_ARG: &str = "emit";
/// The flag that picks the run-time checks.
const CHECKS_ARG: &str = "checks";
/// The flag that sets the most nested calls before the executable stops.
const MAX_DEPTH_ARG: &str = "max_depth";
/// The flag that caps the heap octets the executable holds at once.
const MAX_HEAP_ARG: &str = "max_heap";
/// The flag of a release build, which leaves out the source line of each
/// statement; `zakac build` gets it from the VM target, which shares it.
const RELEASE_ARG: &str = "release";
/// The flag that names the C compiler.
const CC_ARG: &str = "cc";
/// The flag that names the program that links.
const LINKER_ARG: &str = "linker";
/// The flag that adds a flag for the C compiler.
const CC_FLAG_ARG: &str = "cc_flag";
/// The flag that adds a flag for the linker.
const LD_FLAG_ARG: &str = "ld_flag";
/// The flag that leaves out the default C flags.
const NO_DEFAULT_FLAGS_ARG: &str = "no_default_flags";

/// The `zakac_codegen_target_c` command: the C target on its own.
/// `zakac build --target c` runs the same build.
pub struct ZakacTargetC
{
    /// The clap command.
    cmd_instance: clap::Command,
}

/// The words of each choice of an option, for its parser.
///
/// # Returns
///
/// The words.
fn words<E: IntoEnumIterator + Into<&'static str>>() -> Vec<&'static str>
{
    E::iter().map(Into::into).collect()
}

/// Check one item of `--checks`: a level, or a check with `+` or `-`.
///
/// # Arguments
///
/// * `item` - The item.
///
/// # Returns
///
/// The item, or the error that names it.
fn check_item(item: &str) -> Result<String, String>
{
    let unknown = || Error::UnknownCheck { name: item.into() }.to_string();
    RunChecks::default().with_item(item).map(|_| item.to_string()).ok_or_else(unknown)
}

/// The choices of the C target from the command line: the checks, the
/// limits, and whether the build keeps the source lines.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
///
/// # Returns
///
/// The choices; the overflow mode and the platform width come later.
pub fn target_options(matches: &ArgMatches) -> TargetCOptions
{
    let items = matches.get_many::<String>(CHECKS_ARG).into_iter().flatten();
    let checks = RunChecks::from_items(items.map(String::as_str)).unwrap_or_default();
    let depth = matches.get_one::<u32>(MAX_DEPTH_ARG).copied().unwrap_or(4096);
    let heap = matches.get_one::<u64>(MAX_HEAP_ARG).copied().unwrap_or(0);
    let release =
        matches.try_get_one::<bool>(RELEASE_ARG).ok().flatten() == Some(&true);
    #[rustfmt::skip]
    let options = TargetCOptions::builder().checks(checks).max_depth(depth).max_heap(heap)
        .debug_info(!release).build();
    options
}

/// Configure the build command.
///
/// # Arguments
///
/// * `cmd` - The command to configure.
///
/// # Returns
///
/// The configured command.
fn configure_build_command(cmd: clap::Command) -> clap::Command
{
    let input = Arg::new("input").value_name("INPUT").help(t!("target_c.input"));
    let output = Arg::new("output")
        .short('o')
        .long("output")
        .value_name("FILE")
        .help(t!("target_c.output"));
    let emit = Arg::new(EMIT_ARG)
        .long("emit")
        .value_name("WHAT")
        .help(t!("target_c.emit"))
        .value_parser(PossibleValuesParser::new(emit_words()));
    let release = Arg::new(RELEASE_ARG)
        .long("release")
        .help(t!("target_c.release"))
        .action(ArgAction::SetTrue);
    let cmd = cmd.arg(input).arg(output).arg(emit).arg(release);
    attach_borrow_check_args(attach_compile_args(attach_target_args(cmd)))
        .about(t!("target_c.about"))
}

/// The words of `--emit` for the C target.
///
/// # Returns
///
/// The words.
pub fn emit_words() -> Vec<&'static str> { words::<Emit>() }

/// The ids of the flags that belong to the C target alone, for a build that
/// names another target.
pub const TARGET_ARGS: &[&str] = &[
    CACHE_ARG,
    CHECKS_ARG,
    MAX_DEPTH_ARG,
    MAX_HEAP_ARG,
    CC_ARG,
    LINKER_ARG,
    CC_FLAG_ARG,
    LD_FLAG_ARG,
    NO_DEFAULT_FLAGS_ARG,
    "allow-read",
    "allow-write",
    "allow-env",
    "allow-ffi",
    "allow-all",
    "no-permission-prompt",
];

/// Add the flags of the C target: the cache, the checks and the toolchain.
/// `zakac build` shares them.
///
/// # Arguments
///
/// * `cmd` - The command.
///
/// # Returns
///
/// The command with the flags.
pub fn attach_target_args(cmd: clap::Command) -> clap::Command
{
    // The rights the executable holds: the build writes them into it.
    let cmd = attach_grant_args(cmd, Granted::Run);
    let cache =
        Arg::new(CACHE_ARG).long("cache").value_name("DIR").help(t!("target_c.cache"));
    let checks = Arg::new(CHECKS_ARG)
        .long("checks")
        .value_name("LIST")
        .help(t!("target_c.checks"))
        .action(ArgAction::Append)
        .value_delimiter(',')
        .allow_hyphen_values(true)
        .value_parser(check_item)
        .default_value("default");
    let depth = Arg::new(MAX_DEPTH_ARG)
        .long("max-depth")
        .value_name("N")
        .help(t!("target_c.max_depth"))
        .value_parser(clap::value_parser!(u32))
        .default_value("4096");
    let heap = Arg::new(MAX_HEAP_ARG)
        .long("max-heap")
        .value_name("OCTETS")
        .help(t!("target_c.max_heap"))
        .value_parser(clap::value_parser!(u64))
        .default_value("0");
    attach_toolchain_args(cmd.arg(cache).arg(checks).arg(depth).arg(heap))
}

/// Add the flags that pick the toolchain and pass flags to it.
///
/// # Arguments
///
/// * `cmd` - The command.
///
/// # Returns
///
/// The command with the flags.
fn attach_toolchain_args(cmd: clap::Command) -> clap::Command
{
    let tool = |id: &'static str, long: &'static str, help| {
        Arg::new(id).long(long).value_name("CMD").help(help)
    };
    let flag = |id: &'static str, long: &'static str, help| {
        let arg = Arg::new(id).long(long).value_name("FLAG").help(help);
        arg.allow_hyphen_values(true).action(ArgAction::Append)
    };
    let no_defaults = Arg::new(NO_DEFAULT_FLAGS_ARG)
        .long("no-default-flags")
        .help(t!("target_c.no_default_flags"))
        .action(ArgAction::SetTrue);
    cmd.arg(tool(CC_ARG, "cc", t!("target_c.cc")))
        .arg(tool(LINKER_ARG, "linker", t!("target_c.linker")))
        .arg(flag(CC_FLAG_ARG, "cc-flag", t!("target_c.cc_flag")))
        .arg(flag(LD_FLAG_ARG, "ld-flag", t!("target_c.ld_flag")))
        .arg(no_defaults)
}

impl Command for ZakacTargetC
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

    /// Get the section of a config file that holds the defaults of this
    /// command.
    ///
    /// # Returns
    ///
    /// The name of its `zakac` subcommand.
    fn config_section() -> Option<&'static str> { Some("build") }

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
        setup.with(configure_build_command)
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

/// Build a program: check it, write its C into the cache, and compile it
/// unless `--emit c`.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `io` - The standard streams.
///
/// # Returns
///
/// The exit code.
pub fn command_main(matches: &ArgMatches, io: &mut CommandIO<'_>) -> ExitCode
{
    let info = CommandInfo::new(ZakacTargetC::name(), ZakacTargetC::version());
    let arg = |name: &str| matches.get_one::<String>(name).map(String::as_str);
    let input = Input::from(arg("input"));
    let mut source_map = SourceMap::new();
    let source = load_artifact::<Arc<SourceFile>>(
        &input,
        ArtifactFormat::Text,
        io.stdin,
        &mut source_map,
    );
    ret_if!(let Err(error) = source, report_command_error(io, &info, error));
    let source = source.unwrap();
    let emit: Emit = arg(EMIT_ARG).unwrap_or_default().parse().unwrap_or_default();
    let lib = emit == Emit::Lib;
    // A lib has no start block: its functions start the runtime.
    let checks = SemcheckOptions { as_module: lib, ..semcheck_options(matches) };
    let loaded = zakac_semcheck::open_chains::load_settled(
        &source,
        &mut source_map,
        &LexerOptions::default(),
        &target_from(matches, BuildTarget::C),
        &checks,
    );
    ret_if!(let Err(error) = loaded, report_command_error(io, &info, error));

    let target = TargetCOptions { lib, ..target_options(matches) };
    let translated = translate(
        loaded.unwrap(),
        source_map.files(),
        &checks,
        &fold_options(matches),
        &borrow_check_options(matches),
        target,
        &grants_from(matches, Granted::Run),
    );
    let built_for = target_from(matches, BuildTarget::C);
    let shown = checked_fixes(matches, &source, built_for, translated.diagnostics());
    let _ = write_diagnostics(io.stderr, &source_map, &shown);
    ret_if!(has_errors(translated.diagnostics()), ExitCode::Failure);
    let Some(files) = translated.files
    else
    {
        return ExitCode::Failure;
    };
    ret_if!(let Err(error) = files, report_command_error(io, &info, error));

    let stem = input_path(&input);
    let cache = arg(CACHE_ARG).map_or_else(|| default_cache(&stem), PathBuf::from);
    let cross =
        other_host(&target_from(matches, BuildTarget::C)).filter(|_| emit != Emit::C);
    ret_if!(let Some(error) = cross, report_command_error(io, &info, error));
    let wasi = wasi::is_wasi(built_for.abi);
    ret_if!(wasi && lib, report_command_error(io, &info, Error::LibOnWasi));
    let folded = translated.borrowchecked.lowered.folded.as_ref();
    let c_fns = wasi && folded.is_some_and(|m| !m.foreign.is_empty());
    ret_if!(c_fns, report_command_error(io, &info, Error::CFunctionsOnWasi));
    let exports = folded.map(exported).unwrap_or_default();
    ret_if!(
        lib && exports.is_empty(),
        report_command_error(io, &info, Error::LibWithNoExports)
    );
    let base = arg("output").map_or_else(|| stem.with_extension(""), PathBuf::from);
    let lib_files = lib.then(|| LibFiles::of(&base, built_for));
    let out =
        tern!(wasi, with_extension_if_bare(base, wasi::EXTENSION), with_exe(base));
    let out = lib_files.as_ref().map_or(out, |f| f.shared.clone());
    let files = files.unwrap();
    let dialect = toolchain::Dialect::of(target_from(matches, BuildTarget::C).abi);
    let libs = folded.map(|m| link_libs_for(&m.foreign, dialect)).unwrap_or_default();
    let header = lib_files.as_ref().map(|f| {
        let links = static_links(built_for.abi, dialect, &libs);
        let text = exports::header(&exports, target.platform_bits, &f.name, &links);
        (f.header.clone(), text)
    });
    #[rustfmt::skip]
    let job = Job::builder().cache(&cache).out(&out).checks(target.checks).emit(emit)
        .files(&files).libs(&libs).maybe_archive(lib_files.as_ref().map(|f| f.archive.as_path()))
        .maybe_header(header).build();
    let built = build(matches, io, &info, job);
    ret_if!(let Err(error) = built, report_command_error(io, &info, error));
    built.unwrap()
}

/// The error of an executable asked for an OS or a CPU other than the
/// host's, which the C compiler does not build for.
///
/// # Arguments
///
/// * `target` - The target.
///
/// # Returns
///
/// The error, or `None` for the host.
fn other_host(target: &zakac_ast::target::Target) -> Option<Error>
{
    let host = zakac_ast::target::Target::host();
    let same = (target.os, target.arch) == (host.os, host.arch);
    // The clang of wasi-sdk builds for WASI on each host.
    let wasi = target.os == TargetOs::Wasi && target.arch == TargetArch::Wasm32;
    let wasi = wasi && wasi::is_wasi(target.abi);
    let carried = crate::driver::runtime_lib(target.abi).is_some();
    ret_if!((same || wasi) && carried, None);
    Some(Error::CForOtherHost { target: target.shown(), host: host.shown() })
}

/// The path of the input: its file, or `main.zaka` in the working folder
/// for standard input.
///
/// # Arguments
///
/// * `input` - The input.
///
/// # Returns
///
/// The path.
pub fn input_path(input: &Input) -> PathBuf
{
    match input
    {
        Input::File(path) => path.clone(),
        _ => PathBuf::from("main.zaka"),
    }
}

/// An executable path with `.exe` on Windows when it has no extension.
///
/// # Arguments
///
/// * `path` - The path.
///
/// # Returns
///
/// The path to write.
pub fn with_exe(path: PathBuf) -> PathBuf
{
    with_extension_if_bare(path, std::env::consts::EXE_EXTENSION)
}

/// A path with an extension when it has none.
///
/// # Arguments
///
/// * `path` - The path.
/// * `extension` - The extension; a blank one leaves the path as it is.
///
/// # Returns
///
/// The path to write.
fn with_extension_if_bare(path: PathBuf, extension: &str) -> PathBuf
{
    let bare = path.extension().is_none() && !extension.is_empty();
    ret_if!(bare, path.with_extension(extension));
    path
}

/// What one build writes.
#[derive(bon::Builder)]
struct Job<'a>
{
    /// The cache folder.
    cache: &'a Path,
    /// The executable.
    out: &'a Path,
    /// The run-time checks.
    checks: RunChecks,
    /// What the build makes.
    emit: Emit,
    /// The generated C files.
    files: &'a [CFile],
    /// The link flag of each lib the C functions of the program come from.
    libs: &'a [String],
    /// The static lib of a C lib build, beside the shared lib `out` names.
    archive: Option<&'a Path>,
    /// The header of a C lib build, and its text.
    header: Option<(PathBuf, String)>,
}

/// The files a C lib build writes, in one folder: the static lib, the shared
/// lib and the header.
pub struct LibFiles
{
    /// The name of the lib, with no `lib` in front and no extension.
    pub name: String,
    /// The static lib: `lib<name>.a`, or `<name>.lib` for MSVC.
    pub archive: PathBuf,
    /// The shared lib: `lib<name>.so`, `lib<name>.dylib`, or `<name>.dll`.
    pub shared: PathBuf,
    /// The header, `<name>.h`.
    pub header: PathBuf,
}

impl LibFiles
{
    /// The files of a lib, from the path `-o` gives or the input names.
    ///
    /// # Arguments
    ///
    /// * `base` - The path, whose last part names the lib.
    /// * `target` - The target, whose OS and ABI pick the names.
    ///
    /// # Returns
    ///
    /// The files.
    pub fn of(base: &Path, target: zakac_ast::target::Target) -> LibFiles
    {
        let name = base.file_stem().map(|s| s.to_string_lossy().into_owned());
        let name = name.unwrap_or_else(|| "zaka".into());
        let at = |file: String| base.with_file_name(file);
        let msvc = target.abi == zakac_ast::target::TargetAbi::Msvc;
        let archive = tern!(msvc, format!("{name}.lib"), format!("lib{name}.a"));
        let shared = match target.os
        {
            TargetOs::Macos => format!("lib{name}.dylib"),
            TargetOs::Windows => format!("{name}.dll"),
            _ => format!("lib{name}.so"),
        };
        #[rustfmt::skip]
        return LibFiles { archive: at(archive), shared: at(shared), header: at(format!("{name}.h")), name };
    }
}

/// What a program that links the static lib of a C lib links with it: the
/// OS libs of the runtime, libm, and the libs of the C functions the program
/// calls.
///
/// # Arguments
///
/// * `abi` - The ABI of the lib.
/// * `dialect` - The flag form of its toolchain.
/// * `libs` - The link flags of the C functions of the program.
///
/// # Returns
///
/// The link flags.
pub fn static_links(
    abi: zakac_ast::target::TargetAbi, dialect: toolchain::Dialect, libs: &[String],
) -> Vec<String>
{
    let os =
        crate::driver::runtime_lib(abi).map(|r| r.native_libs().collect::<Vec<_>>());
    let os = os.unwrap_or_default().into_iter().map(String::from);
    let m = (dialect == toolchain::Dialect::Gnu).then(|| "-lm".to_string());
    os.chain(m).chain(libs.iter().cloned()).collect()
}

/// Write the C into the cache, then compile it unless `--emit c`.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `io` - The standard streams.
/// * `info` - The command, for warnings.
/// * `job` - What to write.
///
/// # Returns
///
/// The exit code, or the error of a write or of the compiler.
fn build(
    matches: &ArgMatches, io: &mut CommandIO<'_>, info: &CommandInfo, job: Job<'_>,
) -> zakac_host_errors::Result<ExitCode>
{
    let abi = target_from(matches, BuildTarget::C).abi;
    let sources = write_cache_for(job.cache, job.files, abi)?;
    ret_if!(job.emit == Emit::C, Ok(ExitCode::Success));
    #[rustfmt::skip]
    let link = Link::builder().cache(job.cache).out(job.out).checks(job.checks).sources(&sources)
        .libs(job.libs).abi(abi).maybe_archive(job.archive).build();
    let built = build_and_link(matches, io, info, link)?;
    let Some((path, text)) = job.header.filter(|_| built == ExitCode::Success)
    else
    {
        return Ok(built);
    };
    std::fs::write(&path, text).map_err(|e| {
        Error::from_io(e, IOErrorContext::new(path.clone(), IOErrorAction::Write))
    })?;
    Ok(built)
}

/// What a step of the C compiler builds: C files and objects into an
/// executable.
#[derive(bon::Builder)]
pub struct Link<'a>
{
    /// The cache folder, for the probe of the sanitizers.
    pub cache: &'a Path,
    /// The executable.
    pub out: &'a Path,
    /// The run-time checks.
    pub checks: RunChecks,
    /// The C files to compile, then the objects and the libs to link.
    pub sources: &'a [PathBuf],
    /// The link flag of each lib the C functions of the program come from.
    pub libs: &'a [String],
    /// The ABI of the executable; this compiler's own when left out.
    #[builder(default = crate::driver::host_abi())]
    pub abi: zakac_ast::target::TargetAbi,
    /// The static lib of a C lib build, beside the shared lib `out` names.
    pub archive: Option<&'a Path>,
}

/// Compile C files and link them with objects and libs into an executable,
/// with the compiler, the linker and the flags of the command line.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `io` - The standard streams.
/// * `info` - The command, for warnings.
/// * `job` - What to build.
///
/// # Returns
///
/// The exit code, or the error of the compiler.
pub fn build_and_link(
    matches: &ArgMatches, io: &mut CommandIO<'_>, info: &CommandInfo, job: Link<'_>,
) -> zakac_host_errors::Result<ExitCode>
{
    let sources = job.sources;
    let arg = |name: &str| matches.get_one::<String>(name).map(String::as_str);
    // A `--cc` from a config file is a default, so `CC` wins over it.
    let on_line = matches.value_source(CC_ARG) == Some(ValueSource::CommandLine);
    let env_cc = std::env::var_os("CC").is_some_and(|v| !v.is_empty());
    let dialect = toolchain::Dialect::of(job.abi);
    let chosen = tern!(on_line || !env_cc, arg(CC_ARG), None);
    // `CC` and a `--cc` of a config file name a compiler of the host.
    let wasi_cc = || wasi::find_cc(arg(CC_ARG).filter(|_| on_line), job.abi);
    let cc = tern!(
        wasi::is_wasi(job.abi),
        wasi_cc(),
        toolchain::find_cc_for(chosen, dialect)
    )?;
    let linker = arg(LINKER_ARG).map(toolchain::named).transpose()?;
    let mem = job.checks.has(RunCheck::Sanitizers);
    let sanitizers = mem && toolchain::has_sanitizers(&cc, job.cache, dialect);
    if mem && !sanitizers
    {
        let warning = format_command_error(info, t!("target_c.no_sanitizers"));
        let _ = writeln!(io.stderr, "{warning}");
    }
    let cflags = user_flags(matches, CFLAGS, CC_FLAG_ARG);
    let ldflags = user_flags(matches, LDFLAGS, LD_FLAG_ARG);
    #[rustfmt::skip]
    let plan = BuildPlan::builder().cc(&cc).maybe_linker(linker.as_ref())
        .sources(sources).out(job.out).checks(job.checks).sanitizers(sanitizers)
        .defaults(!matches.get_flag(NO_DEFAULT_FLAGS_ARG)).cflags(&cflags)
        .ldflags(&ldflags).libs(job.libs).abi(job.abi).maybe_archive(job.archive).build();
    let built = toolchain::build(&plan)?;
    // `cl` writes the name of each file it compiles, and its errors, to
    // standard output, so a step of MSVC that succeeds shows none of it.
    let quiet = |o: &std::process::Output| {
        dialect == toolchain::Dialect::Msvc && o.status.success()
    };
    for output in &built.outputs
    {
        do_if!(!quiet(output), _ = io.stderr.write_all(&output.stdout));
        let _ = io.stderr.write_all(&output.stderr);
    }
    ret_if!(let Some(cc) = built.failed, Err(Error::CCompilerFailed { cc }));
    Ok(ExitCode::Success)
}

/// The user's flags for one step: those of the variable, then those of each
/// use of the option, so the option wins where the tool takes the last one.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `var` - The variable, `CFLAGS` or `LDFLAGS`.
/// * `arg` - The option, `--cc-flag` or `--ld-flag`.
///
/// # Returns
///
/// The flags.
fn user_flags(matches: &ArgMatches, var: &str, arg: &str) -> Vec<String>
{
    let given = matches.get_many::<String>(arg).into_iter().flatten().cloned();
    toolchain::env_flags(var).into_iter().chain(given).collect()
}
