//! The threads of a program, run as green threads: each thread has its own
//! operand stack, calls and stack region, over the one heap, and the threads
//! take turns of a fixed count of ops on one host thread. The order of the
//! turns is fixed, so a run gives the same output each time; a seed
//! (`--shuffle-thread-order`) shuffles them, and the same seed gives the same
//! order again.
//!
//! The running thread keeps its state in the fields of the machine; each other
//! thread keeps its state in its row of [`Threads`], and a switch swaps the
//! two. A wait for a thread that has not ended parks the waiter: the op goes
//! back one, so it runs again once the thread it waits for ends.

use zakac_rust_sugars::ret_false;

use super::pipes::Pipe;
use super::*;
use crate::mem::StackRange;

/// The ops one thread runs before the next one takes its turn.
const TURN: u32 = 100;

/// What a thread does when it traps, as `--trap` chooses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum TrapMode
{
    /// The trap ends its thread alone; a wait for that thread traps in turn. A
    /// trap of the main thread still stops the program.
    #[default]
    Unwind,
    /// The trap of a thread stops the whole program.
    Abort,
}

/// What a parked thread waits for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Waits
{
    /// Nothing: the thread can run.
    #[default]
    Nothing,
    /// The end of the thread with this number.
    Thread(usize),
    /// The end of each other thread: the main thread at the end of the program.
    Each,
    /// A value in the channel with this number, or its close.
    Receive(usize),
    /// Room in the channel with this number, or its close.
    Send(usize),
    /// The lock of the object at this address.
    Lock(u64),
    /// The clock of the world at this count of milliseconds: a sleep.
    Until(u64),
}

/// Where a thread stands.
#[derive(Debug, Clone, PartialEq, Default)]
enum End
{
    /// It runs, or waits for its turn.
    #[default]
    Runs,
    /// It returned, with its result: a value, or nothing.
    Done(Option<Value>),
    /// It returned a list or a struct, kept as its octets: the block it was
    /// in sat in the stack region of the thread, which the next thread takes.
    Kept(Vec<u8>),
    /// It stopped at a trap.
    Trapped(Box<Trap>),
}

/// One thread.
#[derive(Debug, Clone, Default, Builder)]
struct Thread
{
    /// Its operand stack, while it is parked.
    #[builder(default)]
    stack: Vec<Value>,
    /// The count of locals of its calls, while it is parked.
    #[builder(default)]
    slots: usize,
    /// Its calls, while it is parked.
    #[builder(default)]
    frames: Vec<Frame>,
    /// Its stack region, while it is parked.
    range: StackRange,
    /// What it waits for.
    #[builder(default)]
    waits: Waits,
    /// Where it stands.
    #[builder(default)]
    end: End,
    /// The kind of its result, `None` for a function that gives nothing.
    ret: Option<DataKind>,
    /// Whether some thread waited for it.
    #[builder(default)]
    awaited: bool,
}

/// A lock of a shared object that a thread holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Held
{
    /// The address of the object.
    obj: u64,
    /// The number of the thread that holds it.
    owner: usize,
    /// The count of times the thread took it.
    depth: u32,
    /// Whether its holder stopped at a trap, so it never comes back.
    broken: bool,
}

/// The threads of a program and their turns.
#[derive(Debug, Clone)]
pub struct Threads
{
    /// Each thread, main at 0; the row of the running one is out of date.
    list: Vec<Thread>,
    /// Each channel, the first under the number 1.
    pub(super) pipes: Vec<Pipe>,
    /// Each lock a thread holds, or that a thread held when it stopped at a
    /// trap.
    locks: Vec<Held>,
    /// The number of the running thread.
    now: usize,
    /// The ops left in the turn of the running thread.
    left: u32,
    /// The stack regions of the threads that ended, for the next threads to
    /// take.
    spare: Vec<StackRange>,
    /// What draws the length and the thread of each turn, when a seed shuffles
    /// them.
    rng: Option<fastrand::Rng>,
}

impl Default for Threads
{
    fn default() -> Self
    {
        #[rustfmt::skip]
        let threads = Threads { list: vec![Thread::default()], pipes: Vec::new(), locks: Vec::new(),
            now: 0, left: TURN, spare: Vec::new(), rng: None };
        threads
    }
}

impl Threads
{
    /// Whether a thread stopped at a trap and no thread waited for it: the
    /// program then ends with code 1.
    ///
    /// # Returns
    ///
    /// `true` if one did.
    pub fn lost_trap(&self) -> bool
    {
        self.list.iter().any(|j| matches!(j.end, End::Trapped(_)) && !j.awaited)
    }

    /// The traps that ended threads, each with the number of its thread.
    ///
    /// # Returns
    ///
    /// The traps, in thread order.
    pub fn traps(&self) -> Vec<(usize, &Trap)>
    {
        self.list
            .iter()
            .enumerate()
            .filter_map(|(n, j)| match &j.end
            {
                End::Trapped(t) => Some((n, t.as_ref())),
                _ => None,
            })
            .collect()
    }

    /// Whether the running thread is the main one.
    ///
    /// # Returns
    ///
    /// `true` for the main thread.
    pub fn on_main(&self) -> bool { self.now == 0 }

    /// Whether a thread other than the running one has not ended.
    ///
    /// # Returns
    ///
    /// `true` if one has not.
    fn others_run(&self) -> bool
    {
        let runs = |(n, j): (usize, &Thread)| n != self.now && j.end == End::Runs;
        self.list.iter().enumerate().any(runs)
    }

    /// Whether a thread can run: it has not ended, and what it waits for came.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of the thread.
    /// * `now` - The clock of the world, in milliseconds.
    ///
    /// # Returns
    ///
    /// `true` if it can.
    fn can_go(&self, n: usize, now: u64) -> bool
    {
        let thread = &self.list[n];
        ret_false!(thread.end != End::Runs);
        match thread.waits
        {
            Waits::Nothing => true,
            Waits::Thread(other) =>
            {
                self.list.get(other).is_none_or(|j| j.end != End::Runs)
            }
            Waits::Each =>
            {
                let ended = |(k, j): (usize, &Thread)| k == n || j.end != End::Runs;
                self.list.iter().enumerate().all(ended)
            }
            Waits::Receive(p) => self.pipes.get(p).is_none_or(Pipe::can_receive),
            Waits::Lock(obj) => self.locks.iter().all(|h| h.obj != obj || h.broken),
            Waits::Send(p) => self.pipes.get(p).is_none_or(Pipe::can_send),
            Waits::Until(t) => now >= t,
        }
    }

    /// The earliest clock at which a sleeping thread wakes.
    ///
    /// # Returns
    ///
    /// The clock, or `None` when no thread sleeps.
    fn first_wake(&self) -> Option<u64>
    {
        let wake = |j: &Thread| match (&j.end, j.waits)
        {
            (End::Runs, Waits::Until(t)) => Some(t),
            _ => None,
        };
        self.list.iter().filter_map(wake).min()
    }

    /// The next thread that can run: after the running one in order, the
    /// running one itself last; with a seed, one of those that can run, drawn
    /// from it.
    ///
    /// # Arguments
    ///
    /// * `now` - The clock of the world, in milliseconds.
    ///
    /// # Returns
    ///
    /// The number of the thread, or `None` when none can run.
    fn next_to_go(&mut self, now: u64) -> Option<usize>
    {
        let count = self.list.len();
        let free = (1..=count)
            .map(|k| (self.now + k) % count)
            .filter(|n| self.can_go(*n, now));
        let free: Vec<usize> = free.collect();
        let Some(rng) = self.rng.as_mut()
        else
        {
            return free.first().copied();
        };
        ret_if!(free.is_empty(), None);
        Some(free[rng.usize(..free.len())])
    }

    /// Shuffle the turns from a seed, or keep them fixed.
    ///
    /// # Arguments
    ///
    /// * `seed` - The seed, or `None` for the fixed turns.
    pub(super) fn shuffle(&mut self, seed: Option<u64>)
    {
        self.rng = seed.map(fastrand::Rng::with_seed);
        self.left = self.turn();
    }

    /// The count of ops of the next turn: a fixed count, or with a seed one
    /// from 1 to twice that count.
    ///
    /// # Returns
    ///
    /// The count.
    fn turn(&mut self) -> u32
    {
        self.rng.as_mut().map_or(TURN, |r| r.u32(1..=2 * TURN))
    }
}

/// The kind of a thread number.
pub(super) const THREAD_ID: IntKind = IntKind::new(false, Width::Platform);

/// What a thread does, as a debugger shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState
{
    /// It runs now.
    Runs,
    /// It waits for its turn.
    Queued,
    /// It waits for the end of the thread with this number.
    WaitsThread(usize),
    /// It waits for each other thread to end: the main thread at the end.
    WaitsEach,
    /// It waits for a value of the channel with this number.
    WaitsReceive(usize),
    /// It waits for room in the channel with this number.
    WaitsSend(usize),
    /// It waits for a lock another thread holds.
    WaitsLock,
    /// It sleeps.
    Sleeps,
    /// It returned.
    Done,
    /// It stopped at a trap.
    Trapped,
}

impl Machine
{
    /// Each thread of the program and what it does, the main thread first, for
    /// a debugger.
    ///
    /// # Returns
    ///
    /// The state of each thread, under its number.
    pub fn thread_states(&self) -> Vec<ThreadState>
    {
        let threads = &self.threads;
        let state = |(n, j): (usize, &Thread)| match (&j.end, j.waits)
        {
            (End::Trapped(_), _) => ThreadState::Trapped,
            (End::Done(_) | End::Kept(_), _) => ThreadState::Done,
            _ if n == threads.now => ThreadState::Runs,
            (_, Waits::Nothing) => ThreadState::Queued,
            (_, Waits::Thread(k)) => ThreadState::WaitsThread(k),
            (_, Waits::Each) => ThreadState::WaitsEach,
            (_, Waits::Receive(p)) => ThreadState::WaitsReceive(p),
            (_, Waits::Send(p)) => ThreadState::WaitsSend(p),
            (_, Waits::Lock(_)) => ThreadState::WaitsLock,
            (_, Waits::Until(_)) => ThreadState::Sleeps,
        };
        threads.list.iter().enumerate().map(state).collect()
    }

    /// The number of the thread that runs now; the main thread is 0.
    ///
    /// # Returns
    ///
    /// The number.
    pub fn running_thread(&self) -> usize { self.threads.now }
}

impl Machine
{
    /// Start a thread that calls a function: its arguments leave the stack,
    /// the thread enters the function at once, and the number of the thread is
    /// pushed. The thread holds each list and struct argument, which the spawn
    /// takes owned.
    ///
    /// # Arguments
    ///
    /// * `f` - The function index.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    pub(super) fn spawn(&mut self, f: u32) -> Step
    {
        let func = self.module.funcs.get(f as usize);
        let func = func.ok_or_else(|| malformed("no function"))?;
        let (params, ret) = (func.params as usize, func.ret);
        let kinds: Vec<DataKind> =
            func.locals.iter().take(params).map(|s| s.kind).collect();
        ret_if!(self.stack.len() < params, Err(malformed("missing arguments")));
        let args = self.stack.split_off(self.stack.len() - params);
        let size = self.options.max_stack;
        let spare = self.threads.spare.pop().map(|r| StackRange { top: r.lo, ..r });
        let range = spare.or_else(|| self.mem.take_region(size));
        let range = range.ok_or(TrapKind::StackFull { size })?;
        let n = self.threads.list.len();
        let thread = Thread::builder().range(range).maybe_ret(ret).build();
        self.threads.list.push(thread);
        let back = self.threads.now;
        self.swap_to(n);
        let entered = self.enter_thread(f, args, &kinds, ret);
        self.swap_to(back);
        entered?;
        let platform = self.module.platform_bits;
        self.push(Value::int(THREAD_ID, n as u128, platform))
    }

    /// Enter the function of a new thread, the running one: the slot of a list
    /// or struct result and the copies of the arguments go to its region.
    ///
    /// # Arguments
    ///
    /// * `f` - The function index.
    /// * `args` - The arguments, as the call takes them.
    /// * `kinds` - The kind of each parameter.
    /// * `ret` - The result kind.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn enter_thread(
        &mut self, f: u32, args: Vec<Value>, kinds: &[DataKind], ret: Option<DataKind>,
    ) -> Step
    {
        let platform = self.module.platform_bits;
        if let Some(kind) = ret.filter(|k| k.is_aggregate())
        {
            let size = u64::from(kind.size(platform));
            let slot =
                self.mem.push_stack(size, 0).ok_or(TrapKind::StackFull { size })?;
            self.push(Value::Ptr(slot))?;
        }
        for (v, kind) in args.into_iter().zip(kinds)
        {
            let v = tern!(kind.is_aggregate(), self.keep_arg(v, *kind)?, v);
            self.push(v)?;
        }
        self.enter(f)
    }

    /// Move a list or struct argument into a block of the running thread: the
    /// spawn takes its arguments owned, as a set does, and the function of
    /// the thread drops each of its parameters as it returns.
    ///
    /// # Arguments
    ///
    /// * `v` - The address of the argument.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The address of the duplicate, or a trap.
    fn keep_arg(&mut self, v: Value, kind: DataKind) -> Result<Value, TrapKind>
    {
        let from = v.as_ptr().ok_or_else(|| malformed("an argument is an address"))?;
        let size = u64::from(kind.size(self.module.platform_bits));
        let to = self.mem.push_stack(size, 0).ok_or(TrapKind::StackFull { size })?;
        self.mem.blit(to, from, kind)?;
        Ok(Value::Ptr(to))
    }

    /// Park the running thread at the op that ran last, so the op runs again on
    /// a later turn, once what the thread waits for comes.
    ///
    /// # Arguments
    ///
    /// * `waits` - What it waits for.
    pub(super) fn park(&mut self, waits: Waits)
    {
        do_if!(let Some(frame) = self.frames.last_mut(), frame.pc -= 1);
        self.threads.list[self.threads.now].waits = waits;
    }

    /// Sleep: pop a count of milliseconds, and park the running thread until
    /// the clock of the world reaches it. The op does not run again, so the
    /// thread goes on after it once it wakes.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap for a stack that lacks the count.
    pub(super) fn sleep(&mut self) -> Step
    {
        let ms = u64::try_from(self.pop_int("sleep")?).unwrap_or(u64::MAX);
        let wake = self.world.now_ms().saturating_add(ms);
        self.threads.list[self.threads.now].waits = Waits::Until(wake);
        Ok(())
    }

    /// Wait for a thread: its result is pushed, or copied to the slot under its
    /// number. A thread that has not ended parks the waiter, and the op runs
    /// again on its next turn.
    ///
    /// # Arguments
    ///
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap: `ThreadTrapped` for a thread that stopped at one.
    pub(super) fn wait(&mut self, kind: Option<DataKind>) -> Step
    {
        let id = self.pop_int("wait")?;
        let n = usize::try_from(id).ok().filter(|n| *n < self.threads.list.len());
        let n = n.ok_or_else(|| malformed("wait needs the number of a thread"))?;
        if self.threads.list[n].end == End::Runs
        {
            self.push(Value::int(THREAD_ID, id, self.module.platform_bits))?;
            self.park(Waits::Thread(n));
            return Ok(());
        }
        // One wait takes the result, as the join of Rust does.
        ret_if!(
            self.threads.list[n].awaited,
            Err(TrapKind::ThreadWaited { thread: id as u64 })
        );
        self.threads.list[n].awaited = true;
        let ret = self.threads.list[n].ret;
        let end = std::mem::take(&mut self.threads.list[n].end);
        // The row keeps that the thread ended, with no result left in it.
        let ended = tern!(matches!(end, End::Trapped(_)), end.clone(), End::Done(None));
        self.threads.list[n].end = ended;
        match (end, kind)
        {
            (End::Done(v), _) => self.take_result(v),
            (End::Kept(raw), Some(_)) => self.take_kept(&raw, kind),
            // A wait whose value no one reads drops the result.
            (End::Kept(raw), None) => ret.map_or(Ok(()), |k| self.drop_kept(&raw, k)),
            _ => Err(TrapKind::ThreadTrapped { thread: id as u64 }),
        }
    }

    /// Take the lock of the shared object whose variable the stack pops: a
    /// lock another thread holds parks the thread, and the op runs again on a
    /// later turn.
    ///
    /// # Returns
    ///
    /// `Ok`, or `LockBroken` for a lock whose holder stopped at a trap.
    pub(super) fn lock(&mut self) -> Step
    {
        let (at, obj) = self.lock_block("lock")?;
        let now = self.threads.now;
        let held = self.threads.locks.iter_mut().find(|h| h.obj == obj);
        let Some(held) = held
        else
        {
            self.threads.locks.push(Held { obj, owner: now, depth: 1, broken: false });
            return Ok(());
        };
        ret_if!(held.broken, Err(TrapKind::LockBroken));
        if held.owner == now
        {
            held.depth += 1;
            return Ok(());
        }
        self.push(Value::Ptr(at))?;
        self.park(Waits::Lock(obj));
        Ok(())
    }

    /// Give back the lock of the shared object whose variable the stack pops,
    /// once.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap for a lock the thread does not hold.
    pub(super) fn unlock(&mut self) -> Step
    {
        let (_, obj) = self.lock_block("unlock")?;
        let now = self.threads.now;
        let at = self.threads.locks.iter().position(|h| h.obj == obj && h.owner == now);
        let at =
            at.ok_or_else(|| malformed("unlock of a lock the thread does not hold"))?;
        self.threads.locks[at].depth -= 1;
        do_if!(self.threads.locks[at].depth == 0, _ = self.threads.locks.remove(at));
        Ok(())
    }

    /// Pop the variable of a shared object and find the block of the object:
    /// which names its lock, the same in each thread that holds the object.
    ///
    /// # Arguments
    ///
    /// * `op` - The name of the op, for the trap text.
    ///
    /// # Returns
    ///
    /// The address of the variable and of the block, or a trap.
    fn lock_block(&mut self, op: &str) -> Result<(u64, u64), TrapKind>
    {
        let at = self.pop_ptr(op)?;
        let (base, _) = slice_parts(&self.mem, at)?;
        Ok((at, base))
    }

    /// Give the waiter the result of a thread that returned a value: it is
    /// pushed. A list or struct result is kept, see [`Machine::take_kept`].
    ///
    /// # Arguments
    ///
    /// * `v` - The result the thread left, `None` for a function that gives
    ///   nothing.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn take_result(&mut self, v: Option<Value>) -> Step
    {
        v.map_or(Ok(()), |v| self.push(v))
    }

    /// Give the waiter the list or struct result of a thread, kept as its
    /// octets: the octets move to the slot under the number.
    ///
    /// # Arguments
    ///
    /// * `raw` - The octets of the result.
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn take_kept(&mut self, raw: &[u8], kind: Option<DataKind>) -> Step
    {
        ret_if!(kind.is_none(), Err(malformed("a kept result has a kind")));
        let to = self.stack.last().and_then(|s| s.as_ptr());
        let to = to.ok_or_else(|| malformed("wait needs the slot for the result"))?;
        self.mem.write_octets(to, raw)
    }

    /// Whether the main thread must wait at its last return: other threads have
    /// not ended. It then parks at the return, which runs again once the
    /// others end.
    ///
    /// # Returns
    ///
    /// `true` when it parks.
    pub(super) fn park_at_end(&mut self) -> bool
    {
        ret_false!(self.frames.len() != 1 || !self.threads.on_main());
        ret_false!(!self.threads.others_run());
        self.park(Waits::Each);
        true
    }

    /// The work of the scheduler after each op: a thread that ended keeps its
    /// result, and a thread that ended, parked or used up its turn gives the
    /// next one its turn.
    ///
    /// # Returns
    ///
    /// `Ok`, or `Deadlock` when no thread can run.
    pub(super) fn take_turns(&mut self) -> Step
    {
        let parked = self.threads.list[self.threads.now].waits != Waits::Nothing;
        ret_if!(self.threads.list.len() == 1 && !parked, Ok(()));
        // The main thread returned, after each other thread ended: the run is
        // over.
        ret_if!(self.frames.is_empty() && self.threads.on_main(), Ok(()));
        let ended = self.frames.is_empty() && !self.threads.on_main();
        do_if!(ended, self.end_thread()?);
        self.threads.left = self.threads.left.saturating_sub(1);
        ret_if!(!ended && !parked && self.threads.left > 0, Ok(()));
        self.next_turn()
    }

    /// Give the turn to the next thread that can run.
    ///
    /// # Returns
    ///
    /// `Ok`, or `Deadlock` when no thread can run.
    fn next_turn(&mut self) -> Step
    {
        let mut now = self.world.now_ms();
        let mut next = self.threads.next_to_go(now);
        // No thread can run but one sleeps: the clock goes on to its wake.
        while let (None, Some(wake)) = (next, self.threads.first_wake())
        {
            self.world.sleep_ms(wake.saturating_sub(now));
            now = self.world.now_ms();
            next = self.threads.next_to_go(now);
        }
        let next = next.ok_or(TrapKind::Deadlock)?;
        self.threads.list[next].waits = Waits::Nothing;
        self.threads.left = self.threads.turn();
        self.swap_to(next);
        Ok(())
    }

    /// End the running thread, which returned: keep its result.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap for a stack that lacks the result.
    fn end_thread(&mut self) -> Step
    {
        let now = self.threads.now;
        let ret = self.threads.list[now].ret;
        let result = ret.map(|_| self.pop()).transpose()?;
        let platform = self.module.platform_bits;
        // A list or struct result leaves the region of the thread before
        // another thread takes the region.
        let block =
            ret.filter(|k| k.is_aggregate()).zip(result.and_then(|v| v.as_ptr()));
        let kept =
            block.map(|(k, at)| self.mem.octets(at, u64::from(k.size(platform))));
        self.threads.list[now].end = match kept
        {
            Some(raw) => End::Kept(raw?),
            None => End::Done(result),
        };
        Ok(())
    }

    /// Stop the running thread at a trap, when `--trap unwind` lets the program
    /// go on: the trap of the main thread, a deadlock, or `--trap abort` stops
    /// the program.
    ///
    /// # Arguments
    ///
    /// * `trap` - The trap.
    ///
    /// # Returns
    ///
    /// The state of the run: it goes on, or it stopped at the trap.
    pub(super) fn thread_trapped(&mut self, trap: Trap) -> State
    {
        let whole = self.threads.on_main()
            || trap.kind == TrapKind::Deadlock
            || self.options.trap == TrapMode::Abort;
        ret_if!(whole, State::Trapped(trap));
        let now = self.threads.now;
        self.threads.list[now].end = End::Trapped(Box::new(trap));
        // The locks it held remain taken, broken: a later lock of each traps.
        self.threads
            .locks
            .iter_mut()
            .filter(|h| h.owner == now)
            .for_each(|h| h.broken = true);
        self.unwind_frames();
        match self.next_turn()
        {
            Ok(()) => State::Running,
            Err(kind) => State::Trapped(self.trap(kind)),
        }
    }

    /// Give back the calls of the running thread, which stopped at a trap.
    /// shortcut: the values its calls held are not dropped, so each remains
    /// held to the end; drop them on unwind if a program traps in a loop.
    fn unwind_frames(&mut self)
    {
        let frames = std::mem::take(&mut self.frames);
        let give = |m: &mut Machine, f: &Frame| m.mem.pop_stack(&f.blocks, f.sp);
        frames.iter().rev().for_each(|f| give(self, f));
        self.stack.clear();
        self.slots = 0;
    }

    /// Drop the list and struct results the threads left, at the end of the
    /// program, so no block remains held.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap a drop hits.
    pub(super) fn drop_thread_results(&mut self) -> Step
    {
        let results: Vec<(Vec<u8>, DataKind)> = self
            .threads
            .list
            .iter()
            .filter_map(|j| match (&j.end, j.ret)
            {
                (End::Kept(raw), Some(k)) => Some((raw.clone(), k)),
                _ => None,
            })
            .collect();
        results.into_iter().try_for_each(|(raw, kind)| self.drop_kept(&raw, kind))?;
        self.drop_pipe_values()
    }

    /// Drop a list or struct result a thread kept as its octets: the octets go
    /// to a stack block of their own for the drop, which leaves after it.
    ///
    /// # Arguments
    ///
    /// * `raw` - The octets.
    /// * `kind` - The kind of the result.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap a drop hits.
    fn drop_kept(&mut self, raw: &[u8], kind: DataKind) -> Step
    {
        let (top, size) = (self.mem.stack_top(), raw.len() as u64);
        let at = self.mem.push_stack(size, 0).ok_or(TrapKind::StackFull { size })?;
        self.mem.write_octets(at, raw)?;
        let dropped = self.drop_value(at, kind);
        self.mem.pop_stack(&[at], top);
        dropped
    }

    /// Make another thread the running one: park the state of the running one
    /// in its row, and take the state of the other from its row.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of the thread.
    fn swap_to(&mut self, n: usize)
    {
        let now = self.threads.now;
        ret_if!(n == now);
        let row = &mut self.threads.list[now];
        row.stack = std::mem::take(&mut self.stack);
        row.frames = std::mem::take(&mut self.frames);
        row.slots = self.slots;
        let next = &mut self.threads.list[n];
        self.stack = std::mem::take(&mut next.stack);
        self.frames = std::mem::take(&mut next.frames);
        self.slots = next.slots;
        let range = next.range;
        let left = self.mem.swap_stack(range);
        self.threads.list[now].range = left;
        self.threads.now = n;
        // A thread that ended runs no more: the next thread takes its region.
        // Its blocks leave the table as the new calls cover them.
        let ended = now != 0 && self.threads.list[now].end != End::Runs;
        do_if!(ended, self.threads.spare.push(left));
    }
}
