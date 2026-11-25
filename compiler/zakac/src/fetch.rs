//! The `zakac fetch` command: clone each git dep of a package and of its deps
//! into the cache, and pin each one to a commit in `zakac.lock`. Builds read
//! the lock and the cache alone, so a build never reaches the network.

use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as Process;

use zakac_cli_common::clap;
use zakac_cli_common::clap::Arg;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::clap_utils::CommandInfo;
use zakac_cli_common::clap_utils::report_command_error;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_cli_common::config::LOCAL_FILE;
use zakac_cli_common::config::cache_root;
use zakac_cli_common::config::nearest_local;
use zakac_host_errors::Error;
use zakac_host_errors::errors::IOErrorAction;
use zakac_host_errors::errors::IOErrorContext;
use zakac_i18n::t;
use zakac_parser::package::Checkout;
use zakac_parser::package::LOCK_FILE;
use zakac_parser::package::Lock;
use zakac_parser::package::Source;
use zakac_parser::package::checkout_dir;
use zakac_parser::package::read_lock;
use zakac_parser::package::read_manifest;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_span::file_loader::RealFileLoader;

/// The argument that names the package, a folder in it or its `zakac.toml`.
const PATH_ARG: &str = "path";

/// The `zakac fetch` command.
pub(crate) struct ZakacFetch
{
    /// The clap command.
    cmd_instance: clap::Command,
}

/// A walk over the manifests of a package and its deps.
struct Walk<'a, 'io>
{
    /// The cache folder the checkouts go in.
    cache: PathBuf,
    /// The lock file as it was, whose checkouts are kept when the cache
    /// still holds them.
    old: Lock,
    /// The checkouts of this walk.
    found: Vec<Checkout>,
    /// The manifests read so far.
    seen: HashSet<PathBuf>,
    /// The standard streams.
    io: &'a mut CommandIO<'io>,
}

impl Walk<'_, '_>
{
    /// Read a manifest and fetch each git dep it lists, then walk the
    /// manifest of each dep.
    ///
    /// # Arguments
    ///
    /// * `file` - The `zakac.toml`.
    ///
    /// # Returns
    ///
    /// Nothing, or the error of a manifest or of git.
    fn visit(&mut self, file: PathBuf) -> zakac_host_errors::Result<()>
    {
        ret_if!(!self.seen.insert(file.clone()), Ok(()));
        let manifest = read_manifest(&RealFileLoader, &file)?;
        for (name, dep) in &manifest.deps
        {
            let root = match dep.source()
            {
                Some(Source::Path(path)) => manifest.root.join(path),
                Some(Source::Git(url, rev)) => self.checkout(name, url, rev)?,
                None => continue,
            };
            let next = root.join(LOCAL_FILE);
            do_if!(next.is_file(), self.visit(next)?);
        }
        Ok(())
    }

    /// The checkout of a rev of a repo: the one the lock pins when the cache
    /// holds it, else a fresh clone at the commit the rev stands at now.
    ///
    /// # Arguments
    ///
    /// * `name` - The dep, as the manifest names it.
    /// * `url` - The URL of the repo.
    /// * `rev` - The rev.
    ///
    /// # Returns
    ///
    /// The folder of the checkout, or the error of git or of the cache.
    fn checkout(
        &mut self, name: &str, url: &str, rev: &str,
    ) -> zakac_host_errors::Result<PathBuf>
    {
        let pinned = self.found.iter().find(|c| c.url == url && c.rev == rev);
        let pinned = pinned
            .map(|c| c.commit.clone())
            .or_else(|| self.old.commit_of(url, rev).map(str::to_string));
        let kept =
            pinned.map(|c| checkout_dir(&self.cache, url, &c)).filter(|d| d.is_dir());
        let dir = match kept
        {
            Some(dir) => dir,
            None => self.clone_at(name, url, rev)?,
        };
        let commit = dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
        self.found.push(Checkout::builder().url(url).rev(rev).commit(commit).build());
        Ok(dir)
    }

    /// Clone a repo, check out a rev, and keep the checkout under its commit.
    ///
    /// # Arguments
    ///
    /// * `name` - The dep, as the manifest names it.
    /// * `url` - The URL of the repo.
    /// * `rev` - The rev.
    ///
    /// # Returns
    ///
    /// The folder of the checkout, or the error of git or of the cache.
    fn clone_at(
        &mut self, name: &str, url: &str, rev: &str,
    ) -> zakac_host_errors::Result<PathBuf>
    {
        let repo = checkout_dir(&self.cache, url, "-");
        let repo = repo.parent().unwrap_or(&self.cache).to_path_buf();
        let made = std::fs::create_dir_all(&repo);
        made.map_err(|e| {
            Error::from_io(e, IOErrorContext::new(repo.clone(), IOErrorAction::Write))
        })?;
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
        let stamp = stamp.map(|d| d.as_nanos()).unwrap_or_default();
        let work = repo.join(format!(".fetch-{}-{stamp}", std::process::id()));
        let work_arg = work.to_string_lossy().into_owned();
        git(url, &["clone", "--quiet", url, &work_arg])?;
        git(url, &["-C", &work_arg, "checkout", "--quiet", "--detach", rev])?;
        let commit = git(url, &["-C", &work_arg, "rev-parse", "HEAD"])?;
        let dir = repo.join(commit.trim());
        // A checkout of the commit from an earlier fetch stands as it is.
        let moved = place_checkout(&work, &dir);
        moved.map_err(|e| {
            Error::from_io(e, IOErrorContext::new(dir.clone(), IOErrorAction::Write))
        })?;
        let short: String = commit.trim().chars().take(12).collect();
        let line =
            t!("fetch.fetched", name = name, url = url, rev = rev, commit = short);
        let _ = writeln!(self.io.stdout, "{line}");
        Ok(dir)
    }
}

/// Move a fresh checkout to its place, unless a checkout stands there; then
/// the fresh one is left for removal.
///
/// # Arguments
///
/// * `work` - The fresh checkout.
/// * `dir` - Its place.
///
/// # Returns
///
/// Nothing, or the error of the move.
fn place_checkout(work: &Path, dir: &Path) -> std::io::Result<()>
{
    if dir.is_dir()
    {
        // shortcut: git locks its object files against writes on Windows, so
        // the removal can fail; the leftover folder costs room alone.
        let _ = std::fs::remove_dir_all(work);
        return Ok(());
    }
    std::fs::rename(work, dir)
}

/// Run git and give back what it printed.
///
/// # Arguments
///
/// * `url` - The repo, for the error.
/// * `args` - The arguments.
///
/// # Returns
///
/// The standard output, or the error of git.
fn git(url: &str, args: &[&str]) -> zakac_host_errors::Result<String>
{
    let failed = |msg: String| Error::GitFailed { url: url.into(), msg };
    let out =
        Process::new("git").args(args).output().map_err(|e| failed(e.to_string()))?;
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    ret_if!(!out.status.success(), Err(failed(stderr)));
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Fetch each git dep of the package a path is in, and write its lock file.
///
/// # Arguments
///
/// * `start` - A folder in the package, or its `zakac.toml`.
/// * `io` - The standard streams.
///
/// # Returns
///
/// Nothing, or the error of a manifest, of git or of the lock file.
fn fetch(start: &Path, io: &mut CommandIO<'_>) -> zakac_host_errors::Result<()>
{
    let start = std::path::absolute(start).unwrap_or_else(|_| start.to_path_buf());
    let folder = if start.is_file()
    {
        start.parent().unwrap_or(&start)
    }
    else
    {
        &start
    };
    let shown = folder.display().to_string().replace(std::path::MAIN_SEPARATOR, "/");
    let file = nearest_local(folder).ok_or(Error::NoManifest { path: shown })?;
    let lock_file = file.with_file_name(LOCK_FILE);
    let old = read_lock(&RealFileLoader, &lock_file)?;
    // shortcut: a user with no cache folder fetches into the temp folder,
    // which the OS can clear; name one through ZAKAC_CACHE to keep them.
    let cache = cache_root().unwrap_or_else(std::env::temp_dir);
    #[rustfmt::skip]
    let mut walk = Walk { cache, old, found: Vec::new(), seen: HashSet::new(), io };
    walk.visit(file)?;
    let mut checkouts = walk.found;
    checkouts.sort();
    checkouts.dedup();
    let pinned = checkouts.len();
    let lock = Lock::builder().checkouts(checkouts).build();
    let written = std::fs::write(&lock_file, lock.text());
    let ctx = IOErrorContext::new(lock_file.clone(), IOErrorAction::Write);
    written.map_err(|e| Error::from_io(e, ctx))?;
    let _ = writeln!(walk.io.stdout, "{}", t!("fetch.done", n = pinned));
    Ok(())
}

/// Run `zakac fetch`.
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
    let info = CommandInfo::new(ZakacFetch::name(), ZakacFetch::version());
    let start =
        matches.get_one::<String>(PATH_ARG).map_or_else(|| ".".into(), PathBuf::from);
    match fetch(&start, io)
    {
        Ok(()) => ExitCode::Success,
        Err(error) => report_command_error(io, &info, error),
    }
}

impl Command for ZakacFetch
{
    /// Get the canonical name of the command.
    ///
    /// # Returns
    ///
    /// The canonical name of the command.
    fn name() -> &'static str { "fetch" }

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
        let path = Arg::new(PATH_ARG).value_name("PATH").help(t!("fetch.path"));
        setup.with(|cmd| cmd.about(t!("fetch.about")).arg(path))
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
