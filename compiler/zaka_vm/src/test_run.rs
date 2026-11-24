//! `zakac test`: find the test blocks of the files it is given, and of the
//! files these import, then run each test in a VM of its own, in place of the
//! start block, so a stop fails that test alone.

use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

use bon::Builder;
use zaka_host::TermWorld;
use zaka_host::World;
use zakac_ast::stmt::Program;
use zakac_ast::stmt::Stmt;
use zakac_ast::target::BuildTarget;
use zakac_borrow_checker::cli::attach_borrow_check_args;
use zakac_borrow_checker::cli::borrow_check_options;
use zakac_cli_common::clap;
use zakac_cli_common::clap::Arg;
use zakac_cli_common::clap::ArgAction;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::clap::value_parser;
use zakac_cli_common::clap_utils::CommandInfo;
use zakac_cli_common::clap_utils::report_command_error;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_cli_common::grants::Granted;
use zakac_cli_common::grants::attach_grant_args;
use zakac_codegen_target_vm::artifact::run_loaded;
use zakac_codegen_target_vm::cli::attach_codegen_args;
use zakac_codegen_target_vm::cli::emit_options;
use zakac_diagnostics::has_errors;
use zakac_host_errors::Error;
use zakac_i18n::t;
use zakac_ir_builder::cli::attach_compile_args;
use zakac_ir_builder::cli::fold_options;
use zakac_ir_builder::cli::semcheck_options;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::Loaded;
use zakac_reporter::render::write_diagnostics;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::cli::target_from;
use zakac_semcheck::open_chains::load_settled;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::load_artifact;
use zakac_session::config::Input;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

use crate::cli::run_gate;
use crate::machine::HostData;
use crate::machine::RunOptions;
use crate::machine::run_in;
use crate::trap::Backtrace;
use crate::trap::Trap;

/// The files and folders the command line names.
const PATHS_ARG: &str = "paths";
/// The files and folders a config file names, `paths` of `[test]`, read when
/// the command line names none.
const SEARCH_ARG: &str = "search";
/// The flag that keeps the tests whose names hold a word.
const FILTER_ARG: &str = "filter";
/// The flag that runs each test as native code, made with Cranelift.
const JIT_ARG: &str = "jit";
/// The most ops one test runs on the VM.
const MAX_STEPS_ARG: &str = "max_steps";
/// The most seconds one test runs as native code, which counts no ops.
const TIME_LIMIT_ARG: &str = "time_limit";
/// The file of the one test a process of its own runs, under `--jit`.
const ONE_FILE_ARG: &str = "one_file";
/// Where the name of that one test starts.
const ONE_AT_ARG: &str = "one_at";

/// The run of one test with the JIT, in place of the start block of a file:
/// the matches, the standard streams, the file and the check options that
/// name the test.
pub type JitTest =
    fn(&ArgMatches, &mut CommandIO<'_>, &Path, SemcheckOptions) -> ExitCode;

/// The JIT run of a test, which `zakac` installs; the VM crate holds no
/// native code of its own.
pub static JIT_TEST: OnceLock<JitTest> = OnceLock::new();

/// The `zakac test` command.
pub struct ZakaTest
{
    /// The clap command.
    cmd_instance: clap::Command,
}

/// One test block of a program.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
struct Found
{
    /// The file that holds it, as diagnostics name the file.
    file: String,
    /// The name of the test.
    name: String,
    /// Where its name starts, which names the test to the lowering.
    at: u32,
}

impl Found
{
    /// The name a report gives the test: `<file>::<name>`.
    ///
    /// # Returns
    ///
    /// The name.
    fn shown(&self) -> String { format!("{}::{}", self.file, self.name) }
}

/// The counts of a run.
#[derive(Debug, Default)]
struct Counts
{
    /// The tests that ran to their end.
    passed: usize,
    /// The tests that stopped, or whose file did not compile.
    failed: usize,
    /// The tests that ran, each once, though several files import its file.
    seen: BTreeSet<(String, String)>,
}

/// Configure the test command.
///
/// # Arguments
///
/// * `cmd` - The command to configure.
///
/// # Returns
///
/// The configured command.
fn configure_test_command(cmd: clap::Command) -> clap::Command
{
    let given =
        Arg::new(PATHS_ARG).value_name("PATH").help(t!("vm.test.paths")).num_args(0..);
    let search = Arg::new(SEARCH_ARG)
        .long("paths")
        .value_name("PATH")
        .help(t!("vm.test.search"))
        .num_args(1..)
        .value_delimiter(',')
        .default_value("tests");
    let filter = Arg::new(FILTER_ARG)
        .long("filter")
        .value_name("WORD")
        .help(t!("vm.test.filter"));
    let jit = Arg::new(JIT_ARG)
        .long("jit")
        .help(t!("vm.test.jit"))
        .action(ArgAction::SetTrue);
    let steps = Arg::new(MAX_STEPS_ARG)
        .long("max-steps")
        .value_name("OPS")
        .help(t!("vm.test.max_steps"))
        .value_parser(value_parser!(u64).range(1..))
        .default_value("100000000");
    let time = Arg::new(TIME_LIMIT_ARG)
        .long("time-limit")
        .value_name("SECONDS")
        .help(t!("vm.test.time_limit"))
        .value_parser(value_parser!(u64).range(1..))
        .default_value("60");
    let one_file = Arg::new(ONE_FILE_ARG).long("one-file").hide(true);
    let one_at =
        Arg::new(ONE_AT_ARG).long("one-at").hide(true).value_parser(value_parser!(u32));
    let cmd = cmd.arg(given).arg(search).arg(filter).arg(jit).arg(steps).arg(time);
    let cmd = cmd.arg(one_file).arg(one_at);
    let cmd = attach_codegen_args(attach_borrow_check_args(attach_compile_args(cmd)));
    attach_grant_args(cmd, Granted::Run).about(t!("vm.test.about"))
}

/// The `.zaka` files under the paths the command line names, or under those
/// of the config file when it names none, sorted.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
///
/// # Returns
///
/// The files, or the error of a path that does not exist.
fn test_files(matches: &ArgMatches) -> zakac_host_errors::Result<Vec<PathBuf>>
{
    let given: Vec<&String> =
        matches.get_many(PATHS_ARG).into_iter().flatten().collect();
    let search =
        || matches.get_many::<String>(SEARCH_ARG).into_iter().flatten().collect();
    let roots: Vec<&String> = tern!(given.is_empty(), search(), given);
    let mut files = Vec::new();
    for root in roots
    {
        let path = PathBuf::from(root);
        ret_if!(!path.exists(), Err(Error::FileNotFound { path }));
        walk(&path, &mut files);
    }
    files.sort();
    files.dedup();
    Ok(files)
}

/// Add a file, or each `.zaka` file under a folder, to a list.
///
/// # Arguments
///
/// * `path` - The file or the folder.
/// * `files` - The list.
fn walk(path: &Path, files: &mut Vec<PathBuf>)
{
    ret_if!(path.is_file(), files.push(path.to_path_buf()));
    let Ok(entries) = std::fs::read_dir(path)
    else
    {
        return;
    };
    for item in entries.flatten().map(|e| e.path())
    {
        let zaka = item.extension().is_some_and(|e| e == "zaka");
        do_if!(item.is_dir() || zaka, walk(&item, files));
    }
}

/// The test blocks of a program: those of the file that is run and of each
/// file it imports.
///
/// # Arguments
///
/// * `program` - The program.
/// * `path` - The file that is run, which names its own tests.
///
/// # Returns
///
/// The tests, in source order.
fn found_tests(program: &Program, path: &Path) -> Vec<Found>
{
    let root = shown_path(path);
    let found = |(i, s): (usize, &Stmt)| {
        let Stmt::Fn(f) = s
        else
        {
            return None;
        };
        let unit = program.unit_of(i);
        let file = tern!(
            program.is_root(unit),
            root.clone(),
            program.units[unit as usize].name.clone()
        );
        #[rustfmt::skip]
        let test = Found::builder().file(file).name(f.name.name.to_string())
            .at(f.name.span.lo.as_u32()).build();
        f.test.then_some(test)
    };
    program.stmts.iter().enumerate().filter_map(found).collect()
}

/// The check options of a test run: each file checks as a module, which
/// needs no start block.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `test_at` - The test to run in place of the start block.
///
/// # Returns
///
/// The options.
fn test_options(matches: &ArgMatches, test_at: Option<u32>) -> SemcheckOptions
{
    let mut options = semcheck_options(matches);
    options.as_module = true;
    options.test_at = test_at;
    options
}

/// Run the tests of the files the command names.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `io` - The standard streams.
///
/// # Returns
///
/// The exit code: a failure when a test failed or a file did not compile.
fn command_main(matches: &ArgMatches, io: &mut CommandIO<'_>) -> ExitCode
{
    let info = CommandInfo::new(ZakaTest::name(), ZakaTest::version());
    ret_if!(let Some(file) = matches.get_one::<String>(ONE_FILE_ARG), one_jit_test(matches, io, file));
    let files = match test_files(matches)
    {
        Ok(files) => files,
        Err(error) => return report_command_error(io, &info, error),
    };
    let mut counts = Counts::default();
    for path in &files
    {
        let ran = test_file(matches, path, io, &mut counts);
        ret_if!(let Err(error) = ran, report_command_error(io, &info, error));
    }
    let ok = counts.failed == 0;
    let state = tern!(ok, t!("vm.test.ok"), t!("vm.test.failed"));
    let line = t!(
        "vm.test.result",
        state = state,
        passed = counts.passed,
        failed = counts.failed
    );
    let _ = writeln!(io.stdout, "\n{line}");
    tern!(ok, ExitCode::Success, ExitCode::Failure)
}

/// Run the tests of one file and of the files it imports, each once.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `path` - The file.
/// * `io` - The standard streams.
/// * `counts` - The counts so far.
///
/// # Returns
///
/// Nothing, or the error of a file that could not be read.
fn test_file(
    matches: &ArgMatches, path: &Path, io: &mut CommandIO<'_>, counts: &mut Counts,
) -> zakac_host_errors::Result<()>
{
    let mut source_map = SourceMap::new();
    let input = Input::File(path.to_path_buf());
    let source = load_artifact::<Arc<SourceFile>>(
        &input,
        ArtifactFormat::Text,
        io.stdin,
        &mut source_map,
    )?;
    let options = test_options(matches, None);
    let target = target_from(matches, BuildTarget::Vm);
    let loaded = load_settled(
        &source,
        &mut source_map,
        &LexerOptions::default(),
        &target,
        &options,
    )?;
    let filter = matches.get_one::<String>(FILTER_ARG);
    let keep = |f: &Found| filter.is_none_or(|w| f.shown().contains(w.as_str()));
    let tests: Vec<Found> =
        found_tests(&loaded.program, path).into_iter().filter(keep).collect();
    let tests: Vec<Found> = tests
        .into_iter()
        .filter(|f| counts.seen.insert((f.file.clone(), f.name.clone())))
        .collect();
    // A file whose parse errors hide its tests fails all the same.
    if has_errors(&loaded.diagnostics)
    {
        let _ = write_diagnostics(io.stderr, &source_map, &loaded.diagnostics);
        let shown = shown_path(path);
        let state = t!("vm.test.failed");
        let _ =
            writeln!(io.stdout, "{}", t!("vm.test.line", name = shown, state = state));
        counts.failed += tests.len().max(1);
        return Ok(());
    }
    ret_if!(tests.is_empty(), Ok(()));
    if !compiles(matches, &loaded, &source_map, io)
    {
        do_for!(f in &tests, report(io, f, false));
        counts.failed += tests.len();
        return Ok(());
    }
    let jit = matches.get_flag(JIT_ARG);
    for found in &tests
    {
        let passed = tern!(
            jit,
            run_jit_test(matches, path, found, io),
            run_test(matches, &loaded, found, (&source_map, &mut *io))
        );
        do_if!(passed, counts.passed += 1);
        do_if!(!passed, counts.failed += 1);
    }
    Ok(())
}

/// Check and build a program with no test in place of its start block, and
/// write its diagnostics, so the diagnostics show once for each file.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `loaded` - The program.
/// * `source_map` - The source map.
/// * `io` - The standard streams.
///
/// # Returns
///
/// `true` when the program has no error.
fn compiles(
    matches: &ArgMatches, loaded: &Loaded, source_map: &SourceMap,
    io: &mut CommandIO<'_>,
) -> bool
{
    let options = test_options(matches, None);
    let mem = borrow_check_options(matches);
    let folding = fold_options(matches);
    let emitted =
        run_loaded(loaded.clone(), &options, &folding, &mem, emit_options(matches));
    let diagnostics = emitted.diagnostics();
    let _ = write_diagnostics(io.stderr, source_map, &diagnostics);
    !has_errors(&diagnostics)
}

/// Run one test in a VM of its own, and write its line: `ok`, or `FAILED`
/// with what it printed and the stop.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `loaded` - The program.
/// * `found` - The test.
/// * `out` - The source map, and the standard streams.
///
/// # Returns
///
/// `true` when the test ran to its end.
fn run_test(
    matches: &ArgMatches, loaded: &Loaded, found: &Found,
    out: (&SourceMap, &mut CommandIO<'_>),
) -> bool
{
    let (source_map, io) = out;
    let options = test_options(matches, Some(found.at));
    let mem = borrow_check_options(matches);
    let folding = fold_options(matches);
    let emitted =
        run_loaded(loaded.clone(), &options, &folding, &mem, emit_options(matches));
    let Some(module) = emitted.module
    else
    {
        report(io, found, false);
        return false;
    };
    let mut printed = Vec::new();
    let mut stops = Vec::new();
    let host = HostData::program(&found.file);
    let reach = (Box::new(TermWorld) as Box<dyn World>, run_gate(matches));
    let backtrace =
        || Backtrace::from_value(std::env::var(Backtrace::VAR).ok().as_deref());
    let mut stopped =
        |_: usize, trap: &Trap| stops.push(trap.report(source_map, backtrace()));
    let steps = matches.get_one::<u64>(MAX_STEPS_ARG).copied();
    let run_options = RunOptions::builder().maybe_max_steps(steps).build();
    let (_, result) =
        run_in(&module, &mut printed, &run_options, &host, reach, &mut stopped);
    do_if!(let Err(trap) = &result, stops.push(trap.report(source_map, backtrace())));
    let passed = result.is_ok() && stops.is_empty();
    report(io, found, passed);
    ret_if!(passed, true);
    let _ = io.stdout.write_all(&printed);
    do_for!(s in &stops, _ = writeln!(io.stdout, "{}", t!("vm.test.stopped", stop = s)));
    false
}

/// A path as a report writes it, with `/` on each host.
///
/// # Arguments
///
/// * `path` - The path.
///
/// # Returns
///
/// The text.
fn shown_path(path: &Path) -> String
{
    path.display().to_string().replace(std::path::MAIN_SEPARATOR, "/")
}

/// Run the one test that `--one-file` and `--one-at` name with the JIT, in
/// this process, which a stop of the test ends.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `io` - The standard streams.
/// * `file` - The file of the test.
///
/// # Returns
///
/// The exit code of the test.
fn one_jit_test(matches: &ArgMatches, io: &mut CommandIO<'_>, file: &str) -> ExitCode
{
    let at = matches.get_one::<u32>(ONE_AT_ARG).copied();
    let options = test_options(matches, at);
    let Some(run) = JIT_TEST.get()
    else
    {
        return ExitCode::Failure;
    };
    run(matches, io, Path::new(file), options)
}

/// Run one test with the JIT in a process of its own: this program again,
/// with the arguments of this run and the test named, so a stop of native
/// code ends that process alone. A test past the time limit is killed.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
/// * `path` - The file that is run.
/// * `found` - The test.
/// * `io` - The standard streams.
///
/// # Returns
///
/// `true` when the test ran to its end.
fn run_jit_test(
    matches: &ArgMatches, path: &Path, found: &Found, io: &mut CommandIO<'_>,
) -> bool
{
    let seconds = matches.get_one::<u64>(TIME_LIMIT_ARG).copied().unwrap_or(60);
    let child = std::env::current_exe().and_then(|exe| {
        std::process::Command::new(exe)
            .args(std::env::args_os().skip(1))
            .arg("--one-file")
            .arg(path)
            .arg("--one-at")
            .arg(found.at.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    });
    let Ok(child) = child
    else
    {
        report(io, found, false);
        return false;
    };
    let (status, printed, stops) = wait_limited(child, Duration::from_secs(seconds));
    let passed = status.is_some_and(|s| s.success());
    report(io, found, passed);
    ret_if!(passed, true);
    let _ = io.stdout.write_all(&printed);
    let stops = String::from_utf8_lossy(&stops);
    do_for!(s in stops.lines(), _ = writeln!(io.stdout, "{}", t!("vm.test.stopped", stop = s)));
    let late = t!("vm.test.time_out", seconds = seconds);
    do_if!(
        status.is_none(),
        _ = writeln!(io.stdout, "{}", t!("vm.test.stopped", stop = late))
    );
    false
}

/// Wait for a process, reading its two output streams, and kill it past a
/// time limit.
///
/// # Arguments
///
/// * `child` - The process.
/// * `limit` - The time limit.
///
/// # Returns
///
/// Its exit status, `None` when it was killed, then what it wrote to standard
/// output and to standard error.
fn wait_limited(
    mut child: std::process::Child, limit: Duration,
) -> (Option<std::process::ExitStatus>, Vec<u8>, Vec<u8>)
{
    let out = read_all(child.stdout.take());
    let err = read_all(child.stderr.take());
    let end = Instant::now() + limit;
    let status = loop
    {
        if let Ok(Some(status)) = child.try_wait()
        {
            break Some(status);
        }
        if Instant::now() >= end
        {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    (status, out.join().unwrap_or_default(), err.join().unwrap_or_default())
}

/// Read a stream to its end on a thread of its own, so a process that fills
/// one pipe does not wait on the other.
///
/// # Arguments
///
/// * `stream` - The stream, when there is one.
///
/// # Returns
///
/// The thread, which gives the u8s it read.
fn read_all(
    stream: Option<impl std::io::Read + Send + 'static>,
) -> std::thread::JoinHandle<Vec<u8>>
{
    std::thread::spawn(move || {
        let mut read = Vec::new();
        do_if!(let Some(mut s) = stream, _ = s.read_to_end(&mut read));
        read
    })
}

/// Write the line of one test.
///
/// # Arguments
///
/// * `io` - The standard streams.
/// * `found` - The test.
/// * `passed` - Whether it passed.
fn report(io: &mut CommandIO<'_>, found: &Found, passed: bool)
{
    let state = tern!(passed, t!("vm.test.ok"), t!("vm.test.failed"));
    let _ = writeln!(
        io.stdout,
        "{}",
        t!("vm.test.line", name = found.shown(), state = state)
    );
}

impl Command for ZakaTest
{
    /// Get the canonical name of the command.
    ///
    /// # Returns
    ///
    /// The canonical name of the command.
    fn name() -> &'static str { "test" }

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
    fn config_section() -> Option<&'static str> { Some("test") }

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
        setup.with(configure_test_command)
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
