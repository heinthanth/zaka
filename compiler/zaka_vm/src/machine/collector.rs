//! The ring collector: trial deletion that halts the program while it runs,
//! as PHP and CPython do it. Counting frees a block once nothing holds it,
//! but two blocks that hold each other never reach a count of 0. A release
//! that leaves a block held, when its cells can reach a box, an object or a
//! closure, puts the block in a buffer. A collection takes off, for a trial,
//! each count the blocks the buffer reaches give one another. A block still
//! held after that is held from outside, and it and the blocks it reaches get
//! their counts back; the rest hold one another alone, so the whole group is
//! garbage and is freed together.
//!
//! Each walk keeps a list of the blocks still to visit, so a long chain of
//! objects needs no deep call stack.

use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_true;

use super::*;
use crate::mem::Block;
use crate::mem::Color;

/// The octets of the blocks that wait in the buffer before a collection runs,
/// unless the program sets its own threshold.
pub const DEFAULT_THRESHOLD: u64 = 16 * 1024;

/// What the ring collector keeps between collections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rings
{
    /// The octets of the blocks in the buffer.
    pub waiting: u64,
    /// The octets in the buffer that run a collection.
    pub threshold: u64,
    /// Whether the program paused the collections a full buffer runs.
    pub paused: bool,
}

impl Default for Rings
{
    fn default() -> Self
    {
        Rings { waiting: 0, threshold: DEFAULT_THRESHOLD, paused: false }
    }
}

/// Whether a value of a kind holds a box, an object or a closure, at once or
/// inside a list or a struct: what a deep duplicate walks into.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` for such a kind.
pub(crate) fn holds_shared(kind: DataKind) -> bool
{
    let kind = kind.repr();
    ret_true!(kind.is_box() || kind.is_fn());
    ret_if!(let Some(l) = kind.as_list(), holds_shared(l.cell));
    ret_if!(let Some(s) = kind.as_seq(), holds_shared(s.cell));
    kind.as_struct().is_some_and(|s| s.fields().iter().any(|(_, k)| holds_shared(*k)))
}

/// Whether a block whose cells are of a kind can be part of a ring: whether
/// the blocks its cells hold a count of lead back to a block of the kind, or
/// hold a closure, which can hold each thing.
///
/// # Arguments
///
/// * `kind` - The kind of the cells of the block.
///
/// # Returns
///
/// `true` for such a kind.
pub(crate) fn can_ring(kind: DataKind) -> bool
{
    let mut seen = vec![kind];
    let mut todo = Vec::new();
    ret_true!(!held_kinds(kind, &mut todo));
    while let Some(k) = todo.pop()
    {
        ret_true!(k == kind);
        do_if!(seen.contains(&k), continue);
        seen.push(k);
        ret_true!(!held_kinds(k, &mut todo));
    }
    false
}

/// Gather the kinds of the cells of the blocks a value holds a count of,
/// through its fields and cells: a box, text or a plain list.
///
/// # Arguments
///
/// * `kind` - The kind of the value.
/// * `out` - The cell kinds found.
///
/// # Returns
///
/// `false` when the value holds a function value, whose closure can hold
/// each thing; else `true`.
fn held_kinds(kind: DataKind, out: &mut Vec<DataKind>) -> bool
{
    let kind = kind.repr();
    ret_false!(kind.is_fn());
    if let Some(s) = kind.as_seq()
    {
        out.push(s.cell);
        return true;
    }
    ret_if!(let Some(l) = kind.as_list(), held_kinds(l.cell, out));
    kind.as_struct().is_none_or(|s| s.fields().iter().all(|(_, k)| held_kinds(*k, out)))
}

impl Machine
{
    /// Note a block that a release left held: it can be part of a ring
    /// that nothing outside holds. A full buffer runs a collection, unless
    /// the program paused them.
    ///
    /// # Arguments
    ///
    /// * `block` - The start of the block.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap a collection hits.
    pub(super) fn note_candidate(&mut self, block: u64) -> Step
    {
        let Some(kind) = self.mem.live_block(block).and_then(|b| b.kind)
        else
        {
            return Ok(());
        };
        ret_if!(!self.kind_can_ring(kind), Ok(()));
        let Some(b) = self.mem.live_block_mut(block).filter(|b| !b.buffered)
        else
        {
            return Ok(());
        };
        b.buffered = true;
        self.rings.waiting += b.len;
        self.candidates.push(block);
        ret_if!(self.rings.paused || self.rings.waiting < self.rings.threshold, Ok(()));
        self.collect_rings()
    }

    /// [`can_ring`], worked out once for each kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the cells of a block.
    ///
    /// # Returns
    ///
    /// `true` when a block of the kind can be part of a ring.
    fn kind_can_ring(&mut self, kind: DataKind) -> bool
    {
        let known = self.ring_kinds.iter().find(|(k, _)| *k == kind).map(|(_, r)| *r);
        ret_if!(let Some(ring) = known, ring);
        let ring = can_ring(kind);
        self.ring_kinds.push((kind, ring));
        ring
    }

    /// Run an op of the ring collector.
    ///
    /// # Arguments
    ///
    /// * `act` - What it does; a threshold pops its octets.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap a collection hits.
    pub(super) fn rings(&mut self, act: RingsAct) -> Step
    {
        match act
        {
            RingsAct::Collect => self.collect_rings(),
            RingsAct::Pause =>
            {
                self.pause_rings();
                Ok(())
            }
            RingsAct::Resume => self.resume_rings(),
            RingsAct::Threshold =>
            {
                let octets = self.pop_int("rings threshold")?;
                self.set_ring_threshold(u64::try_from(octets).unwrap_or(u64::MAX))
            }
        }
    }

    /// Pause the collections a full buffer runs; the buffer still fills,
    /// and the end of the program still collects.
    fn pause_rings(&mut self) { self.rings.paused = true; }

    /// Resume the collections a full buffer runs, and collect at once when
    /// the buffer filled while they were paused.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap a collection hits.
    fn resume_rings(&mut self) -> Step
    {
        self.rings.paused = false;
        ret_if!(self.rings.waiting < self.rings.threshold, Ok(()));
        self.collect_rings()
    }

    /// Set the octets in the buffer that run a collection.
    ///
    /// # Arguments
    ///
    /// * `octets` - The threshold.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap a collection hits, when the buffer holds that much
    /// now.
    fn set_ring_threshold(&mut self, octets: u64) -> Step
    {
        self.rings.threshold = octets;
        ret_if!(self.rings.paused || self.rings.waiting < octets, Ok(()));
        self.collect_rings()
    }

    /// Free each group of blocks in the buffer, or reached from it, that
    /// holds itself alone.
    ///
    /// # Returns
    ///
    /// `Ok`, or the trap a free hits.
    pub(super) fn collect_rings(&mut self) -> Step
    {
        self.rings.waiting = 0;
        let buffered = std::mem::take(&mut self.candidates);
        let mut roots = Vec::new();
        for r in buffered
        {
            let Some(b) = self.mem.live_block_mut(r)
            else
            {
                continue;
            };
            b.buffered = false;
            do_if!(b.rc > 0, roots.push(r));
        }
        do_for!(r in &roots, self.mark_pending(*r));
        do_for!(r in &roots, self.scan(*r));
        let mut white = Vec::new();
        do_for!(r in &roots, self.collect_white(*r, &mut white));
        do_for!(b in white, self.mem.free(b)?);
        Ok(())
    }

    /// The blocks a counted block holds a count of: the owned text, plain
    /// lists, boxes and closure records in its cells. A lent value holds no
    /// count.
    ///
    /// # Arguments
    ///
    /// * `block` - The start of the block.
    ///
    /// # Returns
    ///
    /// The starts of the blocks.
    fn children(&self, block: u64) -> Vec<u64>
    {
        let Some(b) = self.mem.live_block(block)
        else
        {
            return Vec::new();
        };
        let Some(cell) = b.kind
        else
        {
            return Vec::new();
        };
        let stride = u64::from(cell.size(self.module.platform_bits)).max(1);
        let cells = (0..b.len / stride).map(|i| block + i * stride);
        let slots = cells.flat_map(|at| self.blocks_in(at, cell));
        slots.filter_map(|(at, kind)| self.held_block(at, kind)).collect()
    }

    /// The block a slot holds a count of: the record of a closure that is
    /// not lent, or the block of text or a plain list that owns it.
    ///
    /// # Arguments
    ///
    /// * `at` - The address of the slot.
    /// * `kind` - Its kind.
    ///
    /// # Returns
    ///
    /// The start of the block, or `None`.
    fn held_block(&self, at: u64, kind: DataKind) -> Option<u64>
    {
        let word = self.mem.load(at, DataKind::OPAQUE).ok()?.as_ptr()?;
        ret_if!(word == 0, None);
        ret_if!(kind.is_fn() && word & 1 == 1, None);
        ret_if!(kind.is_fn(), self.mem.closure_record(word));
        tern!(self.room_of(at).ok()? > 0, Some(word), None)
    }

    /// Take off, for a trial, each count a block reached from a root gives
    /// another: paint each such block pending.
    ///
    /// # Arguments
    ///
    /// * `root` - A block of the buffer.
    fn mark_pending(&mut self, root: u64)
    {
        let mut todo = vec![root];
        while let Some(b) = todo.pop()
        {
            let Some(block) =
                self.mem.live_block_mut(b).filter(|b| b.color != Color::Pending)
            else
            {
                continue;
            };
            block.color = Color::Pending;
            for c in self.children(b)
            {
                do_if!(let Some(child) = self.mem.live_block_mut(c), child.rc = child.rc.saturating_sub(1));
                todo.push(c);
            }
        }
    }

    /// Paint white each pending block that nothing outside holds, and give a
    /// held one and what it reaches their counts back.
    ///
    /// # Arguments
    ///
    /// * `root` - A block of the buffer.
    fn scan(&mut self, root: u64)
    {
        let mut todo = vec![root];
        while let Some(b) = todo.pop()
        {
            let Some(block) =
                self.mem.live_block_mut(b).filter(|b| b.color == Color::Pending)
            else
            {
                continue;
            };
            if block.rc > 0
            {
                self.scan_black(b);
                continue;
            }
            block.color = Color::White;
            todo.extend(self.children(b));
        }
    }

    /// Give a block held from outside, and each block it reaches, the counts
    /// the trial took off, and paint them black.
    ///
    /// # Arguments
    ///
    /// * `root` - The held block.
    fn scan_black(&mut self, root: u64)
    {
        do_if!(let Some(block) = self.mem.live_block_mut(root), block.color = Color::Black);
        let mut todo = vec![root];
        while let Some(b) = todo.pop()
        {
            for c in self.children(b)
            {
                let Some(child) = self.mem.live_block_mut(c)
                else
                {
                    continue;
                };
                child.rc += 1;
                do_if!(child.color != Color::Black, {
                    child.color = Color::Black;
                    todo.push(c);
                });
            }
        }
    }

    /// Gather the white blocks a root reaches, painting them black so each
    /// is gathered once.
    ///
    /// # Arguments
    ///
    /// * `root` - A block of the buffer.
    /// * `out` - The blocks to free.
    fn collect_white(&mut self, root: u64, out: &mut Vec<u64>)
    {
        let mut todo = vec![root];
        while let Some(b) = todo.pop()
        {
            let white =
                |block: &&mut Block| block.color == Color::White && !block.buffered;
            let Some(block) = self.mem.live_block_mut(b).filter(white)
            else
            {
                continue;
            };
            block.color = Color::Black;
            out.push(b);
            todo.extend(self.children(b));
        }
    }
}
