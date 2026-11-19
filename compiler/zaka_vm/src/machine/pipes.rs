//! The channels of a program: queues of values between jobs. A send moves
//! its value into a heap block at the back of the queue; a receive moves the
//! value of the front block into an optional and frees the block. A send to
//! a full channel and a receive from an open one that holds none park the job
//! until the channel changes. The values a channel still holds at the end of
//! the program are dropped.

use std::collections::VecDeque;

use super::jobs::JOB_ID;
use super::jobs::Waits;
use super::*;

/// One channel.
#[derive(Debug, Clone, Default, Builder)]
pub(super) struct Pipe
{
    /// The heap block of each value that waits, the oldest first.
    #[builder(default)]
    queue: VecDeque<u64>,
    /// The most values that wait at once, or `None` for no limit.
    room: Option<u32>,
    /// The kind of its values, known from the first send.
    kind: Option<DataKind>,
    /// Whether it is closed.
    #[builder(default)]
    closed: bool,
}

impl Pipe
{
    /// Whether a receive can go on: a value waits, or the channel is closed.
    ///
    /// # Returns
    ///
    /// `true` if it can.
    pub(super) fn can_receive(&self) -> bool { self.closed || !self.queue.is_empty() }

    /// Whether a send can go on: the channel has room, or it is closed, so the
    /// send stops the program.
    ///
    /// # Returns
    ///
    /// `true` if it can.
    pub(super) fn can_send(&self) -> bool
    {
        let full = self.room.is_some_and(|r| self.queue.len() >= r as usize);
        self.closed || !full
    }
}

impl Machine
{
    /// Make a new channel and push its number.
    ///
    /// # Arguments
    ///
    /// * `room` - The most values that wait in it, or `None` for no limit.
    ///
    /// # Returns
    ///
    /// `Ok`.
    pub(super) fn pipe_new(&mut self, room: Option<u32>) -> Step
    {
        self.jobs.pipes.push(Pipe::builder().maybe_room(room).build());
        let n = self.jobs.pipes.len() as u128;
        self.push(Value::int(JOB_ID, n, self.module.platform_bits))
    }

    /// Pop a channel number and find its row.
    ///
    /// # Arguments
    ///
    /// * `op` - The name of the op, for the trap text.
    ///
    /// # Returns
    ///
    /// The number as the program holds it, and the index of the row.
    fn pop_pipe(&mut self, op: &str) -> Result<(u128, usize), TrapKind>
    {
        let id = self.pop_int(op)?;
        let at = usize::try_from(id).ok().and_then(|n| n.checked_sub(1));
        let at = at.filter(|n| *n < self.jobs.pipes.len());
        let at = at
            .ok_or_else(|| malformed(&format!("{op} needs the number of a channel")))?;
        Ok((id, at))
    }

    /// Push a channel number back, for an op that parks and runs again.
    ///
    /// # Arguments
    ///
    /// * `id` - The number.
    ///
    /// # Returns
    ///
    /// `Ok`.
    fn push_pipe(&mut self, id: u128) -> Step
    {
        self.push(Value::int(JOB_ID, id, self.module.platform_bits))
    }

    /// Move a value into a channel, leaving the default of its kind in its
    /// place; a full channel parks the job, and a closed one stops it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the value.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap: `PipeClosed` for a closed channel.
    pub(super) fn pipe_send(&mut self, kind: DataKind) -> Step
    {
        let src = self.pop_ptr("pipe_send")?;
        let (id, at) = self.pop_pipe("pipe_send")?;
        let pipe = &self.jobs.pipes[at];
        ret_if!(pipe.closed, Err(TrapKind::PipeClosed { pipe: id as u64 }));
        if !pipe.can_send()
        {
            self.push_pipe(id)?;
            self.push(Value::Ptr(src))?;
            self.park(Waits::Send(at));
            return Ok(());
        }
        let size = u64::from(kind.size(self.module.platform_bits));
        let block = self.mem.alloc(size, Origin::Heap, 0);
        let block = block.ok_or(TrapKind::OutOfMem { size })?;
        self.mem.blit(block, src, kind)?;
        self.mem.zero(src, kind)?;
        let pipe = &mut self.jobs.pipes[at];
        pipe.kind = Some(kind);
        pipe.queue.push_back(block);
        Ok(())
    }

    /// Write the next value of a channel as an optional to the slot under its
    /// number: an open channel that holds none parks the job, and a closed and
    /// drained one gives no data.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the values.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap.
    pub(super) fn pipe_receive(&mut self, kind: DataKind) -> Step
    {
        let (id, at) = self.pop_pipe("pipe_receive")?;
        let slot = self.stack.last().and_then(|v| v.as_ptr());
        let slot = slot.ok_or_else(|| malformed("pipe_receive needs the slot"))?;
        let optional = DataKind::optional(kind);
        let pipe = &mut self.jobs.pipes[at];
        if !pipe.can_receive()
        {
            self.push_pipe(id)?;
            self.park(Waits::Receive(at));
            return Ok(());
        }
        let Some(block) = pipe.queue.pop_front()
        else
        {
            return self.mem.zero(slot, optional);
        };
        let shape =
            optional.as_struct().ok_or_else(|| malformed("an optional is a struct"))?;
        let platform = self.module.platform_bits;
        let field = |name| {
            shape.field(name).map(|(i, k)| (u64::from(shape.offset(i, platform)), k))
        };
        let ((tag, tag_kind), (some, _)) = field("tag")
            .zip(field("some"))
            .ok_or_else(|| malformed("an optional has a tag and data"))?;
        let DataKind::Int(int) = tag_kind
        else
        {
            return Err(malformed("the tag of an optional is an integer"));
        };
        self.mem.zero(slot, optional)?;
        self.mem.store(slot + tag, Value::int(int, 1, platform))?;
        self.mem.blit(slot + some, block, kind)?;
        self.mem.free(block)
    }

    /// Close a channel; a second close stops the program.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap: `PipeClosed` for a channel closed before.
    pub(super) fn pipe_close(&mut self) -> Step
    {
        let (id, at) = self.pop_pipe("pipe_close")?;
        let pipe = &mut self.jobs.pipes[at];
        ret_if!(pipe.closed, Err(TrapKind::PipeClosed { pipe: id as u64 }));
        pipe.closed = true;
        Ok(())
    }

    /// Drop and free each value the channels still hold, at the end of the
    /// program.
    ///
    /// # Returns
    ///
    /// `Ok`, or a trap a drop hits.
    pub(super) fn drop_pipe_values(&mut self) -> Step
    {
        let left: Vec<(u64, DataKind)> = self
            .jobs
            .pipes
            .iter_mut()
            .filter_map(|p| Some((std::mem::take(&mut p.queue), p.kind?)))
            .flat_map(|(queue, kind)| queue.into_iter().map(move |b| (b, kind)))
            .collect();
        left.into_iter().try_for_each(|(block, kind)| {
            self.drop_value(block, kind)?;
            self.mem.free(block)
        })
    }
}
