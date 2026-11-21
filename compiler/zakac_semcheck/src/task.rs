//! The kinds of the handles of threads and channels: a thread (`<kind> ပြန်ပေးမည့်
//! အပြိုင်လုပ်ငန်း`) and a channel (`<kind> ပို့နိုင်သော ဒေတာလမ်းကြောင်း`). A value of
//! either is a number that names a thread or a channel to the scheduler, one
//! word wide; it holds no count. The room of a channel is no part of its kind:
//! the declaration that makes the channel gives it.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;

use zakac_lexer::spelling::canonical;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::kind::DataKind;

/// The start of the name of a thread kind, in signatures: `%thread(<kind>)`.
pub const THREAD: &str = "%thread";

/// The start of the name of a channel kind: `%pipe(<kind>)`.
pub const PIPE: &str = "%pipe";

/// The name of the thread kind in phrases, which the core declares.
pub const THREAD_WORD: &str = "အပြိုင်လုပ်ငန်း";

/// The name of the channel kind in phrases.
pub const PIPE_WORD: &str = "ဒေတာလမ်းကြောင်း";

/// The start of the name of an atomic number kind: `%atomic(<kind>)`.
pub const ATOMIC: &str = "%atomic";

/// The word of an atomic number kind, `အက်တမ်မစ် ဖြစ်သော <kind>`, a term of
/// computer science.
pub const ATOMIC_WORD: &str = zakac_ast::threads::ATOMIC_WORDS[0];

/// Which of the kinds of this file a kind is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor
{
    /// A thread.
    Thread,
    /// A channel.
    Pipe,
    /// An atomic number.
    Atomic,
}

/// Whether a name in a kind phrase is the thread, the channel or the atomic
/// number kind, in the spelling the compiler keeps.
///
/// # Arguments
///
/// * `name` - The name.
///
/// # Returns
///
/// The kind, or `None` for another name.
pub fn task_word(name: &str) -> Option<Flavor>
{
    ret_if!(name == canonical(PIPE_WORD), Some(Flavor::Pipe));
    ret_if!(name == ATOMIC_WORD, Some(Flavor::Atomic));
    tern!(name == THREAD_WORD, Some(Flavor::Thread), None)
}

/// A thread or a channel kind.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TaskKind
{
    /// `true` for a channel.
    pub pipe: bool,
    /// `true` for an atomic number: a platform-sized integer whose adds and
    /// subtractions threads make at once lose no change.
    pub atomic: bool,
    /// The kind a channel carries, or the kind the function of a thread gives
    /// back; `None` for a thread whose function gives nothing.
    pub of: Option<DataKind>,
    /// The signature, such as `%thread(isize)`.
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

    /// Whether the kind is a thread.
    ///
    /// # Returns
    ///
    /// `true` for a thread.
    pub fn is_thread(&self) -> bool { !self.pipe && !self.atomic }
}

/// The kinds made so far, under their signatures.
static MADE: OnceLock<Mutex<HashMap<String, &'static TaskKind>>> = OnceLock::new();

/// The thread or channel kind of a carried kind, made once.
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
    made(tern!(pipe, Flavor::Pipe, Flavor::Thread), of)
}

/// The atomic number kind of an integer kind, made once.
///
/// # Arguments
///
/// * `of` - The integer kind, of the platform width.
///
/// # Returns
///
/// The kind.
pub fn make_atomic(of: DataKind) -> &'static TaskKind { made(Flavor::Atomic, Some(of)) }

/// A kind of this file, made once.
///
/// # Arguments
///
/// * `flavor` - Which kind.
/// * `of` - The kind it carries, gives back or holds.
///
/// # Returns
///
/// The kind.
fn made(flavor: Flavor, of: Option<DataKind>) -> &'static TaskKind
{
    let (pipe, atomic) = (flavor == Flavor::Pipe, flavor == Flavor::Atomic);
    let head = match flavor
    {
        Flavor::Thread => THREAD,
        Flavor::Pipe => PIPE,
        Flavor::Atomic => ATOMIC,
    };
    let inner = of.map(|k| k.sig_name()).unwrap_or_default();
    let sig = format!("{head}({inner})");
    let made = MADE.get_or_init(Mutex::default);
    let mut made = made.lock().unwrap_or_else(PoisonError::into_inner);
    let fresh = || -> &'static TaskKind {
        Box::leak(Box::new(TaskKind { pipe, atomic, of, sig: sig.clone() }))
    };
    made.entry(sig.clone()).or_insert_with(fresh)
}

/// Read a thread or channel kind from its signature.
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
    let heads =
        [(PIPE, Flavor::Pipe), (ATOMIC, Flavor::Atomic), (THREAD, Flavor::Thread)];
    let (flavor, rest) =
        heads.iter().find_map(|(h, f)| text.strip_prefix(h).map(|r| (*f, r)))?;
    let inner = rest.strip_prefix('(')?.strip_suffix(')')?;
    ret_if!(inner.is_empty() && flavor == Flavor::Thread, Some(make(false, None)));
    Some(made(flavor, Some(DataKind::parse(inner)?)))
}
