//! The jobs and channels of a native program, each job on a thread of its
//! own. One lock guards the state of each job and channel, and one condition
//! wakes each thread that waits when the state changes. The state counts the
//! threads that run: a thread that would wait while no other runs stops the
//! program with the trap `deadlock`.
//!
//! A job runs a step the compiler writes, under a guard the compiler writes:
//! the step takes the arguments out of the data block of the job and writes
//! the result at its start; the guard keeps the place a trap of the job
//! jumps back to under `--trap unwind`.

use std::cell::Cell;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::PoisonError;
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

/// The step of a job: it takes the data block of the job.
pub type ZkStep = extern "C" fn(*mut c_void);

/// The guard of a job: it runs the step on the data block, and gives 0, or
/// a value other than 0 once a trap jumped back to it.
pub type ZkGuard = extern "C" fn(ZkStep, *mut c_void) -> i32;

/// Where a job stands.
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

/// One job.
struct Job
{
    /// Where it stands.
    end: End,
    /// Its data block: the result at its start, then the arguments.
    data: usize,
    /// The kind of its result, or NULL for a function that gives nothing.
    ret: usize,
    /// Whether some wait saw it.
    awaited: bool,
    /// Its thread, until a wait or the end of the program joins it.
    thread: Option<std::thread::JoinHandle<()>>,
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

/// A lock of a shared object that a thread holds.
struct Held
{
    /// The address of the object.
    obj: usize,
    /// The number of the job that holds it, 0 for the main thread.
    owner: usize,
    /// The count of times the job took it.
    depth: u32,
    /// Whether its holder stopped at a trap, so it never comes back.
    broken: bool,
}

/// The state of the jobs and channels.
struct State
{
    /// Each lock a thread holds, or that a job held when it stopped at a trap.
    locks: Vec<Held>,
    /// Each job; the job a program holds as `n` is at `n - 1`.
    jobs: Vec<Job>,
    /// The numbers of the jobs of parallel loops that ended, which no program
    /// holds, for the next jobs to take.
    spare: Vec<usize>,
    /// Each channel, as the jobs are numbered.
    pipes: Vec<Pipe>,
    /// The threads that run and do not wait, the main one included.
    running: usize,
    /// The threads that wait for the state to change.
    parked: usize,
    /// The count of changes so far, which tells a waiter its wake is real.
    changes: u64,
}

/// The state of the jobs and channels.
static STATE: Mutex<State> = Mutex::new(State {
    locks: Vec::new(),
    jobs: Vec::new(),
    spare: Vec::new(),
    pipes: Vec::new(),
    running: 1,
    parked: 0,
    changes: 0,
});

/// Wakes each thread that waits once the state changes.
static CHANGED: Condvar = Condvar::new();

thread_local! {
    /// The function that jumps back to the guard of the job of this thread
    /// under `--trap unwind`, and the number of the job.
    static LEAVE: Cell<Option<(extern "C" fn(i32), usize)>> = const { Cell::new(None) };
    /// The trap that ended the job of this thread.
    static TRAPPED: Cell<i32> = const { Cell::new(0) };
    /// The number of the job of this thread, 0 for the main thread.
    static JOB: Cell<usize> = const { Cell::new(0) };
}

/// The state, locked.
///
/// # Returns
///
/// The guard.
fn state() -> MutexGuard<'static, State>
{
    STATE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Mark the state changed: each thread that waits runs again and looks.
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

/// Wait until `came` holds; a wait while no other thread runs stops the
/// program with the trap `deadlock`.
///
/// # Arguments
///
/// * `st` - The state, locked.
/// * `came` - What the thread waits for.
///
/// # Returns
///
/// The state, locked, with `came` holding.
fn wait_until(
    mut st: MutexGuard<'static, State>, came: impl Fn(&State) -> bool,
) -> MutexGuard<'static, State>
{
    while !came(&st)
    {
        st.running -= 1;
        st.parked += 1;
        if st.running == 0
        {
            drop(st);
            deadlock();
        }
        let seen = st.changes;
        while st.changes == seen
        {
            st = CHANGED.wait(st).unwrap_or_else(PoisonError::into_inner);
        }
    }
    st
}

/// Stop the whole program with the trap `deadlock`, from whichever thread.
fn deadlock() -> !
{
    LEAVE.set(None);
    stop(trap::DEADLOCK, &[])
}

/// The function that jumps back to the guard of the job of this thread, and
/// its number, when a trap ends the job alone.
///
/// # Returns
///
/// The function and the number, or `None` on the main thread and under
/// `--trap abort`.
pub(crate) fn leave() -> Option<(extern "C" fn(i32), usize)> { LEAVE.get() }

/// Keep the trap that ends the job of this thread, for its guard.
///
/// # Arguments
///
/// * `trap` - The trap.
pub(crate) fn keep_trap(trap: i32) { TRAPPED.set(trap) }

/// Make a data block of a job: zeros, which the spawn fills.
///
/// # Arguments
///
/// * `size` - The octets of the block.
///
/// # Returns
///
/// The block.
#[unsafe(export_name = stamped!("job_data"))]
pub extern "C" fn zk_job_data(size: usize) -> *mut u8
{
    let block = vec![0u8; size.max(1)].into_boxed_slice();
    Box::leak(block).as_mut_ptr()
}

/// Start a job on a thread of its own.
///
/// # Arguments
///
/// * `guard` - The guard the compiler writes.
/// * `jump` - The function that jumps back to the guard.
/// * `step` - The step of the job.
/// * `data` - Its data block, from [`zk_job_data`].
/// * `ret` - The kind of its result, or NULL for none.
/// * `abort` - 1 when a trap of the job stops the whole program.
///
/// # Returns
///
/// The number of the job.
#[unsafe(export_name = stamped!("job_start"))]
pub extern "C" fn zk_job_start(
    guard: ZkGuard, jump: extern "C" fn(i32), step: ZkStep, data: *mut u8,
    ret: *const ZkKind, abort: i32,
) -> usize
{
    zk_threads_begin();
    let mut st = state();
    #[rustfmt::skip]
    let job = Job { end: End::Runs, data: data as usize, ret: ret as usize, awaited: false,
        thread: None };
    let next = st.jobs.len() + 1;
    let n = st.spare.pop().unwrap_or(next);
    if n == next
    {
        st.jobs.push(job);
    }
    else
    {
        st.jobs[n - 1] = job;
    }
    st.running += 1;
    let data = data as usize;
    let run = move || run_job(guard, (jump, abort != 0), step, data, n);
    let thread = std::thread::Builder::new().stack_size(64 << 20).spawn(run);
    let Ok(thread) = thread
    else
    {
        drop(st);
        stop_owned(trap::OUT_OF_MEM, vec![named("size", 64usize << 20)])
    };
    st.jobs[n - 1].thread = Some(thread);
    n
}

/// The work of the thread of a job: run the guard, then mark the job ended.
///
/// # Arguments
///
/// * `guard` - The guard.
/// * `(jump, abort)` - The function that jumps back to the guard, and whether a
///   trap stops the whole program.
/// * `step` - The step.
/// * `data` - The data block.
/// * `n` - The number of the job.
fn run_job(
    guard: ZkGuard, (jump, abort): (extern "C" fn(i32), bool), step: ZkStep,
    data: usize, n: usize,
)
{
    do_if!(!abort, LEAVE.set(Some((jump, n))));
    JOB.set(n);
    let status = guard(step, data as *mut c_void);
    LEAVE.set(None);
    // The thread ends here, and its calls with it.
    crate::ctx::release_frames();
    let mut st = state();
    st.jobs[n - 1].end = if status == 0 { End::Done } else { End::Trapped };
    // The locks a job held when it stopped remain taken, broken.
    st.locks.iter_mut().filter(|h| h.owner == n).for_each(|h| h.broken = true);
    st.running -= 1;
    changed(&mut st);
}

/// Wait for a job to end.
///
/// # Arguments
///
/// * `n` - The number of the job.
///
/// # Returns
///
/// The address of its result, at the start of its data block; a job that
/// stopped at a trap stops the waiter with the trap `job trapped`.
#[unsafe(export_name = stamped!("job_wait"))]
pub extern "C" fn zk_job_wait(n: usize) -> *mut u8
{
    let st = state();
    let at = n.checked_sub(1).filter(|i| *i < st.jobs.len());
    let Some(at) = at
    else
    {
        drop(st);
        stop_owned(trap::JOB_TRAPPED, vec![named("job", n)])
    };
    let mut st = wait_until(st, |s| s.jobs[at].end != End::Runs);
    st.jobs[at].awaited = true;
    let (end, data) = (st.jobs[at].end, st.jobs[at].data);
    let thread = st.jobs[at].thread.take();
    drop(st);
    join(thread);
    ret_if!(end == End::Done, data as *mut u8);
    stop_owned(trap::JOB_TRAPPED, vec![named("job", n)])
}

/// Take the lock of a shared object: the thread waits while another holds
/// it, and takes once more a lock it holds.
///
/// # Arguments
///
/// * `obj` - The variable that holds the object.
#[unsafe(export_name = stamped!("lock"))]
pub extern "C" fn zk_lock(obj: *mut u8)
{
    // The block of the object names the lock, which each job that
    // holds the object shares; the variable differs from job to job.
    let (obj, me) = (zk_seq_base(obj) as usize, JOB.get());
    let free =
        |s: &State| s.locks.iter().all(|h| h.obj != obj || h.owner == me || h.broken);
    let mut st = wait_until(state(), free);
    let held = st.locks.iter_mut().find(|h| h.obj == obj);
    let Some(held) = held
    else
    {
        st.locks.push(Held { obj, owner: me, depth: 1, broken: false });
        return;
    };
    if held.broken
    {
        drop(st);
        stop(trap::LOCK_BROKEN, &[]);
    }
    held.depth += 1;
}

/// Give back the lock of a shared object once.
///
/// # Arguments
///
/// * `obj` - The variable that holds the object.
#[unsafe(export_name = stamped!("unlock"))]
pub extern "C" fn zk_unlock(obj: *mut u8)
{
    let (obj, me) = (zk_seq_base(obj) as usize, JOB.get());
    let mut st = state();
    let at = st.locks.iter().position(|h| h.obj == obj && h.owner == me);
    let Some(at) = at
    else
    {
        return;
    };
    st.locks[at].depth -= 1;
    ret_if!(st.locks[at].depth > 0);
    st.locks.remove(at);
    changed(&mut st);
}

/// Add a value to an atomic number of the platform width, or subtract it,
/// in one step no other thread can split; the result wraps.
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
    // each thread changes with atomic ops alone.
    let n = unsafe { AtomicIsize::from_ptr(at) };
    _ = tern!(
        sub != 0,
        n.fetch_sub(v, Ordering::SeqCst),
        n.fetch_add(v, Ordering::SeqCst)
    );
}

/// Read an atomic number of the platform width in one step that sees each
/// change another thread made before it.
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

/// Set an atomic number of the platform width in one step, which each other
/// thread sees whole and in order.
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

/// The rounds of a parallel loop that its jobs share: each job takes the next
/// block of rounds until none is left, so a job whose rounds run fast takes
/// more of them.
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
    /// The first round no job took so far.
    next: AtomicUsize,
}

/// The step of a job of a parallel loop: take a block of rounds and run them,
/// until no round is left.
///
/// # Arguments
///
/// * `data` - The rounds.
extern "C" fn run_rounds(data: *mut c_void)
{
    // SAFETY: `zk_par_for` hands each job the rounds, which outlive the jobs.
    let r = unsafe { &*data.cast::<Rounds>() };
    loop
    {
        let lo = r.next.fetch_add(r.block, Ordering::Relaxed);
        ret_if!(lo >= r.count);
        let hi = (lo + r.block).min(r.count);
        (lo..hi).for_each(|i| (r.step)(r.data as *mut u8, i));
    }
}

/// Join the thread of a job, once the job ended.
///
/// # Arguments
///
/// * `thread` - The thread, or `None` when a wait joined it before.
fn join(thread: Option<std::thread::JoinHandle<()>>)
{
    do_if!(let Some(t) = thread, _ = t.join());
}

/// Run the rounds of a parallel loop: with no count of threads, fewer rounds
/// than cores run here, in order, and else each core gets a job; with a count,
/// a job for each thread it names, never more jobs than rounds, and one runs
/// the rounds here, in order. Each job takes the next block of rounds until
/// none is left. A trap
/// in a round acts as in a job: the rounds of its job end, the other jobs
/// finish, and the loop then stops with `job trapped`; under `--trap abort` the
/// program stops at once.
///
/// # Arguments
///
/// * `count` - The count of rounds.
/// * `step` - The step of a round.
/// * `data` - The data block of the loop, which the step reads.
/// * `guard` - The guard of a job, from the glue.
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
    // Eight takes for each job leave rounds for a job that ends first to take.
    let block = (count / (workers * 8)).max(1);
    #[rustfmt::skip]
    let mut rounds = Rounds { step, data: data as usize, count, block, next: AtomicUsize::new(0) };
    let at: *mut Rounds = &mut rounds;
    let start =
        |_| zk_job_start(guard, jump, run_rounds, at.cast(), std::ptr::null(), abort);
    let jobs: Vec<usize> = (0..workers).map(start).collect();
    // Each job ends before the loop goes on, so a trap waits for the others,
    // and the rounds outlive each job.
    let ended: Vec<bool> = jobs.iter().map(|n| job_ended_well(*n)).collect();
    let trapped = jobs.iter().zip(ended).find(|(_, ok)| !ok).map(|(n, _)| *n);
    ret_if!(trapped.is_none());
    stop_owned(trap::JOB_TRAPPED, vec![named("job", trapped.unwrap_or_default())])
}

/// Wait for a job to end and mark it awaited.
///
/// # Arguments
///
/// * `n` - The number of the job.
///
/// # Returns
///
/// `true` when it ran to its end, `false` when a trap ended it.
fn job_ended_well(n: usize) -> bool
{
    let mut st = wait_until(state(), |s| s.jobs[n - 1].end != End::Runs);
    st.jobs[n - 1].awaited = true;
    let well = st.jobs[n - 1].end == End::Done;
    let thread = st.jobs[n - 1].thread.take();
    // No program holds the number of a job of a loop, so the next job takes it.
    st.spare.push(n);
    drop(st);
    join(thread);
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
/// * `guard` - The guard of a job, from the glue.
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

/// Wait for each job at the end of the program, then drop each result the
/// jobs left and each value the channels hold.
///
/// # Returns
///
/// Whether a job stopped at a trap that no wait saw.
pub(crate) fn end() -> bool
{
    let st = state();
    ret_if!(st.jobs.is_empty() && st.pipes.is_empty(), false);
    let st = wait_until(st, |s| s.jobs.iter().all(|j| j.end != End::Runs));
    let mut st = st;
    let threads: Vec<_> = st.jobs.iter_mut().filter_map(|j| j.thread.take()).collect();
    let jobs = std::mem::take(&mut st.jobs);
    let pipes = std::mem::take(&mut st.pipes);
    drop(st);
    threads.into_iter().for_each(|t| _ = t.join());
    let lost = jobs.iter().any(|j| j.end == End::Trapped && !j.awaited);
    let results = jobs.iter().filter(|j| j.end == End::Done && j.ret != 0);
    results.for_each(|j| zk_drop(j.data as *mut u8, j.ret as *const ZkKind));
    let held = pipes.into_iter().filter(|p| p.kind != 0);
    for pipe in held
    {
        let k = pipe.kind as *const ZkKind;
        pipe.queue.into_iter().for_each(|mut v| zk_drop(v.as_mut_ptr(), k));
    }
    lost
}

/// Whether a job thread other than this one can run, the rounds of a
/// parallel loop or map among them, so the ring collector waits: it walks the
/// counts with plain ops.
///
/// # Returns
///
/// `true` while a job runs.
pub(crate) fn jobs_run() -> bool { state().jobs.iter().any(|j| j.end == End::Runs) }
