//! The light tasks that wait for a change of the state of the threads, filed
//! under what each waits on, so a change looks at the tasks it concerns and not
//! at each task that waits.

use std::collections::BTreeMap;

use zakac_rust_sugars::do_if;

use crate::threads::Key;

/// Where a task that waits is filed.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Bucket
{
    /// The end of the thread with this number.
    Thread(usize),
    /// The channels at the indices whose rest after a division with 64 is this.
    PipeBit(u32),
    /// The lock of the object at this address.
    Lock(usize),
    /// The end of a time limit.
    Timer,
    /// Each change.
    Each,
}

/// The buckets that what a task waits on files it under, or that a change of
/// it looks at.
///
/// # Arguments
///
/// * `on` - What the task waits on, or what changed.
///
/// # Returns
///
/// The buckets.
fn buckets(on: Key) -> Vec<Bucket>
{
    match on
    {
        Key::Thread(n) => vec![Bucket::Thread(n)],
        Key::Pipe(p) => vec![Bucket::PipeBit((p % 64) as u32)],
        Key::Lock(obj) => vec![Bucket::Lock(obj)],
        Key::Pipes { mask, timed } => (0..64)
            .filter(|b| mask >> b & 1 == 1)
            .map(Bucket::PipeBit)
            .chain(timed.then_some(Bucket::Timer))
            .collect(),
        Key::Timer => vec![Bucket::Timer],
        Key::Each => vec![Bucket::Each],
    }
}

/// The light tasks that wait, each under an id, and the ids under each bucket.
pub(crate) struct Waiters<T>
{
    /// Each task with what it waits on.
    tasks: BTreeMap<u64, (Key, T)>,
    /// The ids of the tasks under each bucket.
    filed: BTreeMap<Bucket, Vec<u64>>,
    /// The id of the next task.
    next: u64,
}

impl<T> Waiters<T>
{
    /// No task waits.
    ///
    /// # Returns
    ///
    /// The table.
    pub(crate) const fn new() -> Waiters<T>
    {
        Waiters { tasks: BTreeMap::new(), filed: BTreeMap::new(), next: 0 }
    }

    /// Keep a task until a change it waits on.
    ///
    /// # Arguments
    ///
    /// * `on` - What it waits on.
    /// * `task` - The task.
    pub(crate) fn park(&mut self, on: Key, task: T)
    {
        let id = self.next;
        self.next += 1;
        buckets(on).into_iter().for_each(|b| self.filed.entry(b).or_default().push(id));
        self.tasks.insert(id, (on, task));
    }

    /// Take the tasks that wait on what changed.
    ///
    /// # Arguments
    ///
    /// * `what` - What changed.
    ///
    /// # Returns
    ///
    /// The tasks.
    pub(crate) fn take(&mut self, what: Key) -> Vec<T>
    {
        if what == Key::Each
        {
            self.filed.clear();
            let tasks = std::mem::take(&mut self.tasks);
            return tasks.into_values().map(|(_, task)| task).collect();
        }
        let looked: Vec<Bucket> =
            buckets(what).into_iter().chain([Bucket::Each]).collect();
        let woken: Vec<u64> =
            looked.iter().flat_map(|b| self.split(*b, what)).collect();
        woken.into_iter().filter_map(|id| self.remove(id, &looked)).collect()
    }

    /// Take out of a bucket the ids of the tasks that a change concerns.
    ///
    /// # Arguments
    ///
    /// * `bucket` - The bucket.
    /// * `what` - What changed.
    ///
    /// # Returns
    ///
    /// The ids.
    fn split(&mut self, bucket: Bucket, what: Key) -> Vec<u64>
    {
        let Some(ids) = self.filed.remove(&bucket)
        else
        {
            return Vec::new();
        };
        let meets = |id: &u64| self.tasks.get(id).is_some_and(|(k, _)| k.meets(what));
        let (woken, kept): (Vec<u64>, Vec<u64>) = ids.into_iter().partition(meets);
        do_if!(!kept.is_empty(), _ = self.filed.insert(bucket, kept));
        woken
    }

    /// Remove a task and its ids from the buckets it was filed under, past
    /// those the change looked at, which no longer hold it.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the task.
    /// * `looked` - The buckets the change looked at.
    ///
    /// # Returns
    ///
    /// The task, or `None` when another bucket took it first.
    fn remove(&mut self, id: u64, looked: &[Bucket]) -> Option<T>
    {
        let (on, task) = self.tasks.remove(&id)?;
        for bucket in buckets(on).into_iter().filter(|b| !looked.contains(b))
        {
            let Some(ids) = self.filed.get_mut(&bucket)
            else
            {
                continue;
            };
            ids.retain(|other| *other != id);
            do_if!(ids.is_empty(), _ = self.filed.remove(&bucket));
        }
        Some(task)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn a_change_takes_the_tasks_it_concerns_alone()
    {
        let mut w = Waiters::new();
        w.park(Key::Pipe(1), "pipe 1");
        w.park(Key::Pipe(65), "pipe 65");
        w.park(Key::Thread(3), "thread 3");
        w.park(Key::Pipes { mask: 1 << 1 | 1 << 2, timed: true }, "select");
        w.park(Key::Each, "each");
        let mut woken = w.take(Key::Pipe(1));
        woken.sort_unstable();
        assert_eq!(woken, ["each", "pipe 1", "select"]);
        assert!(w.take(Key::Pipe(2)).is_empty() && w.take(Key::Timer).is_empty());
        assert_eq!(w.take(Key::Thread(3)), ["thread 3"]);
        assert_eq!(w.take(Key::Each), ["pipe 65"]);
        assert!(w.filed.is_empty() && w.tasks.is_empty());
    }
}
