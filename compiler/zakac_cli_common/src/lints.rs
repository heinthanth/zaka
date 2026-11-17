//! The flags that set the level of a lint group or code: `--lint-allow`,
//! `--lint-warn` and `--lint-error`, each taking names such as `leaks`,
//! `naturalness` or `ZK0708`. A config file writes them in a `[lint]` section.

use std::path::Path;

use clap::Arg;
use clap::ArgAction;
use clap::ArgMatches;
use clap::parser::ValueSource;
use zakac_diagnostics::lint::BadLintName;
use zakac_diagnostics::lint::LintGroup;
use zakac_diagnostics::lint::LintLevel;
use zakac_diagnostics::lint::LintLevels;
use zakac_diagnostics::lint::lint_codes;
use zakac_host_errors::Error;
use zakac_i18n::t;

use crate::config::nearest_local;
use crate::config::parse_text;
use crate::config::values;

/// The flags that set the level of a lint group or code, each with its level.
const LINT_ARGS: [(&str, LintLevel); 3] = [
    ("lint-allow", LintLevel::Off),
    ("lint-warn", LintLevel::Warn),
    ("lint-error", LintLevel::Error),
];

/// Add `--lint-allow`, `--lint-warn` and `--lint-error`.
///
/// # Arguments
///
/// * `cmd` - The command.
///
/// # Returns
///
/// The command with the flags.
pub fn attach_lint_args(cmd: clap::Command) -> clap::Command
{
    let lint = |(id, _): (&'static str, LintLevel), help| {
        Arg::new(id)
            .long(id)
            .value_name("NAME")
            .help(help)
            .action(ArgAction::Append)
            .value_delimiter(',')
            .value_parser(lint_name)
    };
    let [allow, warn, error] = LINT_ARGS;
    #[rustfmt::skip]
    return cmd.arg(lint(allow, t!("cli.allow"))).arg(lint(warn, t!("cli.warn")))
        .arg(lint(error, t!("cli.error")));
}

/// Check one name of `--lint-allow`, `--lint-warn` or `--lint-error`.
///
/// # Arguments
///
/// * `name` - A group, or a code such as `ZK0708`.
///
/// # Returns
///
/// The name, or the error that tells what is wrong with it.
fn lint_name(name: &str) -> Result<String, String>
{
    let error = |bad| match bad
    {
        BadLintName::Unknown => Error::UnknownLintName { name: name.into() },
        BadLintName::NotALint => Error::NotALint { code: name.into() },
    };
    lint_codes(name).map(|_| name.to_string()).map_err(|bad| error(bad).to_string())
}

/// The levels `--lint-allow`, `--lint-warn` and `--lint-error` set: the groups
/// first, then the codes, so a code wins over a group; within each, a flag on
/// the command line wins over a config file, and the later flag wins.
///
/// # Arguments
///
/// * `matches` - The matches of the CLI arguments.
///
/// # Returns
///
/// The levels.
pub fn lint_levels(matches: &ArgMatches) -> LintLevels
{
    let given = |(id, level): (&'static str, LintLevel)| {
        // A value from a config file is a default, which clap places after
        // the command line, so it gets no place: it comes first.
        let on_line = matches.value_source(id) == Some(ValueSource::CommandLine);
        let at = matches.indices_of(id).filter(|_| on_line);
        let at: Vec<usize> = at.map(Iterator::collect).unwrap_or_default();
        let names = matches.get_many::<String>(id).into_iter().flatten();
        let placed =
            names.enumerate().map(move |(i, n)| (at.get(i).copied(), level, n));
        placed.collect::<Vec<_>>()
    };
    let mut all: Vec<_> = LINT_ARGS.into_iter().flat_map(given).collect();
    all.sort_by_key(|(at, _, name)| (name.parse::<LintGroup>().is_err(), *at));
    let set = |levels: LintLevels, (_, level, name): &(_, LintLevel, &String)| {
        levels.with(name, *level).unwrap_or(levels)
    };
    all.iter().fold(LintLevels::default(), set)
}

/// The levels the nearest `zakac.toml` from a folder up sets with the
/// `allow`, `warn` and `error` lists of its `[lint]` section: the groups first,
/// then the codes, as on the command line. The editor and the tests read a
/// project so, with no command line.
///
/// # Arguments
///
/// * `dir` - The folder of the file being checked.
///
/// # Returns
///
/// The levels; none set when no file is found or it does not read.
pub fn project_lint_levels(dir: &Path) -> LintLevels
{
    let table = nearest_local(dir)
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| parse_text(&text, "").ok());
    let Some(table) = table
    else
    {
        return LintLevels::default();
    };
    let named = |(id, level): (&'static str, LintLevel)| {
        let words = table.get(id).and_then(values).unwrap_or_default();
        words.into_iter().map(move |w| (level, w))
    };
    let mut all: Vec<(LintLevel, String)> =
        LINT_ARGS.into_iter().flat_map(named).collect();
    all.sort_by_key(|(_, name)| name.parse::<LintGroup>().is_err());
    let set = |levels: LintLevels, (level, name): &(LintLevel, String)| {
        levels.with(name, *level).unwrap_or(levels)
    };
    all.iter().fold(LintLevels::default(), set)
}

#[cfg(test)]
mod tests
{
    use zakac_diagnostics::Code;

    use super::*;

    #[test]
    fn a_project_file_sets_the_levels_of_its_folder_and_below()
    {
        let dir = std::env::temp_dir().join("zakac_lints_project");
        let inner = dir.join("src");
        std::fs::create_dir_all(&inner).unwrap();
        let toml = "[lint]\nallow = [\"naturalness\"]\nwarn = \"ZK0505\"\n";
        std::fs::write(dir.join("zakac.toml"), toml).unwrap();
        let levels = project_lint_levels(&inner);
        assert_eq!(levels.get(Code::SpokenForm), Some(LintLevel::Off));
        assert_eq!(levels.get(Code::PrintSentenceWithKo), Some(LintLevel::Warn));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
