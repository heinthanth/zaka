//! Config files: TOML that sets the default of each flag of a command, and
//! the settings that env vars hold.
//!
//! The files are read from the lowest to the highest: `/etc/zakac/config.toml`,
//! `~/Library/Application Support/zakac/config.toml` on macOS, the user's
//! config folder (`$XDG_CONFIG_HOME` or `~/.config` on Linux and macOS,
//! `%APPDATA%` on Windows) with `zakac/config.toml`, the nearest `zakac.toml`
//! from the working folder up, and last the file `--config` names. A later
//! file wins over an earlier one; a flag on the command line wins over each.

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use etcetera::BaseStrategy;
use toml::Table;
use toml::Value;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::clap;
use crate::clap_utils::active_command;

/// The config file searched for from the working folder up.
pub const LOCAL_FILE: &str = "zakac.toml";

/// The flag that names a config file.
pub const CONFIG_FLAG: &str = "config";

/// The flags each command shares, which a config file cannot set.
const SHARED: &[&str] = &["help", "version", CONFIG_FLAG];

/// The config names of the settings that env vars hold, with their vars. A
/// var that is set wins over the file.
const ENV_NAMES: &[(&str, &str)] = &[
    ("carets", "ZAKAC_CARETS"),
    ("backtrace", "ZAKA_BACKTRACE"),
    ("cflags", "CFLAGS"),
    ("ldflags", "LDFLAGS"),
    ("color", "NO_COLOR"),
];

/// Read each config file and set the defaults of `cmd` and the env vars
/// from them.
///
/// # Arguments
///
/// * `cmd` - The command tree, before parsing.
/// * `section` - The section of the root command: `None` for `zakac`, the name
///   of its `zakac` subcommand for the executable of one stage.
/// * `args` - The command line, the program name first.
///
/// # Returns
///
/// The command tree with the defaults set, or the error of a file.
pub fn configure(
    cmd: &clap::Command, section: Option<&str>, args: &[String],
) -> zakac_host_errors::Result<clap::Command>
{
    let active = active_command(cmd, args);
    let active =
        tern!(active.get_name() == cmd.get_name(), section, Some(active.get_name()));
    let mut configured = cmd.clone();
    let mut env = BTreeMap::new();
    for path in files(args)?
    {
        let table = read_file(&path, cmd, section)?;
        configured = set_defaults(configured, &table, section);
        env.extend(env_values(&table, active));
    }
    set_env(env);
    Ok(configured)
}

/// The config files to read, the lowest first: each file of the search that
/// is there, then the file `--config` names, which must be there.
///
/// # Arguments
///
/// * `args` - The command line, the program name first.
///
/// # Returns
///
/// The paths, or the error when the file `--config` names is missing.
fn files(args: &[String]) -> zakac_host_errors::Result<Vec<PathBuf>>
{
    let mut paths: Vec<PathBuf> =
        search_paths().into_iter().filter(|p| p.is_file()).collect();
    let Some(given) = given_path(args)
    else
    {
        return Ok(paths);
    };
    let ctx = IOErrorContext::new(given.clone(), IOErrorAction::Read);
    ret_if!(!given.is_file(), Err(Error::from_io(not_found(), ctx)));
    paths.push(given);
    Ok(paths)
}

/// The error of a file that is not there.
///
/// # Returns
///
/// The IO error.
fn not_found() -> std::io::Error { std::io::ErrorKind::NotFound.into() }

/// The places a config file can be, the lowest first.
///
/// # Returns
///
/// The paths, whether or not a file is there.
fn search_paths() -> Vec<PathBuf>
{
    let mut paths = Vec::new();
    #[cfg(unix)]
    paths.push(Path::new("/etc").join("zakac").join("config.toml"));
    #[cfg(target_os = "macos")]
    paths.extend(etcetera::home_dir().ok().map(|home| {
        home.join("Library")
            .join("Application Support")
            .join("zakac")
            .join("config.toml")
    }));
    let base = etcetera::choose_base_strategy().ok();
    paths.extend(base.map(|b| b.config_dir().join("zakac").join("config.toml")));
    let cwd = std::env::current_dir().ok();
    paths.extend(cwd.and_then(|dir| nearest_local(&dir)));
    paths
}

/// The nearest [`LOCAL_FILE`] in a folder or a folder above it.
///
/// # Arguments
///
/// * `dir` - The folder to start from.
///
/// # Returns
///
/// The path, or `None` when no folder holds one.
pub fn nearest_local(dir: &Path) -> Option<PathBuf>
{
    dir.ancestors().map(|d| d.join(LOCAL_FILE)).find(|p| p.is_file())
}

/// The file `--config` names on the command line, before `--`; the last one
/// wins, as with each flag.
///
/// # Arguments
///
/// * `args` - The command line, the program name first.
///
/// # Returns
///
/// The path, or `None`.
pub fn given_path(args: &[String]) -> Option<PathBuf>
{
    let flag = format!("--{CONFIG_FLAG}");
    let joined = format!("{flag}=");
    let mut found = None;
    let mut rest = args.iter().skip(1).take_while(|a| a.as_str() != "--");
    while let Some(arg) = rest.next()
    {
        do_if!(*arg == flag, found = rest.next().map(PathBuf::from));
        do_if!(let Some(p) = arg.strip_prefix(&joined), found = Some(PathBuf::from(p)));
    }
    found
}

/// A path as text for a reader, with `/` between its parts on each host.
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

/// Read one config file and check each name in it against the command tree.
///
/// # Arguments
///
/// * `path` - The file.
/// * `cmd` - The command tree.
/// * `section` - The section of the root command.
///
/// # Returns
///
/// The table of the file, or the error of the read, the TOML or a name.
pub fn read_file(
    path: &Path, cmd: &clap::Command, section: Option<&str>,
) -> zakac_host_errors::Result<Table>
{
    let ctx = IOErrorContext::new(path.to_path_buf(), IOErrorAction::Read);
    let text = std::fs::read_to_string(path).map_err(|e| Error::from_io(e, ctx))?;
    let table = parse_text(&text, &shown(path))?;
    check_file(&table, cmd, section, &shown(path))?;
    Ok(table)
}

/// Parse the text of a config file.
///
/// # Arguments
///
/// * `text` - The text.
/// * `path` - The file, as the error shows it.
///
/// # Returns
///
/// The table, or the error of the TOML.
pub fn parse_text(text: &str, path: &str) -> zakac_host_errors::Result<Table>
{
    #[rustfmt::skip]
    let not_toml = |e: toml::de::Error| Error::ConfigNotToml {
        path: path.into(), msg: e.message().to_string() };
    text.parse::<Table>().map_err(not_toml)
}

/// Check each name of a config file. A section names a `zakac` subcommand.
/// `zakac` checks each name; the executable of one stage sees a file written
/// for all of `zakac`, so it checks its own section alone.
///
/// # Arguments
///
/// * `table` - The file.
/// * `cmd` - The command tree.
/// * `section` - The section of the root command.
/// * `path` - The file, as an error shows it.
///
/// # Returns
///
/// Nothing, or the error of the first name that is wrong.
pub fn check_file(
    table: &Table, cmd: &clap::Command, section: Option<&str>, path: &str,
) -> zakac_host_errors::Result<()>
{
    let whole = cmd.has_subcommands();
    table.iter().try_for_each(|(name, value)| match value
    {
        Value::Table(inner) =>
        {
            let found = section_command(cmd, section, name);
            // A table under the name of a flag is a value of the wrong form,
            // not a section.
            ret_if!(
                found.is_none() && takes(cmd, name),
                check_value(name, value, path)
            );
            check_section(name, inner, found, whole, path)
        }
        _ if whole && !takes(cmd, name) => Err(unknown(path, name)),
        _ => check_value(name, value, path),
    })
}

/// The command a section names: the root command when the section is its
/// own, else a subcommand.
///
/// # Arguments
///
/// * `cmd` - The root command.
/// * `own` - The section of the root command.
/// * `name` - The section.
///
/// # Returns
///
/// The command, or `None`.
fn section_command<'a>(
    cmd: &'a clap::Command, own: Option<&str>, name: &str,
) -> Option<&'a clap::Command>
{
    ret_if!(own == Some(name), Some(cmd));
    cmd.find_subcommand(name)
}

/// Check the names of one section.
///
/// # Arguments
///
/// * `name` - The section.
/// * `inner` - Its table.
/// * `found` - The command it names, if one in this tree.
/// * `whole` - Whether the tree is the whole of `zakac`.
/// * `path` - The file, as an error shows it.
///
/// # Returns
///
/// Nothing, or the error of the first name that is wrong.
fn check_section(
    name: &str, inner: &Table, found: Option<&clap::Command>, whole: bool, path: &str,
) -> zakac_host_errors::Result<()>
{
    let Some(sub) = found
    else
    {
        return tern!(whole, Err(unknown(path, name)), Ok(()));
    };
    inner.iter().try_for_each(|(flag, value)| {
        let known = is_env_name(flag) || own_longs(sub).any(|l| l == flag);
        ret_if!(!known, Err(unknown(path, &format!("{name}.{flag}"))));
        check_value(flag, value, path)
    })
}

/// Check that a value is text, a number, a truth value or a list of these.
///
/// # Arguments
///
/// * `name` - The name the value is set to.
/// * `value` - The value.
/// * `path` - The file, as an error shows it.
///
/// # Returns
///
/// Nothing, or the error of a value of the wrong form.
fn check_value(name: &str, value: &Value, path: &str) -> zakac_host_errors::Result<()>
{
    #[rustfmt::skip]
    let bad = || Error::ConfigBadValue { path: path.into(), name: name.into() };
    values(value).map(drop).ok_or_else(bad)
}

/// The error of a name no command takes.
///
/// # Arguments
///
/// * `path` - The file, as the error shows it.
/// * `name` - The name.
///
/// # Returns
///
/// The error.
fn unknown(path: &str, name: &str) -> Error
{
    Error::ConfigUnknownName { path: path.into(), name: name.into() }
}

/// Whether a name is that of a setting an env var holds.
///
/// # Arguments
///
/// * `name` - The name.
///
/// # Returns
///
/// `true` when it is.
fn is_env_name(name: &str) -> bool { ENV_NAMES.iter().any(|(n, _)| *n == name) }

/// Whether a command or one of its subcommands takes a flag, or the name is
/// that of an env setting.
///
/// # Arguments
///
/// * `cmd` - The command.
/// * `name` - The long name of the flag.
///
/// # Returns
///
/// `true` when one does.
fn takes(cmd: &clap::Command, name: &str) -> bool
{
    is_env_name(name)
        || own_longs(cmd).any(|l| l == name)
        || cmd.get_subcommands().any(|sub| takes(sub, name))
}

/// The long names of the flags of one command that a config file can set.
///
/// # Arguments
///
/// * `cmd` - The command.
///
/// # Returns
///
/// The names.
fn own_longs(cmd: &clap::Command) -> impl Iterator<Item = &str>
{
    cmd.get_arguments().filter_map(clap::Arg::get_long).filter(|l| !SHARED.contains(l))
}

/// A value as the words of a flag: one for text, a number or a truth value,
/// one for each item of a list.
///
/// # Arguments
///
/// * `value` - The value.
///
/// # Returns
///
/// The words, or `None` for a value of another form.
fn values(value: &Value) -> Option<Vec<String>>
{
    match value
    {
        Value::Array(items) => items.iter().map(scalar).collect(),
        _ => scalar(value).map(|word| vec![word]),
    }
}

/// A value that is not a list as the word of a flag.
///
/// # Arguments
///
/// * `value` - The value.
///
/// # Returns
///
/// The word, or `None` for a list, a table or a date.
fn scalar(value: &Value) -> Option<String>
{
    match value
    {
        Value::String(text) => Some(text.clone()),
        Value::Integer(n) => Some(n.to_string()),
        Value::Float(f) => Some(f.to_string()),
        Value::Boolean(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Set the default of each flag a config file names, in a command and its
/// subcommands. A name in the section of a command wins over the same name
/// at the top of the file.
///
/// # Arguments
///
/// * `cmd` - The command.
/// * `table` - The file.
/// * `section` - The section of the command.
///
/// # Returns
///
/// The command with the defaults set.
pub fn set_defaults(
    cmd: clap::Command, table: &Table, section: Option<&str>,
) -> clap::Command
{
    let own = section.and_then(|s| table.get(s)).and_then(Value::as_table);
    let top = |long: &str| table.get(long).filter(|v| !v.is_table());
    let found: Vec<(clap::Id, Vec<String>)> = cmd
        .get_arguments()
        .filter_map(|arg| {
            let long = arg.get_long().filter(|l| !SHARED.contains(l))?;
            let value = own.and_then(|t| t.get(long)).or_else(|| top(long))?;
            Some((arg.get_id().clone(), values(value)?))
        })
        .collect();
    let set = |cmd: clap::Command, (id, words): (clap::Id, Vec<String>)| {
        cmd.mut_arg(id, |arg| arg.default_values(words))
    };
    let cmd = found.into_iter().fold(cmd, set);
    let names: Vec<String> =
        cmd.get_subcommands().map(|s| s.get_name().into()).collect();
    names.into_iter().fold(cmd, |cmd, name| {
        cmd.mut_subcommand(&name, |sub| set_defaults(sub, table, Some(&name)))
    })
}

/// The env settings of a config file: those at the top, then those in the
/// section of the command that runs.
///
/// # Arguments
///
/// * `table` - The file.
/// * `active` - The section of the command that runs.
///
/// # Returns
///
/// Each var with the value the file gives it.
fn env_values(table: &Table, active: Option<&str>) -> Vec<(&'static str, String)>
{
    let own = active.and_then(|s| table.get(s)).and_then(Value::as_table);
    let pick = |t: &Table| -> Vec<(&'static str, String)> {
        ENV_NAMES
            .iter()
            .filter_map(|&(name, var)| env_value(name, var, t.get(name)?))
            .collect()
    };
    let mut found = pick(table);
    found.extend(own.map(pick).unwrap_or_default());
    found
}

/// The value of one env var from its config value. `color = false` sets
/// `NO_COLOR`; `color = true` leaves it as it is.
///
/// # Arguments
///
/// * `name` - The config name.
/// * `var` - The env var.
/// * `value` - The config value.
///
/// # Returns
///
/// The var and its value, or `None` when the file sets nothing.
fn env_value(
    name: &str, var: &'static str, value: &Value,
) -> Option<(&'static str, String)>
{
    ret_if!(
        name == "color",
        value.as_bool().is_some_and(|b| !b).then(|| (var, "1".into()))
    );
    values(value).map(|words| (var, words.join(" ")))
}

/// Set each env var that is not set, so a var set in the shell wins.
///
/// # Arguments
///
/// * `env` - Each var with its value.
fn set_env(env: BTreeMap<&'static str, String>)
{
    for (var, value) in
        env.into_iter().filter(|(var, _)| std::env::var_os(var).is_none())
    {
        // SAFETY: this runs before the command does its work. The one other
        // thread is the main thread, blocked in `join` on this one, so no
        // thread reads or writes the env at the same time.
        unsafe { std::env::set_var(var, value) };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A small tree: `zakac` with `build` and `run`.
    fn tree() -> clap::Command
    {
        let flag = |id: &'static str| {
            clap::Arg::new(id).long(id).action(clap::ArgAction::SetTrue)
        };
        let opt = |id: &'static str| clap::Arg::new(id).long(id);
        let list = |id: &'static str| opt(id).action(clap::ArgAction::Append);
        let build = clap::Command::new("build")
            .arg(opt("target"))
            .arg(flag("release"))
            .arg(list("cc-flag"))
            .arg(opt("max-depth"));
        let run = clap::Command::new("run").arg(opt("max-depth"));
        clap::Command::new("zakac").subcommand(build).subcommand(run)
    }

    /// Parse a command line against a tree with the defaults of a file.
    fn parse(text: &str, line: &[&str]) -> clap::ArgMatches
    {
        let table = parse_text(text, "t.toml").unwrap();
        let cmd = set_defaults(tree(), &table, None);
        cmd.try_get_matches_from(line.iter().copied()).unwrap()
    }

    fn sub<'a>(m: &'a clap::ArgMatches, name: &str) -> &'a clap::ArgMatches
    {
        m.subcommand_matches(name).unwrap()
    }

    fn one(m: &clap::ArgMatches, id: &str) -> Option<String>
    {
        m.get_one::<String>(id).cloned()
    }

    #[test]
    fn a_file_value_is_the_default_and_the_command_line_wins()
    {
        let m = parse("target = \"c\"", &["zakac", "build"]);
        assert_eq!(one(sub(&m, "build"), "target").as_deref(), Some("c"));
        let m = parse("target = \"c\"", &["zakac", "build", "--target", "vm"]);
        assert_eq!(one(sub(&m, "build"), "target").as_deref(), Some("vm"));
    }

    #[test]
    fn a_section_wins_over_the_top_and_reaches_its_command_alone()
    {
        let text = "max-depth = 10\n[run]\nmax-depth = 20";
        let m = parse(text, &["zakac", "run"]);
        assert_eq!(one(sub(&m, "run"), "max-depth").as_deref(), Some("20"));
        let m = parse(text, &["zakac", "build"]);
        assert_eq!(one(sub(&m, "build"), "max-depth").as_deref(), Some("10"));
    }

    #[test]
    fn truth_values_and_lists_fill_their_flags()
    {
        let m = parse(
            "[build]\nrelease = true\ncc-flag = [\"-a\", \"-b\"]",
            &["zakac", "build"],
        );
        let b = sub(&m, "build");
        assert!(b.get_flag("release"));
        let words: Vec<&String> = b.get_many("cc-flag").unwrap().collect();
        assert_eq!(words, ["-a", "-b"]);
    }

    #[test]
    fn a_later_file_wins()
    {
        let first = parse_text("target = \"c\"", "a.toml").unwrap();
        let second = parse_text("target = \"vm\"", "b.toml").unwrap();
        let cmd = set_defaults(set_defaults(tree(), &first, None), &second, None);
        let m = cmd.try_get_matches_from(["zakac", "build"]).unwrap();
        assert_eq!(one(sub(&m, "build"), "target").as_deref(), Some("vm"));
    }

    #[test]
    fn unknown_names_and_bad_values_are_refused()
    {
        let check = |text: &str| {
            let table = parse_text(text, "t.toml").unwrap();
            check_file(&table, &tree(), None, "t.toml")
        };
        assert!(
            check("target = \"c\"\ncarets = \"off\"\n[run]\nmax-depth = 3").is_ok()
        );
        assert!(matches!(check("max-dept = 3"), Err(Error::ConfigUnknownName { .. })));
        assert!(matches!(check("[lex]\nx = 1"), Err(Error::ConfigUnknownName { .. })));
        assert!(matches!(
            check("[run]\ntarget = \"c\""),
            Err(Error::ConfigUnknownName { .. })
        ));
        assert!(matches!(
            check("target = { a = 1 }"),
            Err(Error::ConfigBadValue { .. })
        ));
        assert!(matches!(
            parse_text("target = ", "t"),
            Err(Error::ConfigNotToml { .. })
        ));
    }

    #[test]
    fn the_executable_of_one_stage_checks_its_own_section_alone()
    {
        let lexer =
            clap::Command::new("zakac_lexer").arg(clap::Arg::new("o").long("o"));
        let table = parse_text("target = \"c\"\n[build]\nx = 1\n[lex]\no = \"f\"", "t")
            .unwrap();
        assert!(check_file(&table, &lexer, Some("lex"), "t").is_ok());
        let table = parse_text("[lex]\nx = 1", "t").unwrap();
        assert!(check_file(&table, &lexer, Some("lex"), "t").is_err());
    }

    #[test]
    fn the_config_flag_is_found_before_the_end_of_flags()
    {
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let found = given_path(&args(&["z", "run", "--config", "a.toml", "x"]));
        assert_eq!(found, Some(PathBuf::from("a.toml")));
        let found = given_path(&args(&["z", "--config=b.toml", "run"]));
        assert_eq!(found, Some(PathBuf::from("b.toml")));
        assert_eq!(given_path(&args(&["z", "run", "--", "--config", "c"])), None);
    }

    #[test]
    fn env_settings_come_from_the_top_and_the_section_that_runs()
    {
        let table =
            parse_text("color = false\n[run]\nbacktrace = \"full\"", "t").unwrap();
        let found = env_values(&table, Some("run"));
        assert!(found.contains(&("NO_COLOR", "1".into())));
        assert!(found.contains(&("ZAKA_BACKTRACE", "full".into())));
        assert!(
            env_values(&table, Some("build"))
                .iter()
                .all(|(v, _)| *v != "ZAKA_BACKTRACE")
        );
    }
}
