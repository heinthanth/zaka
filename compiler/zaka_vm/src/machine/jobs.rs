//! The jobs of a program, run as green threads: each job has its own
//! operand stack, calls and stack region, over the one heap, and the jobs
//! take turns of a fixed count of ops on one host thread. The order of the
//! turns is fixed, so a run gives the same output each time.
//!
//! The running job keeps its state in the fields of the machine; each other
//! job keeps its state in its row of [`Jobs`], and a switch swaps the two. A
//! wait for a job that has not ended parks the waiter: the op goes back one,
//! so it runs again once the job it waits for ends.

use zakac_rust_sugars::ret_false;

use super::*;
use crate::mem::StackRange;

/// The ops one job runs before the next one takes its turn.
const TURN: u32 = 100;

/// What a job does when it traps, as `--trap` chooses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum TrapMode
{
    /// The trap ends its job alone; a wait for that job traps in turn. A
    /// trap of the main job still stops the program.
    #[default]
    Unwind,
    /// The trap of a job stops the whole program.
    Abort,
}

/// What a parked job waits for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Waits
{
    /// Nothing: the job can run.
    #[default]
    Nothing,
    /// The end of the job with this number.
    Job(usize),
    /// The end of each other job: the main job at the end of the program.
    Each,
}

/// Where a job stands.
#[derive(Debug, Clone, PartialEq, Default)]
enum End
{
    /// It runs, or waits for its turn.
    #[default]
    Runs,
    /// It returned, with its result: a value, or for a list or struct
    /// result the address of the block that holds it.
    Done(Option<Value>),
    /// It stopped at a trap.
    Trapped(Box<Trap>),
}

/// One job.
#[derive(Debug, Clone, Default, Builder)]
struct Job
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
    /// The blocks that hold its own copies of its list and struct
    /// arguments, each with its kind, dropped when it ends.
    #[builder(default)]
    kept: Vec<(u64, DataKind)>,
    /// Whether some job waited for it.
    #[builder(default)]
    awaited: bool,
}

/// The jobs of a program and their turns.
#[derive(Debug, Clone)]
pub struct Jobs
{
    /// Each job, main at 0; the row of the running one is out of date.
    list: Vec<Job>,
    /// The number of the running job.
    now: usize,
    /// The ops left in the turn of the running job.
    left: u32,
}

impl Default for Jobs
{
    fn default() -> Self { Jobs { list: vec![Job::default()], now: 0, left: TURN } }
}

impl Jobs
{
    /// Whether a job stopped at a trap and no job waited for it: the program
    /// then ends with code 1.
    ///
    /// # Returns
    ///
    /// `true` if one did.
    pub fn lost_trap(&self) -> bool
    {
        self.list.iter().any(|j| matches!(j.end, End::Trapped(_)) && !j.awaited)
    }

    /// The traps that ended jobs, each with the number of its job.
    ///
    /// # Returns
    ///
    /// The traps, in job order.
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

    /// Whether the running job is the main one.
    ///
    /// # Returns
    ///
    /// `true` for the main job.
    pub fn on_main(&self) -> bool { self.now == 0 }

    /// Whether a job other than the running one has not ended.
    ///
    /// # Returns
    ///
    /// `true` if one has not.
    fn others_run(&self) -> bool
    {
        let runs = |(n, j): (usize, &Job)| n != self.now && j.end == End::Runs;
        self.list.iter().enumerate().any(runs)
    }

    /// Whether a job can run: it has not ended, and what it waits for came.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of the job.
    ///
    /// # Returns
    ///
    /// `true` if it can.
    fn ready(&self, n: usize) -> bool
    {
        let job = &self.list[n];
        ret_false!(job.end != End::Runs);
        match job.waits
        {
            Waits::Nothing => true,
            Waits::Job(other) =>
            {
                self.list.get(other).is_none_or(|j| j.end != End::Runs)
            }
            Waits::Each =>
            {
                let ended = |(k, j): (usize, &Job)| k == n || j.end != End::Runs;
                self.list.iter().enumerate().all(ended)
            }
        }
    }

    /// The next job that can run, after the running one in order; the
    /// running one itself comes last.
    ///
    /// # Returns
    ///
    /// The number of the job, or `None` when none can run.
    fn next_ready(&self) -> Option<usize>
    {
        let count = self.list.len();
        (1..=count).map(|k| (self.now + k) % count).find(|n| self.ready(*n))
    }
}

/// The kind of a job number.
const JOB_ID: IntKind = IntKind::new(false, Width::Platform);

impl Machine
{
    /// Start a job that calls a function: its arguments leave the stack,
    /// the job enters the function at once, and the number of the job is
    /// pushed. The job holds its own copy of each list and struct argument.
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
        let range = self.mem.take_region(size).ok_or(TrapKind::StackFull { size })?;
        let n = self.jobs.list.len();
        let job = Job::builder().range(range).maybe_ret(ret).build();
        self.jobs.list.push(job);
        let back = self.jobs.now;
        self.swap_to(n);
        let entered = self.enter_job(f, args, &kinds, ret);
        self.swap_to(back);
        entered?;
        let platform = self.module.platform_bits;
        self.push(Value::int(JOB_ID, n as u128, platform))
    }

    /// Enter the function of a new job, the running one: the slot of a list
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
    fn enter_job(
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

    /// Copy a list or struct argument into a block of the running job, which
    /// holds its own count of what the argument holds.
    ///
    /// # Arguments
    ///
    /// * `v` - The address of the argument.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The address of the copy, or a trap.
    fn keep_arg(&mut self, v: Value, kind: DataKind) -> Result<Value, TrapKind>
    {
        let from = v.as_ptr().ok_or_else(|| malformed("an argument is an address"))?;
        let size = u64::from(kind.size(self.module.platform_bits));
        let to = self.mem.push_stack(size, 0).ok_or(TrapKind::StackFull { size })?;
        self.mem.blit(to, from, kind)?;
        self.dup_value(to, kind)?;
        self.jobs.list[self.jobs.now].kept.push((to, kind));
        Ok(Value::Ptr(to))
    }

    /// Wait for a job: its result is pushed, or copied to the slot under its
    /// number. A job that has not ended parks the waiter, and the op runs
    /// again on its next turn.
    ///
    /// # Arguments
    ///
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap: `JobTrapped` for a job that stopped at one.
    pub(super) fn wait(&mut self, kind: Option<DataKind>) -> Step
    {
        let id = self.pop_int("wait")?;
        let n = usize::try_from(id).ok().filter(|n| *n < self.jobs.list.len());
        let n = n.ok_or_else(|| malformed("wait needs the number of a job"))?;
        if self.jobs.list[n].end == End::Runs
        {
            self.push(Value::int(JOB_ID, id, self.module.platform_bits))?;
            do_if!(let Some(frame) = self.frames.last_mut(), frame.pc -= 1);
            self.jobs.list[self.jobs.now].waits = Waits::Job(n);
            return Ok(());
        }
        self.jobs.list[n].awaited = true;
        match self.jobs.list[n].end.clone()
        {
            End::Done(v) => self.take_result(v, kind),
            _ => Err(TrapKind::JobTrapped { job: id as u64 }),
        }
    }

    /// Give the waiter the result of a job that returned.
    ///
    /// # Arguments
    ///
    /// * `v` - The result the job left.
    /// * `kind` - The result kind.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    fn take_result(&mut self, v: Option<Value>, kind: Option<DataKind>) -> Step
    {
        let (Some(v), Some(kind)) = (v, kind)
        else
        {
            return Ok(());
        };
        ret_if!(!kind.is_aggregate(), self.push(v));
        let from = v.as_ptr().ok_or_else(|| malformed("a job result is an address"))?;
        let to = self.stack.last().and_then(|s| s.as_ptr());
        let to = to.ok_or_else(|| malformed("wait needs the slot for the result"))?;
        self.mem.blit(to, from, kind)?;
        self.dup_value(to, kind)
    }

    /// Whether the main job must wait at its last return: other jobs have not
    /// ended. It then parks at the return, which runs again once they end.
    ///
    /// # Returns
    ///
    /// `true` when it parks.
    pub(super) fn park_at_end(&mut self) -> bool
    {
        ret_false!(self.frames.len() != 1 || !self.jobs.on_main());
        ret_false!(!self.jobs.others_run());
        do_if!(let Some(frame) = self.frames.last_mut(), frame.pc -= 1);
        self.jobs.list[0].waits = Waits::Each;
        true
    }

    /// The work of the scheduler after each op: a job that ended keeps its
    /// result, and a job that ended, parked or used up its turn gives the
    /// next one its turn.
    ///
    /// # Returns
    ///
    /// `Ok`, or `Deadlock` when no job can run.
    pub(super) fn take_turns(&mut self) -> Step
    {
        ret_if!(self.jobs.list.len() == 1, Ok(()));
        // The main job returned, after each other job ended: the run is over.
        ret_if!(self.frames.is_empty() && self.jobs.on_main(), Ok(()));
        let ended = self.frames.is_empty() && !self.jobs.on_main();
        do_if!(ended, self.end_job()?);
        let parked = self.jobs.list[self.jobs.now].waits != Waits::Nothing;
        self.jobs.left = self.jobs.left.saturating_sub(1);
        ret_if!(!ended && !parked && self.jobs.left > 0, Ok(()));
        self.next_turn()
    }

    /// Give the turn to the next job that can run.
    ///
    /// # Returns
    ///
    /// `Ok`, or `Deadlock` when no job can run.
    fn next_turn(&mut self) -> Step
    {
        let next = self.jobs.next_ready().ok_or(TrapKind::Deadlock)?;
        self.jobs.list[next].waits = Waits::Nothing;
        self.jobs.left = TURN;
        self.swap_to(next);
        Ok(())
    }

    /// End the running job, which returned: keep its result and drop its
    /// copies of its arguments.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap a drop hits.
    fn end_job(&mut self) -> Step
    {
        let now = self.jobs.now;
        let ret = self.jobs.list[now].ret;
        let result = ret.map(|_| self.pop()).transpose()?;
        self.jobs.list[now].end = End::Done(result);
        let kept = std::mem::take(&mut self.jobs.list[now].kept);
        kept.into_iter().try_for_each(|(at, kind)| self.drop_value(at, kind))
    }

    /// Stop the running job at a trap, when `--trap unwind` lets the program
    /// go on: the trap of the main job, a deadlock, or `--trap abort` stops
    /// the program.
    ///
    /// # Arguments
    ///
    /// * `trap` - The trap.
    ///
    /// # Returns
    ///
    /// The state of the run: it goes on, or it stopped at the trap.
    pub(super) fn job_trapped(&mut self, trap: Trap) -> State
    {
        let whole = self.jobs.on_main()
            || trap.kind == TrapKind::Deadlock
            || self.options.trap == TrapMode::Abort;
        ret_if!(whole, State::Trapped(trap));
        let now = self.jobs.now;
        self.jobs.list[now].end = End::Trapped(Box::new(trap));
        self.unwind_frames();
        match self.next_turn()
        {
            Ok(()) => State::Running,
            Err(kind) => State::Trapped(self.trap(kind)),
        }
    }

    /// Give back the calls of the running job, which stopped at a trap.
    /// shortcut: the values its calls held are not dropped, so they stay
    /// held to the end; drop them on unwind if a program traps in a loop.
    fn unwind_frames(&mut self)
    {
        let frames = std::mem::take(&mut self.frames);
        let give = |m: &mut Machine, f: &Frame| m.mem.pop_stack(&f.blocks, f.sp);
        frames.iter().rev().for_each(|f| give(self, f));
        self.stack.clear();
        self.slots = 0;
    }

    /// Drop the list and struct results the jobs left, at the end of the
    /// program, so no block stays held.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap a drop hits.
    pub(super) fn drop_job_results(&mut self) -> Step
    {
        let results: Vec<(u64, DataKind)> = self
            .jobs
            .list
            .iter()
            .filter_map(|j| match (&j.end, j.ret)
            {
                (End::Done(Some(Value::Ptr(at))), Some(k)) if k.is_aggregate() =>
                {
                    Some((*at, k))
                }
                _ => None,
            })
            .collect();
        results.into_iter().try_for_each(|(at, kind)| self.drop_value(at, kind))
    }

    /// Make another job the running one: park the state of the running one
    /// in its row, and take the state of the other from its row.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of the job.
    fn swap_to(&mut self, n: usize)
    {
        let now = self.jobs.now;
        ret_if!(n == now);
        let row = &mut self.jobs.list[now];
        row.stack = std::mem::take(&mut self.stack);
        row.frames = std::mem::take(&mut self.frames);
        row.slots = self.slots;
        let next = &mut self.jobs.list[n];
        self.stack = std::mem::take(&mut next.stack);
        self.frames = std::mem::take(&mut next.frames);
        self.slots = next.slots;
        let range = next.range;
        self.jobs.list[now].range = self.mem.swap_stack(range);
        self.jobs.now = n;
    }
}
