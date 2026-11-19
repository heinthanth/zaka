//! The rights of a run: the grants of the build, or the gate a JIT hands in,
//! and the question to the person at the terminal for a right the run lacks.

use std::ffi::CStr;
use std::ffi::c_char;
use std::io::BufRead;
use std::io::IsTerminal;
use std::io::Write;
use std::sync::Mutex;

use zaka_host::Answer;
use zaka_host::Asker;
use zaka_host::Gate;
use zaka_host::Grant;
use zaka_host::Grants;
use zaka_host::Right;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::out::flush_out;
use crate::program::ZkGrant;
use crate::program::prog;
use crate::program::row;
use crate::program::text;
use crate::traps::fill;
use crate::traps::stop_owned;
use crate::traps::trap;

/// The gate of the run: set from a JIT before the program starts, else made
/// from the grants of the build at the first check.
static GATE: Mutex<Option<Gate>> = Mutex::new(None);

/// Hand the runtime the gate of a run, before the program starts: a JIT
/// grants what `zakac run` was told to grant, and asks through its own asker.
///
/// # Arguments
///
/// * `gate` - The gate.
pub fn set_gate(gate: Gate)
{
    if let Ok(mut held) = GATE.lock()
    {
        *held = Some(gate);
    }
}

/// The rights a program can hold, as the table of the program lists them.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(missing_docs)]
pub enum ZkRight
{
    Read,
    Write,
    Env,
    Ffi,
    /// The count of rights, the length of a table of them.
    Count,
}

/// The right of a number, as [`ZkRight`] counts them.
///
/// # Arguments
///
/// * `n` - The number.
///
/// # Returns
///
/// The right.
fn right_of(n: i32) -> Right
{
    match n
    {
        n if n == ZkRight::Read as i32 => Right::Read,
        n if n == ZkRight::Write as i32 => Right::Write,
        n if n == ZkRight::Env as i32 => Right::Env,
        _ => Right::Ffi,
    }
}

/// The number of a right, as [`ZkRight`] counts them.
///
/// # Arguments
///
/// * `right` - The right.
///
/// # Returns
///
/// The number.
fn number_of(right: Right) -> usize
{
    let id = match right
    {
        Right::Read => ZkRight::Read,
        Right::Write => ZkRight::Write,
        Right::Env => ZkRight::Env,
        Right::Ffi => ZkRight::Ffi,
    };
    id as usize
}

/// The grant of one right, as the build wrote it.
///
/// # Arguments
///
/// * `g` - The grant in the table.
///
/// # Returns
///
/// The grant.
fn grant_of(g: &ZkGrant) -> Grant
{
    ret_if!(g.all != 0, Grant::All);
    ret_if!(g.count <= 0, Grant::None);
    let items = (0..g.count as usize).map(|i| String::from_utf8_lossy(row(g.items, i)));
    Grant::Some(items.map(|s| s.into_owned()).collect())
}

/// The gate of a build: its grants, asking through the terminal when the build
/// lets the program ask.
///
/// # Returns
///
/// The gate.
fn build_gate() -> Gate
{
    let p = prog();
    // SAFETY: the table holds a grant for each of the four rights.
    let g = unsafe { std::slice::from_raw_parts(p.grants, 4) };
    #[rustfmt::skip]
    let grants = Grants::builder().read(grant_of(&g[0])).write(grant_of(&g[1]))
        .env(grant_of(&g[2])).ffi(g[3].all != 0).prompt(p.grant_prompt != 0).build();
    Gate::new(grants).with_asker(Box::new(TermAsker))
}

/// Check an effect against the gate of the run.
///
/// # Arguments
///
/// * `right` - The right it needs.
/// * `subject` - Its path or name; blank for C calls or the whole environment.
///
/// # Returns
///
/// `true` when the run holds the right.
pub fn allowed(right: Right, subject: &str) -> bool
{
    with_gate(|g| g.check(right, subject).is_ok()).unwrap_or(false)
}

/// Run a step with the gate of the run: the gate of the C lib whose call is
/// in progress, or else the gate of the program; each is made from the grants
/// of its build at the first check.
///
/// # Arguments
///
/// * `step` - The step.
///
/// # Returns
///
/// What the step gave, or `None` when the gate is out of reach.
fn with_gate<R>(step: impl FnOnce(&mut Gate) -> R) -> Option<R>
{
    let lib = crate::libs::current().map(|l| &l.gate);
    let mut held = lib.unwrap_or(&GATE).lock().ok()?;
    Some(step(held.get_or_insert_with(build_gate)))
}

/// The grant of the environment in the gate of the run.
///
/// # Returns
///
/// The grant.
fn env_grant() -> Grant { with_gate(|g| g.grants.env.clone()).unwrap_or(Grant::None) }

/// What an effect of a right does, as a message names it.
///
/// # Arguments
///
/// * `right` - The right.
/// * `subject` - The path, the name, or blank for the whole environment.
///
/// # Returns
///
/// The text.
fn what(right: Right, subject: &str) -> Vec<u8>
{
    let whole = right == Right::Env && subject.is_empty();
    let n = if whole { 4 } else { number_of(right) };
    fill(row(prog().permission_text, n), &[("s", subject.as_bytes())])
}

/// Stop the run for a right it lacks, naming the flag that grants it.
///
/// # Arguments
///
/// * `right` - The right.
/// * `subject` - The C name, or blank for the environment.
pub fn permission_trap(right: Right, subject: &str) -> !
{
    let flag = row(prog().flag_text, number_of(right)).to_vec();
    let what = what(right, subject);
    stop_owned(
        trap::PERMISSION,
        vec![("what".to_string(), what), ("flag".to_string(), flag)],
    )
}

/// Asks the person at the terminal, with the texts of the build.
struct TermAsker;

impl Asker for TermAsker
{
    fn ask(&mut self, right: Right, subject: &str) -> Answer
    {
        ret_if!(
            !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal(),
            Answer::No
        );
        let question = fill(text(prog().ask_text), &[("what", &what(right, subject))]);
        flush_out();
        let mut err = std::io::stderr().lock();
        let _ = err.write_all(&question);
        let _ = err.flush();
        let mut line = String::new();
        match std::io::stdin().lock().read_line(&mut line)
        {
            Ok(0) | Err(_) => Answer::No,
            Ok(_) => Answer::typed(&line),
        }
    }
}

/// The text of a C string, for a check.
///
/// # Arguments
///
/// * `p` - The text.
///
/// # Returns
///
/// The text.
fn subject_of(p: *const c_char) -> String
{
    String::from_utf8_lossy(text(p)).into_owned()
}

/// Whether the program holds a right for a path or a name.
///
/// # Arguments
///
/// * `right` - The right, an `enum zk_right`.
/// * `subject` - The path or name, as the program gave it; blank for C calls.
///
/// # Returns
///
/// 1 when it does.
#[unsafe(export_name = stamped!("allowed"))]
pub extern "C" fn zk_allowed(right: i32, subject: *const c_char) -> i32
{
    i32::from(allowed(right_of(right), &subject_of(subject)))
}

/// Stop the run unless the program holds the right to call a C function; each
/// C call checks this first.
///
/// # Arguments
///
/// * `name` - The C name.
#[unsafe(export_name = stamped!("need_ffi"))]
pub extern "C" fn zk_need_ffi(name: *const c_char)
{
    let name = subject_of(name);
    do_if!(!allowed(Right::Ffi, &name), permission_trap(Right::Ffi, &name));
}

/// Whether a row of the environment names one of the variables listed.
///
/// # Arguments
///
/// * `row` - The row, a NUL-ended `NAME=value`.
/// * `names` - The variables.
///
/// # Returns
///
/// `true` when it does.
fn named(row: *mut c_char, names: &[String]) -> bool
{
    // SAFETY: each row of the environment ends in NUL.
    let pair = unsafe { CStr::from_ptr(row) }.to_bytes();
    let name = pair.split(|c| *c == b'=').next().unwrap_or(pair);
    names.iter().any(|n| n.as_bytes() == name)
}

/// The environment the start block gets: each variable with the env right,
/// the variables a list names, else the whole environment once the person at
/// the terminal allows it; the run stops when the answer is no.
///
/// # Arguments
///
/// * `envp` - The environment `main` was given.
/// * `count` - Where the count of variables goes.
///
/// # Returns
///
/// The variables.
#[unsafe(export_name = stamped!("allowed_env"))]
pub extern "C" fn zk_allowed_env(
    envp: *mut *mut c_char, count: *mut i32,
) -> *mut *mut c_char
{
    let mut all = Vec::new();
    // SAFETY: `envp` is the NULL-ended list `main` was given.
    while !envp.is_null() && !unsafe { *envp.add(all.len()) }.is_null()
    {
        all.push(unsafe { *envp.add(all.len()) });
    }
    let kept: Vec<*mut c_char> = match env_grant()
    {
        Grant::All => all,
        Grant::None if allowed(Right::Env, "") => all,
        Grant::None => permission_trap(Right::Env, ""),
        Grant::Some(names) => all.into_iter().filter(|v| named(*v, &names)).collect(),
    };
    // SAFETY: the caller hands room for the count.
    unsafe { *count = kept.len() as i32 };
    let mut rows = kept;
    rows.push(std::ptr::null_mut());
    Box::leak(rows.into_boxed_slice()).as_mut_ptr()
}
