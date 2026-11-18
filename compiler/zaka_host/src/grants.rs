//! The rights a run holds, as Deno grants them: reading and writing files,
//! reading the environment, and calling C. A run holds none unless its
//! command grants them; a right can cover each path (or each variable) or a
//! list of them. A run on a terminal can ask for a right it lacks.

use std::collections::HashSet;
use std::fmt;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use strum::IntoStaticStr;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

/// A right a run can hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Right
{
    /// Read files, list folders and tell what a path holds.
    Read,
    /// Write, add to, remove and move files, and make folders.
    Write,
    /// Read the environment.
    Env,
    /// Call C functions.
    Ffi,
}

/// What a run holds of one right.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Grant
{
    /// Nothing.
    #[default]
    None,
    /// Each path, or each variable.
    All,
    /// The paths under these folders or files, or these variables.
    Some(Vec<String>),
}

/// What a denied effect does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnDenial
{
    /// The effect fails with its error result, as an effect the OS refuses.
    #[default]
    Fault,
    /// The run stops: a comptime run, where a denial stops the build.
    Stop,
}

/// The rights of a run.
#[derive(Debug, Clone, PartialEq, Eq, Default, bon::Builder)]
pub struct Grants
{
    /// Reading files.
    #[builder(default)]
    pub read: Grant,
    /// Writing files.
    #[builder(default)]
    pub write: Grant,
    /// Reading the environment.
    #[builder(default)]
    pub env: Grant,
    /// Calling C functions.
    #[builder(default)]
    pub ffi: bool,
    /// Whether a run on a terminal can ask for a right it lacks.
    #[builder(default)]
    pub prompt: bool,
    /// What a denied effect does.
    #[builder(default)]
    pub on_denial: OnDenial,
}

impl Grants
{
    /// Each right, on each path and variable, with no question asked.
    ///
    /// # Returns
    ///
    /// The grants.
    pub fn all() -> Self
    {
        #[rustfmt::skip]
        return Grants::builder().read(Grant::All).write(Grant::All).env(Grant::All).ffi(true)
            .build();
    }

    /// Whether the grants cover an effect.
    ///
    /// # Arguments
    ///
    /// * `right` - The right the effect needs.
    /// * `subject` - Its path, as an absolute normal path, or its variable;
    ///   blank for a C call or the whole environment.
    ///
    /// # Returns
    ///
    /// `true` when they do.
    pub fn allows(&self, right: Right, subject: &str) -> bool
    {
        let grant = match right
        {
            Right::Read => &self.read,
            Right::Write => &self.write,
            Right::Env => &self.env,
            Right::Ffi => return self.ffi,
        };
        match grant
        {
            Grant::None => false,
            Grant::All => true,
            Grant::Some(_) if subject.is_empty() => false,
            Grant::Some(list) if right == Right::Env =>
            {
                list.iter().any(|n| n == subject)
            }
            Grant::Some(list) => list.iter().any(|p| Path::new(subject).starts_with(p)),
        }
    }

    /// Grant a right on each path and variable, as the answer "all" does.
    ///
    /// # Arguments
    ///
    /// * `right` - The right.
    pub fn grant_all(&mut self, right: Right)
    {
        match right
        {
            Right::Read => self.read = Grant::All,
            Right::Write => self.write = Grant::All,
            Right::Env => self.env = Grant::All,
            Right::Ffi => self.ffi = true,
        }
    }
}

/// The answer to a question for a right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer
{
    /// This path, variable or C function, for the rest of the run: `Y`.
    This,
    /// This effect alone; the next one asks again: `O`.
    Once,
    /// Each effect of this right, for the rest of the run: `A`.
    All,
    /// No, for this path, variable or C function, for the rest of the run:
    /// `N`.
    No,
}

impl Answer
{
    /// The answer a person typed: its first letter, in either case; each
    /// other line is no.
    ///
    /// # Arguments
    ///
    /// * `line` - The line.
    ///
    /// # Returns
    ///
    /// The answer.
    pub fn typed(line: &str) -> Answer
    {
        match line.trim().chars().next().map(|c| c.to_ascii_uppercase())
        {
            Some('Y') => Answer::This,
            Some('O') => Answer::Once,
            Some('A') => Answer::All,
            _ => Answer::No,
        }
    }
}

/// What asks the person at the terminal for a right.
pub trait Asker: Send + Sync
{
    /// Ask for a right.
    ///
    /// # Arguments
    ///
    /// * `right` - The right.
    /// * `subject` - Its path or variable, blank for C calls or the whole
    ///   environment.
    ///
    /// # Returns
    ///
    /// The answer.
    fn ask(&mut self, right: Right, subject: &str) -> Answer;
}

/// An asker that gives answers set in advance, then "no": for tests.
#[derive(Debug, Clone, Default)]
pub struct ScriptAsker
{
    /// The answers, first first.
    pub answers: Vec<Answer>,
    /// The questions asked so far.
    pub asked: Vec<(Right, String)>,
}

impl Asker for ScriptAsker
{
    fn ask(&mut self, right: Right, subject: &str) -> Answer
    {
        self.asked.push((right, subject.to_string()));
        tern!(self.answers.is_empty(), Answer::No, self.answers.remove(0))
    }
}

/// What asks for a right, shared by each clone of a run.
pub type SharedAsker = Arc<Mutex<Box<dyn Asker>>>;

/// The rights of a run, the asker, the answers so far, and the folder its
/// relative paths start from.
#[derive(Clone)]
pub struct Gate
{
    /// The rights.
    pub grants: Grants,
    /// What asks for a right the run lacks; `None` asks nothing.
    pub asker: Option<SharedAsker>,
    /// The folder relative paths start from: the folder of the program for a
    /// comptime run, whose world then sees each path under it; `None` for
    /// the folder the run started in, with paths handed on as written.
    pub root: Option<PathBuf>,
    /// The paths, variables and C functions answered `Y` so far.
    allowed: HashSet<(Right, String)>,
    /// The effects answered "no" so far.
    refused: HashSet<(Right, String)>,
}

/// A denied effect: the right it needed and its subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Denial
{
    /// The right.
    pub right: Right,
    /// The path or variable, as the program gave it; blank for C calls or
    /// the whole environment.
    pub subject: String,
}

impl fmt::Debug for Gate
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("Gate")
            .field("grants", &self.grants)
            .field("root", &self.root)
            .finish()
    }
}

impl Default for Gate
{
    fn default() -> Self { Gate::new(Grants::default()) }
}

impl Gate
{
    /// The gate with an asker.
    ///
    /// # Arguments
    ///
    /// * `asker` - What asks for a right the run lacks.
    ///
    /// # Returns
    ///
    /// The gate.
    pub fn with_asker(mut self, asker: Box<dyn Asker>) -> Self
    {
        self.asker = Some(Arc::new(Mutex::new(asker)));
        self
    }

    /// A gate with rights, no asker, and the starting folder of the run.
    ///
    /// # Arguments
    ///
    /// * `grants` - The rights.
    ///
    /// # Returns
    ///
    /// The gate.
    pub fn new(grants: Grants) -> Self
    {
        #[rustfmt::skip]
        return Gate { grants, asker: None, root: None, allowed: HashSet::new(),
            refused: HashSet::new() };
    }

    /// Check an effect, asking for the right when the run can: the path the
    /// world gets, or the denial.
    ///
    /// # Arguments
    ///
    /// * `right` - The right the effect needs.
    /// * `subject` - Its path or variable, as the program gave it; blank for C
    ///   calls or the whole environment.
    ///
    /// # Returns
    ///
    /// The path or variable to hand the world, or the denial.
    pub fn check(&mut self, right: Right, subject: &str) -> Result<String, Denial>
    {
        let paths = matches!(right, Right::Read | Right::Write) && !subject.is_empty();
        let full = tern!(paths, self.full_path(subject), subject.to_string());
        let handed =
            tern!(self.root.is_some() && paths, full.clone(), subject.to_string());
        let key = (right, full.clone());
        let allowed = self.grants.allows(right, &full) || self.allowed.contains(&key);
        ret_if!(allowed, Ok(handed));
        let denial = Denial { right, subject: subject.to_string() };
        let ask = self.grants.prompt && !self.refused.contains(&key);
        let asker = self.asker.as_ref().filter(|_| ask);
        let answer =
            asker.map(|a| a.lock().map_or(Answer::No, |mut a| a.ask(right, subject)));
        match answer
        {
            Some(Answer::This) => _ = self.allowed.insert(key),
            Some(Answer::Once) =>
            {}
            Some(Answer::All) => self.grants.grant_all(right),
            Some(Answer::No) =>
            {
                self.refused.insert(key);
                return Err(denial);
            }
            None => return Err(denial),
        }
        Ok(handed)
    }

    /// A path as an absolute normal path: joined to the root, or to the
    /// folder the run started in, with `.` and `..` worked out.
    ///
    /// # Arguments
    ///
    /// * `path` - The path.
    ///
    /// # Returns
    ///
    /// The path, as text.
    fn full_path(&self, path: &str) -> String
    {
        let start = self.root.clone().or_else(|| std::env::current_dir().ok());
        let joined = start.unwrap_or_default().join(path);
        normal(&joined).to_string_lossy().into_owned()
    }
}

/// A path with `.` parts removed and `..` parts applied, without reading the
/// disk.
///
/// # Arguments
///
/// * `path` - The path.
///
/// # Returns
///
/// The path.
pub fn normal(path: &Path) -> PathBuf
{
    let mut out = PathBuf::new();
    for part in path.components()
    {
        match part
        {
            Component::CurDir =>
            {}
            Component::ParentDir => _ = out.pop(),
            other => out.push(other),
        }
    }
    out
}

/// The paths of a grant as given on the command line, each made absolute
/// against the folder the command runs in.
///
/// # Arguments
///
/// * `list` - The paths.
///
/// # Returns
///
/// The grant: each path when the list is blank, else those paths.
pub fn path_grant(list: &[String]) -> Grant
{
    ret_if!(list.is_empty(), Grant::All);
    let here = std::env::current_dir().unwrap_or_default();
    let full =
        list.iter().map(|p| normal(&here.join(p)).to_string_lossy().into_owned());
    Grant::Some(full.collect())
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn a_path_grant_covers_the_paths_under_it()
    {
        let grants = Grants::builder().read(Grant::Some(vec!["/a/b".into()])).build();
        assert!(grants.allows(Right::Read, "/a/b"));
        assert!(grants.allows(Right::Read, "/a/b/c.txt"));
        assert!(!grants.allows(Right::Read, "/a/bc"));
        assert!(!grants.allows(Right::Write, "/a/b/c.txt"));
    }

    #[test]
    fn a_path_that_climbs_out_is_checked_where_it_lands()
    {
        let mut gate =
            Gate::new(Grants::builder().read(Grant::Some(vec!["/r".into()])).build());
        gate.root = Some(PathBuf::from("/r"));
        // The path is the host's, with `\` on Windows, so it compares as a
        // path.
        let landed = gate.check(Right::Read, "x/../y.txt").map(PathBuf::from);
        assert_eq!(landed, Ok(PathBuf::from("/r/y.txt")));
        assert!(gate.check(Right::Read, "../s/y.txt").is_err());
    }

    #[test]
    fn an_env_grant_names_its_variables()
    {
        let grants = Grants::builder().env(Grant::Some(vec!["HOME".into()])).build();
        assert!(grants.allows(Right::Env, "HOME"));
        assert!(!grants.allows(Right::Env, "PATH"));
        assert!(!grants.allows(Right::Env, ""));
    }

    #[test]
    fn answers_grant_once_for_good_or_not_at_all()
    {
        let answers =
            vec![Answer::Once, Answer::No, Answer::This, Answer::Once, Answer::All];
        let asker = Box::new(ScriptAsker { answers, asked: Vec::new() });
        let mut gate =
            Gate::new(Grants::builder().prompt(true).build()).with_asker(asker);
        assert!(gate.check(Right::Ffi, "abs").is_ok());
        assert!(gate.check(Right::Env, "HOME").is_err());
        // A refused effect is not asked again.
        assert!(gate.check(Right::Env, "HOME").is_err());
        // "Y" holds for the rest of the run; "O" is asked again.
        assert!(gate.check(Right::Env, "PATH").is_ok());
        assert!(gate.check(Right::Env, "PATH").is_ok());
        assert!(gate.check(Right::Ffi, "abs").is_ok());
        // "A" grants each variable.
        assert!(gate.check(Right::Env, "USER").is_ok());
        assert!(gate.check(Right::Env, "LANG").is_ok());
    }

    #[test]
    fn a_typed_answer_reads_its_first_letter_in_either_case()
    {
        assert_eq!(Answer::typed("Y\n"), Answer::This);
        assert_eq!(Answer::typed("o"), Answer::Once);
        assert_eq!(Answer::typed("a"), Answer::All);
        assert_eq!(Answer::typed("N"), Answer::No);
        assert_eq!(Answer::typed(""), Answer::No);
    }
}
