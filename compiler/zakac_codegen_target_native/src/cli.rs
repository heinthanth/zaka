//! The commands of the native target: `zakac run --jit`, and
//! `zakac build --target native`.

use std::sync::Arc;

#[cfg(feature = "jit")]
use zaka_host::Gate;
use zakac_ast::target::BuildTarget;
use zakac_ast::target::Target;
use zakac_borrow_checker::cli::borrow_check_options;
use zakac_borrow_checker::cli::checked_fixes;
use zakac_cli_common::clap::ArgMatches;
#[cfg(feature = "jit")]
use zakac_cli_common::clap::parser::ValueSource;
use zakac_cli_common::clap_utils::CommandInfo;
use zakac_cli_common::clap_utils::report_command_error;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::ExitCode;
use zakac_codegen_target_c::options::TargetCOptions;
use zakac_diagnostics::has_errors;
use zakac_host_errors::Error;
use zakac_ir_builder::cli::fold_options;
use zakac_ir_builder::cli::semcheck_options;
use zakac_lexer::lexer::LexerOptions;
use zakac_reporter::render::write_diagnostics;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::cli::target_from;
use zakac_session::artifact::ArtifactFormat;
use zakac_session::artifact::load_artifact;
use zakac_session::config::Input;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

/// The error of a target the native code cannot be made for: a CPU with no
/// Cranelift backend.
///
/// # Arguments
///
/// * `target` - The target.
///
/// # Returns
///
/// The error, or `None` for a CPU Cranelift builds for.
pub fn arch_error(target: &Target) -> Option<Error>
{
    ret_if!(crate::aot::ARCHES.contains(&target.arch), None);
    let arch: &'static str = target.arch.into();
    Some(Error::ArchNotForNative { arch: arch.into() })
}

/// Run a program with the JIT: the source is checked as `zakac run` checks
/// it, with the output fact `"native"`, then lowered and run in this
/// process.
///
/// # Arguments
///
/// * `matches` - The matches of `zakac run`.
/// * `io` - The standard streams.
/// * `gate` - The rights of the run, and what asks for one it lacks.
/// * `args` - The arguments of the start block: the program name first.
///
/// # Returns
///
/// The exit code.
#[cfg(feature = "jit")]
pub fn run_jit(
    matches: &ArgMatches, io: &mut CommandIO<'_>, gate: Gate, args: Vec<String>,
) -> ExitCode
{
    let info = CommandInfo::new("zakac run", env!("CARGO_PKG_VERSION"));
    let input = Input::from(matches.get_one::<String>("input").map(String::as_str));
    let mut source_map = SourceMap::new();
    let source = load_artifact::<Arc<SourceFile>>(
        &input,
        ArtifactFormat::Text,
        io.stdin,
        &mut source_map,
    );
    ret_if!(let Err(error) = source, report_command_error(io, &info, error));
    let target = target_from(matches, BuildTarget::Native);
    ret_if!(let Some(error) = arch_error(&target), report_command_error(io, &info, error));
    ret_if!(let Some(error) = other_host(&target), report_command_error(io, &info, error));
    let semcheck = semcheck_options(matches);
    let source = source.unwrap();
    let loaded = zakac_semcheck::open_chains::load_settled(
        &source,
        &mut source_map,
        &LexerOptions::default(),
        &target,
        &semcheck,
    );
    ret_if!(let Err(error) = loaded, report_command_error(io, &info, error));
    let mem = borrow_check_options(matches);
    let checked = zakac_borrow_checker::artifact::run_loaded(
        loaded.unwrap(),
        &semcheck,
        &fold_options(matches),
        &mem,
    );
    let shown = checked_fixes(matches, &source, target.clone(), &checked.diagnostics);
    let _ = write_diagnostics(io.stderr, &source_map, &shown);
    ret_if!(has_errors(&checked.diagnostics), ExitCode::Failure);
    let Some(program) = checked.lowered.folded.as_ref()
    else
    {
        return ExitCode::Failure;
    };
    let explicit =
        |id: &str| matches.value_source(id) == Some(ValueSource::CommandLine);
    let max_depth =
        matches.try_get_one::<u32>("max_depth").ok().flatten().copied().unwrap_or(4096);
    let max_heap = matches
        .try_get_one::<u64>("max_heap")
        .ok()
        .flatten()
        .copied()
        .filter(|_| explicit("max_heap"));
    let release = matches.try_get_one::<bool>("release").ok().flatten() == Some(&true);
    #[rustfmt::skip]
    let options = TargetCOptions::builder().overflow(mem.overflow).platform_bits(semcheck.platform_bits)
        .max_depth(max_depth).max_heap(max_heap.unwrap_or(0)).debug_info(!release).build();
    match crate::jit::run(program, source_map.files(), options, gate, &args)
    {
        Ok(status) => ExitCode::from((status & 0xff) as u8),
        Err(error) => report_command_error(io, &info, error),
    }
}

#[cfg(feature = "jit")]
/// The error of a program settled for an OS or a CPU other than the host's,
/// which the JIT cannot run.
///
/// # Arguments
///
/// * `target` - The target.
///
/// # Returns
///
/// The error, or `None` for the host.
fn other_host(target: &Target) -> Option<Error>
{
    ret_if!(target.is_host(), None);
    Some(Error::BuiltForOtherHost {
        built: target.shown(),
        host: Target::host().shown(),
    })
}

/// What an AOT build makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Emit
{
    /// The executable: the object linked with the runtime lib.
    #[default]
    Bin,
    /// The object file alone.
    Obj,
    /// The CLIF text of each function.
    Clif,
}

/// The words of `--emit` the native target takes.
///
/// # Returns
///
/// The words.
pub fn emit_words() -> Vec<&'static str> { vec!["bin", "obj", "clif"] }

/// Build a program with the native target: checked as the C target checks it,
/// with the output fact `"native"`, lowered to an object file for the target,
/// then linked with the runtime lib unless `--emit obj` or `--emit clif`.
///
/// # Arguments
///
/// * `matches` - The matches of `zakac build`.
/// * `io` - The standard streams.
///
/// # Returns
///
/// The exit code.
pub fn build_main(matches: &ArgMatches, io: &mut CommandIO<'_>) -> ExitCode
{
    use std::path::PathBuf;

    use zakac_cli_common::grants::Granted;
    use zakac_cli_common::grants::grants_from;
    use zakac_codegen_target_c::cli::Link;
    use zakac_codegen_target_c::cli::build_and_link;
    use zakac_codegen_target_c::cli::input_path;
    use zakac_codegen_target_c::cli::target_options;
    use zakac_codegen_target_c::cli::with_exe;
    use zakac_codegen_target_c::driver::default_cache;
    use zakac_codegen_target_c::driver::write_cache_for;
    use zakac_codegen_target_c::ffi::link_libs_for;
    use zakac_codegen_target_c::options::RunCheck;
    use zakac_codegen_target_c::toolchain::Dialect;

    let info = CommandInfo::new("zakac build", env!("CARGO_PKG_VERSION"));
    let arg = |name: &str| matches.get_one::<String>(name).map(String::as_str);
    let emit = match arg("emit")
    {
        Some("obj") => Emit::Obj,
        Some("clif") => Emit::Clif,
        _ => Emit::Bin,
    };
    let input = Input::from(arg("input"));
    let mut source_map = SourceMap::new();
    let source = load_artifact::<Arc<SourceFile>>(
        &input,
        ArtifactFormat::Text,
        io.stdin,
        &mut source_map,
    );
    ret_if!(let Err(error) = source, report_command_error(io, &info, error));
    let target = target_from(matches, BuildTarget::Native);
    ret_if!(let Some(error) = arch_error(&target), report_command_error(io, &info, error));
    let cross = native_other_host(&target).filter(|_| emit == Emit::Bin);
    ret_if!(let Some(error) = cross, report_command_error(io, &info, error));
    let semcheck = semcheck_options(matches);
    let source = source.unwrap();
    let loaded = zakac_semcheck::open_chains::load_settled(
        &source,
        &mut source_map,
        &LexerOptions::default(),
        &target,
        &semcheck,
    );
    ret_if!(let Err(error) = loaded, report_command_error(io, &info, error));
    let mem = borrow_check_options(matches);
    let checked = zakac_borrow_checker::artifact::run_loaded(
        loaded.unwrap(),
        &semcheck,
        &fold_options(matches),
        &mem,
    );
    let shown = checked_fixes(matches, &source, target.clone(), &checked.diagnostics);
    let _ = write_diagnostics(io.stderr, &source_map, &shown);
    ret_if!(has_errors(&checked.diagnostics), ExitCode::Failure);
    let Some(program) = checked.lowered.folded.as_ref()
    else
    {
        return ExitCode::Failure;
    };
    let options = TargetCOptions {
        overflow: mem.overflow,
        platform_bits: semcheck.platform_bits,
        ..target_options(matches)
    };
    let grants = grants_from(matches, Granted::Run);
    let made = crate::aot::object(
        program,
        source_map.files(),
        options,
        &grants,
        &target,
        emit == Emit::Clif,
    );
    ret_if!(let Err(error) = made, report_command_error(io, &info, error));
    let made = made.unwrap();
    let stem = input_path(&input);
    let written = |path: PathBuf, octets: &[u8]| {
        std::fs::write(&path, octets).map_err(|e| {
            let context = zakac_host_errors::errors::IOErrorContext::new(
                path,
                zakac_host_errors::errors::IOErrorAction::Write,
            );
            Error::from_io(e, context)
        })
    };
    let out = |ext: &str| {
        arg("output").map_or_else(|| stem.with_extension(ext), PathBuf::from)
    };
    let done = match emit
    {
        Emit::Clif =>
        {
            written(out("clif"), made.clif.as_bytes()).map(|_| ExitCode::Success)
        }
        Emit::Obj => written(out("o"), &made.octets).map(|_| ExitCode::Success),
        Emit::Bin =>
        {
            let cache =
                arg("cache").map_or_else(|| default_cache(&stem), PathBuf::from);
            let exe = with_exe(out(""));
            // Cranelift code holds no sanitizer marks, so the link leaves them
            // out.
            let checks = options.checks.with(RunCheck::Sanitizers, false);
            let dialect = Dialect::of(target.abi);
            write_cache_for(&cache, &[], target.abi).and_then(|lib| {
                let obj = cache.join(format!("main.{}", dialect.object()));
                written(obj.clone(), &made.octets)?;
                let sources: Vec<PathBuf> = std::iter::once(obj).chain(lib).collect();
                let libs = link_libs_for(&program.foreign, dialect);
                #[rustfmt::skip]
                let link = Link::builder().cache(&cache).out(&exe).checks(checks)
                    .sources(&sources).libs(&libs).abi(target.abi).build();
                build_and_link(matches, io, &info, link)
            })
        }
    };
    match done
    {
        Ok(code) => code,
        Err(error) => report_command_error(io, &info, error),
    }
}

/// The error of an executable asked for an OS or a CPU other than the host's,
/// which the C toolchain does not link for.
///
/// # Arguments
///
/// * `target` - The target.
///
/// # Returns
///
/// The error, or `None` for the host.
fn native_other_host(target: &Target) -> Option<Error>
{
    let host = Target::host();
    let same = (target.os, target.arch) == (host.os, host.arch);
    let carried = zakac_codegen_target_c::driver::runtime_lib(target.abi).is_some();
    ret_if!(same && carried, None);
    Some(Error::NativeForOtherHost {
        target: target.shown(),
        host: Target::host().shown(),
    })
}
