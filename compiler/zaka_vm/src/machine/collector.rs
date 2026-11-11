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

use zakac_rust_sugars::ret_true;

use super::*;
use crate::mem::Block;
use crate::mem::Color;

/// The count of blocks that wait in the buffer before a collection runs.
const BUFFER: usize = 1024;

/// Whether a value of a kind can hold a block that holds the value back: a
/// box, an object or a closure, at once or inside a list or a struct.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` for such a kind.
pub(crate) fn can_ring(kind: DataKind) -> bool
{
    let kind = kind.repr();
    ret_true!(kind.is_box() || kind.is_fn());
    ret_if!(let Some(l) = kind.as_list(), can_ring(l.cell));
    ret_if!(let Some(s) = kind.as_seq(), can_ring(s.cell));
    kind.as_struct().is_some_and(|s| s.fields().iter().any(|(_, k)| can_ring(*k)))
}

impl Machine
{
    /// Note a block that a release left held: it can be part of a ring
    /// that nothing outside holds. A full buffer runs a collection.
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
        let Some(b) = self.mem.live_block_mut(block)
        else
        {
            return Ok(());
        };
        ret_if!(b.buffered || !b.kind.is_some_and(can_ring), Ok(()));
        b.buffered = true;
        self.candidates.push(block);
        ret_if!(self.candidates.len() < BUFFER, Ok(()));
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
