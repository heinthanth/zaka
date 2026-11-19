//! The kinds of the handles of threads: a job (`<kind> ပြန်ပေးမည့်
//! အပြိုင်လုပ်ငန်း`) and a channel (`<kind> ပို့နိုင်သော ဒေတာလမ်းကြောင်း`). A value of
//! either is a number that names a job or a channel to the scheduler, one word
//! wide; it holds no count. The room of a channel is no part of its kind: the
//! declaration that makes the channel gives it.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;

use zakac_lexer::spelling::canonical;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::kind::DataKind;

/// The start of the name of a job kind, in signatures: `%job(<kind>)`.
pub const JOB: &str = "%job";

/// The start of the name of a channel kind: `%pipe(<kind>)`.
pub const PIPE: &str = "%pipe";

/// The name of the job kind in phrases, which the core declares.
pub const JOB_WORD: &str = "အပြိုင်လုပ်ငန်း";

/// The name of the channel kind in phrases.
pub const PIPE_WORD: &str = "ဒေတာလမ်းကြောင်း";

/// Whether a name in a kind phrase is the job or the channel kind, in the
/// spelling the compiler keeps.
///
/// # Arguments
///
/// * `name` - The name.
///
/// # Returns
///
/// `Some(true)` for the channel kind, `Some(false)` for the job kind, `None`
/// for another name.
pub fn task_word(name: &str) -> Option<bool>
{
    ret_if!(name == canonical(PIPE_WORD), Some(true));
    tern!(name == JOB_WORD, Some(false), None)
}

/// A job or a channel kind.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TaskKind
{
    /// `true` for a channel, `false` for a job.
    pub pipe: bool,
    /// The kind a channel carries, or the kind the function of a job gives
    /// back; `None` for a job whose function gives nothing.
    pub of: Option<DataKind>,
    /// The signature, such as `%job(isize)`.
    sig: String,
}

impl TaskKind
{
    /// The signature, which names the kind in artifacts.
    ///
    /// # Returns
    ///
    /// The signature.
    pub fn sig(&self) -> &str { &self.sig }
}

/// The kinds made so far, under their signatures.
static MADE: OnceLock<Mutex<HashMap<String, &'static TaskKind>>> = OnceLock::new();

/// The job or channel kind of a carried kind, made once.
///
/// # Arguments
///
/// * `pipe` - `true` for a channel.
/// * `of` - The kind it carries or gives back.
///
/// # Returns
///
/// The kind.
pub fn make(pipe: bool, of: Option<DataKind>) -> &'static TaskKind
{
    let head = tern!(pipe, PIPE, JOB);
    let inner = of.map(|k| k.sig_name()).unwrap_or_default();
    let sig = format!("{head}({inner})");
    let made = MADE.get_or_init(Mutex::default);
    let mut made = made.lock().unwrap_or_else(PoisonError::into_inner);
    let fresh = || -> &'static TaskKind {
        Box::leak(Box::new(TaskKind { pipe, of, sig: sig.clone() }))
    };
    made.entry(sig.clone()).or_insert_with(fresh)
}

/// Read a job or channel kind from its signature.
///
/// # Arguments
///
/// * `text` - The signature.
///
/// # Returns
///
/// The kind, or `None` for a text that is no such signature.
pub fn parse_sig(text: &str) -> Option<&'static TaskKind>
{
    let (pipe, rest) = text
        .strip_prefix(PIPE)
        .map(|r| (true, r))
        .or_else(|| text.strip_prefix(JOB).map(|r| (false, r)))?;
    let inner = rest.strip_prefix('(')?.strip_suffix(')')?;
    ret_if!(inner.is_empty() && !pipe, Some(make(false, None)));
    Some(make(pipe, Some(DataKind::parse(inner)?)))
}
