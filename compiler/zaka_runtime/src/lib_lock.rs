//! The lock of each C lib. The calls of a lib that write its globals run one
//! at a time under the lock of that lib, while the calls of other libs go on
//! at the same time. The OS thread that holds a lock can take it again, as C
//! that a call calls can call the lib again on the same OS thread.

use std::cell::RefCell;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::thread::ThreadId;

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::do_while;
use zakac_rust_sugars::ret_if;

use crate::libs::ZkLib;

/// The lock of one lib.
pub(crate) struct Lock
{
    /// The OS thread that holds it, and how deep its calls go.
    held: Mutex<(Option<ThreadId>, u32)>,
    /// Wakes the OS threads that wait once no call of the lib runs.
    freed: Condvar,
}

impl Lock
{
    /// A lock that no OS thread holds.
    ///
    /// # Returns
    ///
    /// The lock.
    pub(crate) const fn new() -> Lock
    {
        Lock { held: Mutex::new((None, 0)), freed: Condvar::new() }
    }

    /// Wait until no other OS thread holds the lock, then hold it at a depth
    /// more.
    ///
    /// # Arguments
    ///
    /// * `depth` - The depth to add.
    fn take(&self, depth: u32)
    {
        let me = std::thread::current().id();
        let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        do_while!(
            held.0.is_some_and(|t| t != me),
            held = self.freed.wait(held).unwrap_or_else(PoisonError::into_inner)
        );
        *held = (Some(me), held.1 + depth);
    }

    /// Hold the lock at a depth less; at 0 the next OS thread that waits goes
    /// on.
    ///
    /// # Returns
    ///
    /// `true` once this OS thread holds it no more.
    fn give(&self) -> bool
    {
        let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        held.1 = held.1.saturating_sub(1);
        ret_if!(held.1 > 0, false);
        held.0 = None;
        self.freed.notify_one();
        true
    }

    /// Let go of the lock at each depth this OS thread holds it.
    ///
    /// # Returns
    ///
    /// The depth it held.
    fn give_all(&self) -> u32
    {
        let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        let depth = held.1;
        *held = (None, 0);
        self.freed.notify_all();
        depth
    }
}

thread_local! {
    /// The libs whose lock this OS thread holds, as addresses, in the order it
    /// took them.
    static HOLDS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

/// The lock of a lib, from its address.
///
/// # Arguments
///
/// * `lib` - The lib, as an address.
///
/// # Returns
///
/// The lock.
fn lock_of<'a>(lib: usize) -> &'a Lock
{
    // SAFETY: a lib lives as long as the program.
    &unsafe { &*(lib as *const ZkLib) }.lock
}

/// Note that this OS thread holds the lock of a lib: the first lock it holds
/// counts it among the OS threads that run a call under a lock.
///
/// # Arguments
///
/// * `lib` - The lib, as an address.
fn note_held(lib: usize)
{
    let first = HOLDS.with(|h| {
        let mut h = h.borrow_mut();
        ret_if!(h.contains(&lib), false);
        h.push(lib);
        h.len() == 1
    });
    do_if!(first, crate::libs::locked_in());
}

/// Wait until no other OS thread runs a call of a lib under its lock, then
/// hold it: the globals of the lib serve one such call at a time.
///
/// # Arguments
///
/// * `lib` - The lib, as `zk_lib_open` made it.
#[unsafe(export_name = stamped!("lib_lock"))]
pub extern "C" fn zk_lib_lock(lib: *mut ZkLib)
{
    lock_of(lib as usize).take(1);
    note_held(lib as usize);
}

/// End the hold of [`zk_lib_lock`] for one call: once the calls of the OS
/// thread in the lib all end, the next OS thread that waits goes on.
///
/// # Arguments
///
/// * `lib` - The lib.
#[unsafe(export_name = stamped!("lib_unlock"))]
pub extern "C" fn zk_lib_unlock(lib: *mut ZkLib)
{
    ret_if!(!lock_of(lib as usize).give());
    let last = HOLDS.with(|h| {
        let mut h = h.borrow_mut();
        h.retain(|l| *l != lib as usize);
        h.is_empty()
    });
    do_if!(last, crate::libs::locked_out());
}

/// Whether this OS thread holds the lock of a lib. An OS thread that ends,
/// whose drops can run the ring collector, holds none.
///
/// # Returns
///
/// `true` when it holds one.
pub(crate) fn holds_lock() -> bool
{
    HOLDS.try_with(|h| !h.borrow().is_empty()).unwrap_or(false)
}

/// The count of the locks of libs this OS thread holds.
///
/// # Returns
///
/// The count.
pub(crate) fn held_count() -> usize { HOLDS.with(|h| h.borrow().len()) }

/// Let go of each lock this OS thread took after the first `n`, at each
/// depth: a trap left them held.
///
/// # Arguments
///
/// * `n` - The count of locks that remain held.
pub(crate) fn let_go_from(n: usize)
{
    let (later, last) = HOLDS.with(|h| {
        let mut h = h.borrow_mut();
        let from = n.min(h.len());
        let later: Vec<usize> = h.drain(from..).collect();
        (later, h.is_empty())
    });
    ret_if!(later.is_empty());
    later.iter().for_each(|lib| _ = lock_of(*lib).give_all());
    do_if!(last, crate::libs::locked_out());
}

/// Let go of the lock of each lib this OS thread holds, at each depth, for a
/// call to C that lets other OS threads into the libs while it runs.
///
/// # Returns
///
/// Each lib it held, as an address, with the depth it held it at.
pub(crate) fn let_go_locks() -> Vec<(usize, u32)>
{
    let held = HOLDS.with(|h| std::mem::take(&mut *h.borrow_mut()));
    ret_if!(held.is_empty(), Vec::new());
    let each = held.into_iter().map(|lib| (lib, lock_of(lib).give_all()));
    let held: Vec<(usize, u32)> = each.collect();
    crate::libs::locked_out();
    held
}

/// Take back the locks a call to C let go of, at the depths it held them,
/// once no other OS thread runs a call of those libs.
///
/// # Arguments
///
/// * `held` - Each lib and depth, as [`let_go_locks`] gave them.
pub(crate) fn take_locks(held: &[(usize, u32)])
{
    held.iter().for_each(|&(lib, depth)| {
        lock_of(lib).take(depth);
        note_held(lib);
    });
}
