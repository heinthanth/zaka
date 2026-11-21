//! Light tasks: a thread whose function calls no C, reads no standard input
//! and calls no function again (the compiler checks) runs on a small stack of
//! its own, and a pool of OS threads, one for each core, takes turns on the
//! tasks. A task gives its OS thread to the next one when it waits for a
//! thread, a channel or a lock, when it sleeps, and at the back edge of a loop
//! once its turn is over while others wait; it can go on on another OS thread
//! of the pool, with what it keeps for itself ([`Local`]).
//!
//! A task reaches the cells of an OS thread through the small functions below
//! alone, which are never inlined, and it holds no lock when it stops: a
//! function that went on on another OS thread must not keep the address of a
//! cell of the first.

#[cfg(not(zk_tasks))]
pub(crate) use none::*;
#[cfg(zk_tasks)]
pub(crate) use pool::*;

use crate::threads::Key;
use crate::threads::ZkGuard;
use crate::threads::ZkStep;

/// The pool, where the stacks of tasks can be made.
#[cfg(zk_tasks)]
mod pool
{
    use std::cell::Cell;
    use std::collections::VecDeque;
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::sync::Condvar;
    use std::sync::Mutex;
    use std::sync::MutexGuard;
    use std::sync::PoisonError;
    use std::sync::atomic::Ordering;
    use std::time::Duration;
    use std::time::Instant;

    use zakac_rust_sugars::do_if;
    use zakac_rust_sugars::ret_if;

    use super::*;
    use crate::fiber::Stack;
    use crate::fiber::prepare;
    use crate::fiber::switch;
    use crate::local::Local;

    /// What a task can stop for.
    pub(crate) enum Stop
    {
        /// A wait for the state of the threads to change, after the change with
        /// this count, on what the key names.
        Park(u64, Key),
        /// A sleep until this time.
        Sleep(Instant),
        /// The end of its turn.
        Turn,
    }

    /// Where a task stands once its OS thread runs it no more.
    enum Outcome
    {
        /// It runs, or did not run so far.
        Runs,
        /// It stopped, for this.
        Stopped(Stop),
        /// It ended, its guard giving this.
        Ended(i32),
    }

    /// The octets of the stack of a task: reserved at once, and filled as it
    /// is used. A thread whose calls go deeper runs on an OS thread.
    const STACK: usize = 1 << 20;

    /// How long a task runs before the back edge of a loop gives its OS thread
    /// to the next task, while others wait.
    const TURN: Duration = Duration::from_millis(10);

    /// One light task. It keeps to its box while it lives, so its stack can
    /// keep the address of the box.
    pub(crate) struct Task
    {
        /// Its stack.
        stack: Stack,
        /// Where its stack stopped, for the next switch to it.
        sp: *mut u8,
        /// The stack of the OS thread that runs it, while one does.
        back: *mut u8,
        /// The guard, the step and the data block it starts with, until it
        /// starts.
        start: Option<(ZkGuard, ZkStep, usize)>,
        /// Where it stands once its OS thread runs it no more.
        outcome: Outcome,
        /// What it keeps for itself while no OS thread runs it.
        local: Local,
        /// The number of its thread.
        n: usize,
    }

    // SAFETY: a task that stopped holds no lock and no reference to an OS
    // thread's cells (see the notes of this file), so the next OS thread of the
    // pool can take it over.
    unsafe impl Send for Task {}

    /// The tasks that can go on, and those that sleep.
    struct Pool
    {
        /// The tasks that wait for an OS thread of the pool, first in front.
        queue: VecDeque<Box<Task>>,
        /// The tasks that sleep, with their time to wake.
        asleep: Vec<(Instant, Box<Task>)>,
        /// Whether the OS threads of the pool started.
        started: bool,
    }

    /// The pool.
    static POOL: Mutex<Pool> =
        Mutex::new(Pool { queue: VecDeque::new(), asleep: Vec::new(), started: false });

    /// Wakes an OS thread of the pool once a task can go on.
    static WAKE: Condvar = Condvar::new();

    /// The bit of the halt flag that asks a task to give its turn on.
    const TURN_BIT: u8 = 2;

    thread_local! {
        /// The task this OS thread of the pool runs now; NULL outside each
        /// task.
        static CURRENT: Cell<*mut Task> = const { Cell::new(null_mut()) };
        /// When the turn of that task began.
        static BEGAN: Cell<Option<Instant>> = const { Cell::new(None) };
    }

    /// The pool, locked.
    ///
    /// # Returns
    ///
    /// The guard.
    fn pool() -> MutexGuard<'static, Pool>
    {
        POOL.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Whether a thread can start as a light task: a program runs, not a C lib,
    /// whose OS threads come from C.
    ///
    /// # Returns
    ///
    /// `true` when it can.
    pub(crate) fn can_start() -> bool { crate::threads::program_runs() }

    /// Start a thread as a light task, and the OS threads of the pool at the
    /// first task.
    ///
    /// # Arguments
    ///
    /// * `guard` - The guard the compiler writes.
    /// * `leave` - The jump back to the guard and the number of the thread,
    ///   `None` when a trap of it stops the whole program.
    /// * `step` - The step of the thread.
    /// * `data` - Its data block.
    /// * `n` - The number of the thread.
    ///
    /// # Returns
    ///
    /// `false` when no stack could be made, so the thread needs an OS thread.
    pub(crate) fn start(
        guard: ZkGuard, leave: Option<(extern "C" fn(i32), usize)>, step: ZkStep,
        data: usize, n: usize,
    ) -> bool
    {
        let Some(stack) = Stack::new(STACK)
        else
        {
            return false;
        };
        let local = Local::of_task(n, leave);
        #[rustfmt::skip]
        let mut task = Box::new(Task { stack, sp: null_mut(), back: null_mut(),
            start: Some((guard, step, data)), outcome: Outcome::Runs, local, n });
        let at: *mut Task = &mut *task;
        task.sp = prepare(&task.stack, task_main, at.cast());
        ret_if!(task.sp.is_null(), false);
        let mut pool = pool();
        do_if!(!pool.started, start_pool(&mut pool));
        pool.queue.push_back(task);
        WAKE.notify_one();
        true
    }

    /// Start the OS threads of the pool, one for each core, and the OS thread
    /// that ends long turns.
    ///
    /// # Arguments
    ///
    /// * `pool` - The pool.
    fn start_pool(pool: &mut Pool)
    {
        pool.started = true;
        crate::fiber::catch_overflow();
        let cores = std::thread::available_parallelism().map_or(1, usize::from);
        let spawn = |f: fn()| std::thread::Builder::new().stack_size(8 << 20).spawn(f);
        (0..cores).for_each(|_| _ = spawn(work));
        let _ = spawn(watch_turns);
    }

    /// The work of an OS thread of the pool: run the next task that can go
    /// on, for ever.
    fn work()
    {
        crate::fiber::enter_thread();
        loop
        {
            run(next_task());
        }
    }

    /// The room of the stack of the running task, when a fault lies on its
    /// guard: on unix the address of the fault tells, and on Windows the OS
    /// tells an overflow of the running stack (`None`).
    ///
    /// # Arguments
    ///
    /// * `at` - The address of the fault, `None` for an overflow on Windows.
    ///
    /// # Returns
    ///
    /// The room, or `None` for a fault off each task.
    pub(crate) fn overflowed(at: Option<usize>) -> Option<usize>
    {
        let task = CURRENT.get();
        ret_if!(task.is_null(), None);
        // SAFETY: the running task lives in its box.
        let stack = unsafe { &(*task).stack };
        at.is_none_or(|at| stack.guards(at)).then(|| stack.room())
    }

    /// The next task that can go on: the first in the queue, or one whose sleep
    /// ended; with none, wait for one.
    ///
    /// # Returns
    ///
    /// The task.
    fn next_task() -> Box<Task>
    {
        let mut pool = pool();
        loop
        {
            wake_sleepers(&mut pool);
            ret_if!(let Some(task) = pool.queue.pop_front(), task);
            let first = pool.asleep.iter().map(|(at, _)| *at).min();
            let wait = first.map(|at| at.saturating_duration_since(Instant::now()));
            pool = match wait
            {
                Some(wait) =>
                {
                    WAKE.wait_timeout(pool, wait)
                        .unwrap_or_else(PoisonError::into_inner)
                        .0
                }
                None => WAKE.wait(pool).unwrap_or_else(PoisonError::into_inner),
            };
        }
    }

    /// Move each task whose sleep ended to the queue.
    ///
    /// # Arguments
    ///
    /// * `pool` - The pool.
    fn wake_sleepers(pool: &mut Pool)
    {
        let now = Instant::now();
        let (woken, asleep): (Vec<_>, Vec<_>) = std::mem::take(&mut pool.asleep)
            .into_iter()
            .partition(|(at, _)| *at <= now);
        pool.asleep = asleep;
        pool.queue.extend(woken.into_iter().map(|(_, task)| task));
    }

    /// The first code on the stack of a task: run its guard, keep what the
    /// guard gave, and switch back for good.
    ///
    /// # Arguments
    ///
    /// * `arg` - The task.
    extern "C" fn task_main(arg: *mut u8) -> !
    {
        let task = arg.cast::<Task>();
        // SAFETY: the task lives in its box until it ended, and its OS thread
        // waits in the switch of `run` while it runs.
        unsafe {
            let start = (*task).start.take();
            let (guard, step, data) = start.expect("a task starts once");
            let status = guard(step, data as *mut c_void);
            (*task).outcome = Outcome::Ended(status);
            switch(&raw mut (*task).sp, (*task).back);
        }
        // No switch comes back to a task that ended.
        std::process::abort()
    }

    /// Run a task until it stops or ends, then put it where it waits.
    ///
    /// # Arguments
    ///
    /// * `task` - The task.
    fn run(mut task: Box<Task>)
    {
        task.local.swap();
        let at: *mut Task = &mut *task;
        CURRENT.set(at);
        BEGAN.set(Some(Instant::now()));
        // SAFETY: the stack of the task stopped in a switch, or is new; the
        // box keeps its place while the task runs.
        unsafe { switch(&raw mut (*at).back, (*at).sp) };
        CURRENT.set(null_mut());
        BEGAN.set(None);
        let outcome = std::mem::replace(&mut task.outcome, Outcome::Runs);
        // A task that ended hands on the line it printed, as a thread does.
        do_if!(matches!(outcome, Outcome::Ended(_)), crate::out::flush_line());
        task.local.swap();
        match outcome
        {
            Outcome::Ended(status) => end(task, status),
            Outcome::Stopped(Stop::Park(seen, key)) =>
            {
                crate::threads::park_task(task, seen, key)
            }
            Outcome::Stopped(Stop::Sleep(at)) => asleep(task, at),
            Outcome::Stopped(Stop::Turn) | Outcome::Runs => requeue(vec![task]),
        }
    }

    /// End a task: free what it held, and mark its thread ended.
    ///
    /// # Arguments
    ///
    /// * `task` - The task.
    /// * `status` - What its guard gave: 0, or a value other than 0 after a
    ///   trap.
    fn end(task: Box<Task>, status: i32)
    {
        let Task { stack, local, n, .. } = *task;
        drop(stack);
        local.free();
        crate::threads::thread_ended(n, status);
    }

    /// Put a sleeping task with the others that sleep.
    ///
    /// # Arguments
    ///
    /// * `task` - The task.
    /// * `at` - Its time to wake.
    fn asleep(task: Box<Task>, at: Instant)
    {
        pool().asleep.push((at, task));
        WAKE.notify_one();
    }

    /// Put tasks back with those that can go on.
    ///
    /// # Arguments
    ///
    /// * `tasks` - The tasks.
    pub(crate) fn requeue(tasks: Vec<Box<Task>>)
    {
        ret_if!(tasks.is_empty());
        let count = tasks.len();
        pool().queue.extend(tasks);
        (0..count).for_each(|_| WAKE.notify_one());
    }

    /// The work of the OS thread that ends long turns: while tasks wait for an
    /// OS thread of the pool, ask the task that runs to give its turn on at
    /// the back edge of its next loop.
    fn watch_turns()
    {
        loop
        {
            std::thread::sleep(TURN);
            let waiting = !pool().queue.is_empty();
            let flag = crate::seqs::halt_flag();
            mark_turn(flag, waiting);
        }
    }

    /// Set or clear the turn bit of the halt flag.
    ///
    /// # Arguments
    ///
    /// * `flag` - The flag.
    /// * `set` - Whether to set it.
    fn mark_turn(flag: &std::sync::atomic::AtomicU8, set: bool)
    {
        match set
        {
            true => flag.fetch_or(TURN_BIT, Ordering::SeqCst),
            false => flag.fetch_and(!TURN_BIT, Ordering::SeqCst),
        };
    }

    /// Whether this OS thread runs a task now.
    ///
    /// # Returns
    ///
    /// `true` when it does.
    #[inline(never)]
    pub(crate) fn on_task() -> bool { !CURRENT.get().is_null() }

    /// Stop the running task, and go on once an OS thread of the pool runs it
    /// again, which can be another one, so the caller reads no cell it read
    /// before.
    ///
    /// # Arguments
    ///
    /// * `reason` - What it stops for.
    #[inline(never)]
    fn stop(reason: Stop)
    {
        let task = CURRENT.get();
        // SAFETY: the running task, whose stack this call runs on; its OS
        // thread waits in the switch of `run`.
        unsafe {
            (*task).outcome = Outcome::Stopped(reason);
            switch(&raw mut (*task).sp, (*task).back);
        }
    }

    /// Stop the running task until the state of the threads changes again.
    ///
    /// # Arguments
    ///
    /// * `seen` - The count of changes so far.
    /// * `key` - What it waits on.
    pub(crate) fn park(seen: u64, key: Key)
    {
        stop(Stop::Park(seen, key));
        // A woken task counted as halted until it ran again.
        crate::seqs::halted_leave();
    }

    /// Put woken tasks back with those that can go on: each counts as halted
    /// until it runs again, so a collection does not wait for it.
    ///
    /// # Arguments
    ///
    /// * `tasks` - The tasks.
    pub(crate) fn wake(tasks: Vec<Box<Task>>)
    {
        crate::seqs::halted_add(tasks.len());
        requeue(tasks);
    }

    /// Stop the running task, which counts as running, while it waits: a
    /// collection need not wait for it, and it halts at a safe point after,
    /// when a collection runs then.
    ///
    /// # Arguments
    ///
    /// * `reason` - What it stops for.
    fn stop_running(reason: Stop) { crate::seqs::halted_while(|| stop(reason)) }

    /// Sleep: the running task stops for a count of milliseconds.
    ///
    /// # Arguments
    ///
    /// * `ms` - The count.
    pub(crate) fn sleep(ms: u64)
    {
        let at = Instant::now() + Duration::from_millis(ms);
        stop_running(Stop::Sleep(at));
    }

    /// At the back edge of a loop: the running task gives its OS thread to the
    /// next task once its turn is over and others wait.
    #[inline(never)]
    pub(crate) fn turn()
    {
        let asked = crate::seqs::halt_flag().load(Ordering::Acquire) & TURN_BIT != 0;
        ret_if!(!asked || !on_task());
        let over = BEGAN.get().is_some_and(|b| b.elapsed() >= TURN);
        do_if!(over, stop_running(Stop::Turn));
    }
}

/// No pool: each thread runs on an OS thread of its own.
#[cfg(not(zk_tasks))]
mod none
{
    use super::*;

    /// A light task, which this host never makes.
    pub(crate) struct Task;

    /// Whether a thread can start as a light task: never on this host.
    ///
    /// # Returns
    ///
    /// `false`.
    pub(crate) fn can_start() -> bool { false }

    /// Start a light task: never on this host.
    ///
    /// # Arguments
    ///
    /// * `_guard`, `_leave`, `_step`, `_data`, `_n` - Unused.
    ///
    /// # Returns
    ///
    /// `false`, so the thread gets an OS thread.
    pub(crate) fn start(
        _guard: ZkGuard, _leave: Option<(extern "C" fn(i32), usize)>, _step: ZkStep,
        _data: usize, _n: usize,
    ) -> bool
    {
        false
    }

    /// Wake tasks: there are none on this host.
    ///
    /// # Arguments
    ///
    /// * `_tasks` - None.
    pub(crate) fn wake(_tasks: Vec<Box<Task>>) {}

    /// Whether this OS thread runs a task: never on this host.
    ///
    /// # Returns
    ///
    /// `false`.
    pub(crate) fn on_task() -> bool { false }

    /// Stop a task: there are none on this host.
    ///
    /// # Arguments
    ///
    /// * `_seen`, `_key` - Unused.
    pub(crate) fn park(_seen: u64, _key: Key) {}

    /// Sleep a task: there are none on this host.
    ///
    /// # Arguments
    ///
    /// * `_ms` - Unused.
    pub(crate) fn sleep(_ms: u64) {}

    /// The end of a turn: there are no tasks on this host.
    pub(crate) fn turn() {}
}
