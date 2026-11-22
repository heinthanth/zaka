//! The `zakafmt` command: files in place, folders, `--check`, standard input
//! and the settings as flags.

use std::io::Cursor;
use std::path::Path;

use termcolor::Buffer;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::ExitCode;
use zakac_formatter::cli::ZakacFormatter;

use crate::common::repo;

/// What one run of the command gave.
struct Ran
{
    code: ExitCode,
    stdout: String,
    stderr: String,
}

/// Run the standalone command.
///
/// # Arguments
///
/// * `args` - The arguments after the command name.
/// * `stdin` - The standard input.
///
/// # Returns
///
/// The exit code and the two outputs.
fn run_cli(args: &[&str], stdin: &str) -> Ran
{
    let mut argv = vec![ZakacFormatter::standalone_name().to_string()];
    argv.extend(args.iter().map(|arg| (*arg).to_string()));
    let command = ZakacFormatter::create_as_standalone();
    let mut stdin = Cursor::new(stdin.to_string());
    let mut stdout = Buffer::no_color();
    let mut stderr = Buffer::no_color();
    #[rustfmt::skip]
    let mut io = CommandIO::builder().stdin(&mut stdin).stdout(&mut stdout)
        .stderr(&mut stderr).build();
    let code = command.execute(&argv, &mut io);
    let stdout = String::from_utf8(stdout.into_inner()).unwrap();
    let stderr = String::from_utf8(stderr.into_inner()).unwrap();
    Ran { code, stdout, stderr }
}

/// A file of the tests folder.
///
/// # Arguments
///
/// * `name` - The path inside `tests/`.
///
/// # Returns
///
/// The text.
fn case(name: &str) -> String
{
    let path = repo().join("compiler/zakac_formatter/tests").join(name);
    std::fs::read_to_string(path).unwrap()
}

/// A folder with a rough file and a clean one.
///
/// # Returns
///
/// The folder.
fn folder() -> tempfile::TempDir
{
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("rough.zaka"), case("inputs/hello.zaka")).unwrap();
    std::fs::write(dir.path().join("clean.zaka"), case("expected/hello.zaka")).unwrap();
    dir
}

/// The text of a file of a folder.
///
/// # Arguments
///
/// * `dir` - The folder.
/// * `name` - The file name.
///
/// # Returns
///
/// The text.
fn read(dir: &Path, name: &str) -> String
{
    std::fs::read_to_string(dir.join(name)).unwrap()
}

/// Standard input is formatted to standard output.
#[test]
fn standard_input_goes_to_standard_output()
{
    let ran = run_cli(&[], &case("inputs/hello.zaka"));
    assert_eq!(ran.code, ExitCode::Success, "{}", ran.stderr);
    assert_eq!(ran.stdout, case("expected/hello.zaka"));
}

/// A folder is walked, and each file that changes is written in place.
#[test]
fn a_folder_is_formatted_in_place()
{
    let dir = folder();
    let ran = run_cli(&[dir.path().to_str().unwrap()], "");
    assert_eq!(ran.code, ExitCode::Success, "{}", ran.stderr);
    assert_eq!(read(dir.path(), "rough.zaka"), case("expected/hello.zaka"));
    assert_eq!(read(dir.path(), "clean.zaka"), case("expected/hello.zaka"));
}

/// `--check` writes nothing, lists the file that would change, and fails.
#[test]
fn check_lists_the_files_that_would_change()
{
    let dir = folder();
    let ran = run_cli(&["--check", dir.path().to_str().unwrap()], "");
    assert_eq!(ran.code, ExitCode::Failure);
    assert!(ran.stdout.trim_end().ends_with("rough.zaka"), "{}", ran.stdout);
    assert_eq!(ran.stdout.lines().count(), 1, "{}", ran.stdout);
    assert_eq!(read(dir.path(), "rough.zaka"), case("inputs/hello.zaka"));
}

/// A file that does not parse is left as it is, with its errors shown.
#[test]
fn a_broken_file_is_left_with_its_errors()
{
    let dir = tempfile::tempdir().unwrap();
    let broken = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n";
    std::fs::write(dir.path().join("broken.zaka"), broken).unwrap();
    let ran = run_cli(&[dir.path().to_str().unwrap()], "");
    assert_eq!(ran.code, ExitCode::Failure);
    assert!(ran.stderr.contains("ZK"), "{}", ran.stderr);
    assert_eq!(read(dir.path(), "broken.zaka"), broken);
}

/// The settings are flags.
#[test]
fn settings_are_flags()
{
    let ran = run_cli(&["--blank-lines-max", "0"], &case("expected/hello.zaka"));
    assert_eq!(ran.code, ExitCode::Success, "{}", ran.stderr);
    assert!(!ran.stdout.contains("\n\n"), "{}", ran.stdout);
}

/// The `[fmt]` section of the nearest `zakac.toml` sets the defaults the
/// language server formats with.
#[test]
fn project_options_read_the_nearest_config()
{
    let dir = tempfile::tempdir().unwrap();
    let config = "[fmt]\nindent = 4\nspeech-mode = \"spoken\"\n";
    std::fs::write(dir.path().join("zakac.toml"), config).unwrap();
    let inner = dir.path().join("src");
    std::fs::create_dir(&inner).unwrap();
    let options = zakac_formatter::cli::project_options(&inner);
    assert_eq!(options.indent, 4);
    assert_eq!(options.speech_mode, zakac_formatter::options::SpeechMode::Spoken);
}
