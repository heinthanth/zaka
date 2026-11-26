//! The flags that grant a run its rights, as Deno names them:
//! `--allow-read[=<paths>]`, `--allow-write[=<paths>]`,
//! `--allow-env[=<names>]`, `--allow-ffi`, `--allow-run[=<programs>]`,
//! `--allow-all` (`-A`) and `--no-permission-prompt` for the run of a
//! program; the same with `comptime-` in front for the runs of comptime calls,
//! which never ask and never run another program.

use clap::Arg;
use clap::ArgAction;
use clap::ArgMatches;
use zaka_host::Grant;
use zaka_host::Grants;
use zaka_host::path_grant;
use zakac_i18n::t;
use zakac_rust_sugars::tern;

/// Whose rights a set of flags grants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Granted
{
    /// The run of the program.
    Run,
    /// The runs of comptime calls.
    Comptime,
}

impl Granted
{
    /// The words in front of `allow-` in each flag.
    ///
    /// # Returns
    ///
    /// The words.
    fn prefix(self) -> &'static str { tern!(self == Granted::Run, "", "comptime-") }

    /// The id of a flag.
    ///
    /// # Arguments
    ///
    /// * `right` - The words after `allow-`.
    ///
    /// # Returns
    ///
    /// The id.
    fn id(self, right: &str) -> String { format!("{}allow-{right}", self.prefix()) }
}

/// Add the flags that grant rights to a command.
///
/// # Arguments
///
/// * `cmd` - The command.
/// * `who` - Whose rights the flags grant.
///
/// # Returns
///
/// The command with the flags.
pub fn attach_grant_args(cmd: clap::Command, who: Granted) -> clap::Command
{
    let listed = |right: &'static str, value: &'static str| {
        Arg::new(who.id(right))
            .long(who.id(right))
            .value_name(value)
            .help(t!("cli.grant", right = right))
            .num_args(0..=1)
            .require_equals(true)
            .value_delimiter(',')
            .action(ArgAction::Append)
    };
    let ffi = Arg::new(who.id("ffi"))
        .long(who.id("ffi"))
        .help(t!("cli.grant_ffi"))
        .action(ArgAction::SetTrue);
    let mut all = Arg::new(who.id("all"))
        .long(who.id("all"))
        .help(t!("cli.grant_all"))
        .action(ArgAction::SetTrue);
    all = tern!(who == Granted::Run, all.short('A'), all);
    let cmd = cmd.arg(listed("read", "PATHS")).arg(listed("write", "PATHS"));
    let cmd = cmd.arg(listed("env", "NAMES")).arg(ffi).arg(all);
    if who == Granted::Comptime
    {
        return cmd;
    }
    let cmd = cmd.arg(listed("run", "PROGRAMS"));
    let prompt = Arg::new("no-permission-prompt")
        .long("no-permission-prompt")
        .help(t!("cli.no_prompt"))
        .action(ArgAction::SetTrue);
    cmd.arg(prompt)
}

/// The rights the flags of a command grant.
///
/// # Arguments
///
/// * `matches` - The parsed flags.
/// * `who` - Whose rights the flags grant.
///
/// # Returns
///
/// The rights: none for a command with no such flags.
pub fn grants_from(matches: &ArgMatches, who: Granted) -> Grants
{
    let known = |id: &str| matches!(matches.try_contains_id(id), Ok(true));
    let flag = |id: &str| known(id) && matches.get_flag(id);
    let list = |right: &str| -> Option<Vec<String>> {
        let id = who.id(right);
        ret_none(known(&id))?;
        let values = matches.get_many::<String>(&id);
        Some(values.map(|v| v.cloned().collect()).unwrap_or_default())
    };
    let paths = |right: &str| list(right).map_or(Grant::None, |l| path_grant(&l));
    let names = |l: Vec<String>| tern!(l.is_empty(), Grant::All, Grant::Some(l));
    let env = list("env").map_or(Grant::None, names);
    let run = list("run").map_or(Grant::None, names);
    let all = flag(&who.id("all"));
    let prompt = who == Granted::Run && !flag("no-permission-prompt");
    // A comptime run never runs another program, even with each other right.
    let run = tern!(who == Granted::Run && all, Grant::All, run);
    if all
    {
        return Grants { prompt, run, ..Grants::all() };
    }
    #[rustfmt::skip]
    return Grants::builder().read(paths("read")).write(paths("write")).env(env)
        .ffi(flag(&who.id("ffi"))).run(run).prompt(prompt).build();
}

/// `Some` when a flag was given, for `?`.
///
/// # Arguments
///
/// * `given` - Whether it was.
///
/// # Returns
///
/// `Some(())` when given.
fn ret_none(given: bool) -> Option<()> { given.then_some(()) }
