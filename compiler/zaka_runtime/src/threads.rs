//! The threads and channels of a native program, each thread on an OS thread of
//! its own. One lock guards the state of each thread and channel, and one
//! condition wakes each OS thread that waits when the state changes. The state
//! counts the OS threads that run: an OS thread that would wait while no other
//! runs stops the program with the trap `deadlock`.
//!
//! A thread runs a step the compiler writes, under a guard the compiler writes:
//! the step takes the arguments out of the data block of the thread and writes
//! the result at its start; the guard keeps the place a trap of the thread
//! jumps back to under `--trap unwind`.

use std::cell::Cell;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::PoisonError;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::heap::zk_threads_begin;
use crate::program::ZkKind;
use crate::seqs::kind;
use crate::seqs::zk_add_cell;
use crate::seqs::zk_drop;
use crate::seqs::zk_seq_base;
use crate::seqs::zk_seq_len;
use crate::traps::named;
use crate::traps::stop;
use crate::traps::stop_owned;
use crate::traps::trap;

/// The step of a thread: it takes the data block of the thread.
pub type ZkStep = extern "C" fn(*mut c_void);

/// The guard of a thread: it runs the step on the data block, and gives 0, or
/// a value other than 0 once a trap jumped back to it.
pub type ZkGuard = extern "C" fn(ZkStep, *mut c_void) -> i32;

/// Where a thread stands.
#[derive(Clone, Copy, PartialEq, Eq)]
enum End
{
    /// It runs.
    Runs,
    /// It returned; its result is at the start of its data block.
    Done,
    /// It stopped at a trap.
    Trapped,
}

/// One thread.
struct Thread
{
    /// Where it stands.
    end: End,
    /// Its data block: the result at its start, then the arguments.
    data: usize,
    /// The kind of its result, or NULL for a function that gives nothing.
    ret: usize,
    /// Whether a wait took its result, or saw its trap.
    awaited: bool,
    /// Whether its data block came from [`zk_thread_data`], so the runtime
    /// frees it; the threads of a parallel loop read a block of the loop.
    owned: bool,
    /// Its OS thread, until a wait or the end of the program joins it.
    os_thread: Option<std::thread::JoinHandle<()>>,
}

/// One channel.
struct Pipe
{
    /// Each value that waits, the oldest first, as its octets.
    queue: VecDeque<Box<[u8]>>,
    /// The most values that wait at once; 0 for no limit.
    room: usize,
    /// The kind of its values, from the first send.
    kind: usize,
    /// Whether it is closed.
    closed: bool,
}

/// A lock of a shared object that an OS thread holds.
struct Held
{
    /// The address of the object.
    obj: usize,
    /// The OS thread that holds it, as [`owner`] names it.
    owner: usize,
    /// The count of times the thread took it.
    depth: u32,
    /// Whether its holder stopped at a trap, so it never comes back.
    broken: bool,
}

/// The state of the threads and channels.
struct State
{
    /// Each thread; the thread a program holds as `n` is at `n - 1`.
    threads: Vec<Thread>,
    /// The numbers of the threads of parallel loops that ended, which no
    /// program holds, for the next threads to take.
    spare: Vec<usize>,
    /// Each channel, as the threads are numbered.
    pipes: Vec<Pipe>,
    /// The OS threads that run and do not wait, the main one included.
    running: usize,
    /// The OS threads that wait for the state to change.
    parked: usize,
    /// The count of changes so far, which tells a waiter its wake is real.
    changes: u64,
}

/// The state of the threads and channels.
static STATE: Mutex<State> = Mutex::new(State {
    threads: Vec::new(),
    spare: Vec::new(),
    pipes: Vec::new(),
    running: 1,
    parked: 0,
    changes: 0,
});

/// Wakes each OS thread that waits once the state changes.
static CHANGED: Condvar = Condvar::new();

thread_local! {
    /// The function that jumps back to the guard of the thread of this OS thread
    /// under `--trap unwind`, and the number of the thread.
    static LEAVE: Cell<Option<(extern "C" fn(i32), usize)>> = const { Cell::new(None) };
    /// The trap that ended the thread of this OS thread.
    static TRAPPED: Cell<i32> = const { Cell::new(0) };
    /// The number of the thread of this OS thread, 0 for the main OS thread.
    static THREAD: Cell<usize> = const { Cell::new(0) };
    /// The number an OS thread that runs no thread holds locks under, from its first
    /// lock; 0 before it.
    static LOCKER: Cell<usize> = const { Cell::new(0) };
    /// Whether this OS thread runs the program: `zk_start` marks it.
    static MAIN: Cell<bool> = const { Cell::new(false) };
}

/// The next number of an OS thread that runs no thread, for its locks: the high
/// half of the numbers, which no thread reaches.
static NEXT_LOCKER: AtomicUsize = AtomicUsize::new(1 << (usize::BITS - 1));

/// Whether the main OS thread of a program runs, as against a C lib, whose
/// OS threads come from C.
static PROGRAM: AtomicBool = AtomicBool::new(false);

/// Mark this OS thread as the main OS thread of the program, which the count of
/// running OS threads holds from the start.
pub(crate) fn mark_main()
{
    MAIN.set(true);
    PROGRAM.store(true, Ordering::SeqCst);
}

/// The OS thread a lock names as its holder: the number of a thread, or, for
/// each other OS thread (the main one and each OS thread of C), a number of its
/// own, so two such OS threads never pass as one.
///
/// # Returns
///
/// The number.
fn owner() -> usize
{
    let thread = THREAD.get();
    ret_if!(thread != 0, thread);
    let mine = LOCKER.get();
    ret_if!(mine != 0, mine);
    let fresh = NEXT_LOCKER.fetch_add(1, Ordering::Relaxed);
    LOCKER.set(fresh);
    fresh
}

/// Whether an OS thread out of the count of running OS threads can still change
/// what a waiter waits for: each OS thread of a C lib, an OS thread of C inside
/// a call of the program now, or a closure that C can call from an OS thread of
/// its own later. A wait then never stops as a deadlock.
///
/// # Returns
///
/// `true` when one can.
fn outside_acts() -> bool
{
    !PROGRAM.load(Ordering::SeqCst)
        || crate::libs::free_calls() > 0
        || lent_to_c_threads()
}

/// Look again at each wait once an OS thread of C left a call of the program: a
/// wait that began while that OS thread could still act checks again for a
/// deadlock. A lib looks at none, as its waits never stop as one.
pub(crate) fn recheck_waits()
{
    ret_if!(!PROGRAM.load(Ordering::SeqCst));
    let mut st = state();
    ret_if!(st.parked == 0);
    changed(&mut st);
}

/// Whether C holds a closure it can call from an OS thread of its own.
///
/// # Returns
///
/// `true` when it does.
#[cfg(not(target_os = "wasi"))]
fn lent_to_c_threads() -> bool { crate::callbacks::lent_to_c_threads() }

/// Whether C holds a closure it can call from an OS thread of its own: never on
/// WASI, which calls no C.
///
/// # Returns
///
/// `false`.
#[cfg(target_os = "wasi")]
fn lent_to_c_threads() -> bool { false }

/// The state, locked.
///
/// # Returns
///
/// The guard.
fn state() -> MutexGuard<'static, State>
{
    STATE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Mark the state changed: each OS thread that waits runs again and looks.
///
/// # Arguments
///
/// * `st` - The state.
fn changed(st: &mut State)
{
    st.running += st.parked;
    st.parked = 0;
    st.changes += 1;
    CHANGED.notify_all();
}

/// Wait until `came` holds; a wait while no other OS thread runs stops the
/// program with the trap `deadlock`.
///
/// # Arguments
///
/// * `st` - The state, locked.
/// * `came` - What the OS thread waits for.
///
/// # Returns
///
/// The state, locked, with `came` holding.
fn wait_until(
    mut st: MutexGuard<'static, State>, came: impl Fn(&State) -> bool,
) -> MutexGuard<'static, State>
{
    // An OS thread of C is out of the count: it waits and changes no count.
    let counted = THREAD.get() != 0 || MAIN.get();
    while !came(&st)
    {
        let seen = st.changes;
        if counted
        {
            st.running -= 1;
            st.parked += 1;
        }
        if counted && st.running == 0 && !outside_acts()
        {
            drop(st);
            deadlock();
        }
        while st.changes == seen
        {
            st = CHANGED.wait(st).unwrap_or_else(PoisonError::into_inner);
        }
        // An OS thread that wakes during a collection halts first, holding no
        // lock.
        if crate::seqs::halt_asked()
        {
            drop(st);
            crate::seqs::safe_point();
            st = state();
        }
    }
    st
}

/// Stop the whole program with the trap `deadlock`, from whichever OS thread.
fn deadlock() -> !
{
    LEAVE.set(None);
    stop(trap::DEADLOCK, &[])
}

/// The function that jumps back to the guard of the thread of this OS thread,
/// and its number, when a trap ends the thread alone.
///
/// # Returns
///
/// The function and the number, or `None` on the main OS thread and under
/// `--trap abort`.
pub(crate) fn leave() -> Option<(extern "C" fn(i32), usize)> { LEAVE.get() }

/// Keep the trap that ends the thread of this OS thread, for its guard.
///
/// # Arguments
///
/// * `trap` - The trap.
pub(crate) fn keep_trap(trap: i32) { TRAPPED.set(trap) }

/// Make a data block of a thread: zeros, which the spawn fills.
///
/// # Arguments
///
/// * `size` - The octets of the block.
///
/// # Returns
///
/// The block.
#[unsafe(export_name = stamped!("thread_data"))]
pub extern "C" fn zk_thread_data(size: usize) -> *mut u8
{
    // The word in front of the block holds its size, for the free.
    let block = vec![0u8; size + WORD].into_boxed_slice();
    let start = Box::leak(block).as_mut_ptr();
    // SAFETY: the block holds a word, then `size` octets.
    unsafe {
        start.cast::<usize>().write_unaligned(size);
        start.add(WORD)
    }
}

/// The octets of a word, in front of a data block of a thread.
const WORD: usize = size_of::<usize>();

/// Free a data block of a thread, from [`zk_thread_data`].
///
/// # Arguments
///
/// * `data` - The block, as the address it gave.
fn free_data(data: usize)
{
    ret_if!(data == 0);
    // SAFETY: the word in front of the block holds its size, and the block
    // is a leaked box of that size and the word.
    unsafe {
        let start = (data as *mut u8).sub(WORD);
        let size = start.cast::<usize>().read_unaligned();
        let slice = std::ptr::slice_from_raw_parts_mut(start, size + WORD);
        drop(Box::from_raw(slice));
    }
}

/// Start a thread on an OS thread of its own.
///
/// # Arguments
///
/// * `guard` - The guard the compiler writes.
/// * `jump` - The function that jumps back to the guard.
/// * `step` - The step of the thread.
/// * `data` - Its data block, from [`zk_thread_data`].
/// * `ret` - The kind of its result, or NULL for none.
/// * `abort` - 1 when a trap of the thread stops the whole program.
///
/// # Returns
///
/// The number of the thread.
#[unsafe(export_name = stamped!("thread_start"))]
pub extern "C" fn zk_thread_start(
    guard: ZkGuard, jump: extern "C" fn(i32), step: ZkStep, data: *mut u8,
    ret: *const ZkKind, abort: i32,
) -> usize
{
    start_thread((guard, jump, step), data, ret, abort, true)
}

/// Start a thread on an OS thread of its own; see [`zk_thread_start`].
///
/// # Arguments
///
/// * `(guard, jump, step)` - The guard, the jump back to it, and the step.
/// * `data` - Its data block.
/// * `ret` - The kind of its result, or NULL.
/// * `abort` - 1 when a trap stops the whole program.
/// * `owned` - Whether the runtime frees the data block.
///
/// # Returns
///
/// The number of the thread.
fn start_thread(
    (guard, jump, step): (ZkGuard, extern "C" fn(i32), ZkStep), data: *mut u8,
    ret: *const ZkKind, abort: i32, owned: bool,
) -> usize
{
    zk_threads_begin();
    let mut st = state();
    #[rustfmt::skip]
    let thread = Thread { end: End::Runs, data: data as usize, ret: ret as usize, awaited: false,
        owned, os_thread: None };
    let next = st.threads.len() + 1;
    let n = st.spare.pop().unwrap_or(next);
    if n == next
    {
        st.threads.push(thread);
    }
    else
    {
        st.threads[n - 1] = thread;
    }
    st.running += 1;
    let data = data as usize;
    let run = move || run_thread(guard, (jump, abort != 0), step, data, n);
    let os_thread = std::thread::Builder::new().stack_size(64 << 20).spawn(run);
    let Ok(os_thread) = os_thread
    else
    {
        drop(st);
        stop_owned(trap::OUT_OF_MEM, vec![named("size", 64usize << 20)])
    };
    st.threads[n - 1].os_thread = Some(os_thread);
    n
}

/// The work of the OS thread of a thread: run the guard, then mark the thread
/// ended.
///
/// # Arguments
///
/// * `guard` - The guard.
/// * `(jump, abort)` - The function that jumps back to the guard, and whether a
///   trap stops the whole program.
/// * `step` - The step.
/// * `data` - The data block.
/// * `n` - The number of the thread.
fn run_thread(
    guard: ZkGuard, (jump, abort): (extern "C" fn(i32), bool), step: ZkStep,
    data: usize, n: usize,
)
{
    do_if!(!abort, LEAVE.set(Some((jump, n))));
    THREAD.set(n);
    let status = guard(step, data as *mut c_void);
    LEAVE.set(None);
    // The OS thread ends here, and its calls and its line with it.
    crate::ctx::release_frames();
    crate::out::flush_line();
    // The locks a thread held when it stopped remain taken, broken.
    for shard in &LOCKS
    {
        let mut held = shard.lock().unwrap_or_else(PoisonError::into_inner);
        held.iter_mut().filter(|h| h.owner == n).for_each(|h| h.broken = true);
    }
    let mut st = state();
    st.threads[n - 1].end = if status == 0 { End::Done } else { End::Trapped };
    st.running -= 1;
    changed(&mut st);
}

/// Wait for a thread to end and take its result: one wait takes it, as the
/// join of Rust does, and a second wait of the thread stops the program with
/// the trap `thread waited`.
///
/// # Arguments
///
/// * `n` - The number of the thread.
///
/// # Returns
///
/// The address of its result, at the start of its data block; the caller
/// moves the result out, then gives the block back with [`zk_thread_taken`]. A
/// thread that stopped at a trap stops the waiter with the trap `thread
/// trapped`.
#[unsafe(export_name = stamped!("thread_wait"))]
pub extern "C" fn zk_thread_wait(n: usize) -> *mut u8
{
    let st = state();
    let at = n.checked_sub(1).filter(|i| *i < st.threads.len());
    let Some(at) = at
    else
    {
        drop(st);
        stop_owned(trap::THREAD_TRAPPED, vec![named("thread", n)])
    };
    let mut st = wait_until(st, |s| s.threads[at].end != End::Runs);
    if st.threads[at].awaited
    {
        drop(st);
        stop_owned(trap::THREAD_WAITED, vec![named("thread", n)])
    }
    st.threads[at].awaited = true;
    let (end, data) = (st.threads[at].end, st.threads[at].data);
    let os_thread = st.threads[at].os_thread.take();
    drop(st);
    join(os_thread);
    ret_if!(end == End::Done, data as *mut u8);
    stop_owned(trap::THREAD_TRAPPED, vec![named("thread", n)])
}

/// Give back the data block of a thread whose result a wait moved out.
///
/// # Arguments
///
/// * `n` - The number of the thread.
#[unsafe(export_name = stamped!("thread_taken"))]
pub extern "C" fn zk_thread_taken(n: usize)
{
    let data = {
        let mut st = state();
        let thread = &mut st.threads[n - 1];
        std::mem::take(&mut thread.data)
    };
    free_data(data);
}

/// Wait for a thread and drop its result, for a wait whose value no one reads.
///
/// # Arguments
///
/// * `n` - The number of the thread.
#[unsafe(export_name = stamped!("thread_drop"))]
pub extern "C" fn zk_thread_drop(n: usize)
{
    let data = zk_thread_wait(n);
    let ret = state().threads[n - 1].ret;
    do_if!(ret != 0, zk_drop(data, ret as *const ZkKind));
    zk_thread_taken(n);
}

/// The count of the shards of the locks.
const SHARDS: usize = 64;

/// Each lock an OS thread holds, or that a thread held when it stopped at a
/// trap, apart from the state and spread over shards on the address of the
/// object: an OS thread takes a lock no other OS thread holds, and gives one
/// back no OS thread waits for, with no lock of the state and none other
/// objects share.
static LOCKS: [Mutex<Vec<Held>>; SHARDS] = [const { Mutex::new(Vec::new()) }; SHARDS];

/// The count of OS threads that wait for a lock, so a give back with none
/// wakes no one.
static LOCK_WAITERS: AtomicUsize = AtomicUsize::new(0);

/// The shard of the locks of an object, locked.
///
/// # Arguments
///
/// * `obj` - The block of the object.
///
/// # Returns
///
/// The guard.
fn locks(obj: usize) -> MutexGuard<'static, Vec<Held>>
{
    // Blocks align to 16 octets, so the low bits name no shard.
    let shard = (obj >> 4) % SHARDS;
    LOCKS[shard].lock().unwrap_or_else(PoisonError::into_inner)
}

/// What an OS thread finds when it takes a lock.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Take
{
    /// It took the lock, or took once more one it holds.
    Taken,
    /// Its holder stopped at a trap.
    Broken,
    /// Another OS thread holds it.
    Other,
}

/// Take a lock when no other OS thread holds it.
///
/// # Arguments
///
/// * `obj` - The block of the object.
/// * `me` - The OS thread, as [`owner`] names it.
///
/// # Returns
///
/// What the OS thread found.
fn take_free(obj: usize, me: usize) -> Take
{
    let mut held = locks(obj);
    let Some(h) = held.iter_mut().find(|h| h.obj == obj)
    else
    {
        held.push(Held { obj, owner: me, depth: 1, broken: false });
        return Take::Taken;
    };
    ret_if!(h.broken, Take::Broken);
    ret_if!(h.owner != me, Take::Other);
    h.depth += 1;
    Take::Taken
}

/// Take the lock of a shared object: the OS thread waits while another holds
/// it, and takes once more a lock it holds.
///
/// # Arguments
///
/// * `obj` - The variable that holds the object.
#[unsafe(export_name = stamped!("lock"))]
pub extern "C" fn zk_lock(obj: *mut u8)
{
    // The block of the object names the lock, which each thread that
    // holds the object shares; the variable differs from thread to thread.
    let (obj, me) = (zk_seq_base(obj) as usize, owner());
    let mut took = take_free(obj, me);
    if took == Take::Other
    {
        // The count goes up before the look, so a give back after the look
        // sees it and wakes this OS thread.
        LOCK_WAITERS.fetch_add(1, Ordering::SeqCst);
        let free = |_: &State| {
            let held = locks(obj);
            held.iter().all(|h| h.obj != obj || h.owner == me || h.broken)
        };
        while took == Take::Other
        {
            drop(wait_until(state(), free));
            took = take_free(obj, me);
        }
        LOCK_WAITERS.fetch_sub(1, Ordering::SeqCst);
    }
    ret_if!(took == Take::Taken);
    stop(trap::LOCK_BROKEN, &[]);
}

/// Give back the lock of a shared object once.
///
/// # Arguments
///
/// * `obj` - The variable that holds the object.
#[unsafe(export_name = stamped!("unlock"))]
pub extern "C" fn zk_unlock(obj: *mut u8)
{
    let (obj, me) = (zk_seq_base(obj) as usize, owner());
    {
        let mut held = locks(obj);
        let Some(at) = held.iter().position(|h| h.obj == obj && h.owner == me)
        else
        {
            return;
        };
        held[at].depth -= 1;
        ret_if!(held[at].depth > 0);
        held.remove(at);
    }
    ret_if!(LOCK_WAITERS.load(Ordering::SeqCst) == 0);
    changed(&mut state());
}

/// Add a value to an atomic number of the platform width, or subtract it,
/// in one step no other OS thread can split; the result wraps.
///
/// # Arguments
///
/// * `at` - The address of the number.
/// * `v` - The value.
/// * `sub` - 1 to subtract the value.
#[unsafe(export_name = stamped!("atomic_add"))]
pub extern "C" fn zk_atomic_add(at: *mut isize, v: isize, sub: i32)
{
    // SAFETY: the number is a word of the program, aligned as its kind, which
    // each OS thread changes with atomic ops alone.
    let n = unsafe { AtomicIsize::from_ptr(at) };
    _ = tern!(
        sub != 0,
        n.fetch_sub(v, Ordering::SeqCst),
        n.fetch_add(v, Ordering::SeqCst)
    );
}

/// Read an atomic number of the platform width in one step that sees each
/// change another OS thread made before it.
///
/// # Arguments
///
/// * `at` - The address of the number.
///
/// # Returns
///
/// The number.
#[unsafe(export_name = stamped!("atomic_load"))]
pub extern "C" fn zk_atomic_load(at: *mut isize) -> isize
{
    // SAFETY: as in `zk_atomic_add`.
    unsafe { AtomicIsize::from_ptr(at) }.load(Ordering::SeqCst)
}

/// Set an atomic number of the platform width to a new value when it holds
/// the expected one, in one step no other OS thread can split.
///
/// # Arguments
///
/// * `at` - The address of the number.
/// * `expected` - The value it must hold.
/// * `new` - The value it takes.
///
/// # Returns
///
/// What it held before: `expected` when it took `new`.
#[unsafe(export_name = stamped!("atomic_cas"))]
pub extern "C" fn zk_atomic_cas(at: *mut isize, expected: isize, new: isize) -> isize
{
    // SAFETY: as in `zk_atomic_add`.
    let n = unsafe { AtomicIsize::from_ptr(at) };
    let swapped = n.compare_exchange(expected, new, Ordering::SeqCst, Ordering::SeqCst);
    swapped.unwrap_or_else(|held| held)
}

/// Set an atomic number of the platform width in one step, which each other
/// OS thread sees whole and in order.
///
/// # Arguments
///
/// * `at` - The address of the number.
/// * `v` - The value.
#[unsafe(export_name = stamped!("atomic_set"))]
pub extern "C" fn zk_atomic_set(at: *mut isize, v: isize)
{
    // SAFETY: as in `zk_atomic_add`.
    unsafe { AtomicIsize::from_ptr(at) }.store(v, Ordering::SeqCst);
}

/// The step of a parallel map: it takes the address of a cell and the address
/// the result goes to.
pub type ZkMapStep = extern "C" fn(*mut u8, *mut u8);

/// The step of a round of a parallel loop: it takes the data block of the
/// loop and the number of the round, from 0.
pub type ZkRoundStep = extern "C" fn(*mut u8, usize);

/// The rounds of a parallel loop that its threads share: each thread takes the
/// next block of rounds until none is left, so a thread whose rounds run fast
/// takes more of them.
struct Rounds
{
    /// The step of a round.
    step: ZkRoundStep,
    /// The data block of the loop, as an address.
    data: usize,
    /// The count of rounds.
    count: usize,
    /// The rounds of one take.
    block: usize,
    /// The first round no thread took so far.
    next: AtomicUsize,
}

/// The step of a thread of a parallel loop: take a block of rounds and run
/// them, until no round is left.
///
/// # Arguments
///
/// * `data` - The rounds.
extern "C" fn run_rounds(data: *mut c_void)
{
    // SAFETY: `zk_par_for` hands each thread the rounds, which outlive the
    // threads.
    let r = unsafe { &*data.cast::<Rounds>() };
    loop
    {
        let lo = r.next.fetch_add(r.block, Ordering::Relaxed);
        ret_if!(lo >= r.count);
        let hi = (lo + r.block).min(r.count);
        (lo..hi).for_each(|i| (r.step)(r.data as *mut u8, i));
    }
}

/// Join the OS thread of a thread, once the thread ended.
///
/// # Arguments
///
/// * `os_thread` - The OS thread, or `None` when a wait joined it before.
fn join(os_thread: Option<std::thread::JoinHandle<()>>)
{
    do_if!(let Some(t) = os_thread, _ = t.join());
}

/// Run the rounds of a parallel loop: with no count of threads, fewer rounds
/// than cores run here, in order, and else each core gets a thread; with a
/// count, a thread for each one it names, never more threads than rounds, and
/// one runs the rounds here, in order. Each thread takes the next block of
/// rounds until none is left. A trap
/// in a round acts as in a thread: the rounds of its thread end, the other
/// threads finish, and the loop then stops with `thread trapped`; under `--trap
/// abort` the program stops at once.
///
/// # Arguments
///
/// * `count` - The count of rounds.
/// * `step` - The step of a round.
/// * `data` - The data block of the loop, which the step reads.
/// * `guard` - The guard of a thread, from the glue.
/// * `jump` - The function that jumps back to the guard on a trap.
/// * `abort` - 1 when a trap stops the whole program.
/// * `threads` - The count of threads the loop was given, 0 for none.
#[unsafe(export_name = stamped!("par_for"))]
pub extern "C" fn zk_par_for(
    count: usize, step: ZkRoundStep, data: *mut u8, guard: ZkGuard,
    jump: extern "C" fn(i32), abort: i32, threads: usize,
)
{
    let cores = std::thread::available_parallelism().map_or(1, usize::from);
    let in_order = tern!(threads == 0, count < cores, threads.min(count) <= 1);
    if in_order
    {
        (0..count).for_each(|i| step(data, i));
        return;
    }
    let workers = tern!(threads == 0, cores, threads.min(count));
    // Eight takes for each thread leave rounds for a thread that ends first to
    // take.
    let block = (count / (workers * 8)).max(1);
    #[rustfmt::skip]
    let mut rounds = Rounds { step, data: data as usize, count, block, next: AtomicUsize::new(0) };
    let at: *mut Rounds = &mut rounds;
    let start = |_| {
        start_thread(
            (guard, jump, run_rounds),
            at.cast(),
            std::ptr::null(),
            abort,
            false,
        )
    };
    let threads: Vec<usize> = (0..workers).map(start).collect();
    // Each thread ends before the loop goes on, so a trap waits for the others,
    // and the rounds outlive each thread.
    let ended: Vec<bool> = threads.iter().map(|n| thread_ended_well(*n)).collect();
    let trapped = threads.iter().zip(ended).find(|(_, ok)| !ok).map(|(n, _)| *n);
    ret_if!(trapped.is_none());
    stop_owned(trap::THREAD_TRAPPED, vec![named("thread", trapped.unwrap_or_default())])
}

/// Wait for a thread to end and mark it awaited.
///
/// # Arguments
///
/// * `n` - The number of the thread.
///
/// # Returns
///
/// `true` when it ran to its end, `false` when a trap ended it.
fn thread_ended_well(n: usize) -> bool
{
    let mut st = wait_until(state(), |s| s.threads[n - 1].end != End::Runs);
    st.threads[n - 1].awaited = true;
    let well = st.threads[n - 1].end == End::Done;
    let os_thread = st.threads[n - 1].os_thread.take();
    // No program holds the number of a thread of a loop, so the next thread
    // takes it.
    st.spare.push(n);
    drop(st);
    join(os_thread);
    well
}

/// What the rounds of a parallel map read and write.
struct MapData
{
    /// The first cell of the plain list the map reads.
    base: usize,
    /// The size of a cell.
    cell: usize,
    /// The first result.
    at: usize,
    /// The size of a result.
    ret: usize,
    /// The step the compiler writes for the function.
    step: ZkMapStep,
}

/// The step of a round of a parallel map: one cell to one result.
///
/// # Arguments
///
/// * `data` - The map.
/// * `i` - The number of the cell.
extern "C" fn map_round(data: *mut u8, i: usize)
{
    // SAFETY: `zk_par_map` hands the rounds its map, which outlives them.
    let m = unsafe { &*data.cast::<MapData>() };
    (m.step)((m.base + i * m.cell) as *mut u8, (m.at + i * m.ret) as *mut u8);
}

/// Run a parallel map: each cell of a plain list goes to the step, the rounds
/// run as those of a parallel loop do, and each result goes to the end of
/// another plain list, in the order of the cells.
///
/// # Arguments
///
/// * `src` - The plain list the map reads.
/// * `src_kind` - Its kind.
/// * `out` - The plain list the map fills, blank before.
/// * `out_kind` - Its kind.
/// * `step` - The step the compiler writes for the function.
/// * `guard` - The guard of a thread, from the glue.
/// * `jump` - The function that jumps back to the guard on a trap.
/// * `abort` - 1 when a trap stops the whole program.
/// * `threads` - The count of threads the map was given, 0 for none.
#[unsafe(export_name = stamped!("par_map"))]
pub extern "C" fn zk_par_map(
    src: *mut u8, src_kind: *const ZkKind, out: *mut u8, out_kind: *const ZkKind,
    step: ZkMapStep, guard: ZkGuard, jump: extern "C" fn(i32), abort: i32,
    threads: usize,
)
{
    let (base, n) = (zk_seq_base(src) as usize, zk_seq_len(src));
    let cell = kind(kind(src_kind).cell).size;
    let ret = kind(kind(out_kind).cell).size;
    let mut results = vec![0u8; (n * ret).max(1)];
    let at = results.as_mut_ptr() as usize;
    let mut map = MapData { base, cell, at, ret, step };
    let data: *mut MapData = &mut map;
    zk_par_for(n, map_round, data.cast(), guard, jump, abort, threads);
    let start = results.as_mut_ptr();
    // SAFETY: each result is `ret` octets of `results`, which the add moves
    // out.
    (0..n).for_each(|i| zk_add_cell(out, out_kind, unsafe { start.add(i * ret) }));
}

/// Make a channel.
///
/// # Arguments
///
/// * `room` - The most values that wait in it; 0 for no limit.
///
/// # Returns
///
/// The number of the channel.
#[unsafe(export_name = stamped!("pipe_new"))]
pub extern "C" fn zk_pipe_new(room: usize) -> usize
{
    let mut st = state();
    st.pipes.push(Pipe { queue: VecDeque::new(), room, kind: 0, closed: false });
    st.pipes.len()
}

/// The index of a channel, or a stop for a number that names none.
///
/// # Arguments
///
/// * `st` - The state.
/// * `n` - The number.
///
/// # Returns
///
/// The index.
fn pipe_at(st: &State, n: usize) -> usize
{
    let at = n.checked_sub(1).filter(|i| *i < st.pipes.len());
    at.unwrap_or_else(|| stop_owned(trap::PIPE_CLOSED, vec![named("pipe", n)]))
}

/// Move a value into a channel, leaving zeros in its place; the send waits
/// while the channel is full, and a closed channel stops the program.
///
/// # Arguments
///
/// * `n` - The number of the channel.
/// * `value` - The address of the value.
/// * `k` - Its kind.
#[unsafe(export_name = stamped!("pipe_send"))]
pub extern "C" fn zk_pipe_send(n: usize, value: *mut u8, k: *const ZkKind)
{
    let size = kind(k).size;
    let st = state();
    let at = pipe_at(&st, n);
    let full = |s: &State| {
        let p = &s.pipes[at];
        p.closed || p.room == 0 || p.queue.len() < p.room
    };
    let mut st = wait_until(st, full);
    if st.pipes[at].closed
    {
        drop(st);
        stop_owned(trap::PIPE_CLOSED, vec![named("pipe", n)]);
    }
    // SAFETY: the value is `size` octets the generated code owns.
    let octets = unsafe { std::slice::from_raw_parts_mut(value, size) };
    let pipe = &mut st.pipes[at];
    pipe.queue.push_back(octets.to_vec().into_boxed_slice());
    pipe.kind = k as usize;
    octets.fill(0);
    changed(&mut st);
}

/// Write the next value of a channel as an optional: it waits while the
/// channel is open and holds none, and writes no data once it is closed and
/// drained.
///
/// # Arguments
///
/// * `n` - The number of the channel.
/// * `out` - The optional.
/// * `k` - The kind of the optional: its fields are the tag and the data.
#[unsafe(export_name = stamped!("pipe_receive"))]
pub extern "C" fn zk_pipe_receive(n: usize, out: *mut u8, k: *const ZkKind)
{
    let optional = kind(k);
    let st = state();
    let at = pipe_at(&st, n);
    let came = |s: &State| s.pipes[at].closed || !s.pipes[at].queue.is_empty();
    let mut st = wait_until(st, came);
    let value = st.pipes[at].queue.pop_front();
    do_if!(value.is_some(), changed(&mut st));
    drop(st);
    // SAFETY: the optional is `size` octets of the generated code.
    let slot = unsafe { std::slice::from_raw_parts_mut(out, optional.size) };
    slot.fill(0);
    let Some(value) = value
    else
    {
        return;
    };
    let fields = optional.field_list();
    let (tag, data) = (fields[0].offset, fields[1].offset);
    slot[tag] = 1;
    slot[data..data + value.len()].copy_from_slice(&value);
}

/// Close a channel; a second close stops the program.
///
/// # Arguments
///
/// * `n` - The number of the channel.
#[unsafe(export_name = stamped!("pipe_close"))]
pub extern "C" fn zk_pipe_close(n: usize)
{
    let mut st = state();
    let at = pipe_at(&st, n);
    if st.pipes[at].closed
    {
        drop(st);
        stop_owned(trap::PIPE_CLOSED, vec![named("pipe", n)]);
    }
    st.pipes[at].closed = true;
    changed(&mut st);
}

/// Wait for each thread at the end of the program, then drop each result the
/// threads left and each value the channels hold.
///
/// # Returns
///
/// Whether a thread stopped at a trap that no wait saw.
pub(crate) fn end() -> bool
{
    let st = state();
    ret_if!(st.threads.is_empty() && st.pipes.is_empty(), false);
    let st = wait_until(st, |s| s.threads.iter().all(|j| j.end != End::Runs));
    let mut st = st;
    let os_threads: Vec<_> =
        st.threads.iter_mut().filter_map(|j| j.os_thread.take()).collect();
    let threads = std::mem::take(&mut st.threads);
    let pipes = std::mem::take(&mut st.pipes);
    drop(st);
    os_threads.into_iter().for_each(|t| _ = t.join());
    let lost = threads.iter().any(|j| j.end == End::Trapped && !j.awaited);
    // The results no wait took are dropped, then each block the runtime owns
    // is freed.
    let untaken =
        |j: &&Thread| j.end == End::Done && j.ret != 0 && !j.awaited && j.data != 0;
    let results = threads.iter().filter(untaken);
    results.for_each(|j| zk_drop(j.data as *mut u8, j.ret as *const ZkKind));
    threads.iter().filter(|j| j.owned).for_each(|j| free_data(j.data));
    let held = pipes.into_iter().filter(|p| p.kind != 0);
    for pipe in held
    {
        let k = pipe.kind as *const ZkKind;
        pipe.queue.into_iter().for_each(|mut v| zk_drop(v.as_mut_ptr(), k));
    }
    lost
}

/// Whether a thread OS thread other than this one can run, the rounds of a
/// parallel loop or map among them, so the ring collector waits: it walks the
/// counts with plain ops.
///
/// # Returns
///
/// `true` while a thread runs.
pub(crate) fn threads_run() -> bool
{
    state().threads.iter().any(|j| j.end == End::Runs)
}

/// Whether this OS thread counts among the running OS threads: the main one, or
/// a thread.
///
/// # Returns
///
/// `true` when it does.
pub(crate) fn counted_thread() -> bool { THREAD.get() != 0 || MAIN.get() }

/// The count of running OS threads other than this one, which counts itself;
/// `None` while another OS thread holds the state, so a wait never blocks on
/// it.
///
/// # Returns
///
/// The count, or `None`.
pub(crate) fn running_others() -> Option<usize>
{
    let st = STATE.try_lock().ok()?;
    Some(st.running.saturating_sub(1))
}
