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
        /// A call that can block, which an OS thread for I/O runs.
        Io(Job),
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

    /// The bit of the halt flag that asks a task to give its turn on. The loops
    /// of each OS thread read the flag, those of the program start block among
    /// them, and take a call while it is set; so it is set just while a task is
    /// past its turn and others wait, and cleared once the task gives it on.
    const TURN_BIT: u8 = 2;

    /// When the task of each OS thread of the pool began its turn, in
    /// milliseconds from [`EPOCH`] plus one; 0 while it runs none.
    static BEGUN: std::sync::OnceLock<Box<[std::sync::atomic::AtomicU64]>> =
        std::sync::OnceLock::new();

    /// The time the turns count from.
    static EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

    /// The milliseconds from [`EPOCH`], plus one, so 0 stands for no turn.
    ///
    /// # Returns
    ///
    /// The count.
    fn now_ms() -> u64
    {
        let epoch = EPOCH.get_or_init(Instant::now);
        u64::try_from(epoch.elapsed().as_millis()).unwrap_or(u64::MAX - 1) + 1
    }

    /// Whether the turn that began at `begun` is over.
    ///
    /// # Arguments
    ///
    /// * `begun` - A value of [`BEGUN`].
    /// * `now` - The time, as [`now_ms`] gives it.
    ///
    /// # Returns
    ///
    /// `true` when it is.
    fn over(begun: u64, now: u64) -> bool
    {
        let turn = u64::try_from(TURN.as_millis()).unwrap_or(u64::MAX);
        begun != 0 && now.saturating_sub(begun) >= turn
    }

    /// Set the turn bit while a task is past its turn and others wait, and
    /// clear it otherwise.
    fn refresh_turn()
    {
        let waiting = !pool().queue.is_empty();
        let now = now_ms();
        let begun = BEGUN.get().map_or(&[][..], |b| &b[..]);
        let past = begun.iter().any(|b| over(b.load(Ordering::Relaxed), now));
        mark_turn(crate::seqs::halt_flag(), waiting && past);
    }

    thread_local! {
        /// The task this OS thread of the pool runs now; NULL outside each
        /// task.
        static CURRENT: Cell<*mut Task> = const { Cell::new(null_mut()) };
        /// The number of this OS thread among those of the pool.
        static SLOT: Cell<usize> = const { Cell::new(0) };
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
        let slots = (0..cores).map(|_| std::sync::atomic::AtomicU64::new(0)).collect();
        let _ = BEGUN.set(slots);
        let builder = || std::thread::Builder::new().stack_size(8 << 20);
        (0..cores).for_each(|i| _ = builder().spawn(move || work(i)));
        let _ = builder().spawn(watch_turns);
    }

    /// The work of an OS thread of the pool: run the next task that can go
    /// on, for ever.
    ///
    /// # Arguments
    ///
    /// * `slot` - Its number among the OS threads of the pool.
    fn work(slot: usize)
    {
        SLOT.set(slot);
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
        begin_turn(now_ms());
        // SAFETY: the stack of the task stopped in a switch, or is new; the
        // box keeps its place while the task runs.
        unsafe { switch(&raw mut (*at).back, (*at).sp) };
        CURRENT.set(null_mut());
        begin_turn(0);
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
            Outcome::Stopped(Stop::Io(job)) => start_io(job, task),
            Outcome::Stopped(Stop::Turn) | Outcome::Runs =>
            {
                requeue(vec![task]);
                refresh_turn();
            }
        }
    }

    /// Note the start of a turn on this OS thread of the pool, or its end (0).
    ///
    /// # Arguments
    ///
    /// * `at` - The time, as [`now_ms`] gives it, or 0.
    fn begin_turn(at: u64)
    {
        let begun = BEGUN.get().and_then(|b| b.get(SLOT.get()));
        do_if!(let Some(begun) = begun, begun.store(at, Ordering::Relaxed));
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
    /// OS thread of the pool and a task is past its turn, ask it to give its
    /// turn on at the back edge of its next loop.
    fn watch_turns()
    {
        loop
        {
            std::thread::sleep(TURN);
            refresh_turn();
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

    /// A call that can block, on the stack of the task that waits for it: the
    /// task stops until an OS thread for I/O ran it, so what it borrows from
    /// the stack lives as long as the call.
    pub(crate) struct Job(*mut (dyn FnMut() + 'static));

    // SAFETY: the task that owns the call waits, stopped, until the OS thread
    // for I/O ran it, and reaches none of it meanwhile.
    unsafe impl Send for Job {}

    /// The most OS threads for I/O: more calls at once wait in the queue.
    const IO_THREADS: usize = 64;

    /// The calls that wait for an OS thread for I/O, each with its task, and
    /// the count of those OS threads, and of those that wait.
    struct Io
    {
        /// The calls, first in front.
        queue: VecDeque<(Job, Box<Task>)>,
        /// The OS threads for I/O.
        threads: usize,
        /// Those of them that wait for a call.
        idle: usize,
    }

    /// The calls for I/O and their OS threads.
    static IO: Mutex<Io> =
        Mutex::new(Io { queue: VecDeque::new(), threads: 0, idle: 0 });

    /// Wakes an OS thread for I/O once a call comes.
    static IO_WAKE: Condvar = Condvar::new();

    /// Run a call that can block, such as one on a file: on a light task an OS
    /// thread for I/O runs it while the OS thread of the pool goes on with
    /// other tasks; elsewhere it runs here.
    ///
    /// # Arguments
    ///
    /// * `f` - The call.
    ///
    /// # Returns
    ///
    /// What the call gave.
    pub(crate) fn blocking<T>(f: impl FnOnce() -> T) -> T
    {
        if !on_task()
        {
            return f();
        }
        let (mut f, mut out) = (Some(f), None);
        let mut call = || out = f.take().map(|f| f());
        let call: &mut dyn FnMut() = &mut call;
        // SAFETY: the task waits until the call ran, so its stack, and what the
        // call borrows from it, outlives the call.
        let job = unsafe {
            std::mem::transmute::<*mut dyn FnMut(), *mut (dyn FnMut() + 'static)>(call)
        };
        stop_running(Stop::Io(Job(job)));
        out.expect("an OS thread for I/O ran the call")
    }

    /// Hand a call of a task to an OS thread for I/O, starting one when none
    /// waits.
    ///
    /// # Arguments
    ///
    /// * `job` - The call.
    /// * `task` - The task that waits for it.
    fn start_io(job: Job, task: Box<Task>)
    {
        let mut io = IO.lock().unwrap_or_else(PoisonError::into_inner);
        io.queue.push_back((job, task));
        let more = io.idle == 0 && io.threads < IO_THREADS;
        do_if!(more, io.threads += 1);
        drop(io);
        do_if!(more, _ = std::thread::Builder::new().spawn(io_work));
        IO_WAKE.notify_one();
    }

    /// The work of an OS thread for I/O: run each call that comes, then put
    /// its task back with those that can go on.
    fn io_work()
    {
        loop
        {
            let (Job(call), task) = next_io();
            // SAFETY: the task of the call waits, stopped, so the call and
            // what it borrows live.
            unsafe { (*call)() };
            requeue(vec![task]);
        }
    }

    /// The next call for I/O; with none, wait for one.
    ///
    /// # Returns
    ///
    /// The call and its task.
    fn next_io() -> (Job, Box<Task>)
    {
        let mut io = IO.lock().unwrap_or_else(PoisonError::into_inner);
        loop
        {
            ret_if!(let Some(next) = io.queue.pop_front(), next);
            io.idle += 1;
            io = IO_WAKE.wait(io).unwrap_or_else(PoisonError::into_inner);
            io.idle -= 1;
        }
    }

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
        let begun = BEGUN.get().and_then(|b| b.get(SLOT.get()));
        let past = begun.is_some_and(|b| over(b.load(Ordering::Relaxed), now_ms()));
        do_if!(past, stop_running(Stop::Turn));
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

    /// Run a call that can block, here: there are no tasks on this host.
    ///
    /// # Arguments
    ///
    /// * `f` - The call.
    ///
    /// # Returns
    ///
    /// What the call gave.
    pub(crate) fn blocking<T>(f: impl FnOnce() -> T) -> T { f() }
}
